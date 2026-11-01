//! Exhaustive reference for the fixed three-slot Bronze jump domain.
use crate::{
    lookup_base_value, validate_program, AdultCategory, BaseValue, BaseValueError,
    BaseValueOutcome, ComponentCode, Coverage, FindingKind, JumpElement, JumpKind, ListedJump,
    PanelRecognition, PlannedElement, Program, RecognitionDetails, RecognitionOutcome, RulesetId,
    ScoringTableId, Season,
};

/// Fixed assumptions for this exhaustive model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDomain {
    pub slot_start_seconds: [u16; 3],
    pub duration_seconds: u16,
    pub uninterrupted_timing_assumed: bool,
    pub no_event_exception_assumed: bool,
    pub clean_recognition_assumed: bool,
    pub exclude_lutz: bool,
}

/// Slot occupancy corresponds to the result's domain times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub slots: [bool; 3],
    pub program: Program,
    pub base_hundredths: u16,
    pub coverage: Coverage,
}

/// Complete enumeration within the documented domain, not official program feasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceOutcome {
    Optimal {
        base_hundredths: u16,
        candidates: Vec<ReferenceCandidate>,
    },
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceResult {
    pub domain: ReferenceDomain,
    pub outcome: ReferenceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    IncompleteValidation,
    UnsupportedElement,
    BaseValue(BaseValueError),
    InvalidLimit,
    UnsupportedVersion,
    InconsistentSolverResult,
}

const BRONZE_RULESET: RulesetId = RulesetId::IsuAdult2026_27July16;
const BRONZE_TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

fn bronze_domain(exclude_lutz: bool) -> ReferenceDomain {
    ReferenceDomain {
        slot_start_seconds: [10, 20, 30],
        duration_seconds: 100,
        uninterrupted_timing_assumed: true,
        no_event_exception_assumed: true,
        clean_recognition_assumed: true,
        exclude_lutz,
    }
}

#[cfg(feature = "optimizer")]
mod pumpkin;
#[cfg(feature = "optimizer")]
pub use pumpkin::*;

/// Enumerate every slot assignment. Exclusion removes 1Lz from every slot.
/// Lookup errors abort the search rather than producing an incomplete optimum.
pub fn enumerate_bronze_reference(exclude_lutz: bool) -> Result<ReferenceResult, ReferenceError> {
    let domain = bronze_domain(exclude_lutz);
    let ruleset = BRONZE_RULESET;
    let table = BRONZE_TABLE;
    let mut best = None;
    let mut candidates = Vec::new();
    for mask in 0..8 {
        let slots = std::array::from_fn(|i| mask & (1 << i) != 0);
        if !slots.iter().any(|&occupied| occupied) || (exclude_lutz && slots.contains(&true)) {
            continue;
        }
        let mut program = bronze_program(slots);
        let validation = validate_program(&program);
        if validation
            .findings
            .iter()
            .any(|f| f.kind != FindingKind::Violation)
        {
            return Err(ReferenceError::IncompleteValidation);
        }
        if !validation.findings.is_empty() {
            continue;
        }
        let mut total = 0;
        let mut sources = vec![crate::validation::bronze_jump_source()];
        for element in &program.elements {
            let value = reference_base_value(element, ruleset, table)?;
            total += value.hundredths;
            if !sources.contains(&value.source) {
                sources.push(value.source);
            }
        }
        program.sources = sources;
        let candidate = ReferenceCandidate {
            slots,
            program,
            base_hundredths: total,
            coverage: validation.coverage,
        };
        match best {
            None => {
                best = Some(total);
                candidates.push(candidate);
            }
            Some(previous) if total > previous => {
                best = Some(total);
                candidates.clear();
                candidates.push(candidate);
            }
            Some(previous) if total == previous => candidates.push(candidate),
            _ => {}
        }
    }
    Ok(ReferenceResult {
        domain,
        outcome: match best {
            Some(base_hundredths) => ReferenceOutcome::Optimal {
                base_hundredths,
                candidates,
            },
            None => ReferenceOutcome::Infeasible,
        },
    })
}

fn bronze_program(slots: [bool; 3]) -> Program {
    let elements = slots
        .iter()
        .filter(|&&occupied| occupied)
        .map(|_| {
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(1),
            }))
        })
        .collect();
    Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: AdultCategory::Bronze,
        ruleset: BRONZE_RULESET,
        scoring_table: BRONZE_TABLE,
        sources: vec![],
        elements,
    }
}

fn reference_base_value(
    element: &PlannedElement,
    ruleset: RulesetId,
    table: ScoringTableId,
) -> Result<BaseValue, ReferenceError> {
    let code = match element {
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1),
        })) => "1Lz",
        _ => return Err(ReferenceError::UnsupportedElement),
    };
    let recognition = PanelRecognition {
        ruleset,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from(code).expect("known code")],
                None,
                Some(vec![]),
            )
            .expect("known recognition"),
        ),
    };
    match lookup_base_value(table, &recognition, &[]).map_err(ReferenceError::BaseValue)? {
        BaseValueOutcome::Value(value) => Ok(value),
        _ => Err(ReferenceError::BaseValue(BaseValueError::Unsupported)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scoring_rejects_elements_outside_the_domain() {
        for element in [
            PlannedElement::Unspecified,
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Flip),
                rotations: Some(1),
            })),
        ] {
            assert_eq!(
                reference_base_value(
                    &element,
                    RulesetId::IsuAdult2026_27July16,
                    ScoringTableId::Isu2786June2_2026
                ),
                Err(ReferenceError::UnsupportedElement)
            );
        }
    }
}
