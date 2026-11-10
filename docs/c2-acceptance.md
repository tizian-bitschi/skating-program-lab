# C2 application acceptance observations (#162)

Recorded 2026-11-10 against the integrated native service and production Vite
application. Technical checks below passed within their stated scope. Overall
acceptance remains open: a real screen-reader/browser walkthrough has not run.
Human feedback on the three language catalogs is also outstanding. No physical
phone, recruited participant, field percentile or blanket WCAG claim is made.

## Implemented interface

The selected C2 mixer now has vertical faders for every service-supplied jump,
regular-spin form and step sequence. Pointer movement changes a local readout;
release/change commits the setting. Checkboxes and explicit maximum selectors
remain available as alternatives. Off and Basic are separate; category changes
retain explicit maxima. All twenty spin forms remain present, including disabled
category-unavailable forms. Names appear on desktop hover/focus and remain
visible in the reflow layout.

The native element/connection picker replaces the permanent tall catalogue.
Adding an element closes it and restores focus. Escape also closes it with a
filled search field. On phones, limits start collapsed and explicit links reach
the program and suggestions; expanding controls does not alter the request.
Desktop headings, completion/assessment and primary actions stay outside the
scrolling content lists. The exclusion builder is named “Excluded connections”.

Both confirmed results and selected comparisons render twenty records per page.
Previous/next and a keyboard-operable page number reach every received record;
the denominator explicitly describes confirmed records. Pagination does not cap
collection or selection. New batches preserve the current page, scroll position,
focused checkbox and selected IDs. The UI retains all received records in memory
until reset/reload, so retained data grows with collection size. It does not
invent client result identities or save that history in localStorage.

Live regions contain concise assessment/BV and search-state/count summaries.
Expanded findings, source links, result records and comparisons sit outside
those regions. Keyboard checks supplement, and do not replace, the pending
screen-reader check.

## Automated behavior and layout

The Stable Chrome browser suite passed all 32 tests before the final Escape and
cancel-request wording fixes. The relevant final focused suite then passed all
11 tests, including the added Escape regression, four resource/reset cycles,
filled desktop layouts, fader commit behavior and all six search/recovery tests.
The suites use actual picker actions rather than operating hidden controls.

Coverage includes six built-in profiles and the independently registered HTTP
test policy, real manual BV/compliance, required occurrences/adoption, exact
connection exclusions, persistence/errors, stale replies, pause/continue/cancel,
connection recovery and source links. The service suite passed 21 integration
tests, including the independent 16/14-composition oracle and owner lifecycle.
The synthetic workload is separate from those correctness oracles.

Filled programs are checked at 1280×720 and 1440×900 in English, German and
Russian. Program headings, the element picker action, calculation and assessment
remain within the viewport, and the workspace container does not scroll.
Phone/reflow checks include 360 and 320 CSS pixels, no horizontal document
overflow, translated names, explicit touch targets and keyboard operation.
Phone retain/compare labels provide at least 44px height; faders and buttons have
tap/key alternatives. The main text/accent pairs meet 4.5:1, and the control
border/focus pairs meet 3:1 against white. Disabled controls use the applicable
contrast exception. This is a scoped contrast check.

Actual Chrome browser zoom was set to 200% through the appearance settings in a
dedicated test profile. The recorded viewport changed from 1264×805/DPR 1 to
632×402 CSS pixels/DPR 2; CDP also reported browser zoom 2. All three locales
retained operation, no horizontal overflow and picker focus restoration.
Playwright's ordinary screenshots cropped the zoomed surface; direct browser
surface captures and control rectangles were used for visual verification.
The user’s browser profile and settings were not changed.

## Controlled production measurements

Reference host: Windows 11 Pro 10.0.26200, Intel i9-13900K (24 cores/32 logical
processors), 68,414,652,416 bytes RAM, Node 24.21.0. Native service built with
`cargo build --release --offline --manifest-path service/Cargo.toml`; static
assets built with `npm --prefix web run build`. No CPU throttle was applied.
Each cold repetition used a fresh browser context/profile, English and an empty
browser workspace. No service worker bypass was allowed.

All browser requests used the same local reverse-proxy shaper: aggregate
4,000,000 bit/s download, 1,000,000 bit/s upload, and 50ms added request plus 50ms
added response latency. Its recorded concurrent calibration measured 4.00204
and 1.00176 Mbit/s after subtracting the empty-response baseline; the baseline
median was 122.57ms including local timer/transport overhead. It models HTTP body
bytes and added application latency, not packet loss, TLS or an exact TCP RTT.

Each row below contains three repetitions under identical English conditions.
Ranges retain every valid sample. Navigation-to-usable and edit-to-assessment
include browser polling/automation observation overhead; local dialog feedback
ends at the next animation frame with the dialog open.

| Configuration | Usable startup ms | Ordinary assessment ms | Local dialog feedback ms |
| --- | ---: | ---: | ---: |
| Chrome Stable 157.0.8103.50, 1280×720 | 825.3–845.5 | 270.6–281.7 | 7.2–7.3 |
| Edge Stable 157.0.4341.46, 1280×720 | 848.9–863.7 | 277.2–303.7 | 8.3–9.0 |
| Firefox Stable 159.0.1 / geckodriver 0.37.1, 1280×720 | 896–927 | 293–312 | 3–5 |
| Android Chrome engine/viewport/touch proxy, Chrome 157.0.8103.50, 360×844 | 829.7–851.0 | 273.8–290.4 | 5.0–6.5 |
| iPhone Safari engine/viewport/touch proxy, WebKit 26.5, 390×844 | 856–894 | 306–323 | 10–15 |

These samples passed the 3s/1s/200ms budgets. The phone rows are engine/viewport
proxies, not physical Android Chrome or iPhone Safari observations. Additional
en/de/ru layout runs are retained separately and are not presented as identical
cold repetitions. The cold matrix preceded the final Escape-handler, cancel
wording and phone target-spacing fixes; final focused behavior/zoom/resource
checks used the subsequent assets. Exact asset/native hashes are retained.

Real constrained Adult Silver collection was also exercised through the same
production shaper. Pause, continue and cancel requested feedback appeared within
2.5–2.8ms; confirmed state appeared 128.4–1336.2ms after the received control
acknowledgement. First collecting control appeared after 2123ms, reported
separately from startup. The cancelled search retained its proved 14.10 BV and
50 confirmed programs without claiming exhaustion. A separate unshaped Release
HTTP diagnostic records proof, first result and exhaustion for finite constrained
workloads in all six profiles; it is not a total solver-time promise.

## Synthetic resources

A clearly marked 10,000-record browser/HTTP fixture delivered one hundred
100-record deltas. It exercised append, inner scrolling, first/last-page
selection, comparison, preserved focus and resets. It does not claim 10,000
computed optimal programs. Both result and comparison DOM paths stay bounded.
The final Stable Chrome run recorded 100 receipt-to-presented-frame samples of
1.6–21.7ms, below the 200ms budget. Receipt is the public response JSON becoming
available; presentation is the matching loaded-record count at the next frame.

CDP heap observations after explicit garbage collection were 2.78MB before
collection, 3.22MB at 100 records and 8.86MB at 10,000 records. Three subsequent
reload/collect cycles recorded 2.70–3.07MB empty and 8.28–8.29MB at 10,000;
the final empty sample was 3.07MB. Records are retained deliberately while
displayable. These samples show the observed reset behavior; they do not define
a universal memory limit or establish leak freedom from process high-water RAM.
Service tests separately verify live-owner accounting, cancellation, disposal
and lease expiry through public HTTP behavior.

## Heuristic walkthroughs

These are agent observations and perspective-based judgments, not observed
children/adults or invented participant quotations. Basic skating knowledge was
assumed, while UI knowledge and memorized codes were not.

| Goal and actual exercised behavior | Twelve-year-old perspective | Adult perspective and corrections |
| --- | --- | --- |
| Category → explicit calculation → proved program; real profile browser tests and the separate Silver production run | Phone initially required traversing every control. Collapsed limits plus named program/suggestions links now expose the task directly; names remain available without memorizing codes. | “Proved maximum” and the explicit unknown-total message distinguish proof from all alternatives being delivered. |
| Retain exact occurrences, set limits, calculate and adopt; browser required-ID assertions and manual reassessment | “Retain for calculation” is attached to each whole entry. Off versus Basic has a separate checkbox/fader position and named slider value. | Explicit maxima survive category changes; adoption keeps required IDs and labels added content. Narrow “M” maximum values were corrected to readable “Max”. |
| Exclude an exact connection, observe a retained conflict, restore and recalculate; real exclusion tests | The generic second “Build a connection” action was ambiguous. Its summary now says “Excluded connections”. Pair components and longer wholes remain available. | Personal-conflict details localize the whole entry; restore is direct and keyboard focus returns to a surviving action. Expanded-builder overlap was fixed by reserving panel space. |
| Enter 1Lz/SSp1/ChSq1, mark complete, add a conflicting spin and correct it; independent complete fixtures | Search names/codes and visible full names in the picker avoid an abbreviation-only path. The native Done/Escape action returns to the program. | The literal Bronze 5.70 BV and separate category/personal findings remain understandable. Filled-program headings/actions were previously hidden by whole-workspace scrolling; only content now scrolls. |
| Pause/continue, compare/adopt, reload and inspect restored content; browser lifecycle/storage tests | The requested-state text appears immediately. Escape from a filled search initially cleared text instead of closing; explicit dismissal and a regression now cover it. | Resume uses the same page-held search; reload restores content without silently starting optimization. Bounded comparison pages retain selection. Cancellation now immediately says it was requested and later reports confirmation. |

No blocking ambiguity remained in these exercised paths after correction. Human
language feedback and actual screen-reader output remain separate pending work.

## Evidence and remaining gates

Local artifacts are in the sibling `issue-154-evidence` directory: the calibrated
shaper/host record; `162-browser-matrix*.json` and runners; Firefox matrix;
state timing before/after correction; profile diagnostics; synthetic resource
JSON; zoom JSON, rectangles and direct-surface screenshots; exact final artifact
hashes; Stable full/focused browser logs. Repository browser tests reproduce the
behavioral assertions using isolated ports 3169/3170 and public DELETE cleanup.
`C2_BROWSER_PATH` optionally selects the verified Stable executable; ordinary
tests retain their existing installed-Chrome default.

Valid failures remain recorded: the initial six desktop language samples had a
one-pixel document overflow (61px navbar versus 60px reserved height); the fixed
identical-English repetitions passed. The first controlled control run exposed
missing immediate cancellation wording. Initial filled-program/browser and
zoom failures are retained rather than converted into passes. A synthetic
fixture initially advertised only delivered records, causing normal 1s polling;
it was corrected to advertise its already-confirmed backlog. This was a fixture
correction, not a product collection-speed change.

`npm run build`, the service tests, service formatting and service Clippy with
warnings denied passed. The parent integration ran the prescribed library
formatting, Clippy, default tests and no-default-features tests successfully.
No CI, security, agent instruction, deployment or main-branch change is included.

The required real Narrator/browser check is blocked by the documented desktop
automation initialization failure and missing alternate `orca` command. Neither
an accessibility tree nor keyboard success is recorded as spoken output. Final
parent acceptance must retain that open gate until a real run is observed.
