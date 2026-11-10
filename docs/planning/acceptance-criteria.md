# Accepted UI acceptance criteria

**Accepted for planning on 2026-11-09, with the user’s explicit Q1/Q4 simplifications. No implementation, new prototype or acceptance run was performed by this document.**

Decision: [Set measurable UI performance and usability acceptance criteria](https://github.com/tizian-bitschi/skating-program-lab/issues/152). Scope: the selected C2 UI, [accepted service contract](service-contract.md), [local workspace/recovery policy](https://github.com/tizian-bitschi/skating-program-lab/issues/149#issuecomment-6750385948), and [language contract](language-contract.md).

The [technical source notes](acceptance-technical-notes.md) distinguish standards, external guidance and the proposed product targets below. The old native service and prototype observations remain feasibility evidence, not acceptance of the integrated application.

## 1. Browser and viewport scope

- Desktop: current stable Chrome, Edge and Firefox at acceptance time. Record actual browser/OS versions.
- Phone targets: Android Chrome and iPhone Safari layouts. Browser-engine, viewport and touch emulation are sufficient for planned acceptance. Real smartphone testing is optional, not a required gate. Record the actual browser/engine and emulation profile; WebKit emulation is not a claim that real iPhone Safari was tested.
- Default-zoom desktop at or above 1280×720 CSS pixels: category/language, complete repertoire controls, program, primary actions and result area fit without document scrolling. Program/result content may scroll inside its panel when necessary.
- Normal phone layout from 360 CSS pixels wide: vertical scrolling is allowed, horizontal page scrolling and clipped essential controls are not.
- Accessible reflow/zoom remains required at an equivalent 320 CSS-pixel width. It may introduce vertical document scrolling instead of enforcing the desktop fixed-height console. Test 200% text/zoom use and 320px reflow; preserve information and functionality.
- All three language layouts receive automated/screenshot checks, including long German names, Cyrillic, state/error messages and language switching.

Record actual tested browsers and emulated viewports. Optional real-device feedback can supplement these checks, but its absence does not block acceptance. Do not describe emulation as a physical-device result or infer certification of other browsers/older versions.

## 2. Accepted timing targets

Use the built production-mode application and a running reference service, not the development server or the old prototype. Record service host characteristics, build, browser/engine, viewport/emulation, reference host, any CPU throttling, cache state, network and workloads. Repeatable network profile: 4 Mbit/s down, 1 Mbit/s up and 100ms round-trip latency. These are test conditions, not a claim about a mobile network generation.

| Target | Start and end |
| --- | --- |
| Usable workspace in at most 3 seconds | Cold navigation to category/repertoire/program controls usable with the correct planning context. Painting an empty shell is insufficient. |
| Visible local feedback in at most 200ms | Tap/key/committed action to the next visible control/readout or requested-state feedback. Includes sliders, search acknowledgement and pause/cancel request feedback. |
| Ordinary BV/rule assessment within 1 second | Committed valid edit to the matching current-revision assessment visible, including debounce, ordinary request time and rendering under the reference profile. |
| New result batch usable within 200ms of receipt | Completed response received to appended, selectable result rows and updated count. Measure separately from result generation/network transfer. |
| State update within 2 seconds of service acknowledgement | A confirmed service transition becomes visible under the normal online polling profile. This is not a core pause/cancel interruption guarantee. |

Run three controlled cold-start repetitions per reference browser configuration and repeated representative interactions. Targets must pass in those recorded valid runs; preserve failed/outlier observations and rerun with an explanation after fixing or identifying an invalid test environment. Do not call a three-run statistic field p75, INP or population coverage.

Capture maximum-proof duration, first confirmed result, collection time, pause/cancel confirmation time and exhaustion separately for the six profiles and bounded fixtures. No arbitrary total optimum/enumeration deadline or total result cap is added here. Do not claim that immediate feedback proves quick optimization, or turn a test timeout into infeasibility, optimality or exhaustion. A bounded control test that hangs is a failure to investigate, not a successful cancellation.

LCP/INP/CLS can be recorded as diagnostics using their proper definitions; these application targets are not identical to Core Web Vitals. No field-performance conformance claim follows from the lab runs.

## 3. Functional and practical accessibility gates

Test behavior at the browser and public HTTP contract boundaries, reusing independent source-derived library/finite-oracle fixtures where appropriate. Internal component layout, helper calls and private solver branching are not acceptance expectations.

### Functional essentials — all must pass

- All four user workflows through the integrated UI/service: unrestricted optimum, exact required whole elements, combined repertoire restrictions, and independent manual program assessment.
- Exercise all six built-in profiles, plus an additional registered test policy without UI logic keyed to its name. Unknown/unsupported profiles or mechanisms fail explicitly and are not advertised as fully supported.
- The service supplies the real repertoire, including twenty regular-spin forms, their category availability, and other supported planned intentions. No hand-picked prototype shortlist or missing Camel/Change Camel/Layback mapping remains.
- Per-form maxima include every lower eligible level; Off differs from Basic. V choices share their form maximum. Required multiplicity, whole compound identity and freely chosen placement survive serialization and adoption.
- Exact connection exclusion does not remove its components elsewhere. Catalog text/code search and display-only filters do not change planning restrictions.
- Real credited BV and category findings agree with the shared Rust assessment. Personal conflicts are separate from category violations. Unknown/incomplete input never produces an invented positive complete verdict; actual execution and duration are not certified.
- An independent bounded oracle verifies the complete equal-optimum identity set, more than five results, no duplicates, provenance and fresh manual reassessment. Interrupted searches do not label unproved candidates optimal or claim complete enumeration.
- Confirmed result IDs/cursors are stable across retries/reconnect; source completion and data still being delivered are not confused. Result transfer uses bounded deltas. Delayed replies for old revisions/searches cannot overwrite current data.
- Pause request/acknowledgement, same-session continuation, cancel, changed-input cancellation, expiry, disposal, unavailable service and temporary overload have truthful distinct UI behavior. Retain confirmed results as agreed.
- Automatic local workspace save/reload, adopted program content, language priority/fallback, manual language change, and no automatic optimization after reload match the accepted contracts. Search access tokens and unbounded result history are not written into the durable workspace.
- Language switching does not cancel/restart a search or alter canonical codes/values. Locale-specific numeric display does not change exact integer arithmetic.
- Malformed requests/storage, unsupported saved schema/context and access/quota failures are handled without silent category fallback, false save success or destruction of the original saved snapshot.
- Search access is isolated between callers. Capacity counts live owners until actual shutdown. Repeated create/cancel/dispose/expiry cycles do not accumulate orphan solver owners or result stores no longer needed for display/delivery.

### Large-list and resource observation

Use a clearly marked synthetic UI/HTTP fixture with at least 10,000 result records to exercise batch append, scrolling, selection and the 200ms presentation/feedback target. This is a test quantity, not a product maximum or proof that 10,000 real optima were computed. Keep DOM rendering bounded through a suitable list strategy. Preserve keyboard focus and reading position during append.

Report retained data and memory growth versus result count, and after repeated disposal/reset cycles. Allocator/process high-water memory alone is not proof of a leak. A fixed universal MB limit is not invented without a deployment baseline. Unrestricted complete enumeration remains subject to honest runtime/resource reporting, never silent truncation or a fabricated completed state.

### Practical accessibility — all listed checks must pass

- Finish the core flows by keyboard, without traps; keep focus order/indicator visible and unobscured. Dialogs close and restore focus correctly.
- Named controls and slider values make sense with at least one real screen reader/browser combination. Announce concise state/result updates without moving focus or reading every new row.
- Touch operations do not require hover or dragging alone. Choose an effective target of about 44×44 CSS pixels for essential phone controls; keyboard and point/tap alternatives remain available.
- Check normal-text contrast at 4.5:1, applicable large-text/control contrast at 3:1, and communicate errors/states through more than color. Apply the actual criterion and its exceptions rather than asserting every pixel has one universal ratio.
- Layout remains operable under the reflow/zoom conditions above. Do not disable zoom to make the console fit.

These are scoped practical gates informed by WCAG 2.2 A/AA. They do not on their own authorize a blanket AA conformance claim; that requires a complete applicable-page/process evaluation.

## 4. Perspective-based understandability review

The user explicitly replaced the planned participant evaluation with perspective-based review. The agent and user walk through the interface from the viewpoint of a twelve-year-old skater and an adult skater. Basic skating knowledge is reasonable; familiarity with this UI or memorized ISU codes must not be assumed. No recruitment, scheduled interviews or minimum number of real participants is required.

Use the same five tasks, expressed as goals rather than click instructions:

1. Choose a category and obtain a confirmed optimal program from an initially unrestricted request; explain what the shown maximum means.
2. Preserve an exact whole element/combination, restrict repertoire through the mixer, and request the best remaining content. Identify retained versus added content.
3. Exclude a specified whole connection while keeping its individual jumps available; identify and recover from a conflicting retained element/restriction.
4. Enter/change a manual plan, recognize a concrete category finding, and correct the responsible input. Distinguish rule findings from personal constraints.
5. Pause/continue collection, adopt a program, reload and recognize the restored workspace/new-search behavior. Explain the difference between a proved maximum and all alternatives being found.

Use these tasks as a structured heuristic walkthrough. Identify unclear actions, redundant text, missing names, misleading states and difficult recovery; correct blocking issues and repeat the affected walkthrough. Do not add long explanations merely to compensate for ambiguous controls.

Record the perspectives/scenarios considered, observations and unresolved assumptions. This is heuristic review, not an observed child/adult test: do not invent participant actions, quotations or completion rates, and do not claim that actual twelve-year-olds have been tested. The separately agreed language workflow still applies.

The user may occasionally ask other people for feedback and feed useful findings into corrections. This is optional and informal; it is not a separately scheduled work package or release gate.

## 5. Acceptance outcome

Keep separate statuses for technical behavior, performance/accessibility checks, language review and heuristic understandability review. Overall acceptance uses the agreed browser/emulation and perspective-based methods. Physical-device and recruited-participant evidence are optional; their absence is not a blocker. Never relabel a heuristic review as empirical user evidence or an emulated viewport as a physical device. Required checks that were not performed remain pending rather than passed.

Wrong score/constraints, false positive compliance or optimum/exhaustion claims, lost saved plan, broken access isolation, unusable essential controls and unresolved blocking ambiguity identified in the perspective walkthrough block final acceptance. Native feasibility timings, a clean automated scan or attractive prototype screenshots do not replace the required checks.

## 6. Acceptance record

The user accepted Q2/Q3 unchanged and Q1/Q4 with explicit simplifications on 2026-11-09:

1. Browser/viewport checks suffice; real smartphones are not mandatory.
2. The 3s usable-start, 200ms feedback/presentation and 1s ordinary-assessment targets and separate solver-time measurement are accepted.
3. Functional testing at browser/HTTP boundaries and the practical accessibility checklist are accepted. These are also the agreed high-level test seams for specification/implementation planning.
4. Use child/adult perspective-taking; no explicit participant-testing plan is required. The user will seek occasional informal feedback at their discretion.

No additional unticketed planning decision is identified by these choices. These criteria define future acceptance; they do not assert that the not-yet-implemented product has passed.
