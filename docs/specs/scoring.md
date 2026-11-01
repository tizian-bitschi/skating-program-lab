# Scoring

## Target contract


The user supplies the recognized code and level, calls, final **single hypothetical judge's integer GOE** from -5 to +5, fall location, program duration and selected rule/table version. The user can provide observed execution separately. This is a scenario calculation, not an actual panel result; panel GOE is a trimmed mean of judges' numerical GOE values and is outside the MVP. Inputs are checked for supported codes, categories, mutually exclusive rotation calls (`q`, `<`, `<<`), edge calls (`!`, `e`), supported combinations and applicable GOE restrictions. An edge call on an inapplicable jump or contradictory marks yields an error; missing or undecidable evidence is unknown. An error must not silently change supplied calls or GOE. For a fall in an element, Communication 2788 pp. 14 and 16 gives a -5 reduction and a maximum starting GOE of +2; a supplied final single-judge GOE above -3 is inconsistent. Other aspects can lower the final GOE; a fall does not automatically make it -5.

For supported inputs, use the recognized element and call-specific entry in the selected Scale of Values, apply the corresponding GOE table value and prescribed rounding, and display base value, adjustments, GOE points and element score. `q` retains base value, `<` selects the under-rotated value, `<<` the downgraded value; `!` retains base value and `e` selects the applicable edge value. Their GOE effects are assessed separately; calls alone do not determine a final GOE. For combinations use the relevant table and reference-jump rule rather than a blanket percentage formula. The 1.1 second-half factor requires the jump-element start time and applicable interruption facts; see [sources](../rule-sources.md). Apply the general rule as the source-based default and check for any event-specific Adult exception. Missing timing or an unresolved applicable exception makes the affected result incomplete. Sum element scores to TES. Show fall and time deductions separately; do not subtract them from TES. A later total-segment score requires separately calculated PCS and deductions. If an exact rule, rounding step or table is not supported, report the calculation as incomplete.


## Acceptance examples

These are target examples. Existing tests cover some cases only in part; see [implementation status](../implementation-status.md). Other findings may coexist. See [rule sources](../rule-sources.md) for source editions. Independent human domain review is required before claiming complete correctness.

4. **Table values:** Using the 2 June 2786 update, a recognized solo 1Lz started in the first half at final GOE +1 has BV 0.60, GOE +0.06 and element score 0.66; 1Lzq retains BV 0.60, 1Lz< has 0.48, 1Lz<< has 0.00, 1Lz! has 0.60, 1Lze has 0.48 and 1Lze< has 0.36 (p. 2). These are unfactored table BVs. Their error-related GOE reductions are not inferred from BV. A recognized ChSp1 with GOE +1 has BV 3.50, GOE +0.50 and score 4.00; CSSp1 with +1 has BV 2.30, GOE +0.23 and score 2.53 (pp. 8 and 14). No fall is assumed in these examples.

5. **Falls and time:** In Bronze, a fall inside a recognized element gives no separate fall deduction; with a supplied final single-judge GOE of -3 the applicable table value is used, while +1 is inconsistent with the fall's -5 reduction and starting cap. An outside fall adds 0.50 to separate deductions. A Bronze duration of 1:50 is within tolerance; 1:51 incurs a 1.00 time deduction, assuming no other time-related consequence (Adult p. 11; Sports Rules 353.1(n), 502; 2788 pp. 14, 16). TES remains the element-score sum.

10. **Silver duration:** 2:10 remains within tolerance; 2:11 produces a 1.00 separate time deduction, assuming no additional timing consequence. A jump eligible for the second-half factor without timing data or with an unresolved applicable exception must produce an incomplete affected result, not a silently unadjusted exact total.


## Implemented subset

See [implementation status](../implementation-status.md#scoring) for supported rows and limitations.
