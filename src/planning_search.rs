//! Complete theoretical composition search; performance recognition is outside scope.
use crate::*;
mod model;
mod senior_bound;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredPlanningElement {
    pub id: String,
    pub element: AssumedRecognition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningExclusion {
    JumpKind(JumpKind),
    ExactJump(ListedJump),
    SpinKind(RegularSpinKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningSearchRequest {
    pub personal_limits: PersonalLimits,
    pub rotation_preference: RotationPreference,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub required: Vec<RequiredPlanningElement>,
    pub exclusions: Vec<PlanningExclusion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningSearchPhase {
    Domain,
    Model,
    Solve,
    /// An internal candidate passed shared reconstruction and manual assessment.
    IncumbentFound,
    Reconstruction,
    MaximumProved,
    Alternatives,
}

/// Polling is cooperative. Phases expose lifecycle only, never an unproved program.
pub trait PlanningSearchControl {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningDiagnostic {
    pub required_input_indices: Vec<usize>,
    pub rule_id: Option<&'static str>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningSearchError {
    PersonalLimitConflict(Vec<PersonalLimitConflict>),
    RotationPreferenceConflict(Vec<RotationPreferenceConflict>),
    InvalidRequest {
        reason: &'static str,
        diagnostics: Vec<PlanningDiagnostic>,
    },
    Profile(ProfileSelectionError),
    UnsupportedCoverage {
        profile: ProfileId,
        reason: &'static str,
    },
    InconsistentAssessment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningProvenance {
    Required { id: String, input_index: usize },
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningProgram {
    pub program: ProfileProgram,
    pub elements: Vec<AssumedRecognition>,
    pub provenance: Vec<PlanningProvenance>,
    /// Source-bound composition coverage and exact per-element credited BV.
    pub assessment: ManualBvAssessment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningAlternativeStatus {
    Exhausted,
    LimitReached,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningMaximum {
    pub personal_limits: PersonalLimits,
    pub rotation_preference: RotationPreference,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub maximum_base_hundredths: u32,
    pub alternatives: Vec<PlanningProgram>,
    pub alternative_status: PlanningAlternativeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningSearchOutcome {
    Cancelled,
    Infeasible {
        diagnostics: Vec<PlanningDiagnostic>,
    },
    ProvenMaximum(PlanningMaximum),
}

pub(super) struct Atom {
    assumption: AssumedRecognition,
    element: PlannedElement,
    jumps: Vec<ListedJump>,
    spin: Option<SpinAbbreviation>,
    eulers: usize,
    value: i32,
}

impl Atom {
    fn family(&self) -> ProfileElementFamily {
        ProfileElementFamily::of(&self.element).expect("selected element family")
    }
    fn sequence(&self) -> bool {
        self.family() == ProfileElementFamily::JumpSequence
    }
    fn combination(&self) -> bool {
        self.family() == ProfileElementFamily::Combination
    }
    fn excluded(&self, exclusions: &[PlanningExclusion]) -> bool {
        exclusions.iter().any(|exclusion| match exclusion {
            PlanningExclusion::JumpKind(kind) => self.jumps.iter().any(|j| j.kind == Some(*kind)),
            PlanningExclusion::ExactJump(jump) => self.jumps.contains(jump),
            PlanningExclusion::SpinKind(kind) => self.spin.is_some_and(|a| a.kind == *kind),
        })
    }
}

fn invalid(reason: &'static str, indices: Vec<usize>) -> PlanningSearchError {
    PlanningSearchError::InvalidRequest {
        reason,
        diagnostics: vec![PlanningDiagnostic {
            required_input_indices: indices,
            rule_id: None,
            message: reason.into(),
        }],
    }
}

fn manual(
    request: &PlanningSearchRequest,
    elements: Vec<AssumedRecognition>,
    submission: SubmissionState,
) -> ManualBvRequest {
    ManualBvRequest {
        personal_limits: request.personal_limits.clone(),
        rotation_preference: request.rotation_preference,
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        submission,
        elements,
    }
}

fn domain(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    profile: &RuleProfile,
    maximum: Option<Level>,
    control: &impl PlanningSearchControl,
) -> Result<Option<Vec<Atom>>, PlanningSearchError> {
    let Some(mut assumptions) = crate::selection::planned_jump_elements(
        registry,
        &request.profile,
        request.scoring_table,
        request.rotation_preference,
        &request.personal_limits,
        &mut || control.is_cancelled(PlanningSearchPhase::Domain),
    )
    .map_err(PlanningSearchError::Profile)?
    else {
        return Ok(None);
    };
    assumptions.extend(
        selectable_elements_with_step_limit(
            registry,
            &request.profile,
            request.scoring_table,
            &request.personal_limits,
            maximum,
        )
        .map_err(PlanningSearchError::Profile)?
        .into_iter()
        .filter(|s| !matches!(s.kind, SelectableElementKind::SoloJump { .. }))
        .map(|s| AssumedRecognition::new(s.row.notation)),
    );
    assumptions.sort_by(|a, b| {
        a.code.cmp(&b.code).then(
            a.direct_transitions_assumed
                .cmp(&b.direct_transitions_assumed),
        )
    });
    let selection =
        crate::selection::Selection::new(registry, &request.profile, request.scoring_table)
            .map_err(PlanningSearchError::Profile)?;
    let mut atoms = Vec::with_capacity(assumptions.len());
    for assumption in assumptions {
        if control.is_cancelled(PlanningSearchPhase::Domain) {
            return Ok(None);
        }
        let (element, evidence, value, restriction) = crate::manual_bv::resolve_selection(
            &selection,
            request.scoring_table,
            profile.scoring.recognition_ruleset,
            &assumption,
        );
        if restriction.is_some() {
            return Err(PlanningSearchError::InconsistentAssessment);
        }
        let value = value
            .map(|v| i32::from(v.hundredths))
            .ok_or(PlanningSearchError::InconsistentAssessment)?;
        let (jumps, eulers) = match &element {
            PlannedElement::Jump(JumpElement::Solo(j)) => (vec![j.clone()], 0),
            PlannedElement::Jump(
                JumpElement::Combination(parts) | JumpElement::Sequence(parts),
            ) => (
                parts
                    .iter()
                    .filter_map(|p| {
                        if let JumpPart::Jump(j) = p {
                            Some(j.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                parts
                    .iter()
                    .filter(|p| matches!(p, JumpPart::Euler))
                    .count(),
            ),
            _ => (vec![], 0),
        };
        let spin = match &evidence {
            ElementPlanEvidence::Spin { spin, .. } => spin.abbreviation,
            _ => None,
        };
        atoms.push(Atom {
            assumption,
            element,
            jumps,
            spin,
            eulers,
            value,
        });
    }
    Ok(Some(atoms))
}

/// Prove a maximum over the complete supported profile repertoire. No timeout or
/// caller-supplied shortlist applies. Cancelled proofs expose no candidate.
pub fn search_maximum_bv(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    control: &impl PlanningSearchControl,
) -> Result<PlanningSearchOutcome, PlanningSearchError> {
    search_maximum_bv_with_step_limit(registry, request, None, control)
}

/// Prove a maximum with an additional personal step-sequence level ceiling.
/// `None` preserves the original search contract and request layout.
pub fn search_maximum_bv_with_step_limit(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    maximum: Option<Level>,
    control: &impl PlanningSearchControl,
) -> Result<PlanningSearchOutcome, PlanningSearchError> {
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(PlanningSearchError::Profile)?;
    let senior = crate::senior::selected(profile, ProfileRule::Repetition);
    if ProfileRule::ALL.iter().any(|rule| {
        profile.binding(*rule).mechanism
            != match (*rule, senior) {
                (ProfileRule::Scoring, true) => RuleMechanism::SeniorScoringV1,
                (ProfileRule::Scoring, false) => RuleMechanism::AdultScoringV1,
                (_, true) => RuleMechanism::SeniorJumpPlanV1,
                (_, false) => RuleMechanism::JumpPlanV1,
            }
    }) {
        return Err(PlanningSearchError::UnsupportedCoverage {
            profile: request.profile.clone(),
            reason: "Complete planning search requires consistent supported jump and scoring mechanisms.",
        });
    }
    let policy = registry
        .program_policy(profile, None)
        .map_err(PlanningSearchError::Profile)?
        .ok_or(PlanningSearchError::UnsupportedCoverage {
            profile: request.profile.clone(),
            reason: "Complete planning search requires an explicit supported program policy.",
        })?;
    // Keep registered policies within the audited arithmetic and search envelope.
    if [
        profile.jumps.max_jumps,
        profile.jumps.max_compounds,
        profile.jumps.max_combinations,
        profile.jumps.max_sequences,
        profile.jumps.max_three_jump_compounds,
        profile.jumps.max_eulers,
        profile.jumps.max_repetitions,
        policy.spins.max_spins,
        policy.max_steps,
        policy.max_choreographic_sequences,
        policy.max_choreographic_spins,
    ]
    .into_iter()
    .any(|n| n > 11)
    {
        return Err(PlanningSearchError::UnsupportedCoverage {
            profile: request.profile.clone(),
            reason: "Planning policy exceeds the audited eleven-occurrence bounds.",
        });
    }
    let mut ids = std::collections::BTreeSet::new();
    for (i, required) in request.required.iter().enumerate() {
        if required.id.trim().is_empty() || !ids.insert(&required.id) {
            return Err(invalid(
                "Required occurrence IDs must be nonempty and unique.",
                vec![i],
            ));
        }
    }
    if request.exclusions.iter().any(|e| matches!(e,PlanningExclusion::ExactJump(j) if j.kind.is_none() || j.rotations.is_none_or(|r| r==0))) { return Err(invalid("Exact jump exclusions require a kind and positive rotation count.", vec![])); }
    let required_inputs = request.required.iter().map(|r| r.element.clone()).collect();
    let required_assessment = assess_manual_bv_with_step_limit(
        registry,
        &manual(request, required_inputs, SubmissionState::Draft),
        maximum,
    )
    .map_err(PlanningSearchError::Profile)?;
    if !required_assessment.personal_conflicts.is_empty() {
        return Err(PlanningSearchError::PersonalLimitConflict(
            required_assessment.personal_conflicts,
        ));
    }
    if !required_assessment.rotation_conflicts.is_empty() {
        return Err(PlanningSearchError::RotationPreferenceConflict(
            required_assessment.rotation_conflicts,
        ));
    }
    let Some(mut atoms) = domain(registry, request, profile, maximum, control)? else {
        return Ok(PlanningSearchOutcome::Cancelled);
    };
    for (i, required) in request.required.iter().enumerate() {
        let atom = match atoms.iter().find(|a| a.assumption == required.element) {
            Some(atom) => atom,
            None => {
                let single = assess_manual_bv(
                    registry,
                    &manual(
                        request,
                        vec![required.element.clone()],
                        SubmissionState::Draft,
                    ),
                )
                .map_err(PlanningSearchError::Profile)?;
                let mut error = invalid(
                    "Required exact element is outside the permitted planning repertoire.",
                    vec![i],
                );
                if let Some(f) = single
                    .legality
                    .validation
                    .validation
                    .findings
                    .iter()
                    .find(|f| f.element_index == Some(0) && f.kind == FindingKind::Violation)
                {
                    if let PlanningSearchError::InvalidRequest { diagnostics, .. } = &mut error {
                        diagnostics[0].rule_id = Some(f.rule_id);
                        diagnostics[0].message = f.explanation.into();
                    }
                }
                return Err(error);
            }
        };
        if atom.excluded(&request.exclusions) {
            return Err(invalid(
                "Required element conflicts with a global exclusion.",
                vec![i],
            ));
        }
    }
    let required: Vec<_> = request.required.iter().map(|r| r.element.clone()).collect();
    let assessment = assess_manual_bv(registry, &manual(request, required, SubmissionState::Draft))
        .map_err(PlanningSearchError::Profile)?;
    let diagnostics: Vec<_> = assessment
        .legality
        .validation
        .validation
        .findings
        .iter()
        .enumerate()
        .filter(|(i, f)| {
            f.kind == FindingKind::Violation && !assessment.legality.repairable_absences.contains(i)
        })
        .map(|(_, f)| PlanningDiagnostic {
            required_input_indices: f
                .element_index
                .map_or_else(|| (0..request.required.len()).collect(), |i| vec![i]),
            rule_id: Some(f.rule_id),
            message: f.explanation.into(),
        })
        .collect();
    if !diagnostics.is_empty() {
        return Ok(PlanningSearchOutcome::Infeasible { diagnostics });
    }
    atoms.retain(|a| !a.excluded(&request.exclusions));
    if profile.jumps.require_axel
        && !atoms
            .iter()
            .any(|a| a.jumps.iter().any(|j| j.kind == Some(JumpKind::Axel)))
    {
        return Ok(PlanningSearchOutcome::Infeasible {
            diagnostics: vec![PlanningDiagnostic {
                required_input_indices: vec![],
                rule_id: None,
                message: "Global exclusions leave no Axel to satisfy the profile obligation."
                    .into(),
            }],
        });
    }
    let mut outcome = model::search(registry, request, profile, &policy, &atoms, control)?;
    if maximum.is_some() {
        if let PlanningSearchOutcome::ProvenMaximum(result) = &mut outcome {
            for alternative in &mut result.alternatives {
                let assessment = assess_manual_bv_with_step_limit(
                    registry,
                    &manual(
                        request,
                        alternative.elements.clone(),
                        SubmissionState::Complete,
                    ),
                    maximum,
                )
                .map_err(PlanningSearchError::Profile)?;
                if !assessment.personal_conflicts.is_empty() {
                    return Err(PlanningSearchError::InconsistentAssessment);
                }
                alternative.assessment = assessment;
            }
        }
    }
    Ok(outcome)
}
