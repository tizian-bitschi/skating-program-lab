//! Bounded mixed-program checks. Qualification is an explicit scenario assumption.
use crate::*;

const REQUIRED_CHOREOGRAPHIC_SPIN_RULE: &str = "program-required-choreographic-spin";

pub(crate) const RULE_IDS: &[&str] = &[
    "program-choreographic-plan-consistency",
    "program-choreographic-sequence",
    "program-choreographic-spin",
    "program-eight-position",
    "program-eight-revolutions",
    "program-family-cap",
    "program-feature-once-per-program",
    "program-feature-per-foot",
    "program-feature-qualification",
    "program-feature-position",
    "program-flying-feature-entry",
    REQUIRED_CHOREOGRAPHIC_SPIN_RULE,
    crate::validation::SEQUENCE_DIRECT_STEP_RULE,
    "program-spin-level-credit",
    "program-spin-level-four-feature",
    "program-step-feature-dependency",
    "program-step-level-credit",
    "program-step-recognition",
];

/// Additional family rules are explicit for custom profiles, never inferred by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramPolicy {
    pub spins: RegularSpinPolicy,
    pub max_steps: usize,
    pub max_choreographic_sequences: usize,
    pub max_choreographic_spins: usize,
    pub require_choreographic_spin: bool,
    pub max_step_level: Level,
    pub step_movement_required: bool,
    pub permitted_step_features: Vec<u8>,
    pub step_source: SourceReference,
    pub choreography_source: SourceReference,
    pub feature_source: SourceReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkatingMovement {
    Spiral,
    Arabesque,
    SpreadEagle,
    InaBauer,
    Hydroblading,
    UnlistedJump,
}

/// Local to one candidate/element. None is missing evidence, not an empty assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementPlanEvidence {
    Jump {
        direct_transitions_assumed: bool,
    },
    Spin {
        spin: RegularSpinEvidence,
        /// Explicitly assumes each supplied successful feature meets its local
        /// technical prerequisites. Program-wide reuse is still checked here.
        local_feature_qualification_assumed: bool,
    },
    Step {
        level: Level,
        clean_difficult_turns: Option<u8>,
        full_ice_coverage: Option<bool>,
        movement: Option<SkatingMovement>,
        /// Already qualified feature identities, under an explicit caller assumption.
        qualified_features: Option<Vec<u8>>,
        qualification_assumed: bool,
    },
    ChoreographicSequence {
        movements: Option<Vec<SkatingMovement>>,
        clearly_visible: Option<bool>,
    },
    ChoreographicSpin {
        /// Each entry is one uninterrupted same-foot hold; never sum across feet.
        consecutive_revolutions_per_foot: Option<Vec<u16>>,
    },
}

fn source(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Communication 2788".into(),
        version: "2026-05-19".into(),
        location: location.into(),
    }
}

struct MixedChecks<'a> {
    findings: &'a mut Vec<Finding>,
    applied_rules: &'a mut Vec<AppliedRule>,
}
impl MixedChecks<'_> {
    fn apply(&mut self, id: &'static str, index: Option<usize>, source: &SourceReference) {
        crate::validation::record_rule(self.applied_rules, id, index, None, source);
    }
    fn add(
        &mut self,
        kind: FindingKind,
        id: &'static str,
        index: Option<usize>,
        text: &'static str,
        source: &SourceReference,
    ) {
        self.apply(id, index, source);
        self.findings.push(Finding {
            kind,
            rule_id: id,
            element_index: index,
            component_index: None,
            explanation: text,
            source: source.clone(),
        });
    }
}

impl SkatingMovement {
    /// Exact supported vocabulary for legacy movement descriptions; no prose inference.
    fn from_description(description: &str) -> Option<Self> {
        let key: String = description
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect();
        Some(match key.as_str() {
            "spiral" => Self::Spiral,
            "arabesque" => Self::Arabesque,
            "spreadeagle" => Self::SpreadEagle,
            "inabauer" => Self::InaBauer,
            "hydroblading" => Self::Hydroblading,
            "unlistedjump" => Self::UnlistedJump,
            _ => return None,
        })
    }
}

/// Reuses regular-spin and jump validation, then checks the ordered mixed plan.
/// No result establishes execution, panel recognition or complete ISU compliance.
pub fn validate_supported_program(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    evidence: &[ElementPlanEvidence],
    policy: &ProgramPolicy,
) -> Result<EvidenceValidationReport, ProfileSelectionError> {
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    let invalid = |reason: &str| ProfileSelectionError {
        profile: program.profile.clone(),
        kind: ProfileErrorKind::Invalid,
        reason: reason.into(),
    };
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        if *profile == adult_profile(category) && *policy != adult_program_policy(category) {
            return Err(invalid("built-in profile policies cannot be overridden; register an explicit custom profile"));
        }
    }
    if [
        &policy.step_source,
        &policy.choreography_source,
        &policy.feature_source,
    ]
    .iter()
    .any(|s| {
        s.document.trim().is_empty() || s.version.trim().is_empty() || s.location.trim().is_empty()
    }) {
        return Err(invalid(
            "program policy needs complete source bindings for each family",
        ));
    }
    if evidence.len() != program.elements.len()
        || policy.spins.profile != profile.id
        || policy
            .permitted_step_features
            .iter()
            .any(|n| !(1..=5).contains(n))
    {
        return Err(invalid(
            "mixed evidence needs one correctly typed entry per element and a profile-bound policy",
        ));
    }
    let mut spin_evidence = ProgramValidationEvidence::default();
    for (index, (element, data)) in program.elements.iter().zip(evidence).enumerate() {
        match (element, data) {
            (PlannedElement::RegularSpin(_), ElementPlanEvidence::Spin { spin, .. }) => {
                spin_evidence.spins.push(IndexedSpinEvidence {
                    element_index: index,
                    spin: spin.clone(),
                })
            }
            (PlannedElement::Jump(_), ElementPlanEvidence::Jump { .. })
            | (PlannedElement::StepSequence, ElementPlanEvidence::Step { .. })
            | (
                PlannedElement::ChoreographicSequence { .. },
                ElementPlanEvidence::ChoreographicSequence { .. },
            )
            | (PlannedElement::ChoreographicSpin, ElementPlanEvidence::ChoreographicSpin { .. }) => {
            }
            _ => {
                return Err(invalid(
                    "element/evidence family mismatch or unspecified element",
                ))
            }
        }
    }
    let assumed_direct_sequences: Vec<_> = evidence
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            matches!(
                e,
                ElementPlanEvidence::Jump {
                    direct_transitions_assumed: true
                }
            )
            .then_some(i)
        })
        .collect();
    let mut report = crate::spin_validation::validate_profile_program_with_assumptions(
        registry,
        program,
        &spin_evidence,
        Some(&policy.spins),
        &assumed_direct_sequences,
        crate::spin_validation::FeatureCreditEvaluation::ProgramWide,
    )?;
    let mut checks = MixedChecks {
        findings: &mut report.validation.findings,
        applied_rules: &mut report.applied_rules,
    };
    let count = |family| {
        program
            .elements
            .iter()
            .filter(|e| ProfileElementFamily::of(e) == Some(family))
            .count()
    };
    for (family, cap) in [
        (ProfileElementFamily::StepSequence, policy.max_steps),
        (
            ProfileElementFamily::ChoreographicSequence,
            policy.max_choreographic_sequences,
        ),
        (
            ProfileElementFamily::ChoreographicSpin,
            policy.max_choreographic_spins,
        ),
    ] {
        let family_source = if family == ProfileElementFamily::StepSequence {
            &policy.step_source
        } else {
            &policy.choreography_source
        };
        checks.apply("program-family-cap", None, family_source);
        if count(family) > cap {
            checks.add(
                FindingKind::Violation,
                "program-family-cap",
                None,
                "The family count exceeds the selected program policy.",
                family_source,
            );
        }
    }
    if policy.require_choreographic_spin {
        checks.apply(
            REQUIRED_CHOREOGRAPHIC_SPIN_RULE,
            None,
            &policy.choreography_source,
        );
    }
    if policy.require_choreographic_spin && count(ProfileElementFamily::ChoreographicSpin) == 0 {
        checks.add(
            FindingKind::Violation,
            REQUIRED_CHOREOGRAPHIC_SPIN_RULE,
            None,
            "The profile requires a choreographic spin.",
            &policy.choreography_source,
        );
    }
    let feature_source = policy.feature_source.clone();
    let mut consumed = Vec::new();
    let mut uncertain_success = Vec::new();
    for (index, (element, data)) in program.elements.iter().zip(evidence).enumerate() {
        match data {
            ElementPlanEvidence::Spin {
                spin,
                local_feature_qualification_assumed,
            } => {
                let PlannedElement::RegularSpin(plan) = element else {
                    unreachable!()
                };
                checks.apply("program-spin-level-credit", Some(index), &feature_source);
                let mut credit = 0;
                let mut level_four_feature = false;
                let mut per_foot = vec![
                    0usize;
                    plan.steps.as_ref().map_or(1, |s| 1 + s
                        .iter()
                        .filter(|s| matches!(s, SpinStep::ChangeFoot))
                        .count())
                ];
                let mut previous_target = None;
                for attempt in spin.feature_attempts.iter().flatten() {
                    checks.apply(
                        "program-feature-qualification",
                        Some(index),
                        &feature_source,
                    );
                    let valid_target = crate::spin_validation::feature_target_validity(
                        plan,
                        attempt,
                        previous_target,
                    );
                    previous_target = Some(attempt.position_step);
                    if valid_target != Some(true) {
                        checks.add(FindingKind::Unknown, "program-feature-qualification", Some(index), "Invalid, unordered or missing position-step evidence prevents feature credit.", &feature_source);
                        continue;
                    }
                    // Variation categories and entrance/exit nature need richer evidence.
                    let supported = matches!(
                        attempt.feature,
                        SpinFeature::JumpWithinSpin
                            | SpinFeature::ChangeOfEdge
                            | SpinFeature::BothDirections
                            | SpinFeature::IncreaseOfSpeed
                            | SpinFeature::EightRevolutions
                            | SpinFeature::DifficultFlyingEntry
                            | SpinFeature::DifficultBlade
                            | SpinFeature::Windmills
                    );
                    let position = plan
                        .steps
                        .as_ref()
                        .and_then(|s| s.get(attempt.position_step));
                    if attempt.feature == SpinFeature::DifficultFlyingEntry {
                        checks.apply("program-flying-feature-entry", Some(index), &feature_source);
                    }
                    if attempt.feature == SpinFeature::EightRevolutions {
                        checks.apply("program-eight-position", Some(index), &feature_source);
                    }
                    if attempt.feature == SpinFeature::DifficultFlyingEntry
                        && plan.entry != Some(SpinEntry::Flying)
                    {
                        checks.add(
                            FindingKind::Unknown,
                            "program-flying-feature-entry",
                            Some(index),
                            "A difficult flying-entry feature needs a planned flying entry.",
                            &feature_source,
                        );
                        continue;
                    }
                    if attempt.feature == SpinFeature::EightRevolutions
                        && !matches!(
                            position,
                            Some(SpinStep::Position {
                                position: Some(SpinPosition::Camel),
                                ..
                            })
                        )
                    {
                        checks.add(FindingKind::Unsupported, "program-eight-position", Some(index), "This bounded eight-revolution subset supports plain camel only; difficult variation and Layback evidence remain unmodeled.", &feature_source);
                        continue;
                    }
                    if !supported {
                        checks.add(FindingKind::Unsupported, "program-feature-qualification", Some(index), "This feature needs unmodeled variation, foot-change or entrance/exit evidence.", &feature_source);
                        continue;
                    }
                    checks.apply(
                        "program-feature-once-per-program",
                        Some(index),
                        &feature_source,
                    );
                    let first_success = attempt.feature == SpinFeature::EightRevolutions;
                    let available = !consumed.contains(&attempt.feature);
                    if !first_success {
                        consumed.push(attempt.feature);
                    }
                    if first_success && attempt.outcome == FeatureAttemptOutcome::Unknown {
                        uncertain_success.push(attempt.feature);
                    }
                    if matches!(
                        attempt.feature,
                        SpinFeature::ChangeOfEdge
                            | SpinFeature::BothDirections
                            | SpinFeature::IncreaseOfSpeed
                            | SpinFeature::DifficultBlade
                    ) && !matches!(
                        position,
                        Some(SpinStep::Position {
                            position: Some(SpinPosition::Camel | SpinPosition::Sit),
                            ..
                        })
                    ) && !spin
                        .abbreviation
                        .is_some_and(|a| a.kind == RegularSpinKind::Layback)
                    {
                        checks.add(FindingKind::Unsupported, "program-feature-position", Some(index), "This feature requires camel, sit, Layback, Biellmann or a difficult upright variation; generic upright or missing position evidence cannot qualify it.", &feature_source);
                        continue;
                    }
                    if attempt.outcome == FeatureAttemptOutcome::Unknown
                        || !local_feature_qualification_assumed
                    {
                        checks.add(FindingKind::Unknown, "program-feature-qualification", Some(index), "Feature outcome or explicit local qualification assumption is missing.", &feature_source);
                    } else if attempt.outcome == FeatureAttemptOutcome::Successful && available {
                        let steps = plan.steps.as_deref().unwrap_or(&[]);
                        let foot = steps
                            .iter()
                            .take(attempt.position_step)
                            .filter(|s| matches!(s, SpinStep::ChangeFoot))
                            .count();
                        if attempt.feature == SpinFeature::EightRevolutions {
                            checks.apply("program-eight-revolutions", Some(index), &feature_source);
                        }
                        if attempt.feature == SpinFeature::EightRevolutions
                            && !matches!(steps.get(attempt.position_step), Some(SpinStep::Position { revolutions: Some(n), .. }) if *n >= 8)
                        {
                            checks.add(FindingKind::Unknown, "program-eight-revolutions", Some(index), "Eight uninterrupted revolutions are not established at the feature target.", &feature_source);
                            continue;
                        }
                        if first_success {
                            if uncertain_success.contains(&attempt.feature) {
                                checks.add(FindingKind::Unknown, "program-feature-once-per-program", Some(index), "An earlier attempt has an unknown outcome; first successful execution is not established.", &feature_source);
                                continue;
                            }
                            consumed.push(attempt.feature);
                        }
                        if per_foot.len() > 1 {
                            checks.apply("program-feature-per-foot", Some(index), &feature_source);
                        }
                        if per_foot.len() > 1 && per_foot[foot] >= 2 {
                            continue;
                        }
                        per_foot[foot] += 1;
                        credit += 1;
                        level_four_feature |= matches!(
                            attempt.feature,
                            SpinFeature::ChangeOfEdge
                                | SpinFeature::BothDirections
                                | SpinFeature::IncreaseOfSpeed
                                | SpinFeature::DifficultFlyingEntry
                                | SpinFeature::Windmills
                        );
                    }
                }
                if spin.planned_level == Some(Level::Four) {
                    checks.apply(
                        "program-spin-level-four-feature",
                        Some(index),
                        &feature_source,
                    );
                }
                if spin.planned_level.is_some_and(|l| {
                    l.number() > credit || (l == Level::Four && !level_four_feature)
                }) {
                    checks.add(FindingKind::Unknown, "program-spin-level-credit", Some(index), "The target level lacks eligible unused feature credit or a required level-four feature.", &feature_source);
                }
            }
            ElementPlanEvidence::Step {
                level: target,
                clean_difficult_turns,
                full_ice_coverage,
                movement,
                qualified_features,
                qualification_assumed,
            } => {
                for id in [
                    "program-step-recognition",
                    "program-step-feature-dependency",
                    "program-step-level-credit",
                ] {
                    checks.apply(id, Some(index), &policy.step_source);
                }
                if clean_difficult_turns.is_none_or(|n| n < 2)
                    || *full_ice_coverage != Some(true)
                    || (policy.step_movement_required && movement.is_none())
                {
                    checks.add(FindingKind::Unknown, "program-step-recognition", Some(index), "Basic step recognition needs two clean difficult turns, ice coverage and the profile movement requirement.", &policy.step_source);
                }
                let mut features = qualified_features.clone().unwrap_or_default();
                checks.apply(
                    "program-step-feature-dependency",
                    Some(index),
                    &source("pp. 2, 5, step features"),
                );
                features.sort_unstable();
                features.dedup();
                if features.iter().any(|n| !(1..=5).contains(n))
                    || (features.contains(&5) && !features.contains(&4))
                {
                    checks.add(
                        FindingKind::Unknown,
                        "program-step-feature-dependency",
                        Some(index),
                        "Unknown feature identity or feature five without feature four.",
                        &source("pp. 2, 5, step features"),
                    );
                }
                features.retain(|n| policy.permitted_step_features.contains(n));
                if !qualification_assumed
                    || qualified_features.is_none()
                    || target.number() > policy.max_step_level.number()
                    || target.number() as usize > features.len()
                    || (*target != Level::Basic && !features.contains(&1))
                {
                    checks.add(FindingKind::Unknown, "program-step-level-credit", Some(index), "The assumed step level lacks supported qualified features or exceeds the profile cap.", &policy.step_source);
                }
            }
            ElementPlanEvidence::ChoreographicSequence {
                movements,
                clearly_visible,
            } => {
                checks.apply(
                    "program-choreographic-sequence",
                    Some(index),
                    &policy.choreography_source,
                );
                checks.apply(
                    "program-choreographic-sequence",
                    Some(index),
                    &source("p. 6, choreographic sequence recognition"),
                );
                if let PlannedElement::ChoreographicSequence {
                    movements: Some(plan),
                } = element
                {
                    let input = SourceReference {
                        document: "Skating Program Lab input contract".into(),
                        version: "issue-49".into(),
                        location:
                            "typed ChSq evidence supplements ordered legacy movement descriptions"
                                .into(),
                    };
                    checks.apply(
                        "program-choreographic-plan-consistency",
                        Some(index),
                        &input,
                    );
                    match plan.iter().map(|m| SkatingMovement::from_description(m)).collect::<Option<Vec<_>>>() {
                        None => checks.add(FindingKind::Unsupported,"program-choreographic-plan-consistency",Some(index),"A legacy movement description is outside the supported exact vocabulary; it cannot be silently reconciled.",&input),
                        Some(parsed) if movements.as_ref().is_some_and(|m| *m != parsed) => checks.add(FindingKind::Violation,"program-choreographic-plan-consistency",Some(index),"Typed ChSq movement evidence contradicts the explicitly ordered plan movements.",&input),
                        _ => {},
                    }
                }
                if *clearly_visible != Some(true)
                    || movements
                        .as_ref()
                        .is_none_or(|m| !m.iter().any(|a| m.iter().any(|b| a != b)))
                {
                    checks.add(FindingKind::Unknown, "program-choreographic-sequence", Some(index), "Recognition needs two different supported movements and a clearly visible sequence.", &source("p. 6, choreographic sequence recognition"));
                }
            }
            ElementPlanEvidence::ChoreographicSpin {
                consecutive_revolutions_per_foot,
            } => {
                let qa = SourceReference {
                    document: "ISU Technical Committee Q&A 1".into(),
                    version: "2026-04-21".into(),
                    location: "p. 1, Choreospins Q1, same foot".into(),
                };
                checks.apply("program-choreographic-spin", Some(index), &qa);
                if consecutive_revolutions_per_foot
                    .as_ref()
                    .is_none_or(|feet| !feet.iter().any(|n| *n >= 3))
                {
                    checks.add(FindingKind::Unknown, "program-choreographic-spin", Some(index), "Recognition needs three consecutive revolutions on the same foot; split holds cannot be added.", &qa);
                }
            }
            ElementPlanEvidence::Jump { .. } => {}
        }
    }
    for coverage in &mut report.coverage {
        if matches!(
            coverage.family,
            ValidationFamily::SpinFeatures | ValidationFamily::StepsAndChoreography
        ) {
            coverage.coverage = FamilyCoverage::Partial;
            coverage.explanation = "Bounded ordered feature and movement checks with explicit qualification assumptions; unmodeled variants remain unsupported.";
        }
    }
    for src in report
        .validation
        .findings
        .iter()
        .map(|f| &f.source)
        .chain(report.applied_rules.iter().map(|r| &r.source))
    {
        if !report.sources.contains(src) {
            report.sources.push(src.clone());
        }
    }
    Ok(report)
}

/// Classification belongs to the shared validator, never to diagnostic prose.
#[cfg(feature = "optimizer")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateFindingScope {
    Element,
    ProgramObligation,
    ExecutionAssumption,
}

#[cfg(feature = "optimizer")]
pub(crate) fn candidate_finding_scope(
    finding: &Finding,
    profile: &RuleProfile,
) -> CandidateFindingScope {
    if finding.rule_id == crate::spin_validation::SPIN_EXECUTION_RULE {
        CandidateFindingScope::ExecutionAssumption
    } else if is_program_obligation(finding, profile) {
        CandidateFindingScope::ProgramObligation
    } else {
        CandidateFindingScope::Element
    }
}

pub(crate) fn is_program_obligation(finding: &Finding, profile: &RuleProfile) -> bool {
    finding.element_index.is_none()
        && (finding.rule_id == profile.binding(ProfileRule::Axel).id
            || finding.rule_id == profile.binding(ProfileRule::Eligibility).id
            || finding.rule_id == crate::spin_validation::SPIN_REQUIRED_ROLE_RULE
            || finding.rule_id == REQUIRED_CHOREOGRAPHIC_SPIN_RULE)
}

pub(crate) fn completion_findings(
    report: &EvidenceValidationReport,
    profile: &RuleProfile,
    program: &ProfileProgram,
    policy: &ProgramPolicy,
) -> Vec<usize> {
    let count = |family| {
        program
            .elements
            .iter()
            .filter(|e| ProfileElementFamily::of(e) == Some(family))
            .count()
    };
    let jump_count = program
        .elements
        .iter()
        .filter(|e| matches!(e, PlannedElement::Jump(_)))
        .count();
    let missing_spins = report
        .validation
        .findings
        .iter()
        .filter(|f| {
            f.kind == FindingKind::Violation
                && matches!(
                    f.rule_id,
                    "spin-required-role" | REQUIRED_CHOREOGRAPHIC_SPIN_RULE
                )
        })
        .count();
    let spin_room = count(ProfileElementFamily::RegularSpin)
        + count(ProfileElementFamily::ChoreographicSpin)
        + missing_spins
        <= policy.spins.max_spins;
    let family_room = profile
        .families
        .required
        .iter()
        .filter(|f| count(**f) == 0)
        .all(|f| match f {
            ProfileElementFamily::SoloJump => jump_count < profile.jumps.max_jumps,
            ProfileElementFamily::Combination => {
                jump_count < profile.jumps.max_jumps && profile.jumps.max_combinations > 0
            }
            ProfileElementFamily::JumpSequence => {
                jump_count < profile.jumps.max_jumps && profile.jumps.max_sequences > 0
            }
            ProfileElementFamily::RegularSpin => spin_room && policy.spins.max_spins > 0,
            ProfileElementFamily::ChoreographicSpin => {
                spin_room && policy.max_choreographic_spins > 0
            }
            ProfileElementFamily::StepSequence => policy.max_steps > 0,
            ProfileElementFamily::ChoreographicSequence => policy.max_choreographic_sequences > 0,
        });
    report
        .validation
        .findings
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == FindingKind::Violation && is_program_obligation(f, profile))
        .filter(|(_, f)| {
            if matches!(
                f.rule_id,
                "spin-required-role" | REQUIRED_CHOREOGRAPHIC_SPIN_RULE
            ) {
                spin_room
                    && (f.rule_id != REQUIRED_CHOREOGRAPHIC_SPIN_RULE
                        || count(ProfileElementFamily::ChoreographicSpin)
                            < policy.max_choreographic_spins)
            } else if f.rule_id == profile.binding(ProfileRule::Axel).id {
                jump_count < profile.jumps.max_jumps
            } else {
                family_room
            }
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(all(test, feature = "optimizer"))]
mod candidate_scope_tests {
    use super::*;
    #[test]
    fn required_family_classification_is_independent_of_diagnostic_prose() {
        let profile = adult_profile(AdultCategory::Masters);
        let mut finding = Finding {
            kind: FindingKind::Violation,
            rule_id: profile.binding(ProfileRule::Eligibility).id,
            element_index: None,
            component_index: None,
            explanation: "changed diagnostic wording",
            source: profile.sources[0].clone(),
        };
        assert_eq!(
            candidate_finding_scope(&finding, &profile),
            CandidateFindingScope::ProgramObligation
        );
        finding.element_index = Some(0);
        assert_eq!(
            candidate_finding_scope(&finding, &profile),
            CandidateFindingScope::Element
        );
        finding.rule_id = "program-family-cap";
        finding.element_index = None;
        assert_eq!(
            candidate_finding_scope(&finding, &profile),
            CandidateFindingScope::Element
        );
    }
}
