# Implementation status

The library provides partial coverage, not complete ISU compliance. The tests cover selected cases, not every domain rule. See [rule sources](rule-sources.md) for editions and open questions.

## Domain


The library stores planned jump type and intended rotations independently; a missing value is unknown, not a legal or illegal jump. The partial validator checks Bronze listed-jump eligibility, the four jump-element limit, combination structure and listed-jump repetitions, not other categories or Bronze spin and sequence recognition rules; it does not validate program-wide scoring consequences. A choreographic sequence can record planned movements, but free text does not establish recognition. Program metadata retains the selected scoring table and source references; compatibility and source contents are not validated.

Supplied recognition can record marks on a whole element or an ordered component and place +COMBO/+SEQ between components. The relevant notation includes q, <, <<, !, e, V, *, +REP, +COMBO and +SEQ (2786 p. 14; Technical Panel Handbook pp. 2, 6, 25, 30–33; Q&A 3 Q2). B and 1–4 are levels, F denotes a fall, and x denotes second-half credit on protocols; these are distinct from rotation and edge calls. x can be recorded as supplied; the calculation must still apply the event rules. RecognitionDetails stores ordered unmarked component codes, a separate level and calls as its canonical representation. ComponentCode rejects marked or unsupported codes. Its constructor rejects missing components, invalid targets, repeated calls, conflicting rotation or edge calls, and a level on multiple components, jumps, Euler or choreographic elements. Unknown calls remain distinct from an explicitly empty list; notation is generated only when calls are known. For a single component, element-level technical marks normalize to that component. Output orders edge marks, rotation marks, V and asterisk independently of input order; repetition remains an element-level annotation. Component codes are limited to the listed jump names, Euler and the explicitly supported spin and sequence abbreviations; unknown codes require an explicit error. The constructor does not establish rule legality, event applicability or whether a call is applicable to a particular element. Future scoring must validate those conditions. The second-half factor requires timing, interruption facts and a check for applicable event-specific exceptions.


The model stores observations and supplied panel decisions separately. Indices refer to caller-provided lists and are not validated. Observed movement is free text; revolutions are whole counts. Source applicability and metadata consistency are not validated. See `src/lib.rs`, `src/plan.rs`, `src/execution.rs`, `tests/domain.rs`, `tests/plan.rs` and `tests/execution.rs`.

## Validation

`src/validation.rs` checks selected Bronze jump eligibility, element count, combination count and structure, Euler prohibition and repetitions. Missing details yield unknown findings; other categories and seasons are unsupported. Findings carry a rule identifier, explanation, location and source. Coverage is always partial; an empty list does not establish compliance. Spin and sequence recognition and program-wide scoring consequences are not validated. `tests/validation.rs` covers selected boundaries and unknowns.

## Scoring


The scoring module selects unfactored values from Communication 2786, updated 2 June 2026. It supports the individually listed 1Lz fixtures, 1A<<, CSSp1 and ChSp1. Results identify the exact row, page and table version and store hundredths as integers. Ignored and no-value outcomes remain distinct from recognized zero values. Unknown recognition, details or calls are reported explicitly. Other entries, combinations, simultaneous calls outside the listed fixtures, V, *, +REP, +COMBO, +SEQ and second-half annotations are unsupported. The lookup does not establish event eligibility or infer a panel decision. Review the cited source and event applicability before use.



The scoring module calculates unfactored scores for unmarked 1Lz, CSSp1 and ChSp1 using the corresponding GOE columns in Communication 2786 pp. 2, 8 and 14. It requires a supplied final integer GOE and an explicit fall observation. A fall caps the final GOE at -3 (Communication 2788 pp. 14 and 16). It does not infer the final GOE or check every execution error. Marked elements are unsupported except the zero-valued 1Lz<< and 1A<< rows; combinations are unsupported. Scores are scenarios, not panel results; independent domain review is required.

The program scoring slice aggregates unmarked 1Lz, CSSp1 and ChSp1 single-judge scenarios, the supported zero-valued jump rows, and supplied ignored and no-value outcomes. Callers provide the complete ordered recognition list, final GOE, fall observations, elapsed jump start times and a count of falls outside recognized elements. TES is absent if any element is unresolved or unsupported. Bronze fall deductions and category time deductions are separate from TES; other categories' fall deductions are not implemented. Time deductions require confirmation that interruption and incomplete-program consequences are resolved. Jump scoring also requires an uninterrupted timing scenario and confirmation that applicable event notices and exceptions were checked. The last three supported jumps starting at or after half the required duration receive the individually rounded 1.1 base-value factor; GOE remains unfactored. A recorded x is not evidence of eligibility. Unsupported elements can affect credit and keep TES incomplete. This is partial coverage and does not validate program legality or calculate PCS or a segment score.

The last-three selection test uses five repeated jumps solely to exercise the algorithm; it is not a legal Bronze program. A separate two-jump fixture covers the half-duration boundary and GOE separation within the supported catalog. Rule 502 timing consequences outside this slice prevent an exact result: programs at least 30 seconds below the lower duration limit, and elements starting after the upper limit. For programs exceeding that limit, each element needs a start time. The caller selects a ruleset explicitly; each recognition must match it. The recorded x is accepted only once on a jump and never establishes eligibility.

All supplied element start times must follow program order, including times on unsupported elements. A test-only ruleset identity exercises mismatch rejection; replace it with a real second identity when another ruleset is supported.


The element scorer supports the zero-valued 1Lz<< and 1A<< rows of Communication 2786, pp. 2-3. Program totals count supplied ignored and no-value outcomes as zero while retaining their distinct element outcomes. A no-value outcome leaves jump bonus selection unresolved when its effect on eligibility is not established; an element known to start before the halfway point or to be a non-jump does not affect selection. Downgraded zero-valued jumps reject a final GOE above -1; with a fall they require -5 (Communication 2788, pp. 14 and 16). Unknown and unsupported outcomes remain incomplete. These inputs do not establish whether a program is legal.

Scoring source references resolve their known document and version to official ISU URLs through `SourceReference::official_url()`. Unknown document editions return no URL rather than a substitute.


See `src/scoring.rs` and `tests/scoring.rs`. These tests cover selected rows and scenarios.

## Optimization

The reference and Pumpkin solver cover only the finite Bronze 1Lz domain in the [optimization specification](specs/optimization.md). The solver is enabled by default; `--no-default-features` omits Pumpkin. `src/optimizer.rs`, `src/optimizer/pumpkin.rs` and `tests/optimizer.rs` cover this domain, slot subsets and result limits. These do not establish complete eligibility, a global maximum or WebAssembly compatibility.
