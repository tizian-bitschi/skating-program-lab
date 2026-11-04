//! Deterministic finite generation; the common validator owns skating rules.
use crate::*;

/// A finite plan/evidence contract, not an inferred panel decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinGenerationPlan {
    pub plan: RegularSpin,
    pub evidence: RegularSpinEvidence,
    pub local_feature_qualification_assumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PersonalSkillLimits {
    pub max_jump_rotations: Option<u8>,
    pub max_position_revolutions: Option<u16>,
}

/// Empty selections generate nothing. Selection order is normalized for jumps;
/// explicit evidence templates and manual candidates retain their input order.
#[derive(Debug, Clone)]
pub struct CandidateGenerationRequest {
    pub season: Season,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub discipline: ProfileDiscipline,
    pub segment: ProgramSegment,
    pub jump_kinds: Vec<JumpKind>,
    pub rotations: Vec<u8>,
    pub solo_jumps: bool,
    pub direct_combinations: bool,
    pub axel_sequences: bool,
    /// Exact ordered jump tuples whose direct steps into Axels are explicitly assumed.
    pub confirmed_sequence_transitions: Vec<Vec<ListedJump>>,
    pub max_compound_length: usize,
    /// Generate explicit eight-revolution Basic plans for selected abbreviations.
    /// Layback is audited as unsupported because its posture is unmodeled.
    pub base_spins: Vec<SpinAbbreviation>,
    pub spin_plans: Vec<SpinGenerationPlan>,
    /// Step, ChSq and ChSp evidence only. Their intended elements are constructed here.
    pub movement_plans: Vec<ElementPlanEvidence>,
    pub manual_variants: Vec<ProgramVariant>,
    pub exclusions: Vec<PersonalExclusion>,
    pub excluded_spin_positions: Vec<SpinPosition>,
    pub excluded_spin_entries: Vec<SpinEntry>,
    pub skill_limits: PersonalSkillLimits,
    /// Explicit hypothetical clean recognition for generated plans.
    pub recognition_assumed: bool,
    pub max_candidates: usize,
    /// Counts automatic definitions, including filtered/duplicate/unsupported ones.
    /// Manual definitions are all checked separately before automatic expansion.
    pub max_expansions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateDomainCompleteness {
    CompleteDeclaredDomain,
    IncompleteCandidateCap,
    IncompleteExpansionCap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateOmission {
    pub variant: ProgramVariant,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct CandidateGenerationAudit {
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub sources: Vec<SourceReference>,
    pub families: Vec<ProfileElementFamily>,
    pub exclusions: Vec<PersonalExclusion>,
    pub excluded_spin_positions: Vec<SpinPosition>,
    pub excluded_spin_entries: Vec<SpinEntry>,
    pub skill_limits: PersonalSkillLimits,
    pub omissions: Vec<CandidateOmission>,
    pub completeness: CandidateDomainCompleteness,
    pub attempted_definitions: usize,
    pub manual_definitions_checked: usize,
    pub max_candidates: usize,
    pub max_expansions: usize,
    pub assumptions: &'static str,
    pub declared_domain: &'static str,
}

#[derive(Debug, Clone)]
pub struct CandidateGenerationResult {
    pub variants: Vec<ProgramVariant>,
    pub audit: CandidateGenerationAudit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateGenerationError {
    InvalidRequest(&'static str),
    Profile(ProfileSelectionError),
    UnsupportedGenerator(&'static str),
    InvalidManualVariant { index: usize, reason: String },
}

const DOMAIN: &str = "Selected clean solos (Adult singles/doubles; Senior through quads); direct two/three-jump combinations with ToeLoop/Loop continuations; Axel sequences with every continuation Axel; explicit finite spin and movement evidence templates; manual subset. No Euler, other transitions, automatic difficult-variation inference or all-physical-program completeness.";
const ASSUMPTIONS: &str = "Clean execution and recognition are explicit hypothetical assumptions. Supplied feature qualification must be supported by shared checks. No candidate establishes program-wide eligibility, feature reuse, physical skateability or a panel call. Capped domains cannot certify an optimum over all eligible alternatives.";

struct Generator<'a> {
    registry: &'a ProfileRegistry,
    request: &'a CandidateGenerationRequest,
    policy: ProgramPolicy,
    result: CandidateGenerationResult,
}
impl Generator<'_> {
    /// false stops expansion; true includes omissions and duplicates.
    fn offer(
        &mut self,
        variant: ProgramVariant,
        manual: Option<usize>,
    ) -> Result<bool, CandidateGenerationError> {
        let variant = super::programs::normalize_variant(variant);
        if manual.is_none()
            && self.result.audit.attempted_definitions == self.request.max_expansions
        {
            self.result.audit.completeness = CandidateDomainCompleteness::IncompleteExpansionCap;
            return Ok(false);
        }
        if manual.is_some() {
            self.result.audit.manual_definitions_checked += 1;
        } else {
            self.result.audit.attempted_definitions += 1;
        }
        let reason = self.check(&variant)?;
        if let Some(reason) = reason {
            if let Some(index) = manual {
                return Err(CandidateGenerationError::InvalidManualVariant { index, reason });
            }
            self.result
                .audit
                .omissions
                .push(CandidateOmission { variant, reason });
            return Ok(true);
        }
        if self
            .request
            .exclusions
            .iter()
            .any(|x| super::programs::excluded(&variant, x, usize::MAX))
            || exceeds_skill(&variant, &self.request.skill_limits)
            || spin_filtered(&variant, self.request)
        {
            self.result.audit.omissions.push(CandidateOmission {
                variant,
                reason: "personal exclusion or explicit skill limit".into(),
            });
            return Ok(true);
        }
        if self.result.variants.contains(&variant) {
            return Ok(true);
        }
        if self.result.variants.len() == self.request.max_candidates {
            self.result.audit.completeness = CandidateDomainCompleteness::IncompleteCandidateCap;
            self.result.audit.omissions.push(CandidateOmission {
                variant,
                reason: if manual.is_some() {
                    "manual variant omitted by candidate cap"
                } else {
                    "automatic variant omitted by candidate cap"
                }
                .into(),
            });
            return Ok(manual.is_some());
        }
        if let Some(family) = ProfileElementFamily::of(&variant.element) {
            if !self.result.audit.families.contains(&family) {
                self.result.audit.families.push(family);
            }
        }
        self.result.variants.push(variant);
        Ok(true)
    }

    fn check(
        &mut self,
        variant: &ProgramVariant,
    ) -> Result<Option<String>, CandidateGenerationError> {
        let prepared = match super::programs::prepare_variant(
            self.registry,
            &self.request.profile,
            self.request.scoring_table,
            &self.policy,
            variant,
        ) {
            Ok(prepared) => prepared,
            Err(ProgramOptimizationError::Profile(e)) => {
                return Err(CandidateGenerationError::Profile(e))
            }
            Err(e) => {
                return Ok(Some(format!(
                    "unsupported recognition/scoring contract: {e:?}"
                )))
            }
        };
        for source in prepared.sources {
            if !self.result.audit.sources.contains(&source) {
                self.result.audit.sources.push(source);
            }
        }
        if prepared.local_findings.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                prepared
                    .local_findings
                    .iter()
                    .map(|f| format!("{}: {}", f.rule_id, f.explanation))
                    .collect::<Vec<_>>()
                    .join("; "),
            ))
        }
    }
}

fn spin_filtered(variant: &ProgramVariant, request: &CandidateGenerationRequest) -> bool {
    let PlannedElement::RegularSpin(spin) = &variant.element else {
        return false;
    };
    spin.entry.is_some_and(|entry| request.excluded_spin_entries.contains(&entry))
        || spin.steps.iter().flatten().any(|step| matches!(step, SpinStep::Position { position: Some(p), .. } if request.excluded_spin_positions.contains(p)))
}

fn exceeds_skill(v: &ProgramVariant, limits: &PersonalSkillLimits) -> bool {
    match &v.element {
        PlannedElement::Jump(j) => {
            let jumps: Vec<_> = match j {
                JumpElement::Solo(j) => vec![j],
                JumpElement::Combination(p) | JumpElement::Sequence(p) => p.iter().filter_map(|p| if let JumpPart::Jump(j) = p { Some(j) } else { None }).collect(),
            };
            jumps.iter().any(|j| limits.max_jump_rotations.is_some_and(|cap| j.rotations.is_some_and(|n| n > cap)))
        }
        PlannedElement::RegularSpin(s) => s.steps.iter().flatten().any(|s| matches!(s, SpinStep::Position { revolutions: Some(n), .. } if limits.max_position_revolutions.is_some_and(|cap| *n > cap))),
        _ => false,
    }
}

fn jump_variant(
    jumps: &[ListedJump],
    sequence: bool,
    request: &CandidateGenerationRequest,
) -> ProgramVariant {
    let parts = || jumps.iter().cloned().map(JumpPart::Jump).collect();
    ProgramVariant {
        element: PlannedElement::Jump(if jumps.len() == 1 {
            JumpElement::Solo(jumps[0].clone())
        } else if sequence {
            JumpElement::Sequence(parts())
        } else {
            JumpElement::Combination(parts())
        }),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: sequence
                && request
                    .confirmed_sequence_transitions
                    .iter()
                    .any(|confirmed| confirmed == jumps),
        },
        recognition_assumed: request.recognition_assumed,
    }
}

/// Generate only the declared finite domain. Complete-program rules stay in
/// `validate_supported_program`; pass the returned variants unchanged to the optimizer.
pub fn generate_program_candidates(
    registry: &ProfileRegistry,
    request: &CandidateGenerationRequest,
) -> Result<CandidateGenerationResult, CandidateGenerationError> {
    use CandidateGenerationError as E;
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?;
    let policy = [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ]
    .into_iter()
    .find(|c| adult_profile(*c) == *profile)
    .map(adult_program_policy)
    .or_else(|| crate::senior::built_in_policy(profile))
    .ok_or(E::UnsupportedGenerator(
        "only exact built-in Adult and Senior 2026/27 profiles have generation contracts",
    ))?;
    let max_rotation = if crate::senior::selected(profile, ProfileRule::Eligibility) {
        4
    } else {
        2
    };
    if request.discipline != ProfileDiscipline::Single
        || request.segment != ProgramSegment::FreeSkating
    {
        return Err(E::UnsupportedGenerator(
            "only Single Free Skating is supported",
        ));
    }
    if request.season != profile.season
        || request.max_candidates == 0
        || request.max_candidates > 1000
        || request.max_expansions == 0
        || request.max_expansions > 100_000
        || request.jump_kinds.len() > 6
        || request.rotations.len() > usize::from(max_rotation)
        || request.exclusions.len() > 1000
        || request.excluded_spin_positions.len() > 3
        || request.excluded_spin_entries.len() > 2
        || request.confirmed_sequence_transitions.len() > 1000
        || request
            .confirmed_sequence_transitions
            .iter()
            .any(|j| !(2..=3).contains(&j.len()))
        || !(2..=3).contains(&request.max_compound_length)
        || request
            .rotations
            .iter()
            .any(|n| !(1..=max_rotation).contains(n))
        || request.skill_limits.max_jump_rotations == Some(0)
        || request.skill_limits.max_position_revolutions == Some(0)
    {
        return Err(E::InvalidRequest(
            "invalid identity, bounds or skill limits; rotations exceed the supported profile generation domain",
        ));
    }
    if request.exclusions.iter().any(|x| {
        matches!(
            x,
            PersonalExclusion::Unsupported(_) | PersonalExclusion::Variant(_)
        )
    }) {
        return Err(E::InvalidRequest("generation requires semantic exclusions, not candidate indices or unsupported restrictions"));
    }
    if request.movement_plans.iter().any(|p| {
        matches!(
            p,
            ElementPlanEvidence::Jump { .. } | ElementPlanEvidence::Spin { .. }
        )
    }) {
        return Err(E::InvalidRequest(
            "movement templates must be step or choreographic evidence",
        ));
    }
    if request.base_spins.len()
        + request.spin_plans.len()
        + request.movement_plans.len()
        + request.manual_variants.len()
        > 1000
    {
        return Err(E::InvalidRequest(
            "at most 1000 explicit templates/manual variants",
        ));
    }
    let mut g = Generator {
        registry,
        request,
        policy,
        result: CandidateGenerationResult {
            variants: vec![],
            audit: CandidateGenerationAudit {
                profile: profile.id.clone(),
                scoring_table: request.scoring_table,
                sources: profile.sources.clone(),
                families: vec![],
                exclusions: request.exclusions.clone(),
                excluded_spin_positions: request.excluded_spin_positions.clone(),
                excluded_spin_entries: request.excluded_spin_entries.clone(),
                skill_limits: request.skill_limits.clone(),
                omissions: vec![],
                completeness: CandidateDomainCompleteness::CompleteDeclaredDomain,
                attempted_definitions: 0,
                manual_definitions_checked: 0,
                max_candidates: request.max_candidates,
                max_expansions: request.max_expansions,
                assumptions: ASSUMPTIONS,
                declared_domain: DOMAIN,
            },
        },
    };
    for (index, variant) in request.manual_variants.iter().enumerate() {
        if !g.offer(variant.clone(), Some(index))? {
            return Ok(g.result);
        }
    }
    let mut jumps = vec![];
    for n in 1..=max_rotation {
        for kind in [
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
            JumpKind::Axel,
        ] {
            if request.jump_kinds.contains(&kind) && request.rotations.contains(&n) {
                jumps.push(ListedJump {
                    kind: Some(kind),
                    rotations: Some(n),
                });
            }
        }
    }
    if request.solo_jumps {
        for j in &jumps {
            if !g.offer(jump_variant(std::slice::from_ref(j), false, request), None)? {
                return Ok(g.result);
            }
        }
    }
    for sequence in [false, true] {
        if !(if sequence {
            request.axel_sequences
        } else {
            request.direct_combinations
        }) {
            continue;
        }
        let continuations: Vec<_> = jumps
            .iter()
            .filter(|j| {
                if sequence {
                    j.kind == Some(JumpKind::Axel)
                } else {
                    matches!(j.kind, Some(JumpKind::ToeLoop | JumpKind::Loop))
                }
            })
            .collect();
        for length in 2..=request.max_compound_length {
            for first in &jumps {
                for second in &continuations {
                    if length == 2 {
                        if !g.offer(
                            jump_variant(&[first.clone(), (*second).clone()], sequence, request),
                            None,
                        )? {
                            return Ok(g.result);
                        }
                    } else {
                        for third in &continuations {
                            if !g.offer(
                                jump_variant(
                                    &[first.clone(), (*second).clone(), (*third).clone()],
                                    sequence,
                                    request,
                                ),
                                None,
                            )? {
                                return Ok(g.result);
                            }
                        }
                    }
                }
            }
        }
    }
    for a in &request.base_spins {
        let positions = match a.kind {
            RegularSpinKind::Camel => vec![SpinPosition::Camel],
            RegularSpinKind::Sit => vec![SpinPosition::Sit],
            RegularSpinKind::Upright | RegularSpinKind::Layback => vec![SpinPosition::Upright],
            RegularSpinKind::Combination => vec![
                SpinPosition::Camel,
                SpinPosition::Sit,
                SpinPosition::Upright,
            ],
        };
        let positions: Vec<_> = positions
            .into_iter()
            .map(|position| SpinStep::Position {
                position: Some(position),
                revolutions: Some(8),
                feature_attempts: Some(vec![]),
            })
            .collect();
        let mut steps = positions.clone();
        if a.change_foot {
            steps.push(SpinStep::ChangeFoot);
            steps.extend(positions);
        }
        if !g.offer(
            ProgramVariant {
                element: PlannedElement::RegularSpin(RegularSpin {
                    entry: Some(a.entry),
                    steps: Some(steps),
                }),
                evidence: ElementPlanEvidence::Spin {
                    spin: RegularSpinEvidence {
                        abbreviation: Some(*a),
                        planned_level: Some(Level::Basic),
                        feature_attempts: Some(vec![]),
                        panel: None,
                        final_upright_wind_up: None,
                    },
                    local_feature_qualification_assumed: false,
                },
                recognition_assumed: request.recognition_assumed,
            },
            None,
        )? {
            return Ok(g.result);
        }
    }
    for p in &request.spin_plans {
        if !g.offer(
            ProgramVariant {
                element: PlannedElement::RegularSpin(p.plan.clone()),
                evidence: ElementPlanEvidence::Spin {
                    spin: p.evidence.clone(),
                    local_feature_qualification_assumed: p.local_feature_qualification_assumed,
                },
                recognition_assumed: request.recognition_assumed,
            },
            None,
        )? {
            return Ok(g.result);
        }
    }
    for evidence in &request.movement_plans {
        let element = match evidence {
            ElementPlanEvidence::Step { .. } => PlannedElement::StepSequence,
            ElementPlanEvidence::ChoreographicSequence { .. } => {
                PlannedElement::ChoreographicSequence { movements: None }
            }
            ElementPlanEvidence::ChoreographicSpin { .. } => PlannedElement::ChoreographicSpin,
            _ => unreachable!("checked movement template"),
        };
        if !g.offer(
            ProgramVariant {
                element,
                evidence: evidence.clone(),
                recognition_assumed: request.recognition_assumed,
            },
            None,
        )? {
            return Ok(g.result);
        }
    }
    Ok(g.result)
}
