# Execution evidence for interrupted jump units

Status: additive API approved by the user on 2026-11-02 and implemented in the fourth #46 milestone. Existing public structs, enum variants and entry points retain their contracts.

## Problem

The existing recognition can store `+COMBO`, `+SEQ`, starred components and component rotation/edge calls. `ProgramElement.fall` records only whether an element contains a fall. It cannot identify a step-out, weight transfer, excess rotation between jumps, Euler step-over or the component on which a fall occurred. TPH pp. 31–33 lists different interruption causes; Communication 2788 pp. 14, 16 and the Judges Handbook pp. 5–6 give their execution-specific reductions. Q&A 3 requires assessment of the entire executed unit, including deleted components.

The original entry points return deterministic BV for a supplied interrupted structure and `UnsupportedMarks` for exact GOE without execution evidence. A hypothetical final GOE does not supply the missing execution evidence needed to check its consistency. Do not infer the cause from the marker, a plan or free-text movement descriptions.

## Additive API decision

Keep existing public structs, functions and their incomplete results. Add separate observed evidence; do not add required fields to `ProgramElement` or reinterpret `PanelCall` as an observation. The scalar interface is:

```rust
pub enum JumpExecutionError {
    Fall { component: usize },
    StepOut { component: usize },
    TouchDownWithWeightTransfer { component: usize },
    FullRevolutionBetween { before_component: usize, full_revolutions: u8 },
    EulerStepOver { component: usize },
}

pub struct ElementExecutionEvidence {
    // None: relevant errors unknown. Some(empty): explicitly no relevant errors.
    pub errors: Option<Vec<JumpExecutionError>>,
}

pub fn score_element_with_execution(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    goe: i8,
    fall: Option<bool>,
    execution: &ElementExecutionEvidence,
) -> Result<ElementScore, ElementScoreError>;
```

The matching additive `score_program_with_execution` and `score_planning_scenario_with_execution` functions use a separate slice of evidence aligned to the ordered elements. Reject a length mismatch, invalid component/transition targets, duplicate/conflicting evidence and inconsistent fall observations. Existing entry points continue using unknown execution evidence. A known error list also supplies the fall status: None on the old fall argument can be resolved, but a contradictory Some value returns InconsistentFall. Unsupported targets, duplicates, ambiguous multiple landing labels on one component and invalid/contradictory structure evidence return UnsupportedMarks. No original plan, panel recognition or observation is mutated.

## Verified consequences and limits

| Additional observation | Minimum reduction / starting cap | Source and boundary |
| --- | --- | --- |
| Located fall | -5 per separately located fall; cap +2 | 2788 pp. 14, 16. The fall boolean is corroborated, never added as another fall |
| Step-out | -3; cap +2 | 2788 p. 14. A followed step-out must match a supplied interrupted/starred tail |
| One full revolution between jumps | -3; cap +2 | Judges Handbook p. 5: full revolution / two three turns. It does not prove an interruption |
| More than one full revolution | -3; cap +2, with a matching interrupted/starred tail | TPH p. 31 plus Judges Handbook p. 5. The count is explicit so one revolution is not misclassified |
| Euler step-over | -2; ordinary single-error cap +4, or +2 with multiple errors | 2788 p. 16; Judges Handbook pp. 2, 5. The target must be the supplied Euler |
| Touch-down with weight transfer alone | UnsupportedMarks | No unambiguous numerical class is verified from this observation alone; [follow-up #56](https://github.com/tizian-bitschi/skating-program-lab/issues/56) |

Reductions combine with supplied rotation/edge calls, including calls on deleted components. Multiple q uses its separate -3..-4 row. Final grade limits clamp at -5. Independent fixtures cover source numbers, invalid targets, unknown vs explicitly empty evidence, inconsistent fall observations, duplicate/conflicting evidence, cumulative errors, retained reference GOE, contextual +REP, the nonterminal +SEQ/deleted-Axel ambiguity, and timed/planning credit with separate deductions.

A concrete interruption cause must explain the actual supplied boundary. Later errors within an already starred tail remain relevant but do not explain an earlier interruption. A step-out/fall or more than one full revolution followed by active credited jumps contradicts the supported supplied recognition and is rejected. No panel call or star is inserted or removed.

Planning rejects a length mismatch with InvalidInput. Timed scoring returns no TES, per-element InvalidInput and invalid-input deductions for a length mismatch. Invalid evidence on one element keeps the relevant program total incomplete; unaffected supported elements retain their scores. Unknown evidence has the same results as the original entry points.

This API does not authorize automatic panel inference, new categories, Pair scoring or a complete execution-quality evaluator. Unmodelled quality remains the responsibility of the supplied hypothetical judge. Ambiguous repeated physical errors on one component need finer evidence; they are not silently double-counted.
