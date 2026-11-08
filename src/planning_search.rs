//! Complete theoretical composition search; performance recognition is outside scope.
use crate::*;
mod model;

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
    control: &impl PlanningSearchControl,
) -> Result<Option<Vec<Atom>>, PlanningSearchError> {
    let Some(mut assumptions) = crate::selection::planned_jump_elements(
        registry,
        &request.profile,
        request.scoring_table,
        &mut || control.is_cancelled(PlanningSearchPhase::Domain),
    )
    .map_err(PlanningSearchError::Profile)?
    else {
        return Ok(None);
    };
    assumptions.extend(
        selectable_elements(registry, &request.profile, request.scoring_table)
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
    let mut atoms = Vec::with_capacity(assumptions.len());
    for assumption in assumptions {
        if control.is_cancelled(PlanningSearchPhase::Domain) {
            return Ok(None);
        }
        let (assessment, program, evidence) = crate::manual_bv::assess_selections(
            registry,
            profile,
            &manual(request, vec![assumption.clone()], SubmissionState::Draft),
        )
        .map_err(PlanningSearchError::Profile)?;
        let value = assessment
            .total
            .exact_hundredths
            .and_then(|v| i32::try_from(v).ok())
            .ok_or(PlanningSearchError::InconsistentAssessment)?;
        let element = program
            .elements
            .into_iter()
            .next()
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
        let spin = match &evidence[0] {
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
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(PlanningSearchError::Profile)?;
    if ![
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ]
    .iter()
    .any(|c| adult_profile(*c) == *profile)
    {
        return Err(PlanningSearchError::UnsupportedCoverage {
            profile: request.profile.clone(),
            reason: "Complete planning search currently supports built-in Adult profiles.",
        });
    }
    let policy = crate::profiles::resolve_program_policy(profile, None)
        .map_err(PlanningSearchError::Profile)?
        .ok_or(PlanningSearchError::InconsistentAssessment)?;
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
    let Some(mut atoms) = domain(registry, request, profile, control)? else {
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
    model::search(registry, request, profile, &policy, &atoms, control)
}
