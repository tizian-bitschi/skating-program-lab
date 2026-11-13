# C2 application acceptance observations (#162)

## Current planning presentation

The [four-area workspace and entry-path acceptance](c2-panels.md) supersedes
the fixed-console layout and collapsed phone mixer recorded below. Earlier
measurements remain historical evidence. Conflict-removal and undo changes
belong to later slices of #164.

Recorded 2026-11-10 against the integrated native service and production Vite
application. Technical checks below passed within their stated scope. On
2026-11-10, the owner removed real-screenreader testing as an acceptance gate.
That check was not performed; the agreed remaining gates are complete.
Optional human feedback on the three language catalogs is outstanding. No physical
phone, recruited participant, field percentile or blanket WCAG claim is made.

## Implemented interface

The 2026-11-10 mixer correction below supersedes the original duplicate-control
presentation. The historical timing tables remain tied to their recorded hashes.

The selected C2 mixer now has vertical faders for every service-supplied jump,
regular-spin form and step sequence. Pointer movement changes a local readout;
release/change commits the setting. Each element has one slider: Off at the
bottom, followed only by the category-eligible classes or levels supplied by
the service. Unlevelled elements have only Off/On. No additional checkbox or
maximum dropdown duplicates the slider. Off and Basic are separate; category changes
retain explicit saved maxima while displaying the effective category ceiling.
All twenty spin forms remain present, including disabled
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
those regions. Keyboard checks do not establish real-screenreader behavior;
the owner waived that separate check as an acceptance gate.

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
Phone retain/compare labels provide at least 44px height; native faders support
pointer and keyboard operation. The main text/accent pairs meet 4.5:1, and the control
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
| Retain exact occurrences, set limits, calculate and adopt; browser required-ID assertions and manual reassessment | “Retain for calculation” is attached to each whole entry. Off and Basic have distinct positions on the same named slider. | Explicit maxima survive category changes; adoption keeps required IDs and labels added content. The later mixer correction removed the duplicate checkbox/dropdown and limits positions to eligible values. |
| Exclude an exact connection, observe a retained conflict, restore and recalculate; real exclusion tests | The generic second “Build a connection” action was ambiguous. Its summary now says “Excluded connections”. Pair components and longer wholes remain available. | Personal-conflict details localize the whole entry; restore is direct and keyboard focus returns to a surviving action. Expanded-builder overlap was fixed by reserving panel space. |
| Enter 1Lz/SSp1/ChSq1, mark complete, add a conflicting spin and correct it; independent complete fixtures | Search names/codes and visible full names in the picker avoid an abbreviation-only path. The native Done/Escape action returns to the program. | The literal Bronze 5.70 BV and separate category/personal findings remain understandable. Filled-program headings/actions were previously hidden by whole-workspace scrolling; only content now scrolls. |
| Pause/continue, compare/adopt, reload and inspect restored content; browser lifecycle/storage tests | The requested-state text appears immediately. Escape from a filled search initially cleared text instead of closing; explicit dismissal and a regression now cover it. | Resume uses the same page-held search; reload restores content without silently starting optimization. Bounded comparison pages retain selection. Cancellation now immediately says it was requested and later reports confirmation. |

No blocking ambiguity remained in these exercised paths after correction. Human
language feedback remains optional; real-screenreader output was not verified
and is no longer an acceptance gate by the owner's decision.

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

The real Narrator/browser check was not performed. The fresh check
in sibling `issue-154-screenreader-fresh-check.md` initialized desktop automation,
then stopped on browser-window access because the current URL could not be
established confidently enough for the tool policy. Narrator was not started.
Earlier initialization failures remain historical evidence. No alternate
presentation or automation path is treated as spoken output. On 2026-11-10,
the owner explicitly removed this check as an acceptance requirement. This
supersedes the earlier screen-reader gate without recording an invented pass.
Informal language feedback remains optional and uncollected.

## Final review fixes and exact-build evidence

Source commit `5508042c9dd98f04aa8239379a9a5dd97cb12c30` aligns whole-intention
permissions with the core's typed `WaltzPolicy`. Axel Off disables called Waltz in
repertoire, connection options and manual conflicts, matching search exclusions.
Ignored Bronze Waltz remains controlled separately by W. Conflicts identify A or
W as responsible; credit and category findings keep their independent meaning.
The core is unchanged. Private typed control conversions retain AtomicU8, memory
ordering, cancellation priority, condition-variable wakeups, command idempotency
and existing wire values.

The public HTTP matrix proves unrestricted, required-only, restricted-only and
combined workflows for all six profiles and the independently registered policy.
It obtains actual keys from repertoire/options, checks independent literal bounds
from `tests/backend_acceptance.rs`, retains exact required occurrence identities,
fetches a confirmed prefix, repeats its cursor, and submits fresh complete manual
assessment. The synthetic Lutz-only policy's literal 950 bound is two 1Lz (120),
two distinct change-foot level-one spins (480), and ChSq1 (350). Capacity-one
admission after DELETE verifies actual owner termination between workflows.
Large domains do not require exhaustive enumeration. Standalone and compound
Waltz regressions cover W/A toggles, preserved entries, optimizer requirements
and responsible settings. Browser recovery disables Axel after adding Waltz,
observes A's conflict, restores Axel and recovers the picker.

The final production assets and Release native executable were rebuilt before
three identical cold repetitions in each of the five reference configurations.
The calibrated shared shaper used 4 Mbit/s down, 1 Mbit/s up and 100ms added RTT
on ports 3175/3176. Fresh browser contexts/profiles, blocked service workers,
English locale, no CPU throttle and the recorded Windows 11/i9-13900K host were
used. These exact-build runs supersede the earlier pre-final-fix cold samples.

| Configuration | Startup ms (three runs) | Ordinary assessment ms | Local feedback ms |
| --- | ---: | ---: | ---: |
| Chrome Stable 157.0.8103.50, 1280×720 | 840.8–864.8 | 275.4–283.6 | 7.9–8.1 |
| Edge Stable 157.0.4341.46, 1280×720 | 857.4–863.5 | 273.7–298.2 | 8.2–9.2 |
| Firefox Stable 159.0.1 / geckodriver 0.37.1, 1280×720 | 905–915 | 291–356 | 3–4 |
| Android Chrome proxy, 360×844 | 816.6–842.2 | 274.9–278.8 | 5.4–5.8 |
| iPhone WebKit 26.5 proxy, 390×844 | 865–870 | 298–310 | 12–17 |

All fifteen cold runs met the 3s/1s/200ms targets. Desktop had no document
overflow; phone had no horizontal overflow. Phone rows remain engine, viewport
and touch emulation, not physical-device or Safari certification.

A real constrained Silver search through the same shaper showed local
pause/continue/cancel requested feedback in 0.9–2.5ms. An independent public
`GET updates?after=0&limit=0` observer acknowledged confirmed paused/running/
cancelled states while ordinary UI polling stayed unchanged. Observer-ack to
visible state measured 774.5ms, 5.2ms and 812.5ms respectively, below 2s.
The command responses themselves still reported running/pause-requested,
paused/continue-requested and running/cancel-requested. Their request-ack to
confirmation-visible durations were 2462.7ms, 128.1ms and 2474.2ms. Earlier
2477.6ms/2459.6ms observations remain preserved. Those durations include
cooperative interruption and are not substituted for confirmed-state propagation
or claimed to meet a solver interruption deadline. First collecting control took
2061ms separately; cancellation preserved 14.10 proved BV and 50 confirmed
programs without implying exhaustion.

The final unshaped Release diagnostic also completed six finite constrained
profile workloads with public HTTP DELETE cleanup. Proof observations ranged
64.2–154.6ms, first confirmed results 112.1–309.5ms, and full finite collection
142.6–8739.0ms (2/21/21/21/122/122 distinct results). These are separate
workloads from the unrestricted literal-bound matrix and do not establish a
solver-time promise; exact inputs and samples are in
`154-review-profile-diagnostics.json`.

The final 10,000-record synthetic browser fixture produced 100 receipt-to-frame
samples of 1.6–10.1ms. It also exercised bounded result/comparison DOM, focus,
selection and three reset/reload cycles. This is presentation/resource evidence,
not 10,000 computed optima or a universal memory bound.

Exact SHA-256 artifacts are in sibling
`issue-154-evidence/154-review-final-artifact-hashes.json`:

| Artifact | SHA-256 |
| --- | --- |
| Release service | `10fbf9df6513b58ff984d648f87445f1fbe0c9d549b46f769c7bba1e95d62cdb` |
| `dist/index.html` | `bd0a252e8bd96c44592690d73b333a1716efe1d41efda881a0d286c51e37cd61` |
| `index-BiwD9y-0.js` | `17e6b5f7c80b0fb1df93e4ec51ec697baf431e86f28f0f122dccd53b154c96a9` |
| `index-Bn6b7tfG.css` | `a5cf3ec57219616d084e9a72e71c21d4c154290557f7df79a1fc942bc22b7f35` |

Final sibling evidence uses the `154-review-` prefix: browser/Firefox matrices,
state timing and command/receipt-only observations, six-profile diagnostics,
synthetic resource observations, service-test log and browser-suite record. Runners target this worktree's own
executable/assets. Sandbox Edge profile launch failed; the normal local runner
succeeded outside that sandbox. The first sandboxed browser suite stalled before
tests; the isolated local suite passed 34/34, including new Waltz recovery.
Initial install-cache and Clippy needless-borrow failures were corrected before
the final build. Earlier timing/tooling failures remain evidence, not passes.
The final Release service suite passed 24/24, including all 28 matrix cases.
Build, service formatting and all-target Clippy with warnings denied passed.
The real Narrator check was not performed and was waived as an acceptance gate
by the owner on 2026-11-10. No screen-reader conformance claim follows.

## Mixer interaction correction (2026-11-10)

Owner review identified a fixed 0–6 slider scale and redundant checkbox/dropdown
controls. Each form now has one range input: Off, then only the service-supplied
eligible levels/classes. Unlevelled content has only Off/On. Saved personal
maxima remain unchanged on category switches; the slider shows the effective
ceiling. During a category refresh, old-context controls cannot be changed.
Home/End behavior is normalized because WebKit's native vertical-range End
behavior differed from Chromium. Pointer preview remains local until release.

The full updated browser suite passed 36/36, including category bounds,
Off/Basic, stored-ceiling restoration, native pointer preview and existing
planning/search/storage flows. Additional Chrome157, Edge157 and WebKit26.5
checks verified physical bottom/top positions, keyboard values, absence of
duplicate controls, desktop1280×720 fit and360px phone reflow. TypeScript/Vite
and diff checks passed. The unchanged Rust service/core suites were not repeated.
The earlier controlled timing tables were not rerun for this focused UI change.
