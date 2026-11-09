# Browser performance and inclusive usability evidence (#147)

Research date: 2026-11-09. Scope: implementation-ready UI specification research, not production UI or measured product acceptance. No participants were recruited and no browser build was tested.

## Contract and evidence boundaries

The proposed product is one program workspace on PC and phone, without accounts, for people aged twelve and older, in German, English and Russian. Repository [requirements](requirements.md), [architecture](architecture.md) and [planning search](specs/planning-search.md) separate domain correctness from UI behavior. The current library does not establish browser/Wasm compatibility. Its session proves maximum BV before delivering confirmed alternatives, collects uncapped batches, retains work on cooperative pause, and establishes exhaustion separately. This report does not replace that contract or prescribe a hosting architecture.

Labels below distinguish **standard** (normative accessibility criteria), **guidance** (official recommendations), and **proposal** (project-specific measurement/evaluation design awaiting human acceptance). No numerical solver SLA follows from browser metrics.

## Performance measurements

**Guidance:** Google defines LCP for loading, INP for interaction responsiveness and CLS for layout stability. Its good thresholds are LCP <=2.5 seconds, INP <=200 milliseconds, CLS <=0.1, assessed at the 75th percentile separately for mobile and desktop. These are external guidance, not selected project gates. [Web Vitals](https://web.dev/articles/vitals)

INP covers click/tap/keyboard interactions through the next paint, including input delay, processing and presentation delay. It does not measure every later asynchronous consequence. A painted searching message can therefore coexist with lengthy proof. Scrolling is outside INP's interaction set. [INP definition](https://web.dev/articles/inp)

**Proposal:** record the following independently with request/session IDs. Use browser User Timing marks/measures for application intervals; the current W3C Level 3 document is a draft. Never compare raw timestamps from different clocks without an explicit alignment method. [User Timing](https://www.w3.org/TR/user-timing/)

| Measure | Start -> end and evidence | Interpretation |
| --- | --- | --- |
| Initial load | navigation -> LCP; separately navigation -> workspace controls usable | Cold and warm loads; LCP alone does not establish readiness |
| Filter response | committed input -> matching count and visible rows updated; trace next paint | Exercise code/text search, facets, clearing, zero matches and rapid edits |
| Search acknowledgement | start activation -> visible searching state | UI response, distinct from proof |
| Proof time | accepted immutable request -> maximum proven or infeasibility proven | Record solver-active and elapsed wall time separately, with pauses identified |
| First confirmed result | accepted request -> first validated alternative visible | Also record proof -> result receipt -> rendering; no premature optimal label |
| Batch presentation | batch received -> new count and usable rows displayed | Record batch size, accumulated results and DOM size; check focus and scrolling |
| Pause | activation -> visible requested state; separately -> solver acknowledgement | Requested is not paused; preserve confirmed results and pending work |
| Resume/cancel | activation -> state acknowledgement | Resume same request; terminal cancellation distinct from retained pause |
| Exhaustion | accepted request -> explicit exhausted event, then visible status | Separate from maximum proof and current collected count; interrupted run is censored, never exhausted |
| Memory | baseline, proof, successive batches, pause/resume, disposal | Separate retained result data, DOM and worker/Wasm memory where available |

**Guidance:** Chrome DevTools offers process memory/JavaScript memory observation, heap snapshots and allocation tools; memory limits depend on device capabilities. **Proposal:** compare retained growth across repeated identical sessions, batch counts and disposal, rather than inventing a universal MB ceiling. JavaScript heap alone does not account for all process or Wasm memory. [Chrome memory diagnostics](https://developer.chrome.com/docs/devtools/memory-problems)

**Proposal workload matrix:** record exact device, OS/browser version, RAM class, viewport, input method, network latency/bandwidth and cache state. Include a real constrained Android phone, iPhone/Safari and desktop browsers; use throttled lab runs for reproducibility and real devices for confirmation. Cross cold/warm load with slow/normal network; small/large repertoires; constrained/unrestricted requests; infeasible requests; few/many equal alternatives; early/late pause; repeated resume and disposal. Include all supported category/profile shapes and large compound-element filters. Retain fixture identity, build version, sample count, raw traces, median and tail distributions. A small lab sample does not establish field p75. Separate browser transport/render overhead from solver-active work. Treat backgrounded/suspended sessions separately. Human owners still select device coverage, repeat counts, regression tolerance and release thresholds.

## Accessibility criteria and practical checks

**Standard:** the following WCAG 2.2 criteria apply to these interactions. This is a scoped checklist, not a complete conformance audit. [WCAG 2.2](https://www.w3.org/TR/WCAG22/)

| Concern | Criteria | Proposed manual check |
| --- | --- | --- |
| Keyboard | 2.1.1, 2.1.2 | Complete filters, ordering, search, pause and result selection without pointer or trap |
| Focus | 2.4.3, 2.4.7, 2.4.11 | Logical order, visible indicator, no sticky panel obscuring focused controls |
| Pointer | 2.5.1, 2.5.2, 2.5.3, 2.5.7 | Single-pointer alternatives; cancellation; visible labels in accessible names; reorder buttons alongside dragging |
| Responsive reading | 1.3.1, 1.3.2, 1.3.4, 1.4.4, 1.4.10, 1.4.12 | Relationships and reading order survive narrow view, zoom, spacing and orientation |
| Visual distinctions | 1.4.1, 1.4.3, 1.4.11 | Status/violations understandable without color; text and control contrast checked |
| Errors and inputs | 3.3.1, 3.3.2, 3.3.3, 4.1.2 | Named controls; indexed error explanation and correction path; no lost values |
| Language | 3.1.1, 3.1.2 | Page and language changes exposed programmatically |

**Standard:** 2.5.8 has a 24-by-24 CSS-pixel minimum with defined exceptions including spacing; 2.5.5's enhanced target is 44-by-44 at AAA, also with exceptions. **Proposal:** inspect crowded filter chips and remove/reorder controls with real touch users before choosing a product target. [W3C target-size explanation](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)

**Standard:** 4.1.3 requires programmatically determinable status messages without moving focus. Search result rows themselves differ from a result-count/status announcement. **Proposal:** announce concise changes such as proof established, batch added, pause acknowledged and exhausted; preserve keyboard focus and reading position while appending. Test screen-reader output during rapid filter edits and large batches; avoid announcing every row or solver step. Unknown progress must not appear as a fabricated percentage. [W3C status messages](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html)

## Internationalization evaluation

**Guidance:** HTML language metadata supports speech synthesis, typography and language-sensitive processing. [W3C language attributes](https://www.w3.org/International/questions/qa-lang-why) W3C's authoring guidance covers Unicode, language declarations, text handling and localization concerns. [Internationalization techniques](https://www.w3.org/International/techniques/authoring-html)

**Proposal:** use UTF-8 and `lang=de`, `en` or `ru`; verify Cyrillic font coverage and German expansion in narrow controls. Keep canonical skating codes and exact arithmetic stable while localizing labels and presentation. Explicitly specify accepted numeric input and locale-aware display; test decimal separators and copy/paste without silent value changes. Test translated names and canonical codes in the repertoire filter, case handling, no-result recovery, diacritics and Cyrillic. Decide cross-language synonyms with domain translators; do not infer them from transliteration. Switch language mid-task and verify preservation of plan, filter selections, search state, errors and accessible names. Native-language domain reviewers must validate distinctions among personal restriction, invalid request, competition finding, unsupported evidence, proven maximum and exhaustive collection. Translation coverage is not comprehension evidence.

## Proposed task-based evaluation

Moderated usability testing observes participants performing concrete tasks; think-aloud helps explain their actions. [GOV.UK method](https://www.gov.uk/service-manual/user-research/using-moderated-usability-testing) W3C recommends combining user involvement with standards evaluation and warns against generalizing from a few participants. [WAI user evaluation](https://www.w3.org/WAI/test-evaluate/involving-users/)

**Proposal:** run short formative rounds against an interactive prototype, then the integrated browser build. Recruit twelve-year-old/teen and adult skaters with varied planning experience across all three languages, including disabled participants and their usual assistive technologies. Arrange appropriate minor assent/guardian consent and recording arrangements before recruitment; applicable local process remains a human decision. No universal participant count or percentage success gate is justified by these sources.

Use the four confirmed planning-search workflows from the current contract, with equivalent domain fixtures:

1. Unrestricted: select profile and obtain a confirmed maximum; explain what has been proven and what remains unknown.
2. Exact-required: find and add required whole elements, including multiplicity; confirm their preservation in results.
3. Restricted: configure personal limits and semantic/exact exclusions through the repertoire filter; explain a zero-match or conflict state and recover.
4. Combined: use requirements and restrictions together, inspect a conflicting fixture, correct it, start search, pause/resume, collect another batch and distinguish maximum proof from exhaustive completion.

Add a manual-plan comprehension probe in the same workspace: change an element and explain BV, a composition violation and an unsupported finding. Ask participants to choose an alternative and explain why equal BV does not establish equal execution quality or official judging. On phone, repeat high-risk filter and result tasks; on desktop include keyboard-only ordering and focused append operations. Test language switching without requiring participants to understand another language. Rotate task ordering to reduce learning effects and keep moderator assistance separate from independent completion.

Record task outcome (independent/assisted/failed), recoverable and critical errors, misinterpretations, assistance, time on task excluding separately logged solver wait, filter backtracking, accidental activations and participant explanations. Report observed counts and denominators by language, age group, device and access mode, with anonymized examples. Small rounds find problems; they do not prove population-wide usability. Retest changed flows and report remaining barriers. Automated checks, browser scores and successful participant tasks each contribute evidence; none establishes complete WCAG conformance alone.

## Decisions still owned by humans

Select supported devices/browsers, formal WCAG conformance scope/level, measurable UI budgets, batch presentation policy and memory budgets after baseline evidence. Agree recruitment coverage, participant counts, minor research arrangements, translation ownership, domain terminology and task-success criteria. Decide how long sessions remain viable on mobile suspension and how uncapped retained results are managed without silently capping collection. There is no selected maximum-proof or exhaustion deadline in this report. No performance, accessibility or user-test results are claimed.

