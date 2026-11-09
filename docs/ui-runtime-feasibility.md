# Browser integration and search-session feasibility (#146)

Research date: 2026-11-09. Baseline: `5bc3333324fe54efab35a6879fe679c0f0e5fbc9` (`origin/main`). Research only; no architecture, framework or public-interface decision.

## Verified core boundary

The [architecture](https://github.com/tizian-bitschi/skating-program-lab/blob/5bc3333324fe54efab35a6879fe679c0f0e5fbc9/docs/architecture.md) places UI/network/storage outside the Rust domain library and explicitly says WASM compatibility is unestablished. [Cargo.toml](https://github.com/tizian-bitschi/skating-program-lab/blob/5bc3333324fe54efab35a6879fe679c0f0e5fbc9/Cargo.toml) pins Pumpkin 0.5.0 behind default-enabled `optimizer`; disabling it is not evidence that complete search works in a browser.

[PlanningSearchSession](https://github.com/tizian-bitschi/skating-program-lab/blob/5bc3333324fe54efab35a6879fe679c0f0e5fbc9/src/planning_search/session.rs) owns its immutable request and prepared solver, borrows `ProfileRegistry`, and exposes synchronous `prove` and `collect(NonZeroUsize, control)`. Collection can pause and resume in that same session; cancellation is terminal. A full batch does not establish exhaustion. Proof has cancellation, not a resumable proof-pause API. There is no persistence or solver-instruction checkpoint promise.

Memory is not bounded by delivery batch size: the session retains every confirmed program, clones each delivery into its outgoing batch, and [Prepared](https://github.com/tizian-bitschi/skating-program-lab/blob/5bc3333324fe54efab35a6879fe679c0f0e5fbc9/src/planning_search/model.rs) retains delivered count vectors and adds exclusion constraints. Therefore pagination/virtualization can bound UI rendering but cannot bound solver/session memory (source-derived inference). Uncapped means no artificial result-count cap, not infinite memory.

Pumpkin's [0.5.0 Solver API](https://docs.rs/pumpkin-solver/0.5.0/pumpkin_solver/struct.Solver.html) explicitly reports `!Send` and `!Sync`. Because Prepared contains Solver, moving an established session between Rust threads or sharing it through ordinary Send/Sync service state is not a credible design. Construct registry and session inside a persistent owner thread; send commands and owned result DTOs. The registry must outlive its sessions; a local registry followed by a local session in an owner loop avoids a self-referential registry/session wrapper. [Rust thread spawning](https://doc.rust-lang.org/std/thread/fn.spawn.html) requires transferred closures/results to be Send and static.

## Feasible approaches and proof gaps

| Approach | Credible integration shape | Required proof before selection |
| --- | --- | --- |
| Browser WASM in one dedicated worker | Worker owns registry and session; UI sends versioned request/command DTOs and receives batches. Existing Rust remains the domain authority. | Compile exact optimizer-enabled dependency graph; run proof, pause/resume, cancellation and exhaustive small oracle; measure phone memory and responsiveness. |
| Native Rust service with browser UI | A persistent native owner thread creates and keeps each session, while transport handles expose opaque session IDs and result DTOs. Network task must not move Solver. | Compile owner-loop adapter; measure concurrency, cleanup, reconnect behavior, command latency and deployment cost. Native existing tests establish core evidence, not service readiness. |

These are candidates, not approved architectures. Internet availability and compute location are open user choices. There is insufficient evidence to justify a third hybrid architecture in this bounded investigation.

### Browser execution and control

The [HTML worker standard](https://html.spec.whatwg.org/multipage/workers.html) defines computation independently of UI scripts, message communication and a worker event loop. It also warns about worker startup and per-instance cost. One long-lived worker per active workspace is a candidate, not a worker per filter item.

Important inference from that event-loop model and the synchronous core: a `postMessage` pause/cancel handler cannot run while the same worker is inside a long synchronous `prove`/`collect` call. Returning between batches permits commands between calls but does not bound time inside a call. A timer in that same worker does not solve this. A cooperative deadline can stop collection through the existing control callback, after which another event-loop task can resume. Resumable proof slicing is not promised by the core. Hard worker termination abandons the session; it is not cooperative pause. An independently observable shared control flag is another experiment, not an established integration.

For shared control, verify host policy and toolchain: [SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer) documents secure-context/cross-origin-isolation requirements for shared memory. Do not make WASM threads or shared-memory deployment a hidden prerequisite. A single worker itself is separate from Rust thread spawning.

The [Rust wasm32-unknown-unknown target guide](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html) supports core/alloc and parts of std, but filesystem operations fail and thread spawning panics. [Pumpkin 0.5.0 documentation](https://docs.rs/pumpkin-solver/0.5.0/pumpkin_solver/) describes pure Rust and a default gzipped-proofs feature. Neither pure Rust nor std availability proves runtime compatibility. Audit transitive randomness, clock, proof/file paths, target-specific dependencies and defaults in a target build; the docs inspected here do not certify this exact graph for WASM.

### Browser differences and costs

Treat current desktop Chrome/Edge, Firefox, Safari, Android Chrome and iPhone Safari as separate acceptance targets. The HTML standard supports the worker model, not equal speed or reliable background completion across devices. No browser/phone performance was measured. Test foreground interaction, background/foreground, screen lock, navigation/reload, low memory, large repertoires, long proof and many tied results. Never promise session survival after a worker/page/process is lost.

Measure compressed WASM/JS transfer, cold compile/instantiate, warm startup, first useful selection, manual validation, first proof, first batch, exhaustion, main-thread long tasks and peak retained memory. Native Release timing cannot be converted into browser timing. For service candidates additionally measure round trips and reconnect/cleanup; a disconnected anonymous client must not leave unbounded retained sessions. These are proposed validation requirements, not measured facts.

## UI-independent adapter proposal

Use one workspace-owned immutable request revision per session. Commands: create, prove, collect(batch size), pause collection, cancel, dispose. Responses carry revision/session identifier, state, maximum separately from alternative exhaustion, new alternatives, diagnostics and errors. Ignore stale responses after editing the request. Do not fabricate percent complete from candidate counts. This protocol is a proposal requiring interface review, not a core change.

Send exact whole-element identities, required occurrences, combined exclusions, jump/spin ceilings and step ceiling to Rust. Required elements remain freely placed under the planning-search contract; do not accidentally translate them into immutable ordered completion. Keep construction validation separate from execution/duration and optional scoring scenarios. Keep locale labels in the UI and stable codes/IDs in requests. No account is needed for either candidate; service session cleanup/expiry remains an engineering requirement.

For the large repertoire, propose local indexed label/code search and windowed result rendering, with Rust supplying permitted choices and authoritative assessment. Measure repertoire payload size and filter latency before choosing full transfer versus paged delivery. Do not truncate valid choices silently.

## Representative frontend shortlist

All candidates need responsive semantic HTML, keyboard/touch support, three translation catalogs (`de`, `en`, `ru`), explicit workspace state and a transport boundary; none is approved and none guarantees usability for age 12+.

- TypeScript + Vite + Vue + Vue I18n: a candidate for structured forms and explicit translation resources. [Vue I18n](https://vue-i18n.intlify.dev/guide/essentials/started) documents messages, locale and fallback locale; its guide includes pluralization/number formatting.
- TypeScript + Vite + React with an independently evaluated i18n package: a candidate when team familiarity favors component/state composition. [React](https://react.dev/learn) documents components, events and shared state. This research does not select an i18n dependency.
- TypeScript + Vite + Svelte with an independently evaluated i18n package: a candidate for a compact single-workspace client. [Svelte](https://svelte.dev/docs/svelte/overview) documents compiled declarative components; no bundle-size superiority is claimed here.

[Vite](https://vite.dev/guide/features.html#web-workers) documents worker bundling and TypeScript transpilation; type checking needs a separate check. The same frontend candidates can consume either integration, so framework selection need not decide compute location.

## Experiments and remaining unknowns

Executed: inspected baseline source, architecture, public session types and Prepared retention; verified local target inventory with `rustup target list --installed` (only `x86_64-pc-windows-msvc`). No WASM target was installed; no download, compile or browser benchmark was attempted. Cached Pumpkin source was absent at the standard user Cargo registry path; docs.rs source-manifest retrieval failed, so transitive target compatibility remains unverified. The selected source clone initially inspected had a different HEAD, so the report clone was explicitly checked out at the required `5bc3333` baseline before final source verification.

Small next feasibility gate: build optimizer-enabled WASM using an approved installed target/toolchain, exercise the same independent finite oracle in a dedicated worker and a native owner thread, prove control responsiveness during one long call, then record representative phone startup/time/memory. Obtain artifact/dependency sizes rather than estimates. No production implementation or architecture approval follows from this report.
