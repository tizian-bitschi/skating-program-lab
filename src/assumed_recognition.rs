//! SOV selections for hypothetical recognition in program construction.
use crate::program_validation::{validate_program_scope, EvaluationScope};
use crate::*;

/// Exact published notation selected for an intended element. The request's
/// profile and scoring table determine where the selection is evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumedRecognition {
    pub code: String,
}

fn spin_abbreviation(code: &str) -> Option<SpinAbbreviation> {
    let (code, _) = code.split_at(code.len().checked_sub(1)?);
    let (entry, code) = match code.strip_prefix('F') {
        Some(code) => (SpinEntry::Flying, code),
        None => (SpinEntry::NonFlying, code),
    };
    let change_foot = matches!(code, "CCSp" | "CCoSp" | "CSSp" | "CUSp" | "CLSp");
    let kind = match if change_foot { &code[1..] } else { code } {
        "CSp" => RegularSpinKind::Camel,
        "SSp" => RegularSpinKind::Sit,
        "USp" => RegularSpinKind::Upright,
        "LSp" => RegularSpinKind::Layback,
        "CoSp" => RegularSpinKind::Combination,
        _ => return None,
    };
    Some(SpinAbbreviation {
        kind,
        entry,
        change_foot,
    })
}

/// Values and diagnostics for one ordered plan under hypothetical recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumedProgramAssessment {
    pub program: ProfileProgram,
    pub recognition: Vec<AssumedRecognition>,
    pub submission: SubmissionState,
    /// Exact SOV rows in plan order, before program-context credit.
    pub unfactored_base: Vec<BaseValue>,
    /// Program BV after supported contextual credit, absent for a violation or unresolved rule.
    pub total_base_hundredths: Option<u32>,
    pub legality: ProgramLegalityReport,
}

impl AssumedRecognition {
    pub(crate) fn resolve(
        &self,
        table: ScoringTableId,
        element: &PlannedElement,
        ruleset: RulesetId,
    ) -> Option<(ElementPlanEvidence, BaseValue)> {
        if matches!(
            element,
            PlannedElement::Jump(JumpElement::Combination(_) | JumpElement::Sequence(_))
        ) {
            let evidence = ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            };
            let details = crate::plan_recognition::intended_details(element, &evidence)?;
            let value = crate::scoring::assumed_jump_unit_value(
                table,
                &details,
                &self.code,
                ruleset == RulesetId::IsuSenior2026_27June12,
            )?;
            return Some((evidence, value));
        }
        let row = catalog_row(table, &self.code)?;
        if row.discipline != CatalogDiscipline::SingleAndPair {
            return None;
        }
        let evidence = match (row.family, element) {
            (CatalogFamily::Jump, PlannedElement::Jump(JumpElement::Solo(jump)))
                if crate::plan_recognition::jump_component(jump)?.as_str() == row.notation =>
            {
                ElementPlanEvidence::Jump {
                    direct_transitions_assumed: false,
                }
            }
            (CatalogFamily::Spin, PlannedElement::RegularSpin(_)) => ElementPlanEvidence::Spin {
                spin: RegularSpinEvidence {
                    abbreviation: Some(spin_abbreviation(row.notation)?),
                    planned_level: Some(row.level?),
                    ..RegularSpinEvidence::default()
                },
                local_feature_qualification_assumed: false,
            },
            (CatalogFamily::StepSequence, PlannedElement::StepSequence) => {
                ElementPlanEvidence::Step {
                    level: row.level?,
                    clean_difficult_turns: None,
                    full_ice_coverage: None,
                    movement: None,
                    qualified_features: None,
                    qualification_assumed: false,
                }
            }
            (
                CatalogFamily::ChoreographicSequence,
                PlannedElement::ChoreographicSequence { .. },
            ) if row.notation == "ChSq1" => ElementPlanEvidence::ChoreographicSequence {
                movements: None,
                clearly_visible: None,
            },
            (CatalogFamily::ChoreographicSpin, PlannedElement::ChoreographicSpin)
                if row.notation == "ChSp1" =>
            {
                ElementPlanEvidence::ChoreographicSpin {
                    consecutive_revolutions_per_foot: None,
                }
            }
            _ => return None,
        };
        Some((
            evidence,
            BaseValue {
                hundredths: row.base_hundredths,
                table,
                notation: row.notation.into(),
                source: row.source.reference(row.notation),
            },
        ))
    }
}

/// Evaluate construction and list source-backed unfactored BV under the exact
/// assumptions supplied for the ordered plan.
pub fn assess_manual_program_with_assumed_recognition(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    recognition: &[AssumedRecognition],
    policy: &ProgramPolicy,
    submission: SubmissionState,
) -> Result<AssumedProgramAssessment, ProfileSelectionError> {
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    let invalid = |reason: String| ProfileSelectionError {
        profile: program.profile.clone(),
        kind: ProfileErrorKind::Invalid,
        reason,
    };
    if program.elements.len() != recognition.len() {
        return Err(invalid("Supply one SOV selection per element.".into()));
    }
    let (evidence, unfactored_base): (Vec<_>, Vec<_>) = program
        .elements
        .iter()
        .zip(recognition)
        .enumerate()
        .map(|(index, (element, assumption))| {
            assumption
                .resolve(
                    program.scoring_table,
                    element,
                    profile.scoring.recognition_ruleset,
                )
                .ok_or_else(|| {
                    invalid(format!(
                        "Element {index}: SOV code {:?} is unsupported or does not match the plan.",
                        assumption.code
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();
    let scope = EvaluationScope::AssumedRecognition;
    let validation = validate_program_scope(registry, program, &evidence, policy, scope)?;
    let legality = crate::legality::report_program_legality(
        registry, program, &evidence, policy, submission, validation, scope,
    )?;
    let total_base_hundredths = legality
        .validation
        .validation
        .findings
        .iter()
        .enumerate()
        .all(|(i, _)| legality.repairable_absences.contains(&i))
        .then(|| {
            let mut total = unfactored_base
                .iter()
                .map(|v| u32::from(v.hundredths))
                .sum::<u32>();
            if profile.scoring.recognition_ruleset == RulesetId::IsuSenior2026_27June12 {
                for (index, value) in unfactored_base.iter().enumerate() {
                    if repeated_advanced_solo(&program.elements, index) {
                        let base = u32::from(value.hundredths);
                        total -= base - (base * 8 + 5) / 10;
                    }
                }
            }
            total
        });
    Ok(AssumedProgramAssessment {
        program: program.clone(),
        recognition: recognition.to_vec(),
        submission,
        unfactored_base,
        total_base_hundredths,
        legality,
    })
}

/// Report covered construction rules under the supplied SOV selections.
pub fn assess_program_legality_with_assumed_recognition(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    recognition: &[AssumedRecognition],
    policy: &ProgramPolicy,
    submission: SubmissionState,
) -> Result<ProgramLegalityReport, ProfileSelectionError> {
    assess_manual_program_with_assumed_recognition(
        registry,
        program,
        recognition,
        policy,
        submission,
    )
    .map(|assessment| assessment.legality)
}

#[cfg(feature = "optimizer")]
pub(crate) fn has_repeated_advanced_solo<'a>(
    elements: impl Iterator<Item = &'a PlannedElement>,
) -> bool {
    let mut seen = Vec::new();
    for element in elements {
        if let PlannedElement::Jump(JumpElement::Solo(jump)) = element {
            if jump.rotations.is_some_and(|r| r >= 3) {
                if seen.contains(&jump) {
                    return true;
                }
                seen.push(jump);
            }
        }
    }
    false
}

fn repeated_advanced_solo(elements: &[PlannedElement], index: usize) -> bool {
    let PlannedElement::Jump(JumpElement::Solo(jump)) = &elements[index] else {
        return false;
    };
    jump.rotations.is_some_and(|r| r >= 3)
        && elements[..index].iter().any(|element| {
            matches!(element, PlannedElement::Jump(JumpElement::Solo(previous)) if previous == jump)
        })
}
