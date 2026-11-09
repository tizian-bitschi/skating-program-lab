# UI acceptance criteria proposal

**DRAFT for live user review. Planning only. No implementation, new prototype, performance run or participant test was performed by this document.**

Decision: [Set measurable UI performance and usability acceptance criteria](https://github.com/tizian-bitschi/skating-program-lab/issues/152). Scope: the selected C2 UI, [accepted service contract](service-contract.md), [local workspace/recovery policy](https://github.com/tizian-bitschi/skating-program-lab/issues/149#issuecomment-6750385948), and [language contract](language-contract.md).

The [technical source notes](acceptance-technical-notes.md) distinguish standards, external guidance and the proposed product targets below. The old native service and prototype observations remain feasibility evidence, not acceptance of the integrated application.

## 1. Proposed supported environments (Q1)

- Desktop: current stable Chrome, Edge and Firefox at acceptance time. Record actual browser/OS versions.
- Phone: current Android Chrome on one real middle-range Android phone, and Safari on one real iPhone. Record actual models, OS/browser versions and viewports. Browser emulation supplements, not replaces, these checks.
- Default-zoom desktop at or above 1280×720 CSS pixels: category/language, complete repertoire controls, program, primary actions and result area fit without document scrolling. Program/result content may scroll inside its panel when necessary.
- Normal phone layout from 360 CSS pixels wide: vertical scrolling is allowed, horizontal page scrolling and clipped essential controls are not.
- Accessible reflow/zoom remains required at an equivalent 320 CSS-pixel width. It may introduce vertical document scrolling instead of enforcing the desktop fixed-height console. Test 200% text/zoom use and 320px reflow; preserve information and functionality.
- All three language layouts receive automated/screenshot checks, including long German names, Cyrillic, state/error messages and language switching.

The user supplies or organizes access to real reference devices when not available in the development environment. Missing environments are reported as pending, not silently counted as passed. Other browsers/older versions are not presumed certified by these samples.

## 2. Proposed timing targets (Q2)

Use the built production-mode application and a running reference service, not the development server or the old prototype. Record service host characteristics, build, browser/device, cache state, network and workloads. Proposed repeatable network profile: 4 Mbit/s down, 1 Mbit/s up and 100ms round-trip latency. These are test conditions, not a claim about a mobile network generation.

| Target | Start and end |
| --- | --- |
| Usable workspace in at most 3 seconds | Cold navigation to category/repertoire/program controls usable with the correct planning context. Painting an empty shell is insufficient. |
| Visible local feedback in at most 200ms | Tap/key/committed action to the next visible control/readout or requested-state feedback. Includes sliders, search acknowledgement and pause/cancel request feedback. |
| Ordinary BV/rule assessment within 1 second | Committed valid edit to the matching current-revision assessment visible, including debounce, ordinary request time and rendering under the reference profile. |
| New result batch usable within 200ms of receipt | Completed response received to appended, selectable result rows and updated count. Measure separately from result generation/network transfer. |
| State update within 2 seconds of service acknowledgement | A confirmed service transition becomes visible under the normal online polling profile. This is not a core pause/cancel interruption guarantee. |

Run three controlled cold-start repetitions per reference device class and repeated representative interactions. Targets must pass in those recorded valid runs; preserve failed/outlier observations and rerun with an explanation after fixing or identifying an invalid test environment. Do not call a three-run statistic field p75, INP or population coverage.

Capture maximum-proof duration, first confirmed result, collection time, pause/cancel confirmation time and exhaustion separately for the six profiles and bounded fixtures. No arbitrary total optimum/enumeration deadline or total result cap is added here. Do not claim that immediate feedback proves quick optimization, or turn a test timeout into infeasibility, optimality or exhaustion. A bounded control test that hangs is a failure to investigate, not a successful cancellation.

LCP/INP/CLS can be recorded as diagnostics using their proper definitions; these application targets are not identical to Core Web Vitals. No field-performance conformance claim follows from the lab runs.

## 3. Functional and practical accessibility gates (Q3)

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

## 4. Proposed small real-user check (Q4)

The user performs or organizes a short evaluation with at least one skater aged 12–17 and one adult skater who did not help develop the interface. Basic skating knowledge is appropriate; memorized ISU codes or knowledge of this UI is not assumed. The agent prepares the tasks and applies identified corrections.

Use the same five tasks, expressed as goals rather than click instructions:

1. Choose a category and obtain a confirmed optimal program from an initially unrestricted request; explain what the shown maximum means.
2. Preserve an exact whole element/combination, restrict repertoire through the mixer, and request the best remaining content. Identify retained versus added content.
3. Exclude a specified whole connection while keeping its individual jumps available; identify and recover from a conflicting retained element/restriction.
4. Enter/change a manual plan, recognize a concrete category finding, and correct the responsible input. Distinguish rule findings from personal constraints.
5. Pause/continue collection, adopt a program, reload and recognize the restored workspace/new-search behavior. Explain the difference between a proved maximum and all alternatives being found.

Pass this small check only when both observed participants finish the core tasks without step-by-step coaching and no critical misunderstanding remains. Self-correction is allowed. Record assistance, wrong turns and unresolved barriers; retest affected tasks after a correction. Do not treat successful reading of an explanation supplied by the evaluator as self-explanatory UI.

Record device/language coverage, and try the mixer, builder and key recovery controls on a phone. The domain-language review of all three catalogs remains the separately agreed responsibility. Two participants are formative/task-specific evidence, not population-wide proof across ages, languages or devices. Do not generalize German-only task observations into Russian/English comprehension evidence.

Use anonymous participant references/age bands in reports. No participant names or recordings are needed for this gate.

## 5. Acceptance outcome

Keep separate statuses for technical behavior, performance/accessibility checks, language review and observed usability. Overall final acceptance requires the agreed critical checks to pass. Missing real-device/participant/language evidence is **pending/unverified**, not passed. Engineering work can be ready for owner validation without being falsely described as fully accepted.

Wrong score/constraints, false positive compliance or optimum/exhaustion claims, lost saved plan, broken access isolation, unusable essential controls and unresolved critical participant confusion block final acceptance. Native feasibility timings, a clean automated scan or attractive prototype screenshots do not replace these checks.

## 6. Review frontier

- **Q1:** proposed browser/device/reflow scope and real reference-device checks?
- **Q2:** 3s usable start, 200ms local feedback/result presentation, 1s ordinary assessment, with the defined reference profile and separate solver timings?
- **Q3:** functional acceptance at UI/HTTP boundaries plus the listed practical keyboard/touch/focus/contrast/reflow/screen-reader checks?
- **Q4:** small two-age-group/five-task evaluation organized by the user, with actual evidence and missing coverage reported honestly?

All four are recommendations awaiting live confirmation. Nothing in this draft implements or executes the gates.
