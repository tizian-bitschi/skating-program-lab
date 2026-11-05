//! SOV selections for hypothetical recognition in program construction.
use crate::program_validation::EvaluationScope;
use crate::*;

/// Exact published notation selected for an intended element. The request's
/// profile and scoring table determine where the selection is evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumedRecognition {
    pub code: String,
    /// Caller assumption for a direct step into the Axel in a jump sequence.
    pub direct_transitions_assumed: bool,
}

pub(crate) fn spin_abbreviation(code: &str) -> Option<SpinAbbreviation> {
    let (code, _) = code.split_at_checked(code.len().checked_sub(1)?)?;
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
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            direct_transitions_assumed: false,
        }
    }

    pub(crate) fn resolve(
        &self,
        table: ScoringTableId,
        element: &PlannedElement,
        ruleset: RulesetId,
        supplied: Option<&ElementPlanEvidence>,
    ) -> Option<(ElementPlanEvidence, BaseValue)> {
        let sequence = matches!(element, PlannedElement::Jump(JumpElement::Sequence(_)));
        if self.direct_transitions_assumed && !sequence {
            return None;
        }
        if matches!(
            element,
            PlannedElement::Jump(JumpElement::Combination(_) | JumpElement::Sequence(_))
        ) {
            let evidence = ElementPlanEvidence::Jump {
                direct_transitions_assumed: sequence
                    && (self.direct_transitions_assumed
                        || matches!(
                            supplied,
                            Some(ElementPlanEvidence::Jump {
                                direct_transitions_assumed: true
                            })
                        )),
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
        resolve_individual_row(row, element, supplied)
    }
}

pub(crate) fn resolve_individual_row(
    row: &CatalogRow,
    element: &PlannedElement,
    supplied: Option<&ElementPlanEvidence>,
) -> Option<(ElementPlanEvidence, BaseValue)> {
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
        (CatalogFamily::StepSequence, PlannedElement::StepSequence) => ElementPlanEvidence::Step {
            level: row.level?,
            clean_difficult_turns: None,
            full_ice_coverage: None,
            movement: None,
            qualified_features: None,
            qualification_assumed: false,
        },
        (CatalogFamily::ChoreographicSequence, PlannedElement::ChoreographicSequence { .. })
            if row.notation == "ChSq1" =>
        {
            ElementPlanEvidence::ChoreographicSequence {
                movements: match supplied {
                    Some(ElementPlanEvidence::ChoreographicSequence { movements, .. }) => {
                        movements.clone()
                    }
                    _ => None,
                },
                clearly_visible: None,
            }
        }
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
            table: row.source.table,
            notation: row.notation.into(),
            source: row.source.reference(row.notation),
        },
    ))
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
    assess_assumed_program(registry, program, recognition, policy, submission, true)
}

fn assess_assumed_program(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    recognition: &[AssumedRecognition],
    policy: &ProgramPolicy,
    submission: SubmissionState,
    check_selection: bool,
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
    let selection = check_selection
        .then(|| {
            crate::selection::Selection::for_policy(
                registry,
                profile,
                program.scoring_table,
                policy,
            )
        })
        .transpose()?;
    let (evidence, unfactored_base): (Vec<_>, Vec<_>) = program
        .elements
        .iter()
        .zip(recognition)
        .enumerate()
        .map(|(index, (element, assumption))| {
            if let Some(selection) = &selection {
                selection
                    .check_assumption(element, assumption)
                    .map_err(|error| {
                        invalid(format!(
                            "Element {index}: {}",
                            error.reason(&assumption.code)
                        ))
                    })?;
            }
            assumption
                .resolve(
                    program.scoring_table,
                    element,
                    profile.scoring.recognition_ruleset,
                    None,
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
    let request = ManualProgramRequest {
        profile: program.profile.clone(),
        scoring_table: program.scoring_table,
        policy: policy.clone(),
        submission,
        entries: program
            .elements
            .iter()
            .cloned()
            .zip(evidence.iter().cloned())
            .map(|(planned, evidence)| ManualProgramEntry {
                planned,
                evidence,
                intended_recognition: None,
                scenario: None,
            })
            .collect(),
        request_hypothetical_total: false,
        second_half_boundary: None,
    };
    let assessment =
        crate::assessment::assess_manual_scope(registry, &request, Some(&unfactored_base))?;
    let scope = EvaluationScope::AssumedRecognition;
    let legality = crate::legality::report_program_legality(
        registry,
        program,
        &evidence,
        policy,
        submission,
        assessment.validation,
        scope,
    )?;
    let total_base_hundredths = legality
        .validation
        .validation
        .findings
        .iter()
        .enumerate()
        .all(|(i, _)| legality.repairable_absences.contains(&i))
        .then(|| {
            assessment
                .intended_base
                .exact_hundredths
                .and_then(|total| u32::try_from(total).ok())
        })
        .flatten();
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
    assess_assumed_program(registry, program, recognition, policy, submission, false)
        .map(|assessment| assessment.legality)
}

#[cfg(feature = "optimizer")]
fn advanced_solo(element: &PlannedElement) -> Option<&ListedJump> {
    match element {
        PlannedElement::Jump(JumpElement::Solo(jump)) if jump.rotations.is_some_and(|r| r >= 3) => {
            Some(jump)
        }
        _ => None,
    }
}

#[cfg(feature = "optimizer")]
pub(crate) fn has_repeated_advanced_solo<'a>(
    elements: impl Iterator<Item = &'a PlannedElement>,
) -> bool {
    let mut seen = Vec::new();
    elements.filter_map(advanced_solo).any(|jump| {
        let repeated = seen.contains(&jump);
        seen.push(jump);
        repeated
    })
}

/// Reuse the Senior scorer's repetition decisions with clean intended details.
pub(crate) fn senior_context(
    elements: &[PlannedElement],
    evidence: &[ElementPlanEvidence],
) -> Vec<Option<PanelRecognition>> {
    let panels: Vec<_> = elements
        .iter()
        .zip(evidence)
        .map(|(element, evidence)| {
            if matches!(element, PlannedElement::Jump(JumpElement::Sequence(_)))
                && matches!(
                    evidence,
                    ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false
                    }
                )
            {
                return None;
            }
            Some(PanelRecognition {
                ruleset: RulesetId::IsuSenior2026_27June12,
                source: None,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Recognized,
                details: Some(crate::plan_recognition::intended_details(
                    element, evidence,
                )?),
            })
        })
        .collect();
    crate::senior::repetition_context(&panels)
}

pub(crate) fn contextual_base(
    table: &BaseValue,
    senior: Option<&Option<PanelRecognition>>,
) -> Result<BaseValueOutcome, BaseValueError> {
    match senior {
        Some(Some(panel))
            if panel
                .details
                .as_ref()
                .and_then(RecognitionDetails::calls)
                .is_some_and(|calls| {
                    calls.iter().any(|call| call.call == PanelCall::Repetition)
                }) =>
        {
            lookup_base_value(table.table, panel, &[])
        }
        Some(None) => Err(BaseValueError::Unsupported),
        _ => Ok(BaseValueOutcome::Value(table.clone())),
    }
}
