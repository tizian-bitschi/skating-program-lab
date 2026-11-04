# Senior numerical protocol replay (#78)

Public API approval: [accepted contract](issue-78-senior-scoring-proposal.md).
`src/scoring/panel.rs` adds `aggregate_panel_goe` and `score_protocol_program`.
Existing scalar GOE, program/manual assessment inputs and public struct layouts
remain unchanged. The new input contains separate judge positions, supplied
recognition, row provenance and printed x. Execution evidence uses the existing
`ElementExecutionEvidence` contract.

## Numerical source review

Reviewed on 2026-11-04 against the editions linked in
[rule sources](../rule-sources.md). Printed page numbers apply.

| Source | Supported numerical use |
| --- | --- |
| Sports Rules, June 12, Rule 353.1(b–h), pp. 9–10 | Map each grade to its numerical SOV column, trim one extreme at each end for panels of at least five; round the mean to hundredths. Printed x affects BV, not GOE. |
| Rule 612, p. 71 | Context distinguishes two identical advanced solos from a compound occurrence. Forbidden repetition contexts remain unresolved. |
| Communication 2786, June 2, pp. 3–5, 14 remarks 4–5 | Triple/quad call rows, compound reference selection and REP BV/reference separation. |
| Technical Panel Handbook, July 15, pp. 27–33 | Supplied recognition, rotation/edge marks, Euler, sequence and deletion semantics. |
| #76 protocol snapshot | Independent printed expectations; exact PDF edition, page, rank and row remain traceable. |

Arithmetic uses signed integer hundredths and a single nearest-hundredth mean
rounding, with exact half ties away from zero. Trimming sorts numerical values
and position together, deleting exactly one low and high value, including ties.
No mean grade is rounded or passed to the scalar scorer.

## Input and output requirements

`JudgePanelGrades.grades` contains one entry per submitted judge position.
Complete lists of one through nine grades in -5..=5 support numerical
aggregation. This does not establish whether that panel composition is permitted
at a particular event. Empty/oversized lists and any missing valued-row grade
return `UnsupportedPanelConfiguration`; out-of-range grades return `InvalidGrade`.
A missing position is never silently removed to fabricate a reduced panel.

`judges: None` requests BV only, leaving GOE, element total and exact TES absent.
Zero-value official rows accept absent grades or all-missing positions and
contribute a known zero. Submitted numerical grades on these rows are rejected.
No final printed GOE is reverse-engineered into judge grades.

Replay requires typed recognition. There is no new production notation parser.
`annotations: [SecondHalfCredit]` is the canonical supplied official x input.
`printed_second_half` is a convenience alias. Either or both apply credit once
and produce identical output/provenance; duplicate annotations are invalid.
Both forms are validated for every recognition outcome, including NoValue and
Ignored: known non-jumps cannot receive x, while starred jumps can.
`table_base` excludes REP and x;
credited BV applies supplied REP and x, with their separate sources. Technical
call rows already contain their source-defined BV reductions. Whole stars return
NoValue. No timestamp, fall or motion evidence is invented.

`score_protocol_program` preserves the supplied calls, including REP and stars.
It reuses profile resolution and the shared SOV/scoring resolver. Its result is
replay of official recognition, not an independent repetition-eligibility or
timing verdict. Ordered profile scoring and manual assessment separately derive
second-solo credit through `senior::repetition_context`. They share advanced
repeat and jump-kind limits with plan validation. A supplied REP alone proves no
eligibility. Senior solo REP is limited to triple/quad/quint recognition; single
and double solo REP return Unsupported. The TPH p. 33 excess-compound consequence
is distinct and can retain a first single/double jump with REP and a starred tail.
Legacy Adult supplied-REP behavior remains unchanged under #78's compatibility
requirement; it is not a Senior repetition-eligibility ruling.

Each replay row retains its `SourceReference` even on error. Any unresolved row
or missing nonzero-row panel prevents exact TES. PCS, deductions and complete
segment score are outside this report.

## Coverage and explicit unknowns

| Row/call | Evidence and support |
| --- | --- |
| Clean triple/quad | All applicable exact catalog rows; #76 printed examples |
| q, <, << | Scalar and panel paths; marked reference rows and contextual second solos |
| ! / e on F/Lz | Exact catalog applicability, including combined edge/rotation marks; e additionally tested with independent SOV expectations |
| Two-/three-jump units | Direct T/Lo, S/F after Euler, supplied Axel sequence; most difficult retained BV reference |
| REP | Supplied replay and separately derived second-solo credit; GOE reference remains unrepeated |
| * / NoValue | Whole/component deletion; #76 includes `2A*` and `3A<*` with absent grades |
| x | Printed official credit, with unverified start-time provenance |
| Spins / StSq / ChSp / ChSq | Shared existing resolver and submitted numerical panel |
| Missing grades on valued row | Explicit unsupported configuration; no guessed panel |
| Unknown calls/recognition, unsupported rows | Explicit row error and absent exact TES |
| Illegal/overlimit planned repeats | Plan violations; contextual scoring stays unresolved rather than inventing zero credit |
| Unsupported execution causes #56/#58 | Existing unsupported consequences remain explicit when supplied; no automatic adjudication |
| Quint calls, timing verification, PCS | No new support from #78 |

`tests/panel_scoring.rs` compares all 132 rows from twelve #76 programs with
independently transcribed BV, GOE and element totals, and sums per-program TES.
Focused SOV tests cover edge marks absent from the selected protocol snapshot,
numerical trimming/rounding, reduced and missing panels, and repeated-solo versus
compound and forbidden-repeat boundaries. Existing Adult fixtures are unchanged.
Review regressions cover invalid solo REP, x on zero-value non-jumps, credit
alias normalization, and component-star reference selection. Component-star
expectations use independent SOV values and the handbook/Q&A deletion rule;
the selected #76 snapshot contains no such component-star observation.
