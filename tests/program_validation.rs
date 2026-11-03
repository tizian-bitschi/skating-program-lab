use skating_program_lab::*;

fn spin(
    kind: RegularSpinKind,
    target: Level,
    attempts: Vec<SpinFeatureAttempt>,
) -> (PlannedElement, ElementPlanEvidence) {
    let position = if kind == RegularSpinKind::Camel {
        SpinPosition::Camel
    } else {
        SpinPosition::Sit
    };
    (
        PlannedElement::RegularSpin(RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: Some(vec![SpinStep::Position {
                position: Some(position),
                revolutions: Some(8),
                feature_attempts: Some(vec![]),
            }]),
        }),
        ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry: SpinEntry::NonFlying,
                    change_foot: false,
                }),
                planned_level: Some(target),
                feature_attempts: Some(attempts),
                panel: None,
                final_upright_wind_up: None,
            },
            local_feature_qualification_assumed: true,
        },
    )
}
fn attempt(feature: SpinFeature, outcome: FeatureAttemptOutcome) -> SpinFeatureAttempt {
    SpinFeatureAttempt {
        feature,
        position_step: 0,
        outcome,
    }
}
fn validate(
    category: AdultCategory,
    entries: Vec<(PlannedElement, ElementPlanEvidence)>,
) -> EvidenceValidationReport {
    let (elements, evidence): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
    validate_supported_program(
        &ProfileRegistry::adult(),
        &ProfileProgram {
            profile: adult_profile(category).id,
            scoring_table: ScoringTableId::Isu2786June2_2026,
            elements,
        },
        &evidence,
        &adult_program_policy(category),
    )
    .unwrap()
}
fn has(report: &EvidenceValidationReport, id: &str, index: Option<usize>) -> bool {
    report
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == id && f.element_index == index)
}

#[test]
fn first_attempt_consumes_feature_even_when_unsuccessful() {
    let a = spin(
        RegularSpinKind::Camel,
        Level::Basic,
        vec![attempt(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Unsuccessful,
        )],
    );
    let b = spin(
        RegularSpinKind::Sit,
        Level::One,
        vec![attempt(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Successful,
        )],
    );
    let report = validate(AdultCategory::Bronze, vec![a.clone(), b.clone()]);
    assert!(has(&report, "program-spin-level-credit", Some(1)));
    let reversed = validate(AdultCategory::Bronze, vec![b, a]);
    assert!(!has(&reversed, "program-spin-level-credit", Some(0)));
    assert!(report
        .sources
        .iter()
        .any(|s| s.document == "ISU Communication 2788" && s.version == "2026-05-19"));
}

#[test]
fn eight_revolution_feature_waits_for_first_success_but_missing_outcome_stays_unknown() {
    let a = spin(
        RegularSpinKind::Camel,
        Level::Basic,
        vec![attempt(
            SpinFeature::EightRevolutions,
            FeatureAttemptOutcome::Unsuccessful,
        )],
    );
    let mut b = spin(
        RegularSpinKind::Camel,
        Level::One,
        vec![attempt(
            SpinFeature::EightRevolutions,
            FeatureAttemptOutcome::Successful,
        )],
    );
    // Distinct abbreviation for the second camel; two foot segments with 8 rev each.
    if let PlannedElement::RegularSpin(plan) = &mut b.0 {
        plan.steps.as_mut().unwrap().extend([
            SpinStep::ChangeFoot,
            SpinStep::Position {
                position: Some(SpinPosition::Camel),
                revolutions: Some(8),
                feature_attempts: Some(vec![]),
            },
        ]);
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut b.1 {
        spin.abbreviation.as_mut().unwrap().change_foot = true;
    }
    let report = validate(AdultCategory::Silver, vec![a.clone(), b.clone()]);
    assert!(!has(&report, "program-spin-level-credit", Some(1)));
    let mut first_success = a.clone();
    if let ElementPlanEvidence::Spin { spin, .. } = &mut first_success.1 {
        spin.feature_attempts.as_mut().unwrap()[0].outcome = FeatureAttemptOutcome::Successful;
    }
    assert!(has(
        &validate(AdultCategory::Silver, vec![first_success, b]),
        "program-spin-level-credit",
        Some(1)
    ));
    let mut unknown = a;
    if let ElementPlanEvidence::Spin { spin, .. } = &mut unknown.1 {
        spin.feature_attempts.as_mut().unwrap()[0].outcome = FeatureAttemptOutcome::Unknown;
    }
    assert!(has(
        &validate(AdultCategory::Silver, vec![unknown]),
        "program-feature-qualification",
        Some(0)
    ));
}

fn step(
    level: Level,
    movement: Option<SkatingMovement>,
    features: Vec<u8>,
) -> (PlannedElement, ElementPlanEvidence) {
    (
        PlannedElement::StepSequence,
        ElementPlanEvidence::Step {
            level,
            clean_difficult_turns: Some(2),
            full_ice_coverage: Some(true),
            movement,
            qualified_features: Some(features),
            qualification_assumed: true,
        },
    )
}

#[test]
fn gold_steps_require_movement_and_ignore_features_three_to_five() {
    let r = validate(AdultCategory::Gold, vec![step(Level::One, None, vec![1])]);
    assert!(has(&r, "program-step-recognition", Some(0)));
    let r = validate(
        AdultCategory::Gold,
        vec![step(
            Level::Two,
            Some(SkatingMovement::Spiral),
            vec![1, 3, 4, 5],
        )],
    );
    assert!(has(&r, "program-step-level-credit", Some(0)));
    let r = validate(
        AdultCategory::Gold,
        vec![step(Level::Two, Some(SkatingMovement::Spiral), vec![1, 2])],
    );
    assert!(!has(&r, "program-step-level-credit", Some(0)));
    assert!(!has(&r, "program-step-recognition", Some(0)));
    let r = validate(
        AdultCategory::Masters,
        vec![step(Level::Two, None, vec![1, 5])],
    );
    assert!(has(&r, "program-step-feature-dependency", Some(0)));
}

#[test]
fn choreographic_families_have_distinct_counts_and_same_foot_recognition() {
    let chsp = |feet| {
        (
            PlannedElement::ChoreographicSpin,
            ElementPlanEvidence::ChoreographicSpin {
                consecutive_revolutions_per_foot: Some(feet),
            },
        )
    };
    assert!(has(
        &validate(AdultCategory::Gold, vec![chsp(vec![2, 2])]),
        "program-choreographic-spin",
        Some(0)
    ));
    assert!(!has(
        &validate(AdultCategory::Gold, vec![chsp(vec![3])]),
        "program-choreographic-spin",
        Some(0)
    ));
    assert!(has(
        &validate(AdultCategory::Bronze, vec![chsp(vec![3])]),
        "program-family-cap",
        None
    ));
    let chsq = |movements| {
        (
            PlannedElement::ChoreographicSequence { movements: None },
            ElementPlanEvidence::ChoreographicSequence {
                movements: Some(movements),
                clearly_visible: Some(true),
            },
        )
    };
    assert!(has(
        &validate(
            AdultCategory::Silver,
            vec![chsq(vec![SkatingMovement::Spiral, SkatingMovement::Spiral])]
        ),
        "program-choreographic-sequence",
        Some(0)
    ));
    let good = chsq(vec![SkatingMovement::Spiral, SkatingMovement::InaBauer]);
    assert!(!has(
        &validate(AdultCategory::Silver, vec![good.clone()]),
        "program-choreographic-sequence",
        Some(0)
    ));
    assert!(has(
        &validate(AdultCategory::Silver, vec![good.clone(), good.clone()]),
        "program-family-cap",
        None
    ));
    assert!(has(
        &validate(AdultCategory::Gold, vec![good]),
        "program-family-cap",
        None
    ));
}

#[test]
fn missing_qualification_or_unmodeled_variations_never_prove_level_credit() {
    let mut entry = spin(
        RegularSpinKind::Sit,
        Level::One,
        vec![attempt(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Successful,
        )],
    );
    if let ElementPlanEvidence::Spin {
        local_feature_qualification_assumed,
        ..
    } = &mut entry.1
    {
        *local_feature_qualification_assumed = false;
    }
    assert!(has(
        &validate(AdultCategory::Bronze, vec![entry]),
        "program-feature-qualification",
        Some(0)
    ));
    let r = validate(
        AdultCategory::Bronze,
        vec![spin(
            RegularSpinKind::Sit,
            Level::One,
            vec![attempt(
                SpinFeature::DifficultVariation,
                FeatureAttemptOutcome::Successful,
            )],
        )],
    );
    assert!(r.validation.findings.iter().any(
        |f| f.kind == FindingKind::Unsupported && f.rule_id == "program-feature-qualification"
    ));
}
