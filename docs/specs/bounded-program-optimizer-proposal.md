# Issue 49: implementation contract for approval

Status: additive API and Option A approved by the user on 2026-11-03. The implemented bounded contract is in bounded-program-optimizer.md.
Source and target: origin/main at 36f62dba877199afd4c7e8067f293fab8fe26ef4.

## Observed dependency gap

Issue 48 is closed, but main documents and implements only regular-spin validation slice 1. `docs/specs/regular-spin-validation.md` explicitly leaves program-wide feature credit/reuse to slice 2 and StSq/ChSq/ChSp recognition and category requirements to slice 3. These checks are required by issue 49. Scoring support alone does not establish plan eligibility or feature recognition.

## Proposed additive public API

Keep existing jump optimizer and plan/profile types compatible. Add `optimize_program` under the existing optimizer feature, with a dedicated `ProgramOptimizationRequest` and `ProgramOptimizationResult`. Use the existing profile registry and explicit regular-spin policy; do not select custom-profile policy by Adult category name.

The request carries exact season/profile/table identity; a finite list of candidate variants; maximum ordered program length; typed personal exclusions and must-have constraints; unfactored base-value objective; ordered-element equivalence; time, proposal and solution limits. Each candidate supplies its existing planned element, typed validation evidence and explicitly assumed panel recognition. Reject unsupported evidence/restrictions rather than discarding them. Do not add timestamps.

The result records the resolved domain, excluded and unsupported variants, per-element values, total objective, profile/table/source editions, applied rule IDs, assumptions and coverage gaps. Keep optimality/infeasibility/search interruption separate from completeness of tie enumeration. All claims apply only to the explicit finite supported model.

## Implementation boundaries

Reuse Pumpkin slot/value modeling and existing solver-common helpers, shared profile validation and catalog-backed scoring. Reindex candidate evidence when constructing each ordered program. Validate whole-program interactions before accepting solver assignments and independently recompute every returned objective. Do not duplicate source tables or category rules.

A separate tiny exhaustive test oracle must enumerate ordered mixed programs independently of Pumpkin. Cover all four Adult profiles plus the synthetic profile, mandatory/forbidden elements, competing once-per-program features, ability exclusions, impossible requests, stopped searches and capped ties.

Full acceptance requires completing the missing shared validation prerequisites first. A narrower mixed jump/regular-spin milestone must explicitly exclude unsupported feature and choreographic claims and must not close issue 49.

## Approval

The user approved the additive `optimize_program` interface and Option A, including missing shared validator prerequisites and the Chrome review workflow. Unsupported movement/recognition cases remain explicitly outside the finite supported domain, as required by issue 49.
