# Manual category planning (#118)

Use the existing selection and assessment entry points as one workflow:

1. Select an exact `ProfileId` from `adult_profile` (Bronze, Silver, Gold,
   Masters) or `senior_profile` (Women, Men). Use its bound scoring table
   and the corresponding registry. Reject unknown serialized versions.
2. Call `selectable_elements` to show supported individual codes and levels.
   Individual availability does not establish compatibility with other choices.
   Call `selectable_jump_elements` for complete atomic jump choices, including
   compound connection intentions and explicit sequence assumptions.
3. Put selected codes into `ManualBvRequest.elements` using
   `AssumedRecognition::new`, then call `assess_manual_bv`. Supported compound
   codes use this same path. No GOE, duration, movement observations or panel
   calls are required. Sequences need an explicit direct-transition assumption.
4. Display each original element index, `table_base`, `credited_base`, and
   the aggregate. `exact_hundredths` is absent when any credit is unresolved;
   `resolved_hundredths` is only the resolved subtotal. Senior repeated
   advanced solos can have different listed and credited values. GOE and
   second-half factors do not enter these values.
5. Read composition findings separately. In a draft, `repairable_absences`
   indexes omissions that can still be completed. Complete submissions retain
   missing-role findings without classifying them as repairable draft omissions.
   Prohibited types/levels, unsupported codes and unknown codes have distinct
   selection errors and retain original indices. A numeric total alone does
   not establish a permitted composition.

This workflow checks intended technical composition within the reported scope.
It does not establish official judging compliance. Detailed evidence-based
assessment and immutable ordered completion retain their separate contracts.
See the [assumed-recognition contract](specs/assumed-recognition.md) for coverage,
transition assumptions and unresolved credit.

`tests/theoretical_manual_planning.rs` exercises real selection, validation and
contextual scoring for all six profiles with independent Adult/Rule 612
composition examples and Communication 2786 values. Existing `manual_bv`,
`element_selection`, `assumed_recognition` and completion tests cover additional
negative cases and compatibility. The [jump domain](specs/planned-jump-domain.md)
defines the complete compound grammar; maximum search belongs to its own #117 workstream.
