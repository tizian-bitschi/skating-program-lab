# Acceptance technical notes (#152)

Source check: 2026-11-09. Planning evidence only. No implementation, prototype, participant evaluation or performance measurement was performed. Product budgets, devices and repeat counts remain decisions for the owner.

## Accessibility scope

Use WCAG 2.2 A and AA criteria within an explicit page/workflow scope. Selected practical checks alone do not establish AA conformance; conformance concerns complete pages and processes. Check keyboard operation without traps (2.1.1–2.1.2), logical focus order and visible focus (2.4.3, 2.4.7), and focused controls not entirely obscured by authored content (2.4.11). AA does not require the stronger AAA focus-appearance criterion 2.4.13. Check pointer cancellation, single-pointer alternatives to gestures/dragging, and accessible names matching visible labels (2.5.1–2.5.3, 2.5.7). Target size 2.5.8 is 24 × 24 CSS pixels with defined exceptions, including spacing; 44 × 44 is enhanced AAA. Text contrast is normally 4.5:1, or 3:1 for large text (1.4.3); relevant control/state and graphical distinctions require 3:1 (1.4.11), subject to their exceptions. Do not communicate meaning through color alone (1.4.1). [Normative WCAG 2.2](https://www.w3.org/TR/WCAG22/)

A desktop workspace designed to fit one viewport can reflow into vertical scrolling under zoom or a narrow viewport. Preserve information and functionality at an equivalent width of 320 CSS pixels, normally without two-dimensional scrolling. This corresponds to a 1280 CSS-pixel viewport at 400% zoom. The exception covers content that needs two-dimensional layout for meaning or use; it does not exempt an entire workspace merely because its desktop design has several panels. A requirement to fit the unzoomed desktop viewport must not prevent accessible reflow. [WAI reflow explanation](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html)

## Timing definitions and evidence

The proposed 3-second usable-start, 200-millisecond visible-feedback and 1-second ordinary-assessment limits are project proposals. Define each start event, visible/usable end condition, workload, environment and treatment of failures before adopting a gate. These sources do not select those product limits.

Core Web Vitals separately assess LCP (loading), INP (interaction responsiveness) and CLS (visual stability). Their good thresholds are 2.5 seconds, 200 milliseconds and 0.1 respectively, assessed at field p75 separately for desktop and mobile. A small set of controlled lab repeats describes only that sample and environment; its percentile must not be presented as field p75 or population coverage. [Web Vitals definitions and measurement guidance](https://web.dev/articles/vitals)

Usable-start is not LCP. An individual feedback interval is not the page's INP, even when both use 200 milliseconds. INP measures latency through the next paint and does not encompass every later asynchronous consequence. A visible acknowledgement and completed assessment need separate intervals; neither establishes a maximum-proof or exhaustion deadline. [INP definition](https://web.dev/articles/inp)

## Participant evidence

A proposed two-participant evaluation involving a person aged twelve or older and an adult can reveal task-specific problems and comprehension failures. It cannot establish usability across the age range, languages, devices, experience levels or disabilities. Report observed outcomes with denominators, participant characteristics, tasks and moderator assistance. This is an application of WAI's explicit caution against generalizing from a couple of participants; WAI also requires combining user involvement with standards evaluation when assessing accessibility. No universal success rate or sufficient participant count follows from this guidance. [WAI user evaluation and reporting guidance](https://www.w3.org/WAI/test-evaluate/involving-users/)


## Subsequent user scope decision

The user later made real-smartphone tests and planned participant studies optional, choosing browser/viewport checks and perspective-based walkthroughs for acceptance. The accepted criteria govern required gates. The source cautions above still apply: emulation and heuristic review must not be presented as physical-device or empirical participant evidence.
