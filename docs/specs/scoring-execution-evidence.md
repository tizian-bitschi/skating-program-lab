# Proposed execution evidence for interrupted jump units

Status: proposal only; public API approval required by AGENTS.md. No new public types or entry points are implemented in this milestone.

## Problem

The existing recognition can store `+COMBO`, `+SEQ`, starred components and component rotation/edge calls. `ProgramElement.fall` records only whether an element contains a fall. It cannot identify a step-out, weight transfer, excess rotation between jumps, Euler step-over or the component on which a fall occurred. TPH pp. 31–33 lists different interruption causes; Communication 2788 pp. 14, 16 and the Judges Handbook pp. 5–6 give their execution-specific reductions. Q&A 3 requires assessment of the entire executed unit, including deleted components.

The scorer currently returns deterministic BV for a supplied interrupted structure and `UnsupportedMarks` for exact GOE. A hypothetical final GOE does not supply the missing execution evidence needed to check its consistency. Do not infer the cause from the marker, a plan or free-text movement descriptions.

## Additive API decision

Keep existing public structs, functions and their incomplete results. Add separate observed evidence; do not add required fields to `ProgramElement` or reinterpret `PanelCall` as an observation. The proposed scalar interface is:

```rust
pub enum JumpExecutionError {
    Fall { component: usize },
    StepOut { component: usize },
    TouchDownWithWeightTransfer { component: usize },
    FullRevolutionBetween { before_component: usize },
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

Provide matching additive `score_program_with_execution` and `score_planning_scenario_with_execution` functions, with a separate slice of evidence aligned to the ordered elements. Reject a length mismatch, invalid component/transition targets, duplicate/conflicting evidence and inconsistent fall observations. Existing entry points continue using unknown execution evidence. No original plan, panel recognition or observation is mutated.

Before implementation, verify the exact reduction classification for weight transfer and excess turns from the identified editions. Preserve source-derived tests for single and cumulative errors, deleted components, no-value vs recognized zero, unknown evidence, timing and separate deductions. If a cause is not covered by the verified set, keep its score incomplete. This proposal does not authorize automatic panel inference, new categories or Pair scoring.
