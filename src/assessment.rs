//! Ordered manual-plan orchestration. No result establishes official compliance.
use crate::program_validation::{validate_program_scope_with_credit, EvaluationScope};
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionState {
    Draft,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentStatus {
    ConfirmedViolation,
    NoViolationInCoveredRules,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualProgramEntry {
    pub planned: PlannedElement,
    pub evidence: ElementPlanEvidence,
    /// Explicit intended recognition assumption, never inferred from execution.
    /// If supplied, planned_element must be this entry's current index;
    /// observed_element must be None because this request has no observation list.
    pub intended_recognition: Option<PanelRecognition>,
    pub scenario: Option<ManualElementScenario>,
}

impl ManualProgramEntry {
    pub(crate) fn recognitions(&self) -> impl Iterator<Item = &PanelRecognition> {
        let evidence_panel = match &self.evidence {
            ElementPlanEvidence::Spin { spin, .. } => spin.panel.as_ref(),
            _ => None,
        };
        self.intended_recognition
            .iter()
            .chain(self.scenario.iter().map(|s| &s.recognition))
            .chain(evidence_panel)
    }

    /// Rebind supplied plan references together; absent references remain absent.
    #[cfg(feature = "optimizer")]
    pub(crate) fn reindex_recognition(&mut self, index: usize) {
        let evidence_panel = match &mut self.evidence {
            ElementPlanEvidence::Spin { spin, .. } => spin.panel.as_mut(),
            _ => None,
        };
        for panel in self
            .intended_recognition
            .iter_mut()
            .chain(self.scenario.iter_mut().map(|s| &mut s.recognition))
            .chain(evidence_panel)
        {
            if panel.planned_element.is_some() {
                panel.planned_element = Some(index);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualElementScenario {
    /// Supplied panel assumption. The current plan index is optional; observation
    /// references are rejected because this request has no indexed observations.
    pub recognition: PanelRecognition,
    pub annotations: Vec<ScoringAnnotation>,
    /// None requests BV only; it is never a final grade of zero.
    pub final_goe: Option<i8>,
    pub fall: Option<bool>,
    pub execution: Option<ElementExecutionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualProgramRequest {
    pub rotation_preference: RotationPreference,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub policy: ProgramPolicy,
    pub submission: SubmissionState,
    pub entries: Vec<ManualProgramEntry>,
    pub request_hypothetical_total: bool,
    /// First element assumed to be in the second half; not timing evidence.
    pub second_half_boundary: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssessmentValueError {
    MissingRecognition,
    PlanRecognitionMismatch,
    /// Indices into the unchanged shared validation report's findings.
    UnresolvedPlanEvidence(Vec<usize>),
    BaseValue(BaseValueError),
}

/// A subtotal is not an exact total when unresolved_indices is nonempty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssessmentAggregate {
    pub resolved_hundredths: i64,
    pub unresolved_indices: Vec<usize>,
    pub exact_hundredths: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualElementAssessment {
    pub element_index: usize,
    pub intended_base: Result<BaseValueOutcome, AssessmentValueError>,
    pub scenario_base: Option<Result<BaseValueOutcome, BaseValueError>>,
    /// Absent when no final GOE was supplied.
    pub scenario_score: Option<Result<ElementScore, ElementScoreError>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualProgramAssessment {
    pub rotation_conflicts: Vec<RotationPreferenceConflict>,
    /// Preserves plans, assumptions, calls and observations separately.
    pub request: ManualProgramRequest,
    pub elements: Vec<ManualElementAssessment>,
    pub intended_base: AssessmentAggregate,
    pub scenario_base: Option<AssessmentAggregate>,
    pub goe_adjustment: Option<AssessmentAggregate>,
    pub hypothetical_total: Option<AssessmentAggregate>,
    /// Full credited element breakdown when a complete boundary scenario is requested.
    pub planning_score: Option<Result<PlanningScore, ProgramScoreError>>,
    pub validation: EvidenceValidationReport,
    pub status: AssessmentStatus,
    /// Finding indices; omissions remain visible even in an extendable draft.
    pub completion_findings: Vec<usize>,
}

pub(crate) fn aggregate(values: impl Iterator<Item = Option<i64>>) -> AssessmentAggregate {
    let mut resolved_hundredths = 0;
    let mut unresolved_indices = vec![];
    for (index, value) in values.enumerate() {
        match value {
            Some(n) => resolved_hundredths += n,
            None => unresolved_indices.push(index),
        }
    }
    AssessmentAggregate {
        resolved_hundredths,
        exact_hundredths: unresolved_indices.is_empty().then_some(resolved_hundredths),
        unresolved_indices,
    }
}

pub(crate) fn base_number(value: &BaseValueOutcome) -> Option<i64> {
    match value {
        BaseValueOutcome::Value(v) => Some(i64::from(v.hundredths)),
        // These outcomes are retained, but never invented as numeric plan values.
        BaseValueOutcome::Ignored | BaseValueOutcome::NoValue => None,
    }
}

/// Assess one ordered program using existing validation and scoring. No official
/// timing, deductions, automatic recognition or full-legality claim is provided.
pub fn assess_manual_program(
    registry: &ProfileRegistry,
    request: &ManualProgramRequest,
) -> Result<ManualProgramAssessment, ProfileSelectionError> {
    assess_manual_scope(registry, request, None)
}

pub(crate) fn assess_manual_scope(
    registry: &ProfileRegistry,
    request: &ManualProgramRequest,
    assumed_values: Option<&[BaseValue]>,
) -> Result<ManualProgramAssessment, ProfileSelectionError> {
    let profile = registry.resolve(&request.profile, request.scoring_table)?;
    let invalid = |reason: &str| ProfileSelectionError {
        profile: request.profile.clone(),
        kind: ProfileErrorKind::Invalid,
        reason: reason.into(),
    };
    if request
        .second_half_boundary
        .is_some_and(|b| b > request.entries.len())
    {
        return Err(invalid("second-half boundary exceeds program length"));
    }
    for (index, entry) in request.entries.iter().enumerate() {
        for panel in entry.recognitions() {
            if panel.ruleset != profile.scoring.recognition_ruleset
                || panel.planned_element.is_some_and(|i| i != index)
                || panel.observed_element.is_some()
            {
                return Err(invalid(
                    "recognition needs the selected ruleset, its current plan index, and no observation reference",
                ));
            }
        }
        if entry
            .scenario
            .as_ref()
            .is_some_and(|s| !s.annotations.is_empty())
        {
            return Err(invalid(
                "use the explicit planning boundary instead of recorded timing credit",
            ));
        }
    }
    let program = ProfileProgram {
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        elements: request.entries.iter().map(|e| e.planned.clone()).collect(),
    };
    let evidence: Vec<_> = request.entries.iter().map(|e| e.evidence.clone()).collect();
    let scope = if assumed_values.is_some() {
        EvaluationScope::AssumedRecognition
    } else {
        EvaluationScope::EvidenceBased
    };
    let (validation, credit_blocks) =
        validate_program_scope_with_credit(registry, &program, &evidence, &request.policy, scope)?;
    let completion_findings = crate::program_validation::completion_findings(
        &validation,
        profile,
        &program,
        &request.policy,
        &evidence,
    );
    let status = if validation
        .validation
        .findings
        .iter()
        .enumerate()
        .any(|(i, f)| {
            f.kind == FindingKind::Violation
                && !(request.submission == SubmissionState::Draft
                    && completion_findings.contains(&i))
        }) {
        AssessmentStatus::ConfirmedViolation
    } else if validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind != FindingKind::Violation)
        || !completion_findings.is_empty()
    {
        AssessmentStatus::Indeterminate
    } else {
        AssessmentStatus::NoViolationInCoveredRules
    };
    let senior = profile.binding(ProfileRule::Scoring).mechanism == RuleMechanism::SeniorScoringV1;
    let assumed_credit = assumed_values.map(|values| {
        crate::assumed_recognition::assumed_credit(
            profile,
            &program.elements,
            &evidence,
            &credit_blocks,
            values.iter().map(Some),
        )
    });
    let contextual = |panels: Vec<Option<PanelRecognition>>| {
        if senior {
            crate::senior::repetition_context(&panels)
        } else {
            panels
        }
    };
    let intended_context = contextual(
        request
            .entries
            .iter()
            .map(|e| e.intended_recognition.clone())
            .collect(),
    );
    let scenario_context = contextual(
        request
            .entries
            .iter()
            .map(|e| e.scenario.as_ref().map(|s| s.recognition.clone()))
            .collect(),
    );
    let elements: Vec<_> = request
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let unresolved: Vec<_> = validation
                .validation
                .findings
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    f.element_index == Some(index)
                        && !matches!(
                            f.rule_id,
                            "spin-execution-recognition"
                                | "spin-panel-reconciliation"
                                | "spin-panel-target"
                        )
                        && (f.kind != FindingKind::Violation
                            || crate::spin_validation::contradicts_spin_evidence(f))
                })
                .map(|(i, _)| i)
                .collect();
            let intended_base = if let Some(credits) = &assumed_credit {
                if unresolved.is_empty() {
                    credits[index]
                        .clone()
                        .map_err(AssessmentValueError::BaseValue)
                } else {
                    Err(AssessmentValueError::UnresolvedPlanEvidence(unresolved))
                }
            } else {
                match &entry.intended_recognition {
                    None => Err(AssessmentValueError::MissingRecognition),
                    Some(_) if !unresolved.is_empty() => {
                        Err(AssessmentValueError::UnresolvedPlanEvidence(unresolved))
                    }
                    Some(panel)
                        if !crate::plan_recognition::matches_intended(
                            &entry.planned,
                            &entry.evidence,
                            panel,
                        ) =>
                    {
                        Err(AssessmentValueError::PlanRecognitionMismatch)
                    }
                    Some(_) if senior && intended_context[index].is_none() => {
                        Err(AssessmentValueError::BaseValue(BaseValueError::Unsupported))
                    }
                    Some(panel) => lookup_base_value(
                        request.scoring_table,
                        intended_context[index].as_ref().unwrap_or(panel),
                        &[],
                    )
                    .map_err(AssessmentValueError::BaseValue),
                }
            };
            let scenario_base = entry.scenario.as_ref().map(|s| {
                if senior && scenario_context[index].is_none() {
                    return Err(BaseValueError::Unsupported);
                }
                lookup_base_value(
                    request.scoring_table,
                    scenario_context[index].as_ref().unwrap_or(&s.recognition),
                    &s.annotations,
                )
            });
            let scenario_score = entry.scenario.as_ref().and_then(|s| {
                let recognition = scenario_context[index].as_ref().unwrap_or(&s.recognition);
                s.final_goe.map(|goe| {
                    if senior && scenario_context[index].is_none() {
                        return Err(ElementScoreError::UnsupportedMarks);
                    }
                    match &s.execution {
                        Some(execution) => score_element_with_execution(
                            request.scoring_table,
                            recognition,
                            &s.annotations,
                            goe,
                            s.fall,
                            execution,
                        ),
                        None => score_element(
                            request.scoring_table,
                            recognition,
                            &s.annotations,
                            goe,
                            s.fall,
                        ),
                    }
                })
            });
            ManualElementAssessment {
                element_index: index,
                intended_base,
                scenario_base,
                scenario_score,
            }
        })
        .collect();
    let intended_base = aggregate(
        elements
            .iter()
            .map(|e| e.intended_base.as_ref().ok().and_then(base_number)),
    );
    let scenario_base = request
        .entries
        .iter()
        .any(|e| e.scenario.is_some())
        .then(|| {
            aggregate(elements.iter().map(|e| {
                e.scenario_base
                    .as_ref()
                    .and_then(|b| b.as_ref().ok())
                    .and_then(base_number)
            }))
        });
    let has_goe = request
        .entries
        .iter()
        .any(|e| e.scenario.as_ref().is_some_and(|s| s.final_goe.is_some()));
    let goe_adjustment = has_goe.then(|| {
        aggregate(elements.iter().map(|e| {
            e.scenario_score
                .as_ref()
                .and_then(|s| s.as_ref().ok())
                .map(|s| i64::from(s.goe_hundredths))
        }))
    });
    // Boundary credit needs a supported complete scenario. Missing GOE is never
    // replaced with a zero grade, including inside the legacy adapter.
    let planning_score = if (!senior || scenario_context.iter().all(Option::is_some))
        && has_goe
        && request.request_hypothetical_total
        && request.second_half_boundary.is_some()
    {
        let inputs: Option<Vec<_>> = request
            .entries
            .iter()
            .enumerate()
            .map(|(index, e)| {
                let s = e.scenario.as_ref()?;
                Some(ProgramElement {
                    recognition: scenario_context[index]
                        .as_ref()
                        .unwrap_or(&s.recognition)
                        .clone(),
                    annotations: vec![],
                    goe: s.final_goe?,
                    fall: s.fall,
                    start_ms: None,
                })
            })
            .collect();
        inputs.map(|inputs| {
            let execution: Vec<_> = request
                .entries
                .iter()
                .map(|e| {
                    e.scenario
                        .as_ref()
                        .and_then(|s| s.execution.clone())
                        .unwrap_or_default()
                })
                .collect();
            score_planning_scenario_with_execution(
                profile.scoring.recognition_ruleset,
                request.scoring_table,
                &inputs,
                request.second_half_boundary,
                &execution,
            )
        })
    } else {
        None
    };
    let hypothetical_total = (has_goe && request.request_hypothetical_total).then(|| {
        aggregate(elements.iter().enumerate().map(|(i, e)| {
            let score = e.scenario_score.as_ref()?.as_ref().ok()?;
            if request.second_half_boundary.is_some() {
                planning_score.as_ref()?.as_ref().ok()?.elements[i]
                    .as_ref()
                    .ok()
                    .map(|s| i64::from(s.score_hundredths))
            } else {
                Some(i64::from(score.score_hundredths))
            }
        }))
    });
    Ok(ManualProgramAssessment {
        rotation_conflicts: crate::rotation::conflicts(
            request.entries.iter().map(|e| &e.planned),
            request.rotation_preference,
        ),
        request: request.clone(),
        elements,
        intended_base,
        scenario_base,
        goe_adjustment,
        hypothetical_total,
        planning_score,
        validation,
        status,
        completion_findings,
    })
}
