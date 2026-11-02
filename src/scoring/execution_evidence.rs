use super::ElementScoreError;
use crate::{
    CallTarget, ElementExecutionEvidence, JumpExecutionError, PanelCall, PanelRecognition,
};

pub(super) struct Adjustment {
    pub minimum_reduction: i16,
    pub errors: usize,
    pub starting_cap: i16,
    pub fall_count: u8,
    pub break_positions: Vec<usize>,
    pub handbook_source: bool,
}

fn target(error: &JumpExecutionError) -> usize {
    match *error {
        JumpExecutionError::Fall { component }
        | JumpExecutionError::StepOut { component }
        | JumpExecutionError::TouchDownWithWeightTransfer { component }
        | JumpExecutionError::EulerStepOver { component } => component,
        JumpExecutionError::FullRevolutionBetween {
            before_component, ..
        } => before_component,
    }
}

fn landing_error(error: &JumpExecutionError) -> bool {
    matches!(
        error,
        JumpExecutionError::Fall { .. }
            | JumpExecutionError::StepOut { .. }
            | JumpExecutionError::TouchDownWithWeightTransfer { .. }
    )
}

pub(super) fn validate(
    recognition: &PanelRecognition,
    fall: Option<bool>,
    evidence: &ElementExecutionEvidence,
) -> Result<Option<Adjustment>, ElementScoreError> {
    let Some(errors) = evidence.errors.as_ref() else {
        return Ok(None);
    };
    let components = recognition
        .details
        .as_ref()
        .map(|details| details.components())
        .unwrap_or(&[]);
    if !errors.is_empty() && components.len() > 4 {
        return Err(ElementScoreError::UnsupportedMarks);
    }
    let calls = recognition
        .details
        .as_ref()
        .and_then(|details| details.calls())
        .unwrap_or(&[]);
    let mut result = Adjustment {
        minimum_reduction: 0,
        errors: 0,
        starting_cap: 5,
        fall_count: 0,
        break_positions: vec![],
        handbook_source: false,
    };
    for (i, error) in errors.iter().enumerate() {
        let index = target(error);
        if index >= components.len()
            || errors[..i].iter().any(|previous| {
                target(previous) == index
                    && (std::mem::discriminant(previous) == std::mem::discriminant(error)
                        || (landing_error(previous) && landing_error(error)))
            })
        {
            return Err(ElementScoreError::UnsupportedMarks);
        }
        let jump = components[index].as_str().starts_with(char::is_numeric)
            || components[index].as_str() == "Eu";
        match *error {
            JumpExecutionError::Fall { .. } => {
                result.fall_count += 1;
                if jump {
                    result.break_positions.push(index + 1);
                }
            }
            JumpExecutionError::StepOut { .. } => {
                if !jump {
                    return Err(ElementScoreError::UnsupportedMarks);
                }
                // Communication 2788 p. 14: -3..-4 and grey starting cap +2.
                result.minimum_reduction += 3;
                result.starting_cap = 2;
                result.errors += 1;
                result.break_positions.push(index + 1);
            }
            JumpExecutionError::TouchDownWithWeightTransfer { .. } => {
                // TPH establishes the interruption, not an unambiguous numeric GOE class.
                return Err(ElementScoreError::UnsupportedMarks);
            }
            JumpExecutionError::FullRevolutionBetween {
                full_revolutions, ..
            } => {
                if index == 0
                    || full_revolutions == 0
                    || !jump
                    || !components[index - 1].as_str().starts_with(char::is_numeric)
                        && components[index - 1].as_str() != "Eu"
                {
                    return Err(ElementScoreError::UnsupportedMarks);
                }
                // Judges Handbook p. 5: full revolution / two three turns, -3..-4.
                result.handbook_source = true;
                result.minimum_reduction += 3;
                result.starting_cap = 2;
                result.errors += 1;
                if full_revolutions > 1 {
                    result.break_positions.push(index);
                }
            }
            JumpExecutionError::EulerStepOver { .. } => {
                if components[index].as_str() != "Eu" {
                    return Err(ElementScoreError::UnsupportedMarks);
                }
                // Communication 2788 p. 16; Judges Handbook p. 5: -2..-3.
                result.handbook_source = true;
                result.minimum_reduction += 2;
                result.errors += 1;
            }
        }
    }
    let observed_fall = result.fall_count > 0;
    if fall.is_some_and(|value| value != observed_fall) {
        return Err(ElementScoreError::InconsistentFall);
    }
    for boundary in &result.break_positions {
        if *boundary >= components.len() {
            continue;
        }
        // TPH pp. 31, 33: calls before the interruption + COMBO/SEQ + starred tail.
        let covered = calls.iter().any(|entry| match entry.target {
            CallTarget::BetweenComponents(at)
                if at <= *boundary
                    && at < components.len()
                    && matches!(entry.call, PanelCall::Combination | PanelCall::Sequence) =>
            {
                (at..components.len()).all(|index| super::component_is_starred(calls, index))
            }
            _ => false,
        });
        if !covered {
            return Err(ElementScoreError::UnsupportedMarks);
        }
    }
    Ok(Some(result))
}
