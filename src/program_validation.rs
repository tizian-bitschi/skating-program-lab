//! Bounded mixed-program checks. Qualification is an explicit scenario assumption.
use crate::*;

const REQUIRED_CHOREOGRAPHIC_SPIN_RULE: &str = "program-required-choreographic-spin";

#[derive(Clone, Copy, PartialEq, Eq)]
enum EvaluationBasis {
    Plan,
    Qualification,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EvidenceRequirement {
    Plan,
    FeatureAttempts,
    PositionFeatures,
    EightRevolutions,
    FlyingEntry,
}

impl EvidenceRequirement {
    fn includes(self, feature: SpinFeature) -> bool {
        match self {
            Self::Plan => false,
            Self::FeatureAttempts => true,
            Self::PositionFeatures => position_limited_feature(feature),
            Self::EightRevolutions => feature == SpinFeature::EightRevolutions,
            Self::FlyingEntry => feature == SpinFeature::DifficultFlyingEntry,
        }
    }
}

fn position_limited_feature(feature: SpinFeature) -> bool {
    matches!(
        feature,
        SpinFeature::ChangeOfEdge
            | SpinFeature::BothDirections
            | SpinFeature::IncreaseOfSpeed
            | SpinFeature::DifficultBlade
    )
}

/// Shared prerequisite classification: absent facts differ from unmodeled variants.
fn feature_position_issue(
    plan: &RegularSpin,
    spin: &RegularSpinEvidence,
    attempt: &SpinFeatureAttempt,
) -> Option<FindingKind> {
    if attempt.feature != SpinFeature::EightRevolutions
        && !position_limited_feature(attempt.feature)
    {
        return None;
    }
    let position = plan
        .steps
        .as_ref()
        .and_then(|steps| steps.get(attempt.position_step));
    match position {
        Some(SpinStep::Position {
            position: Some(SpinPosition::Camel),
            ..
        }) => None,
        Some(SpinStep::Position {
            position: Some(SpinPosition::Sit),
            ..
        }) if attempt.feature != SpinFeature::EightRevolutions => None,
        Some(SpinStep::Position {
            position: Some(SpinPosition::Upright),
            ..
        }) if attempt.feature != SpinFeature::EightRevolutions
            && spin
                .abbreviation
                .is_some_and(|a| a.kind == RegularSpinKind::Layback) =>
        {
            None
        }
        Some(SpinStep::Position {
            position: Some(_), ..
        }) => Some(FindingKind::Unsupported),
        _ => Some(FindingKind::Unknown),
    }
}

struct RuleDescriptor {
    id: &'static str,
    basis: EvaluationBasis,
    evidence: EvidenceRequirement,
}

// One metadata declaration supplies optimizer inventory and coverage semantics.
macro_rules! program_rules {
    ($(($id:expr, $basis:ident, $evidence:ident)),* $(,)?) => {
        #[cfg(feature = "optimizer")]
        pub(crate) const RULE_IDS: &[&str] = &[$($id),*];
        const RULES: &[RuleDescriptor] = &[$(RuleDescriptor { id: $id, basis: EvaluationBasis::$basis, evidence: EvidenceRequirement::$evidence }),*];
    };
}
program_rules![
    ("program-choreographic-plan-consistency", Plan, Plan),
    ("program-choreographic-sequence", Qualification, Plan),
    ("program-choreographic-spin", Qualification, Plan),
    ("program-eight-position", Plan, EightRevolutions),
    ("program-eight-revolutions", Qualification, EightRevolutions),
    ("program-family-cap", Plan, Plan),
    (
        "program-feature-once-per-program",
        Qualification,
        FeatureAttempts
    ),
    ("program-feature-per-foot", Qualification, FeatureAttempts),
    (
        "program-feature-qualification",
        Qualification,
        FeatureAttempts
    ),
    ("program-feature-position", Plan, PositionFeatures),
    ("program-flying-feature-entry", Plan, FlyingEntry),
    (REQUIRED_CHOREOGRAPHIC_SPIN_RULE, Plan, Plan),
    (
        crate::validation::SEQUENCE_DIRECT_STEP_RULE,
        Qualification,
        Plan
    ),
    ("program-spin-level-credit", Qualification, Plan),
    ("program-spin-level-four-feature", Qualification, Plan),
    ("program-step-feature-dependency", Qualification, Plan),
    ("program-step-level-credit", Qualification, Plan),
    ("program-step-recognition", Qualification, Plan),
];

pub(crate) fn assumption_rule(id: &str) -> bool {
    RULES
        .iter()
        .any(|rule| rule.id == id && rule.basis == EvaluationBasis::Qualification)
}

pub(crate) fn unevaluated_rule_state(
    id: &str,
    element: &PlannedElement,
    evidence: &ElementPlanEvidence,
) -> RuleCoverageState {
    if let Some(rule) = RULES
        .iter()
        .find(|r| r.id == id && r.evidence != EvidenceRequirement::Plan)
    {
        if let (PlannedElement::RegularSpin(plan), ElementPlanEvidence::Spin { spin, .. }) =
            (element, evidence)
        {
            return match &spin.feature_attempts {
                None => RuleCoverageState::MissingEvidence,
                Some(attempts) => {
                    let relevant: Vec<_> = attempts
                        .iter()
                        .filter(|a| rule.evidence.includes(a.feature))
                        .collect();
                    if relevant.iter().any(|a| {
                        crate::spin_validation::feature_target_validity(plan, a, None) != Some(true)
                            || feature_position_issue(plan, spin, a) == Some(FindingKind::Unknown)
                    }) {
                        RuleCoverageState::MissingEvidence
                    } else if relevant.iter().any(|a| {
                        feature_position_issue(plan, spin, a) == Some(FindingKind::Unsupported)
                    }) {
                        RuleCoverageState::Unsupported
                    } else {
                        RuleCoverageState::NotApplicable
                    }
                }
            };
        }
    }
    RuleCoverageState::MissingEvidence
}

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
    validate_program_scope(
        registry,
        program,
        evidence,
        policy,
        EvaluationScope::EvidenceBased,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvaluationScope {
    EvidenceBased,
    AssumedRecognition,
}

pub(crate) fn validate_program_scope(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    evidence: &[ElementPlanEvidence],
    policy: &ProgramPolicy,
    scope: EvaluationScope,
) -> Result<EvidenceValidationReport, ProfileSelectionError> {
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    let invalid = |reason: &str| ProfileSelectionError {
        profile: program.profile.clone(),
        kind: ProfileErrorKind::Invalid,
        reason: reason.into(),
    };
    crate::profiles::resolve_program_policy(profile, Some(policy))?;
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
        match scope {
            EvaluationScope::EvidenceBased => {
                crate::spin_validation::FeatureCreditEvaluation::ProgramWide
            }
            EvaluationScope::AssumedRecognition => {
                crate::spin_validation::FeatureCreditEvaluation::AssumedRecognition
            }
        },
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
    let mut eight_success_seen = false;
    let mut eight_uncertain_seen = false;
    let mut unknown_attempts_seen = false;
    let mut uncertain_attempts = Vec::new();
    for (index, (element, data)) in program.elements.iter().zip(evidence).enumerate() {
        if scope == EvaluationScope::AssumedRecognition {
            if let ElementPlanEvidence::Step { level, .. } = data {
                checks.apply("program-step-level-cap", Some(index), &policy.step_source);
                if level.number() > policy.max_step_level.number() {
                    checks.add(
                        FindingKind::Violation,
                        "program-step-level-cap",
                        Some(index),
                        "The selected SOV level exceeds the profile cap.",
                        &policy.step_source,
                    );
                }
            }
            continue;
        }
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
                let mut eight_success_in_spin = false;
                let mut eight_uncertain_in_spin = spin.feature_attempts.is_none();
                let mut eight_success_feet = Vec::new();
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
                        if valid_target.is_none() {
                            if attempt.feature == SpinFeature::EightRevolutions {
                                eight_uncertain_in_spin |=
                                    attempt.outcome != FeatureAttemptOutcome::Unsuccessful;
                            } else {
                                uncertain_attempts.push(attempt.feature);
                            }
                        }
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
                    let position_issue = feature_position_issue(plan, spin, attempt);
                    if attempt.feature == SpinFeature::EightRevolutions {
                        if let Some(kind) = position_issue {
                            eight_uncertain_in_spin |=
                                attempt.outcome != FeatureAttemptOutcome::Unsuccessful;
                            checks.add(kind, "program-eight-position", Some(index), if kind == FindingKind::Unknown { "The feature target has no supplied basic position." } else { "This bounded eight-revolution subset supports plain camel only; difficult variation and Layback evidence remain unmodeled." }, &feature_source);
                            continue;
                        }
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
                    let prior_attempt_uncertain = !first_success
                        && !consumed.contains(&attempt.feature)
                        && (unknown_attempts_seen || uncertain_attempts.contains(&attempt.feature));
                    let available = if first_success {
                        !eight_success_seen
                    } else {
                        !consumed.contains(&attempt.feature)
                    };
                    if !first_success {
                        consumed.push(attempt.feature);
                    }
                    if first_success
                        && (attempt.outcome == FeatureAttemptOutcome::Unknown
                            || (attempt.outcome == FeatureAttemptOutcome::Successful
                                && !local_feature_qualification_assumed))
                    {
                        eight_uncertain_in_spin = true;
                    }
                    let position_limited = position_limited_feature(attempt.feature);
                    if position_limited {
                        checks.apply("program-feature-position", Some(index), &feature_source);
                    }
                    if position_limited {
                        if let Some(kind) = position_issue {
                            checks.add(kind, "program-feature-position", Some(index), if kind == FindingKind::Unknown { "The feature target has no supplied basic position." } else { "This feature requires camel, sit, Layback, Biellmann or a difficult upright variation; generic upright evidence cannot qualify it." }, &feature_source);
                            continue;
                        }
                    }
                    if prior_attempt_uncertain {
                        checks.add(FindingKind::Unknown, "program-feature-once-per-program", Some(index), "Missing earlier feature-list or target evidence prevents establishing the first attempt.", &feature_source);
                    }
                    if attempt.outcome == FeatureAttemptOutcome::Unknown
                        || !local_feature_qualification_assumed
                    {
                        checks.add(FindingKind::Unknown, "program-feature-qualification", Some(index), "Feature outcome or explicit local qualification assumption is missing.", &feature_source);
                    } else if attempt.outcome == FeatureAttemptOutcome::Successful
                        && available
                        && !prior_attempt_uncertain
                    {
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
                            eight_uncertain_in_spin |= matches!(
                                steps.get(attempt.position_step),
                                Some(SpinStep::Position {
                                    revolutions: None,
                                    ..
                                })
                            );
                            checks.add(FindingKind::Unknown, "program-eight-revolutions", Some(index), "Eight uninterrupted revolutions are not established at the feature target.", &feature_source);
                            continue;
                        }
                        if per_foot.len() > 1 {
                            checks.apply("program-feature-per-foot", Some(index), &feature_source);
                        }
                        if first_success {
                            eight_success_in_spin = true;
                            if eight_uncertain_seen {
                                checks.add(FindingKind::Unknown, "program-feature-once-per-program", Some(index), "An earlier attempt has an unknown outcome; first successful execution is not established.", &feature_source);
                                continue;
                            }
                            eight_success_feet.push(foot);
                            continue;
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
                // Allocate feature 10 after the first-attempt features, so its
                // choice of foot is independent of assertion order on that foot.
                if eight_success_feet
                    .iter()
                    .any(|foot| per_foot.len() == 1 || per_foot[*foot] < 2)
                {
                    credit += 1;
                }
                // Feature 10 is reserved by the first successful spin, while
                // either foot in that spin can receive its single credit.
                eight_success_seen |= eight_success_in_spin;
                eight_uncertain_seen |= eight_uncertain_in_spin && !eight_success_in_spin;
                unknown_attempts_seen |= spin.feature_attempts.is_none();
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
        if scope == EvaluationScope::AssumedRecognition {
            match coverage.family {
                ValidationFamily::RegularSpins => {
                    coverage.explanation = "Assumed recognition: spin counts, roles, abbreviations and level caps are checked; execution is unverified.";
                }
                ValidationFamily::SpinFeatures => {
                    coverage.explanation =
                        "Feature qualification is outside the assumed-recognition evaluation.";
                }
                ValidationFamily::StepsAndChoreography => {
                    coverage.coverage = FamilyCoverage::Partial;
                    coverage.explanation = "Assumed recognition: family counts and step level caps are checked; movements are unverified.";
                }
                _ => {}
            }
        } else if matches!(
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
    evidence: &[ElementPlanEvidence],
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
    let spin_slots = policy.spins.max_spins.saturating_sub(
        count(ProfileElementFamily::RegularSpin) + count(ProfileElementFamily::ChoreographicSpin),
    );
    let Some(missing_regular_roles) =
        crate::spin_validation::required_spin_insertions(&policy.spins, evidence, spin_slots)
    else {
        return vec![];
    };
    let missing =
        |family| usize::from(profile.families.required.contains(&family) && count(family) == 0);
    let missing_axel = usize::from(report.validation.findings.iter().any(|f| {
        f.rule_id == profile.binding(ProfileRule::Axel).id && f.kind == FindingKind::Violation
    }));
    let missing_chsp = missing(ProfileElementFamily::ChoreographicSpin).max(usize::from(
        policy.require_choreographic_spin && count(ProfileElementFamily::ChoreographicSpin) == 0,
    ));
    let missing_regular = missing(ProfileElementFamily::RegularSpin).max(missing_regular_roles);
    let missing_combos = missing(ProfileElementFamily::Combination);
    let missing_sequences = missing(ProfileElementFamily::JumpSequence);
    let needed_jumps =
        (missing(ProfileElementFamily::SoloJump) + missing_combos + missing_sequences)
            .max(missing_axel);
    // Missing families share the same capacity. An Axel can occupy one of the
    // required jump families; ChSp and regular roles cannot share an element.
    let jointly_insertable = jump_count + needed_jumps <= profile.jumps.max_jumps
        && (!crate::senior::selected(profile, ProfileRule::Count)
            || program.elements.len()
                + needed_jumps
                + missing_regular
                + missing_chsp
                + missing(ProfileElementFamily::StepSequence)
                + missing(ProfileElementFamily::ChoreographicSequence)
                <= crate::senior::DEFINITION.max_elements)
        && count(ProfileElementFamily::Combination) + missing_combos
            <= profile.jumps.max_combinations
        && count(ProfileElementFamily::JumpSequence) + missing_sequences
            <= profile.jumps.max_sequences
        && count(ProfileElementFamily::Combination)
            + count(ProfileElementFamily::JumpSequence)
            + missing_combos
            + missing_sequences
            <= profile.jumps.max_compounds
        && count(ProfileElementFamily::RegularSpin)
            + count(ProfileElementFamily::ChoreographicSpin)
            + missing_regular
            + missing_chsp
            <= policy.spins.max_spins
        && count(ProfileElementFamily::ChoreographicSpin) + missing_chsp
            <= policy.max_choreographic_spins
        && count(ProfileElementFamily::StepSequence) + missing(ProfileElementFamily::StepSequence)
            <= policy.max_steps
        && count(ProfileElementFamily::ChoreographicSequence)
            + missing(ProfileElementFamily::ChoreographicSequence)
            <= policy.max_choreographic_sequences;
    report
        .validation
        .findings
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == FindingKind::Violation && is_program_obligation(f, profile))
        .filter(|_| jointly_insertable)
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
