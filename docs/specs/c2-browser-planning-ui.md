# C2 Mixer browser UI with native Rust planning service

## Problem Statement

Skaters need one understandable browser workspace to find the best technical program for a category, retain selected elements, restrict their repertoire and check a manually prepared program. The existing Rust library supplies the domain behavior, but there is no production browser/service application. The C2 prototype established the interaction direction; its illustrative data and simulated calculations do not satisfy the product workflows.

The UI must work on desktop and smartphone layouts, remain responsive during long searches, and make its actions understandable from twelve-year-old and adult skater perspectives. It must support German, English and Russian without requiring an account.

## Solution

Build the selected compact C2 Mixer as a Vue 3/TypeScript/Vite browser application backed by one Axum/Tokio Rust service. Use the existing library as the authority for repertoire, construction rules, credited base value and optimum search. Automatically save one browser-local workspace. Provide asynchronous proof and automatic incremental collection of every equal optimum, with truthful pause, cancellation, expiry and completion behavior.

This issue is the implementation specification produced from the accepted planning decisions. Product implementation and implementation-ticket decomposition are subsequent explicitly initiated phases. The `ready-for-agent` label identifies specified work; it does not override the user's current planning-only instruction or authorize starting product code automatically.

## User Stories

1. As a skater, I want one program-centered workspace, so that planning and checking do not require separate starting journeys.
2. As a skater, I want to choose a supported category and rule edition, so that my plan uses the intended rules.
3. As a skater, I want unrestricted optimization, so that I can explore the category's complete permitted planning repertoire.
4. As a skater, I want the highest theoretical credited base value, so that the objective is precise and comparable.
5. As a skater, I want every equally optimal composition progressively, so that an arbitrary result limit does not hide alternatives.
6. As a skater, I want whole-element reorderings deduplicated while meaningful internal compound order remains distinct, so that the list contains genuine alternatives.
7. As a skater, I want to retain an exact solo jump, so that it remains a solo in the result.
8. As a skater, I want to retain an exact combination or sequence, so that its contents remain intact.
9. As a skater, I want repeated required occurrences preserved separately and placed freely, so that multiplicity does not accidentally fix program positions.
10. As a skater, I want named mixer controls for repertoire limits, so that I can adjust them directly without typing technical constraints.
11. As a skater, I want a maximum listed class per jump kind, so that lower eligible classes remain available in solos and compounds.
12. As a skater, I want a maximum level per exact spin form and for steps, so that Basic and lower eligible levels remain included.
13. As a skater, I want Off distinct from Basic or Single, so that excluding an element type is unambiguous.
14. As a skater, I want all twenty regular-spin forms represented, including Camel, Change Camel and Layback, so that the UI does not omit supported choices.
15. As a skater, I want category-unavailable forms visibly distinguished, so that personal settings never appear to override the rules.
16. As a skater, I want unlevelled choreographic content controlled without invented levels, so that controls match the elements.
17. As a skater, I want to assemble an unwanted connection by clicking jump choices, so that I can exclude it without memorizing a code.
18. As a skater, I want exact connection exclusions to leave their components available elsewhere, so that a specific dislike does not remove my whole repertoire.
19. As a skater, I want to restore an excluded connection directly, so that restrictions are easy to undo.
20. As a skater, I want to search element names or canonical codes without changing my restrictions, so that finding content and excluding it stay separate.
21. As a skater, I want requirements and restrictions combined, so that one request captures my intended program.
22. As a skater, I want conflicts localized to the responsible entries/settings, so that I can correct them without silently relaxed constraints.
23. As a skater, I want an explicit infeasible outcome distinct from errors and cancellation, so that an impossible request is not confused with a failed service.
24. As a skater, I want a complete entered plan assessed independently of optimization, so that I can check my own composition.
25. As a skater, I want ordinary BV and rule assessment refreshed after committed changes, so that the current plan stays understandable without repeatedly pressing Check.
26. As a skater, I want personal conflicts separated from category violations and scoring, so that my preferences do not redefine the rules.
27. As a skater, I want short translated findings with original rule sources available, so that I can understand and verify a problem.
28. As a skater, I want a maximum labeled proved only after proof, so that a provisional candidate is never presented as optimal.
29. As a skater, I want confirmed results visible while more are collected, so that I can inspect or adopt them before enumeration ends.
30. As a skater, I want to pause and continue alternative collection, so that I retain progress in the same live session.
31. As a skater, I want to cancel a search while keeping confirmed information visible, so that I control ongoing work.
32. As a skater, I want input changes to stop the old optimization and mark its results out of date, so that they cannot masquerade as current recommendations.
33. As a skater, I want short connection failures to reattach to the same live search, so that I do not lose progress unnecessarily.
34. As a skater, I want an expired session to offer a new calculation while preserving displayed results, so that recovery is explicit.
35. As a skater, I want my category, plan, restrictions, exclusions and language saved automatically in this browser, so that I need no account or Save button.
36. As a skater, I want reload to restore my workspace without automatically restarting optimization, so that I choose when computation runs.
37. As a skater, I want an adopted alternative saved as program content, so that my chosen composition survives reload.
38. As a skater, I want German, English and Russian names and controls with unchanged international codes, so that language choice does not alter my plan.
39. As a skater, I want my saved language preferred, then my supported browser language, then English, so that the initial interface is predictable.
40. As a skater, I want language and presentation changes to preserve the active search, so that translation is not treated as a new optimization request.
41. As a skater, I want the desktop tool to fit the browser viewport and phone controls to reflow, so that I can use the same workspace on different screens.
42. As a skater, I want keyboard, touch, focus and accessible control names to work throughout the core flows, so that precise dragging or memorized abbreviations are not required.
43. As a skater, I want usable startup and immediate control feedback, so that long server work does not freeze the interface.
44. As a skater, I want honest save, capacity, request and service errors without losing my entered plan, so that failures remain recoverable.

## Implementation Decisions

- **Selected architecture:** one same-origin static browser application and native Rust HTTP service using the accepted Vue/TypeScript/Vite and Axum/Tokio basis. Package versions and hosting provider are not selected here. Preserve current library callers; the experimental handwritten transport is not production code to promote.
- **Selected UI:** C2 Mixer only, with one navbar containing category/season and language, compact full-width controls and program/results panels. Do not restore discarded A/B/C variants, a comparison toolbar, the duplicate main page heading or persistent prototype banner. Use the captured prototype as a primary interaction reference, then implement the behavior on the chosen stack.
- **Complete repertoire:** the service supplies modeled forms, category availability, eligible levels/classes, exact intent identities, values and source metadata from the Rust core. Do not ship a hand-picked sample list, category-name conditionals or a second rule engine in the UI. Include supported category-specific planned intentions and all regular-spin distinctions. New categories are advertised only after all four workflows are supported and verified.
- **Objective and identity:** maximize theoretical credited program BV, including program-context credit adjustments; do not sum standalone catalog values in the client. Preserve the core's composition identity, required multiplicity/provenance, ignored required movements and counted zero-value distinctions. Whole-element reordering alone does not add an alternative.
- **Retained elements:** stable workspace IDs identify exact whole required occurrences. Only retained content becomes a search requirement. Do not accidentally turn an entire editable/adopted program into required content or substitute ordered immutable completion for free-placement search.
- **Personal controls:** Off is separate from Basic/Single. Optional maxima intersect category rules and include lower eligible choices. Every exact spin abbreviation has its own maximum; V variants share it. Explicit user maxima survive category changes. No arbitrary lower-level spin holes are introduced as a separate product control.
- **Connections:** use backend-supplied prefix choices and exact whole keys for combination/sequence selection and exclusion. An optional third jump is not mandatory. Existing exclusions remain visible/restorable, and exact two-jump exclusion does not silently exclude longer wholes sharing a prefix.
- **Versioned boundary:** use the accepted versioned HTTP/JSON contract for profiles, repertoire, connection options, independent program assessment, search creation, delta updates and pause/continue/cancel/disposal. Every operation uses explicit profile/rule/table references; unknown context never falls back silently.
- **State and provenance:** correlate replies by request/revision/search identity. Use immutable optimization inputs, stable result IDs and cursor deltas; retries are idempotent and do not duplicate work/results. Changed calculation inputs cancel/invalidate the previous search. Presentation/language and freely placed whole-entry ordering do not create a new optimization problem; ordered manual assessment can still refresh where its inputs change.
- **Ordinary assessment:** automatically assess the current described program after committed edits/restoration, with a short debounce where appropriate. Keep pointer-drag preview local. Fully described supported plans receive the shared binary construction verdict; missing/unsupported input is not fabricated compliance. Keep personal conflicts separate from category findings and preserve exact credited values.
- **Asynchronous search:** start optimization only on explicit user action. Publish no unproved candidate as optimal. After proof, collect automatically in internal chunks and deliver bounded result deltas until pause/cancel/exhaustion. Batch fullness and an unknown total never imply completion; there is no overall result-count cap or arbitrary total solver deadline.
- **Ownership and control:** construct the registry and non-Send session on one dedicated native owner thread. HTTP handlers share only safe handles/control flags/DTOs. Cancellation, pause and lease expiry reach busy owners independently of their work queue. Count live owners until actual termination, not merely removal from a lookup map.
- **Search lifecycle:** proof supports cancellation; resumable pause applies to collection. Distinguish requested control from confirmed state, mathematical infeasibility from input/internal failure, and enumeration completion from expiry/cancellation. Preserve a valid already-proved maximum across later cancellation where the core establishes it.
- **Lease and access:** expire after 15 minutes without valid browser contact; valid polling/paused heartbeat renews the lease. This is not a 15-minute computation limit. Use unpredictable instance/restart-safe identity and page-held access/retry capabilities. Never expose search secrets in links, durable workspace data or ordinary logs. A stale identity cannot attach to another job.
- **Capacity and failures:** global temporary overload has an explicit service-unavailable/retry response, not an implicit admission queue. A separate client rate policy may use its appropriate response. Typed errors carry translation keys, field/entry/rule/source context and current correlation, not raw Rust Debug text. Internal reconstruction/assessment mismatch is an error, never grounds to silently drop an inconvenient candidate.
- **Local data:** automatically store one small versioned localStorage workspace containing context, program/retention flags, personal limits, excluded connections and locale. An adopted alternative is program content. Do not durably store live sessions/tokens or the potentially huge result history. Handle inaccessible/quota/corrupt/unsupported storage honestly without silently destroying the original saved data.
- **Reload and disconnection:** reload restores the workspace and ordinary assessment, with a fresh optimization only on user action. A temporary network loss while the page remains open reattaches to the same available session and deduplicates deltas. Previously paused work stays paused. If unavailable, retain already displayed data and offer a new calculation; do not silently recompute.
- **Languages:** support saved locale first, supported browser locale second and English fallback; retain a manual switch. Translate names/UI/findings and locale-specific numeric display, preserving canonical element codes, exact arithmetic and original source editions. The agent prepares three catalogs; the user performs or organizes content feedback against screens, with corrections applied directly rather than through a separate editorial platform.
- **Acceptance methods:** use the approved browser/viewport and perspective-based methods. Real smartphones and planned recruited-participant studies are optional. Evaluate understandability through the five task scenarios from twelve-year-old/adult perspectives; informal feedback may inform revisions. Do not label heuristic review as empirical user evidence or browser emulation as a real-device measurement.
- **Scope of current work:** this specification and its referenced contracts are planning artifacts. No production implementation, CI/security change, public deployment or merge is carried out by publishing them. Subsequent implementation must follow the approved behavior and repository change/verification rules.

## Testing Decisions

- The user approved browser and public HTTP behavior as the high-level acceptance seams in Q3. Test observable inputs, values, identities, findings, state/control and persistence. Reuse existing public Rust selection/manual-assessment/search-session behavior and independent fixtures as dependency oracles; do not assert private helpers, component arrangement or solver branching.
- Cover all six initial profiles and at least one independently defined registered test policy through the same UI/service pathways. Verify unsupported mechanisms and unavailable versions explicitly, without built-in-name fallbacks.
- Cover unrestricted, required-only, restricted-only and combined optimization, plus independent manual assessment. Verify exact solos/compounds, required duplicates, every supported control type, downward-inclusive maxima, complete spin forms and current category availability. Check Camel CSp, Change Camel CCSp, Layback LSp and Change Layback CLSp names/identities.
- Reuse independent finite optimality/enumeration oracle practice: the known constrained 10.70 BV/16-composition case and its 14-composition exact-exclusion variant are prior integration evidence. Compare complete identity sets, requirements, credited BV and fresh manual reassessment rather than captured solver output. Keep synthetic presentation fixtures separate from domain correctness fixtures.
- Exercise truthful invalid/infeasible/unsupported/cancelled/expired/completed results, in-call control, pause/resume, idempotent retries, cursor reconciliation, cross-search access isolation and stale replies. No unproved maximum or silently truncated complete result set may pass.
- Check real live-owner shutdown/capacity accounting and repeated create/cancel/dispose/expiry cycles. Use controlled time/configuration for lease tests without changing the accepted product policy. Report retained data/memory growth; do not invent a constant memory bound for unbounded enumeration.
- Use a clearly marked 10,000-record synthetic UI/HTTP fixture for append, scrolling and selection. This is a workload, not a maximum product result count or proof that those records are real optima. Keep rendering bounded and preserve focus/reading position.
- Verify automatic workspace save/restore and adoption, missing/bad schema/context, storage failures, independent locale formatting, language changes during active work and no automatic optimization on reload. Check that search secrets/result history are absent from durable workspace data.
- Browser scope: current stable desktop Chrome/Edge/Firefox and suitable Android-Chrome/iPhone-Safari engine/viewport proxies, with exact versions/configuration recorded. Physical-phone tests are not required. At 100% zoom and at least 1280×720 CSS pixels, all main desktop tool regions/actions fit without document scrolling; long inner lists may scroll. Phone layout from 360px supports vertical reflow without horizontal overflow; zoom and equivalent 320px reflow preserve operation.
- Accepted application budgets under the recorded reference-browser setup and 4 Mbit/s down, 1 Mbit/s up, 100ms RTT: usable cold startup at most 3s; visible local feedback and received-batch presentation at most 200ms; ordinary assessment at most 1s; confirmed state reflected within 2s of service acknowledgment. Run three controlled cold-start repetitions and representative repeated interactions, preserving failures/outliers. These are product/lab targets, not field p75/INP evidence or total solver-time limits.
- Practical accessibility checks include complete keyboard flows, logical visible/unobscured focus, dialog restoration, named controls/slider values with at least one real screen-reader/browser combination, concise status announcements, touch alternatives to hover/dragging, suitable effective phone targets, contrast and reflow. A technical screen-reader check does not require recruited participants. Do not claim blanket WCAG AA conformance from a subset of checks.
- Perform heuristic walkthroughs of five goals: unrestricted optimum; whole-element retention plus limits; exact connection exclusion/conflict recovery; manual plan/rule correction; pause/continue/adopt/reload with proof-versus-exhaustion interpretation. Use child/adult perspectives, record actual observations/assumptions and fix blocking ambiguities. No recruitment, minimum participant count or formal user-study schedule is a required gate; owner feedback is optional and informal.
- Record which technical, language and heuristic checks actually ran. Optional physical/participant checks do not block acceptance; required unperformed checks remain pending. Wrong scores/constraints, false compliance/proof/completion, lost saved plan, broken access isolation, unusable essential controls and unresolved blocking walkthrough issues prevent final acceptance. No evidence is fabricated.
- Run the relevant application build/type-check/test/format/static-analysis checks for the implementation, and the repository-prescribed checks when affected. The old prototype and bounded native service are references, not proof that the integrated product already passes these criteria.

## Out of Scope

- Product implementation or implementation-ticket decomposition during this specification publication; automatic work initiation from an issue label.
- A/B/C alternatives or renewed layout exploration; direct promotion of throwaway UI/transport code.
- Accounts, mandatory sign-in, cloud program storage, cross-device synchronization, saved-program collections, export/sharing features not separately agreed.
- GOE/PCS/expected-success optimization, duration validation, actual execution verification or official judging claims.
- New real disciplines/seasons/categories beyond the initial supported six without separate mechanisms/sources and four-workflow acceptance; arbitrary new rules by metadata alone.
- Durable solver checkpoints or restoration of a lost service process; full offline optimization.
- Browser/WASM solver execution, multi-service/distributed-worker architecture, deployment-provider selection and production deployment execution.
- Mandatory real-phone testing, recruited-participant studies, editorial infrastructure, blanket accessibility certification or unsubstantiated performance guarantees.

## Further Notes

The user approved C2, local workspace behavior, service/language contracts and acceptance methods through live discussion. Their latest instruction removes mandatory real-smartphone and planned participant tests; it supersedes earlier proposed methods. All work in this handoff is documentation/planning. Before implementation, refresh the actual target branch and reconcile current library behavior without overwriting concurrent work.

Primary decision sources:

- [Wayfinder decision map](https://github.com/tizian-bitschi/skating-program-lab/issues/145)
- [Selected C2 and interaction evidence](https://github.com/tizian-bitschi/skating-program-lab/issues/148)
- [Complete regular-spin correction and category-selection evidence](https://github.com/tizian-bitschi/skating-program-lab/issues/148#issuecomment-6748895480)
- [Native-service planning proof and its explicit limits](https://github.com/tizian-bitschi/skating-program-lab/issues/153)
- [Confirmed local workspace/recovery behavior](https://github.com/tizian-bitschi/skating-program-lab/issues/149#issuecomment-6750385948)
- [Accepted service contract](https://github.com/tizian-bitschi/skating-program-lab/blob/af0923d820217e324bb25986211110d4b33c0fdd/docs/planning/service-contract.md)
- [Accepted language contract](https://github.com/tizian-bitschi/skating-program-lab/blob/8e84b9d8faa148d82ffb39f059189c607a5d03b4/docs/planning/language-contract.md)
- [Accepted simplified acceptance criteria](https://github.com/tizian-bitschi/skating-program-lab/blob/51cdd83a245bc7c2e658179df74be805f93d8076/docs/planning/acceptance-criteria.md)

The library baseline originally assessed was `5bc3333`; later commits in the evidence branches add prototypes, experiments or planning documents. No production UI/service implementation is asserted by those artifacts. The next flow is an approved breakdown into small end-to-end implementation tickets; product code starts only when that subsequent phase is explicitly initiated.
