# Assumed recognition for program construction

Use an exact published SOV selection such as `CCoSp3` with the intended
`PlannedElement`. The selected `ProfileId` and `ScoringTableId` bind the
selection to a category, segment and SOV edition. `AssumedRecognition` does
not supply observations, GOE, verified timing, positions, feature attempts
or panel decisions.

## API migration

- For manual BV, call `assess_manual_bv` with a `ManualBvRequest` containing
  the profile, table, submission state and ordered selections. Each result
  carries its original index, table BV when available, and contextual BV or
  an error. The subtotal includes resolved credit only, and an exact total
  requires every element to resolve. Read composition findings separately
  from the numeric result. Draft omissions that can still be repaired are
  listed by finding index.
- Use `assess_manual_program_with_assumed_recognition` if you already have
  a typed `ProfileProgram` and explicit policy. That entry point returns
  unfactored values and reports an exact total only when construction
  checks resolve, allowing repairable draft omissions.
- For just construction diagnostics, call
  `assess_program_legality_with_assumed_recognition` with the same inputs.
- In an optimizer build, pass the exact allowed individual SOV codes to
  `generate_program_candidates_with_assumed_recognition` with an empty
  `manual_variants` list. Each selected level is a separate choice, including
  Basic when published. Profile-ineligible types and levels are audited as
  omissions. The result returns the accepted domain, original selection indices
  and matching selections in domain order. For compound jump units, supply
  typed manual variants and their exact selections in corresponding order.
- Use `optimize_program_with_assumed_recognition` for a finite variant
  domain and `complete_program_with_assumed_recognition` for additions to
  immutable originals. The latter receives one ordered selection list for
  originals and a second for additions. Variant indices and original IDs
  remain the stable references in returned placements. The completion request
  preserves the original entries and their detailed evidence. The draft and
  completed assessments contain the separate SOV-resolved evidence used for
  their BV calculations. Use provenance to look up the unchanged original
  entry in the returned request.
- Existing entry points keep the evidence-based contract. Use them when
  the task requires technical position, movement, revolution, feature
  reuse or level-credit checks.

## Evaluated scope

The shared validator checks profile family eligibility, jump composition
and repetition, program and family counts, regular-spin roles and
abbreviation distinctness, step and spin level caps, and required
Axel, spin and movement families where applicable. A selected code
above a cap is reported as a construction violation. It is not downgraded.
Selections cover supported clean singles jump units, regular spin Basic
and levels 1–4, StSq Basic and levels 1–4, and fixed ChSq1 and ChSp1.
The exact catalog row must exist for the selected edition. Compound
notation is resolved through the same component scoring path as detailed
assessment. For a sequence, supply a direct-step assumption through
`AssumedRecognition::direct_transitions_assumed` or the existing jump evidence
on a variant or locked original. The sequence code alone leaves the direct
step unresolved. Coverage reports an explicit direct-step assumption as
conditional.

Spin position holds, variation qualification, first attempts and first
successes, actual movement and execution, timing and panel recognition
remain outside this mode. Position, revolution and spin-feature filters
require the detailed inputs. A report covers only the listed construction
rules and cannot establish official ISU legality. In optimizer results,
`supported_rules` lists the active scope; `applied_rules` records the rules
actually evaluated for returned candidates. Execution checks excluded by
this scope remain available through the evidence-based APIs.

Senior manual totals apply the published 80% value for the second of two
identical advanced solo jumps in an otherwise supported repetition context.
Both manual entry points share a credit resolver and the Rule 612
repetition decision used by the detailed Senior scorer. Known illegal
compound structures, excess Senior jump slots and uncertain repetition
credit leave affected values unresolved. An unknown earlier jump can
consume a repetition slot. The listed value of a known compound is
retained even when its credit is unresolved.
Both assumed-recognition search APIs maximize credited BV over the supplied
finite domain. A second identical advanced solo can receive 80% BV, including
when the first or second attempt is a locked original. If the repeated jump
appears in a combination or sequence, the two-solo reduction does not apply.
The shared validator still checks repetition and composition limits.
The returned per-element values include the contextual notation (such as
`3A+REP`), while the rule trace retains the catalog and Rule 612 sources.
The original assumed selections stay unchanged. If credit cannot be resolved,
the search reports an error rather than substituting a table value or zero.

Pumpkin's listed sum is an upper bound on credited BV; it does not determine
which validated word wins. Optimality, completeness of ordered ties, and
coverage of the declared variant domain are separate claims. Evidence-based
entry points retain their unfactored objective and detailed validation scope.

## Profile-filtered choices

Call `selectable_elements` with the exact profile and SOV edition to list
individual options in published table order. Each option identifies its
semantic kind and exact catalog row, including its published level when
applicable. Fixed codes such as ChSp1 and ChSq1 are not level ranges.
Reduced variants and pair rows are outside this selection contract.
`check_selectable_element` checks a saved code against the same rules and
distinguishes an unknown code, an unsupported row, a disallowed type and a
credited level above the profile cap. Type restrictions take precedence.

Manual BV assessment flags unavailable selections at their original
indices without substituting a different row. It retains known composition
findings and the BV of independent selections. A listed row can be
available even when its credit is unresolved. Known zero, NoValue,
Ignored and unresolved credit have distinct representations. As in detailed
manual assessment, only numeric Value outcomes enter the numeric subtotal.
Clean manual selections do not generate panel NoValue or Ignored decisions. Actual timing,
GOE scenarios and protocol replay use separate entry points.
Assumed candidate generation rejects unavailable manually supplied variants. The separate
legality report still reports construction findings for a supplied plan.
An available choice does not reserve a slot or satisfy any required role;
those checks need the whole program. Compound jump units continue through
the existing compound scoring and validation path.

For a registered profile that uses supported rule mechanisms and supplies
its own program policy, use `selectable_elements_with_policy` and
`check_selectable_element_with_policy`. Both use the same checks as manual BV
assessment. The explicit policy must be bound to that profile; it cannot
override a built-in policy.

## Planned V combination spins

Deliberately planned two-basic-position combination spins accept the published
CoSp, CCoSp, FCoSp and FCCoSp V rows at category-permitted levels. The exact
V row supplies the reduced BV. The ordinary abbreviation determines type
eligibility, required roles and distinctness, so full/V pairs cannot bypass
a repeated-spin prohibition. This selection infers neither execution errors
nor a level downgrade. Other execution-related V forms remain outside the
planned selection domain. Detailed evidence-based assessment is unchanged.

