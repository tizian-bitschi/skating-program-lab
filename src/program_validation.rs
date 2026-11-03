//! Bounded mixed-program checks. Qualification is an explicit scenario assumption.
use crate::*;

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
}

pub fn adult_program_policy(category: AdultCategory) -> ProgramPolicy {
    let advanced = matches!(category, AdultCategory::Gold | AdultCategory::Masters);
    ProgramPolicy {
        spins: adult_regular_spin_policy(category),
        max_steps: usize::from(advanced),
        max_choreographic_sequences: usize::from(!advanced),
        max_choreographic_spins: usize::from(advanced),
        require_choreographic_spin: advanced,
        max_step_level: if category == AdultCategory::Gold {
            Level::Two
        } else {
            Level::Four
        },
        step_movement_required: category == AdultCategory::Gold,
        permitted_step_features: if category == AdultCategory::Gold {
            vec![1, 2]
        } else {
            vec![1, 2, 3, 4, 5]
        },
    }
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

fn finding(
    kind: FindingKind,
    id: &'static str,
    index: Option<usize>,
    text: &'static str,
    source: &SourceReference,
) -> Finding {
    Finding {
        kind,
        rule_id: id,
        element_index: index,
        component_index: None,
        explanation: text,
        source: source.clone(),
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
    let mut report = validate_profile_program_with_evidence(
        registry,
        program,
        &spin_evidence,
        Some(&policy.spins),
    )?;
    // Only this dedicated evaluator supersedes the old feature-credit placeholder.
    report
        .validation
        .findings
        .retain(|f| f.rule_id != "spin-feature-credit");
    let mut add = |kind, id, index, text, src: &SourceReference| {
        report
            .validation
            .findings
            .push(finding(kind, id, index, text, src))
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
        if count(family) > cap {
            add(
                FindingKind::Violation,
                "program-family-cap",
                None,
                "The family count exceeds the selected program policy.",
                &policy.spins.source,
            );
        }
    }
    if policy.require_choreographic_spin && count(ProfileElementFamily::ChoreographicSpin) == 0 {
        add(
            FindingKind::Violation,
            "program-required-choreographic-spin",
            None,
            "The profile requires a choreographic spin.",
            &policy.spins.source,
        );
    }
    let feature_source =
        source("pp. 2, 4-5, Single spin feature credit, reuse, quotas and level 4");
    let mut consumed = Vec::new();
    for (index, (element, data)) in program.elements.iter().zip(evidence).enumerate() {
        match data {
            ElementPlanEvidence::Spin {
                spin,
                local_feature_qualification_assumed,
            } => {
                let PlannedElement::RegularSpin(plan) = element else {
                    unreachable!()
                };
                let mut credit = 0;
                let mut level_four_feature = false;
                let mut per_foot = vec![
                    0usize;
                    plan.steps.as_ref().map_or(1, |s| 1 + s
                        .iter()
                        .filter(|s| matches!(s, SpinStep::ChangeFoot))
                        .count())
                ];
                for attempt in spin.feature_attempts.iter().flatten() {
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
                    if attempt.feature == SpinFeature::DifficultFlyingEntry
                        && plan.entry != Some(SpinEntry::Flying)
                    {
                        add(
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
                        add(FindingKind::Unsupported, "program-eight-position", Some(index), "This bounded eight-revolution subset supports plain camel only; difficult variation and Layback evidence remain unmodeled.", &feature_source);
                        continue;
                    }
                    if !supported {
                        add(FindingKind::Unsupported, "program-feature-qualification", Some(index), "This feature needs unmodeled variation, foot-change or entrance/exit evidence.", &feature_source);
                        continue;
                    }
                    let first_success = attempt.feature == SpinFeature::EightRevolutions;
                    let available = !consumed.contains(&attempt.feature);
                    if !first_success || attempt.outcome == FeatureAttemptOutcome::Successful {
                        consumed.push(attempt.feature);
                    }
                    if attempt.outcome == FeatureAttemptOutcome::Unknown
                        || !local_feature_qualification_assumed
                    {
                        add(FindingKind::Unknown, "program-feature-qualification", Some(index), "Feature outcome or explicit local qualification assumption is missing.", &feature_source);
                    } else if attempt.outcome == FeatureAttemptOutcome::Successful && available {
                        let steps = plan.steps.as_deref().unwrap_or(&[]);
                        let foot = steps
                            .iter()
                            .take(attempt.position_step)
                            .filter(|s| matches!(s, SpinStep::ChangeFoot))
                            .count();
                        if attempt.feature == SpinFeature::EightRevolutions
                            && !matches!(steps.get(attempt.position_step), Some(SpinStep::Position { revolutions: Some(n), .. }) if *n >= 8)
                        {
                            add(FindingKind::Unknown, "program-eight-revolutions", Some(index), "Eight uninterrupted revolutions are not established at the feature target.", &feature_source);
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
                if spin
                    .planned_level
                    .is_some_and(|l| level(l) > credit || (l == Level::Four && !level_four_feature))
                {
                    add(FindingKind::Unknown, "program-spin-level-credit", Some(index), "The target level lacks eligible unused feature credit or a required level-four feature.", &feature_source);
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
                if clean_difficult_turns.is_none_or(|n| n < 2)
                    || *full_ice_coverage != Some(true)
                    || (policy.step_movement_required && movement.is_none())
                {
                    add(FindingKind::Unknown, "program-step-recognition", Some(index), "Basic step recognition needs two clean difficult turns, ice coverage and the profile movement requirement.", &policy.spins.source);
                }
                let mut features = qualified_features.clone().unwrap_or_default();
                features.sort_unstable();
                features.dedup();
                if features.iter().any(|n| !(1..=5).contains(n))
                    || (features.contains(&5) && !features.contains(&4))
                {
                    add(
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
                    || level(*target) > level(policy.max_step_level)
                    || level(*target) as usize > features.len()
                    || (*target != Level::Basic && !features.contains(&1))
                {
                    add(FindingKind::Unknown, "program-step-level-credit", Some(index), "The assumed step level lacks supported qualified features or exceeds the profile cap.", &policy.spins.source);
                }
            }
            ElementPlanEvidence::ChoreographicSequence {
                movements,
                clearly_visible,
            } => {
                if *clearly_visible != Some(true)
                    || movements
                        .as_ref()
                        .is_none_or(|m| !m.iter().any(|a| m.iter().any(|b| a != b)))
                {
                    add(FindingKind::Unknown, "program-choreographic-sequence", Some(index), "Recognition needs two different supported movements and a clearly visible sequence.", &source("p. 6, choreographic sequence; Adult pp. 10-11"));
                }
            }
            ElementPlanEvidence::ChoreographicSpin {
                consecutive_revolutions_per_foot,
            } => {
                if consecutive_revolutions_per_foot
                    .as_ref()
                    .is_none_or(|feet| !feet.iter().any(|n| *n >= 3))
                {
                    let qa = SourceReference {
                        document: "ISU Technical Committee Q&A 1".into(),
                        version: "2026-04-21".into(),
                        location: "p. 1, Choreospins Q1, same foot".into(),
                    };
                    add(FindingKind::Unknown, "program-choreographic-spin", Some(index), "Recognition needs three consecutive revolutions on the same foot; split holds cannot be added.", &qa);
                }
            }
            ElementPlanEvidence::Jump {
                direct_transitions_assumed,
            } => {
                if matches!(element, PlannedElement::Jump(JumpElement::Sequence(_)))
                    && !direct_transitions_assumed
                {
                    add(FindingKind::Unknown, "program-sequence-direct-step", Some(index), "A planned sequence needs an explicit direct Axel-step assumption; recognition is not inferred.", &source("p. 3, jump sequences; Adult pp. 8-10"));
                }
            }
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
        .chain(std::iter::once(&feature_source))
    {
        if !report.sources.contains(src) {
            report.sources.push(src.clone());
        }
    }
    Ok(report)
}

pub(crate) fn level(l: Level) -> u8 {
    match l {
        Level::Basic => 0,
        Level::One => 1,
        Level::Two => 2,
        Level::Three => 3,
        Level::Four => 4,
    }
}
