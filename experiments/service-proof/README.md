# Native service feasibility — planning experiment

**Throwaway, loopback-only technical prototype. Not production implementation, not an approved API, no production deployment or PR.** The selected C2 UI and Rust domain library are unchanged. This experiment resolves the native-service evidence question in [Demonstrate native service integration and search-session control](https://github.com/tizian-bitschi/skating-program-lab/issues/153). The user favored a service and explicitly reaffirmed the planning-only boundary; no WASM comparison was attempted.

## Finding

The existing Rust library can serve real selection, manual construction assessment and incremental optimum search over HTTP while the browser and control plane remain usable. A session's registry and non-Send solver can be constructed, retained and dropped on the same owner thread. Independent request threads send plain commands and update atomic control flags; they never move or share the solver itself.

This supports the preferred native-service direction. It does not finish the runtime/API decision, establish production capacity or certify browser/phone performance.

## Experiment shape

- A separate binary depends on the existing Rust crate; the public library is not modified.
- Individual owner threads hold immutable requests and `PlanningSearchSession`. HTTP threads exchange commands and JSON snapshots only.
- Existing public APIs perform repertoire selection, personal-limit filtering, manual assessment and optimizer proof/collection. No scoring or skating rule engine is reproduced in the server.
- `POST /api/jobs` creates an unproved job; explicit prove/collect calls run asynchronously. Pause/cancel can signal a synchronous core call independently of its owner command queue. Proof can be cancelled; resumable pause belongs to collection only.
- Whole required IDs, exact exclusions, jump/spin/step maxima and rotation preference are translated into the existing types. The sample transport uses built-in profile keys; it is not the proposed final versioned public schema.
- Loopback binding only. Two registered jobs, batch sizes 1–100, explicit disposal and idle expiry keep the test small. There is no overall alternative-count cap.

The HTML page is an engineering harness, not a new product design or an integration of real computation into C2.

## Reproduce

From the experiment directory, in PowerShell:

```powershell
$env:SKATING_PROOF_IDLE_SECONDS='4'
cargo run --offline --release --manifest-path Cargo.toml
```

In another terminal in this directory:

```powershell
python verify.py
python verify_edges.py
```

The expiry check expects the four-second test setting. The binary's normal default is 120 seconds. Open `http://127.0.0.1:8767/` for the engineering harness. No framework installation or production configuration is required. The binary and dependency graph are pinned by the nested Cargo.lock; build provenance is in [environment.json](evidence/environment.json).

## Functional evidence

| Boundary | Observed result |
| --- | --- |
| Public individual selection | Real lists for all six built-in profiles; independent Camel/Change Camel maxima and exclusions verified. |
| Manual construction assessment | Six independently entered category fixtures return Compliant with numerical BV. Prohibited Bronze 2A returns Noncompliant. Personal spin conflicts preserve BV and rule verdict. |
| Complete finite optimization | Independently enumerated atomic fixture proves 1070 hundredths (10.70 BV) and delivers exactly all 16 identities, with required IDs intact and no duplicates. Every delivery is freshly reassessed via HTTP. |
| Combined requirements/restrictions | Jump/spin/step maxima and exact 1F+1F exclusion retain maximum 1070 and exactly the independently expected 14 remaining identities. |
| Invalid/infeasible/cancelled distinction | Unknown profiles and malformed field types reject; required/excluded conflict becomes Failed; removing all mandatory Masters Axels becomes Infeasible without a maximum; cancellation remains distinct. |
| Pause/resume during a call | A test barrier parks an actual collection control callback. While busy, HTTP profile requests and another owner's proof still succeed. Pause acknowledges Paused; the same session resumes to the exact original 16-identity set. |
| Cancellation | In-call collection cancellation retains proved maximum and no unconfirmed delivery. A separate Senior proof cancellation exposes no alternatives; terminal collection requests reject. |
| Lifecycle | Disposal invalidates the old job ID; idle expiry becomes Expired and refuses further work. Separate job identities remain isolated within one server lifetime. |

Final `verify.py` run passed with **134 HTTP observations**; expected 400/404/409 responses are assertions, not failed tests. The edge runner also passed. Raw observations: [main run](observations.json), [edge cases](evidence/edge-observations.json).

The independent oracle uses the existing source-derived Bronze fixture: fixed CCSp1 + CLSp1 + ChSq1 value 830, singles T/S=40 and F/Lo=50, stock at most two of each, at most four whole jump entries and one ordered pair. It independently enumerates maximum 240 in jumps and 16 whole-element multisets. It does not learn expected values from service output.

## Browser observation

In connected desktop Chrome, the harness proved 1070. During an actual collection call held at the marked test barrier, the text field accepted “Bedienbar während des Rust-Aufrufs”. Pause transitioned to Paused. Six subsequent batches delivered 3/6/9/12/15/16 cumulative results and Exhausted. The session was then disposed. A real manual assessment of `1Lz,SSp1,ChSq1` returned Compliant and BV 570.

- [Paused browser screenshot](evidence/browser-paused.jpg)
- [Exhausted browser screenshot](evidence/browser-exhausted.jpg)
- [DOM-observed browser result](evidence/browser-result.json)

Frame counters advanced, but the automation/backgrounding environment also reported gaps around one second. This is qualitative evidence of continued interaction, **not** an INP, frame-rate, responsiveness-budget or physical-phone acceptance result. Browser evidence was captured on the functional build; the HTTP checks were replayed after final formatting and rebuild.

## Bounded native measurements

Windows x64, Rust/Cargo 1.99.1, optimized Release build, local loopback. Host load was uncontrolled. These are observations from a functional run, not a repeated benchmark or an SLA.

- First ungated core prove call: **8.013 ms**. This excludes HTTP parsing and registry setup; it includes the core's domain/model/proof/representative assessment.
- Ungated collection of all 16 through exhaustion: **6.0123 ms** summed across six core calls, separate from proof and HTTP/polling.
- Pause/cancel barrier-to-state-ack wall times: **1.097 / 0.9863 ms**. These measure an injected synchronization case, not worst-case solver interruption latency. Gate waits are excluded from the ungated figures.
- Mixed-fixture successful HTTP medians: profiles **8.403 ms**, individual selection **15.444 ms**, manual assessment **5.102 ms**. Workloads differ; these aggregates are not comparable product targets.
- Largest individual-selection response in this run: **881 bytes**, codes plus profile/table metadata only. This is not the complete compound repertoire or final localized catalog payload.
- Observed service process peak working set: **8,339,456 bytes** for these bounded cases. It is not an unrestricted enumeration memory bound or a leak proof.

Derived summaries and resource evidence: [summary.json](evidence/summary.json), [native-process.json](evidence/native-process.json). Binary size/hash and dependency-lock hash are recorded in environment.json.

## Limits carried into the next planning decision

1. Final API/DTOs, typed/localizable diagnostics, public transport framework, opaque session identity, instance/restart behavior and request-revision handling still need a concrete contract. The handwritten HTTP parser and convenience defaults must not become production validation.
2. The capacity bound is for registered jobs. Disposal removes a lookup immediately; a busy owner winds down cooperatively afterward. Production must count live owners until they actually exit. Idle release drops solver ownership; last-view snapshots can remain until disposal or the next create cleanup.
3. GET currently retransmits all retained results, while the core itself retains confirmed compositions and count vectors. Large result sets need a cursor/delta presentation contract and resource measurements. No unrestricted Senior exhaustion or memory ceiling was established.
4. Browser-local persistence and recovery after service restart are not implemented. Expiry/disposal within one server lifetime was checked; durable solver recovery and stale IDs across server lifetimes were not.
5. No full C2 service adapter, physical-phone measurement, remote network/load test, participant test or full CI suite was run. The Rust core is unchanged and existing independent library evidence remains separate.
6. WASM remains unverified. This experiment establishes one viable preferred direction; it does not establish that browser-local execution is impossible or inferior.

Build, rustfmt check, Release Clippy with warnings denied, black-box verification and whitespace checks passed. All new code is under this experiments directory. The next artifact is a reviewed service/session and browser-data contract for the implementation specification, not a production merge of this experiment.
