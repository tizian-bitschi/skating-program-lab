# Assumed recognition for program construction

Use an exact published SOV selection such as `CCoSp3` with the intended
`PlannedElement`. The selected `ProfileId` and `ScoringTableId` bind the
selection to a category, segment and SOV edition. `AssumedRecognition` does
not supply observations, GOE, verified timing, positions, feature attempts
or panel decisions.

## API migration

- For manual BV, call `assess_manual_program_with_assumed_recognition` with
  a `ProfileProgram` and one selection per element, in program order.
  Read `unfactored_base` for exact SOV values and source rows. Read
  `total_base_hundredths` for the supported contextual total when
  construction checks resolve, including repairable omissions in drafts.
- For just construction diagnostics, call
  `assess_program_legality_with_assumed_recognition` with the same inputs.
- In an optimizer build, pass explicit selections in manual variant order
  to `generate_program_candidates_with_assumed_recognition`. Automatic
  expansion and evidence templates belong to the existing generator.
  The result returns the accepted domain, input indices and matching
  selections in domain order.
- Use `optimize_program_with_assumed_recognition` for a finite variant
  domain and `complete_program_with_assumed_recognition` for additions to
  immutable originals. The latter receives one ordered selection list for
  originals and a second for additions. Variant indices and original IDs
  remain the stable references in returned placements.
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
assessment. The sequence code does not establish a direct step into the
Axel. Missing transition evidence remains visible.

Spin position holds, variation qualification, first attempts and first
successes, actual movement and execution, timing and panel recognition
remain outside this mode. Position, revolution and spin-feature filters
require the detailed inputs. A report covers only the listed construction
rules and cannot establish official ISU legality.

Senior manual totals apply the published 80% value for the second of two
identical advanced solo jumps in an otherwise supported repetition context.
The current ordered solver has an unfactored objective. It rejects assumed
domains containing repeated advanced solo choices before claiming an
optimum, including repetitions involving locked originals.
