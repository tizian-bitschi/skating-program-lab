# Scoring

## Target contract


The user supplies the recognized code and level, calls, final **single hypothetical judge's integer GOE** from -5 to +5, fall location, program duration and selected rule/table version. The user can provide observed execution separately. This is a scenario calculation, not an actual panel result; panel GOE is a trimmed mean of judges' numerical GOE values and is outside the MVP. Inputs are checked for supported codes, categories, mutually exclusive rotation calls (`q`, `<`, `<<`), edge calls (`!`, `e`), supported combinations and applicable GOE restrictions. An edge call on an inapplicable jump or contradictory marks yields an error; missing or undecidable evidence is unknown. An error must not silently change supplied calls or GOE. For a fall in an element, Communication 2788 pp. 14 and 16 gives a -5 reduction and a maximum starting GOE of +2; a supplied final single-judge GOE above -3 is inconsistent. Other aspects can lower the final GOE; a fall does not automatically make it -5.

For supported inputs, use the recognized element and call-specific entry in the selected Scale of Values, apply the corresponding GOE table value and prescribed rounding, and display base value, adjustments, GOE points and element score. `q` retains base value, `<` selects the under-rotated value, `<<` the downgraded value; `!` retains base value and `e` selects the applicable edge value. Their GOE effects are assessed separately; calls alone do not determine a final GOE. For combinations use the relevant table and reference-jump rule rather than a blanket percentage formula. The 1.1 second-half factor requires the jump-element start time and applicable interruption facts; see [sources](../rule-sources.md). Apply the general rule as the source-based default and check for any event-specific Adult exception. Missing timing or an unresolved applicable exception makes the affected result incomplete. Sum element scores to TES. Show fall and time deductions separately; do not subtract them from TES. A later total-segment score requires separately calculated PCS and deductions. If an exact rule, rounding step or table is not supported, report the calculation as incomplete.


## Acceptance examples

These are target examples. Existing tests cover some cases only in part; see [implementation status](../implementation-status.md). Other findings may coexist. See [rule sources](../rule-sources.md) for source editions. Independent human domain review is required before claiming complete correctness.

- **AC-04 — Table values:** Using the 2 June 2786 update, a recognized solo 1Lz started in the first half at final GOE +1 has BV 0.60, GOE +0.06 and element score 0.66; 1Lzq retains BV 0.60, 1Lz< has 0.48, 1Lz<< has 0.00, 1Lz! has 0.60, 1Lze has 0.48 and 1Lze< has 0.36 (p. 2). These are unfactored table BVs. Their error-related GOE reductions are not inferred from BV. A recognized ChSp1 with GOE +1 has BV 3.50, GOE +0.50 and score 4.00; CSSp1 with +1 has BV 2.30, GOE +0.23 and score 2.53 (pp. 8 and 14). No fall is assumed in these examples.

- **AC-05 — Falls and time:** In Bronze, a fall inside a recognized element gives no separate fall deduction; with a supplied final single-judge GOE of -3 the applicable table value is used, while +1 is inconsistent with the fall's -5 reduction and starting cap. An outside fall adds 0.50 to separate deductions. A Bronze duration of 1:50 is within tolerance; 1:51 incurs a 1.00 time deduction, assuming no other time-related consequence (Adult p. 11; Sports Rules 353.1(n), 502; 2788 pp. 14, 16). TES remains the element-score sum.

- **AC-10 — Silver duration:** 2:10 remains within tolerance; 2:11 produces a 1.00 separate time deduction, assuming no additional timing consequence. A jump eligible for the second-half factor without timing data or with an unresolved applicable exception must produce an incomplete affected result, not a silently unadjusted exact total.


## Recognition representation

Supplied recognition can record marks on a whole element or an ordered component and place +COMBO/+SEQ between components. The relevant notation includes q, <, <<, !, e, V, *, +REP, +COMBO and +SEQ (2786 p. 14; Technical Panel Handbook pp. 2, 6, 25, 30–33; Q&A 3 Q2). B and 1–4 are levels, F denotes a fall, and x denotes second-half credit on protocols; these are distinct from rotation and edge calls. x can be recorded as supplied; the calculation must still apply the event rules. RecognitionDetails stores ordered unmarked component codes, a separate level and calls as its canonical representation. ComponentCode rejects marked or unsupported codes. Its constructor rejects missing components, invalid targets, repeated calls, conflicting rotation or edge calls, and a level on multiple components, jumps, Euler or choreographic elements. Unknown calls remain distinct from an explicitly empty list; notation is generated only when calls are known. For a single component, element-level technical marks normalize to that component. Output orders edge marks, rotation marks, V and asterisk independently of input order; repetition remains an element-level annotation. Component codes are limited to the listed jump names, Euler and the explicitly supported spin and sequence abbreviations; unknown codes require an explicit error. The constructor does not establish rule legality, event applicability or whether a call is applicable to a particular element. Future scoring must validate those conditions. The second-half factor requires timing, interruption facts and a check for applicable event-specific exceptions.


## Implemented scoring details

The scoring module selects unfactored values from Communication 2786, updated 2 June 2026. It supports the individually listed 1Lz fixtures, 1A<<, CSSp1 and ChSp1. Results identify the exact row, page and table version and store hundredths as integers. Ignored and no-value outcomes remain distinct from recognized zero values. Unknown recognition, details or calls are reported explicitly. Other entries, combinations, simultaneous calls outside the listed fixtures, V, *, +REP, +COMBO, +SEQ and second-half annotations are unsupported. The lookup does not establish event eligibility or infer a panel decision. Review the cited source and event applicability before use.



The scoring module calculates unfactored scores for unmarked 1Lz, CSSp1 and ChSp1 using the corresponding GOE columns in Communication 2786 pp. 2, 8 and 14. It requires a supplied final integer GOE and an explicit fall observation. A fall caps the final GOE at -3 (Communication 2788 pp. 14 and 16). It does not infer the final GOE or check every execution error. Marked elements are unsupported except the zero-valued 1Lz<< and 1A<< rows; combinations are unsupported. Scores are scenarios, not panel results; independent domain review is required.

The program scoring slice aggregates unmarked 1Lz, CSSp1 and ChSp1 single-judge scenarios, the supported zero-valued jump rows, and supplied ignored and no-value outcomes. Callers provide the complete ordered recognition list, final GOE, fall observations, elapsed jump start times and a count of falls outside recognized elements. TES is absent if any element is unresolved or unsupported. Bronze fall deductions and category time deductions are separate from TES; other categories' fall deductions are not implemented. Time deductions require confirmation that interruption and incomplete-program consequences are resolved. Jump scoring also requires an uninterrupted timing scenario and confirmation that applicable event notices and exceptions were checked. The last three supported jumps starting at or after half the required duration receive the individually rounded 1.1 base-value factor; GOE remains unfactored. A recorded x is not evidence of eligibility. Unsupported elements can affect credit and keep TES incomplete. This is partial coverage and does not validate program legality or calculate PCS or a segment score.

The last-three selection test uses five repeated jumps solely to exercise the algorithm; it is not a legal Bronze program. A separate two-jump fixture covers the half-duration boundary and GOE separation within the supported catalog. Rule 502 timing consequences outside this slice prevent an exact result: programs at least 30 seconds below the lower duration limit, and elements starting after the upper limit. For programs exceeding that limit, each element needs a start time. The caller selects a ruleset explicitly; each recognition must match it. The recorded x is accepted only once on a jump and never establishes eligibility.

All supplied element start times must follow program order, including times on unsupported elements. A test-only ruleset identity exercises mismatch rejection; replace it with a real second identity when another ruleset is supported.


The element scorer supports the zero-valued 1Lz<< and 1A<< rows of Communication 2786, pp. 2-3. Program totals count supplied ignored and no-value outcomes as zero while retaining their distinct element outcomes. A no-value outcome leaves jump bonus selection unresolved when its effect on eligibility is not established; an element known to start before the halfway point or to be a non-jump does not affect selection. Downgraded zero-valued jumps reject a final GOE above -1; with a fall they require -5 (Communication 2788, pp. 14 and 16). Unknown and unsupported outcomes remain incomplete. These inputs do not establish whether a program is legal.

Scoring source references resolve their known document and version to official ISU URLs through `SourceReference::official_url()`. Unknown document editions return no URL rather than a substitute.



## Implemented subset

See [implementation status](../implementation-status.md#scoring) for supported rows and limitations.
