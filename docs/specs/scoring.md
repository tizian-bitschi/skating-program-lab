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

The scorer uses the common immutable catalog from Communication 2786, updated 2 June 2026. Each value records the exact table row, page and version. The supplied final GOE selects a table column; it is not inferred from a call. Catalog presence does not establish category eligibility or recognition support.

| Family / mark | BV and single-judge GOE support | Rule and evidence boundary |
| --- | --- | --- |
| Regular spins, all variants including CLSp/FCLSp, B and 1–4 | Exact published row and GOE column, pp. 6–9 | Supplied level required; no feature inference or category validation |
| Spin V | Exact reduced row and GOE column, pp. 6–9 and p. 14 remark 3 | Handbook p. 25 limits V to flying, combination and change-foot cases; only published V rows supported. No deduction inferred from V alone: its cause can be a chosen two-position combination, not a GOE error. Caller supplies final GOE after assessing execution errors |
| StSq B and 1–4 | Exact row and GOE column, p. 9 | Supplied level required; no turn/feature inference (2788 pp. 2–3; Q&A 2 p. 1) |
| ChSq1 and ChSp1 | Exact fixed BV and choreographic GOE scale, p. 14 | Recognition supplied; no movement inference (2788 pp. 3–6; Q&A 1 p. 1) |
| Falls in each supported family | Final GOE must be at most -3 | 2788 pp. 14, 16: starting cap +2 and reduction -5. Other execution errors remain the caller's responsibility |
| q, <, <<, !, e on non-jumps; V on StSq/choreographic elements or an absent spin variant | Unsupported | Marks never select a plausible substitute row |
| Pair families | Catalog only; unsupported recognition/scoring | No Pair rules or fabricated Single-style score |
| Sole * on one whole element | NoValue | Supplied final panel call; TPH Singles p. 2. No panel call is inferred |
| +REP on one supported jump | 80% applicable BV; original applicable GOE row | Supplied final panel call; SOV p. 14 remark 5. No repetition or category eligibility is inferred |
| Euler inside a supplied recognized unit | Published zero row; no independent reference GOE | At most one Euler between listed jumps; no rotation/edge/star calls on Euler. Supported following jumps are single/double S or F; no footwork or eligibility is inferred |
| Supplied full +SEQ | Full component BV and one reference GOE | Two or three listed single/double jumps, optionally one non-listed Euler; a later Axel is required. No historical 80% sequence factor |
| Component * in an unbroken unit | Exclude BV/reference; retain calls for whole-element GOE | TPH pp. 2, 29 and October Q&A 3 Q2–3. All listed components starred gives NoValue |
| Contextual +REP with first jump retained and later jumps starred | 80% retained BV; original retained-row GOE | TPH p. 33 and SOV p. 14 remark 5; no repetition or star call is inferred |
| Interrupted +COMBO / +SEQ with later components explicitly starred | Component BV is available; element score requires the additive execution-evidence API | The structural marker does not identify the execution error or its cumulative GOE consequences; see the execution-evidence contract |

`tests/scoring.rs` checks all 187 representable non-jump rows and all eleven GOE columns against the independent PDF extraction, including rounded V values (for example FCSpBV 1.43 and GOE +1 = 0.14). Each row also tests fall limits and missing fall observations. Mixed-family timed and timestamp-free fixtures keep non-jumps outside the second-half selection and deductions outside TES. Published levels and V are accepted panel inputs, not assertions about the observed movement or program legality. Unimplemented jump consequences remain explicit below and need a separate contextual-recognition slice before support expands.

| Variant of a supported single jump | Base value | Single-judge score |
| --- | --- | --- |
| Clean, q, <, << | Supported | Supported when final GOE is consistent with the calls and fall |
| Flip/Lutz ! or e alone | Supported | Supported when final GOE is consistent with the call and fall |
| Flip/Lutz ! or e combined with q, < or << | Supported from the named table row | Supported with cumulative GOE limits from 2788 pp. 14, 16 |
| ! or e on other jumps; other unlisted marks or combinations | Unsupported | Unsupported |
| Missing details or calls | Unknown | Unknown |

Communication 2788, updated 19 May 2026, pp. 14 and 16, supplies error reductions and starting-GOE limits. The scorer rejects impossible final GOE values and does not infer a final mark. A fall with a call is subject to the reductions and starting cap. Contradictory calls are rejected by the recognition constructor. Unknown recognition, details and calls remain distinct; recognized zero, ignored and no-value outcomes are not interchangeable. Invalid table identifiers are rejected at parsing. Recognized two- or three-jump combinations whose later jumps are 1T, 2T, 1Lo or 2Lo are supported with a single reference-jump GOE; multiple supported component calls can be scored. Three-jump combinations are supported when each later jump is a single/double toe loop or loop. Supplied Euler/sequence structures and component stars are supported as specified above. Triple/quadruple jumps, unsupported transitions, +REP with multiple retained jumps and interrupted-unit GOE without verified execution evidence remain unsupported. These are hypothetical scenarios, not panel results; independent domain review remains required.

The program scoring slice aggregates supported single-jump, supported jump-unit, regular/V spin, StSq, ChSq1 and ChSp1 single-judge scenarios, and supplied ignored and no-value outcomes. Callers provide the complete ordered recognition list, final GOE, fall observations, elapsed jump start times and a count of falls outside recognized elements. TES is absent if any element is unresolved or unsupported. Bronze fall deductions and category time deductions are separate from TES; other categories' fall deductions are not implemented. Time deductions require confirmation that interruption and incomplete-program consequences are resolved. Jump scoring also requires an uninterrupted timing scenario and confirmation that applicable event notices and exceptions were checked. The last three supported jumps starting at or after half the required duration receive the individually rounded 1.1 base-value factor; GOE remains unfactored. A recorded x is not evidence of eligibility. Unsupported elements can affect credit and keep TES incomplete. This is partial coverage and does not validate program legality or calculate PCS or a segment score.

The last-three selection test uses five repeated jumps solely to exercise the algorithm; it is not a legal Bronze program. A separate two-jump fixture covers the half-duration boundary and GOE separation within the supported catalog. Rule 502 timing consequences outside this slice prevent an exact result: programs at least 30 seconds below the lower duration limit, and elements starting after the upper limit. For programs exceeding that limit, each element needs a start time. The caller selects a ruleset explicitly; each recognition must match it. The recorded x is accepted only once on a jump and never establishes eligibility.

All supplied element start times must follow program order, including times on unsupported elements. A test-only ruleset identity exercises mismatch rejection; replace it with a real second identity when another ruleset is supported.


The element scorer preserves the zero-valued 1Lz<< and 1A<< rows of Communication 2786, pp. 2-3. Program totals count supplied ignored and no-value outcomes as zero while retaining their distinct element outcomes. A no-value outcome leaves jump bonus selection unresolved when its effect on eligibility is not established; an element known to start before the halfway point or to be a non-jump does not affect selection. Downgraded zero-valued jumps reject a final GOE above -1; with a fall they require -5 (Communication 2788, pp. 14 and 16). Unknown and unsupported outcomes remain incomplete. These inputs do not establish whether a program is legal.

Scoring source references resolve their known document and version to official ISU URLs through `SourceReference::official_url()`. Unknown document editions return no URL rather than a substitute.



## Implemented subset

See [implementation status](../implementation-status.md#scoring) for supported rows and limitations.

## Timestamp-free planning scenarios

`score_planning_scenario` calculates a hypothetical single-judge sum without skating times. With no boundary it uses unfactored values and categorizes no element as second half. An optional index marks the first element assumed to be in the second half; the resulting credit is conditional on that assumption. The index is not evidence of an actual start time, event applicability or an official TES. Unknown elements can leave the conditional selection incomplete. `score_program` remains the separate time-dependent path and requires timing and applicable rule evidence for exact claims; duration remains available for deductions.

## Remaining contextual jump work (#46)

The fourth milestone adds explicit execution evidence:

- Separately supplied structured execution errors for interrupted +COMBO/+SEQ validate targets and fall consistency and constrain cumulative GOE without inferring errors from a marker or movement description.
- Independently source-derived fixtures cover supported contextual outcomes; unverified weight-transfer and multiple-full-revolution consequences remain explicit UnsupportedMarks and are tracked in [#56](https://github.com/tizian-bitschi/skating-program-lab/issues/56) and [#58](https://github.com/tizian-bitschi/skating-program-lab/issues/58).

Missing levels on a regular spin or StSq return the existing `UnknownDetails` error, as required for missing panel evidence; fully specified but unimplemented variants remain `Unsupported`. No new public type or signature is introduced. New families use `CatalogSource::reference` for page, section, numeric value-row and exact code. Previously supported solo jump/CSSp1/ChSp1 source strings retain their compatibility format. Combination sources now identify each actual component row instead of describing a synthetic code as a published table row.

## Supplied jump calls and cumulative GOE (#46, second milestone)

Communication 2788 (May 19), p. 14 explicitly adds reductions for multiple errors; p. 16 caps the starting GOE at +2 for multiple errors in any element. The scorer checks the highest possible final grade using the minimum applicable reductions: ! = 1, e = 2, q = 2, < = 2 and << = 3. The starting cap also applies to e, << and falls. The Judges Handbook pp. 2 and 6 additionally excludes a starting +5 for any execution error and limits a single q to starting +3; final maxima are therefore q +1, < +2 and ! +3. Clean recognition and a +REP call without execution error can still use final +5. Grades cannot go below -5. Multiple q signs in a combination use the distinct p. 14 row (-3 to -4, starting cap +2), not two separate -2 reductions. The supplied final GOE remains a hypothetical judge input; no mark or execution quality is inferred. A fall adds its -5 reduction once to the element, not once per marked component.

Communication 2786 p. 14 remark 5 and the Singles Technical Panel Handbook p. 32 establish the numerical effect of a supplied final +REP call: 80% of the applicable jump BV, rounded to two decimals, with GOE based on the original applicable row. The scorer supports +REP on one recognized supported jump, including a multi-component supplied unit whose first jump is the only retained listed jump, with existing rotation/edge calls. It does not infer a repetition or its Adult applicability from program order. Second-half BV credit, when separately established, follows the rounded reduced BV; GOE remains unchanged. +REP with multiple retained listed jumps stays unsupported because no supported rule gives that supplied structure a numerical consequence.

A sole supplied * on one whole recognized element returns NoValue, distinct from a recognized zero row (Singles Technical Panel Handbook p. 2). Stars on listed components of supported units exclude their BV/reference contribution while retaining all technical calls for GOE. A single whole-element star mixed with other calls remains unsupported in the scalar path. The October 2026 Q&A 3, p. 1 Q2–3, requires calls and judge assessment for the entire executed combination even when later components are starred; do not silently delete those calls. No Pair scoring is introduced.

Two- and three-jump recognized combinations sum component rows and select the highest applicable component BV for a single GOE adjustment (Sports Rule 353.1(h)(i); SOV p. 14 remark 4). Later direct jumps remain restricted to single/double toe loops and loops; recognition does not establish category legality. The following Euler/sequence slice extends these recognized structures; program legality and automatic panel inference stay separate. Independent fixtures cover all 24 single/double Flip/Lutz edge/rotation pairs, every GOE grade, multiple marked components, the multiple-q special case, +REP rounding and reference GOE, three-jump reference selection, and no-value whole elements in timed and planning totals.

Internally, catalog, jump-unit and repetition provenance are typed separately. Catalog lookup, jump-unit aggregation, repetition BV reduction and GOE consistency are separate functions. No rule is selected from the display notation suffix. Combination BASE sources list every printed component value row; GOE sources identify the exact selected reference row and remark 4. Repetition sources cite the original row and remark 5. Judges-Handbook consistency references resolve the exact May 19 edition to its official PDF.

## Supplied Euler, sequence and component-deletion units (#46, third milestone)

The Singles Technical Panel Handbook pp. 28–29 establishes full sequence values, Euler as an unlisted zero-valued transition, one Euler between listed jumps, no foot change after it, and normal rotation/edge calls. Supplied recognition is the input: the scorer does not establish those facts from a plan or movement description. It accepts up to three listed single/double jumps plus one Euler, later direct T/Lo jumps, S/F after Euler, and an explicitly supplied sequence with a later Axel. Scalar numerical support does not establish Adult eligibility, once-per-program Euler legality or the sequence/combinations quota. The existing validator remains separate.

Sequence markers at the end of a complete unit or before its Axel connection are accepted. Multiple structural markers, an unmarked Axel transition, calls on Euler, misplaced/repeated Euler, too many listed jumps and unsupported direct transitions produce Unsupported. Unknown calls remain UnknownCalls.

TPH pp. 2, 29, 33 and Q&A 3 Q2–3 separate deleted value from whole-element execution. Explicitly starred listed components contribute no BV and do not provide a reference value; all of their edge/rotation calls remain in the GOE consistency check. The most difficult retained row supplies the one numerical GOE scale (Sports Rule 353.1(h)(i), SOV remark 4). Excluded sources are labelled as excluded rather than presented as a credited BASE. An all-deleted unit is NoValue; retained downgraded zero rows remain recognized zero.

For contextual +REP, only a first retained listed jump with all other listed jumps explicitly starred is supported. Apply 80% once to its applicable BV, round to hundredths, and retain its original GOE scale. A supplied trailing +SEQ with such stars can represent an excess-sequence panel decision; the scorer does not infer it from program order.

A +COMBO marker must precede only starred later components. A +SEQ before a deleted tail also records an interrupted/ambiguous connection. Their BV is deterministic from supplied calls, but exact element/program GOE requires a verified execution-error cause supplied through the additive API. A fall boolean alone does not locate the fall or identify other errors in the unit. This includes the single/double analogue of the Q&A 3 structure, 2S+SEQ+2T<*+2Aq*: the marked tail is preserved, not silently erased. [Execution evidence](scoring-execution-evidence.md) records the approved additive API and verified reductions for the final slice. The original entry points keep their incomplete outcomes.

## Scoring with explicit execution evidence

`score_element_with_execution`, `score_program_with_execution` and `score_planning_scenario_with_execution` accept separate `ElementExecutionEvidence`. Its errors are observed execution facts, not new panel marks. Unknown evidence preserves the old results. A known complete additional-error list resolves the fall status or rejects contradictory fall data. The count of full revolutions distinguishes a permitted connection with a GOE reduction from an interruption requiring supplied structural/deletion calls. Existing public functions and structs remain available unchanged.

The shared core applies verified extra reductions to the same numerical catalog/reference rows and timing algorithms. A matching observed cause enables an interrupted score; unknown or unsupported causes keep the total incomplete. The marker itself does not invent a GOE reduction. Execution evidence never changes supplied recognition, annotations, program order or the hypothetical final single-judge GOE. Weight transfer and more than one full revolution without a verified numerical classification remain unsupported under #56/#58. The one-revolution GOE rule is not extrapolated to multiple revolutions. See the [full contract and matrix](scoring-execution-evidence.md).
