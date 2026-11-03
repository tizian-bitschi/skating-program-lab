# Immutable completion in the ordered optimizer

Accepted: 2026-11-03, public interface approved by the user for issue #66.

The completion workflow needs stable original identities, relative order and
explicit allowed gaps. Mandatory candidate presence cannot express those facts.

Extend the existing ordered-slot Pumpkin search with one channelled position per
original entry, increasing positions, optional absolute positions and forbidden
gap constraints. Retain the same scorer, validator, budgets and optimality loop.
Do not add a second solver or pre-enumerate completed words. Completion owns
manual-entry preservation, typed input conflicts and ordered provenance.

Preserve every original entry, including unlocked entries; editing/deletion is a
separate caller action. Addition exclusions cannot remove originals. Whole-plan
validation governs repetitions and ordered feature use. Reassessment through #64
checks numerical consistency. Deduplicate by visible ordered entry semantics,
including recognition and scenarios, before applying the result cap.

The finite-domain and supported-rule limits of the existing optimizer remain.
Unknown original evidence prevents a proof; it is not a fabricated infeasibility.
