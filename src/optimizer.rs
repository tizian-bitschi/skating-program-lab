//! Exhaustive reference for the fixed three-slot Bronze jump domain.
use crate::{
    lookup_base_value, validate_program, AdultCategory, BaseValueError, BaseValueOutcome,
    ComponentCode, Coverage, FindingKind, JumpElement, JumpKind, ListedJump, PanelRecognition,
    PlannedElement, Program, RecognitionDetails, RecognitionOutcome, RulesetId, ScoringTableId,
    Season,
};

/// Slot positions correspond to starts at 10, 20 and 30 seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub slots: [bool; 3],
    pub program: Program,
    pub base_hundredths: u16,
    pub coverage: Coverage,
}

/// Complete enumeration within the documented domain, not official program feasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceResult {
    Optimal {
        base_hundredths: u16,
        candidates: Vec<ReferenceCandidate>,
    },
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    IncompleteValidation,
    BaseValue(BaseValueError),
}

/// Enumerate every slot assignment. Exclusion removes 1Lz from every slot.
/// Lookup errors abort the search rather than producing an incomplete optimum.
pub fn enumerate_bronze_reference(exclude_lutz: bool) -> Result<ReferenceResult, ReferenceError> {
    let ruleset = RulesetId::IsuAdult2026_27July16;
    let table = ScoringTableId::Isu2786June2_2026;
    let mut best = None;
    let mut candidates = Vec::new();
    for mask in 0..8 {
        let slots = std::array::from_fn(|i| mask & (1 << i) != 0);
        if !slots.iter().any(|&occupied| occupied) || (exclude_lutz && slots.contains(&true)) {
            continue;
        }
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
        let program = Program {
            season: Season {
                start_year: 2026,
                end_year: 2027,
            },
            category: AdultCategory::Bronze,
            ruleset,
            scoring_table: table,
            sources: vec![],
            elements,
        };
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
        let recognition = PanelRecognition {
            ruleset,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: Some(
                RecognitionDetails::new(
                    vec![ComponentCode::try_from("1Lz").expect("known code")],
                    None,
                    Some(vec![]),
                )
                .expect("known recognition"),
            ),
        };
        let mut total = 0;
        for _ in &program.elements {
            match lookup_base_value(table, &recognition, &[]).map_err(ReferenceError::BaseValue)? {
                BaseValueOutcome::Value(value) => total += value.hundredths,
                _ => return Err(ReferenceError::BaseValue(BaseValueError::Unsupported)),
            }
        }
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
    Ok(match best {
        Some(base_hundredths) => ReferenceResult::Optimal {
            base_hundredths,
            candidates,
        },
        None => ReferenceResult::Infeasible,
    })
}
