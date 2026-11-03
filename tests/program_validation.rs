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

#[test]
fn feature_evaluation_scope_preserves_local_checks_and_legacy_limits() {
    let (element, data) = spin(
        RegularSpinKind::Sit,
        Level::One,
        vec![attempt(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Successful,
        )],
    );
    let ElementPlanEvidence::Spin {
        spin: mut evidence, ..
    } = data.clone()
    else {
        unreachable!()
    };
    let profile = adult_profile(AdultCategory::Bronze);
    let program = ProfileProgram {
        profile: profile.id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        elements: vec![element.clone()],
    };
    let local = |spin| {
        validate_profile_program_with_evidence(
            &ProfileRegistry::adult(),
            &program,
            &ProgramValidationEvidence {
                spins: vec![IndexedSpinEvidence {
                    element_index: 0,
                    spin,
                }],
            },
            None,
        )
        .unwrap()
    };
    let legacy = local(evidence.clone());
    assert!(legacy
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-feature-credit" && f.kind == FindingKind::Unsupported));
    let mixed = validate(AdultCategory::Bronze, vec![(element.clone(), data)]);
    assert!(!mixed
        .applied_rules
        .iter()
        .any(|r| r.rule_id == "spin-feature-credit"));
    assert!(!mixed
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-feature-credit"));
    assert!(mixed
        .applied_rules
        .iter()
        .any(|r| r.rule_id == "program-spin-level-credit"));
    assert!(!has(&mixed, "program-spin-level-credit", Some(0)));

    evidence.feature_attempts.as_mut().unwrap()[0].position_step = 99;
    let invalid_local = local(evidence.clone());
    let invalid_mixed = validate(
        AdultCategory::Bronze,
        vec![(
            element,
            ElementPlanEvidence::Spin {
                spin: evidence,
                local_feature_qualification_assumed: true,
            },
        )],
    );
    for report in [invalid_local, invalid_mixed] {
        assert!(report
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == "spin-feature-target"
                && f.kind == FindingKind::Violation
                && f.component_index == Some(99)));
    }
}

#[test]
fn sequence_assumptions_resolve_only_the_supported_direct_step_uncertainty() {
    let element = PlannedElement::Jump(JumpElement::Sequence(vec![
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(1),
        }),
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        }),
    ]));
    let missing = validate(
        AdultCategory::Silver,
        vec![(
            element.clone(),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
        )],
    );
    let diagnostics: Vec<_> = missing
        .validation
        .findings
        .iter()
        .filter(|f| f.element_index == Some(0))
        .collect();
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = diagnostics[0];
    assert_eq!(diagnostic.kind, FindingKind::Unknown);
    assert_eq!(
        diagnostic.rule_id,
        "isu-adult-2026-27-combination-structure"
    );
    assert_eq!(
        diagnostic.explanation,
        "The plan does not establish the direct step into the Axel or panel recognition."
    );
    assert_eq!(
        diagnostic.source.document,
        "International Adult Figure Skating Competitions Technical Announcement"
    );
    assert_eq!(diagnostic.source.version, "2026-07-16");
    assert_eq!(
        diagnostic.source.location,
        "p. 10, Silver Free Skating, jumps"
    );
    let supplied = validate(
        AdultCategory::Silver,
        vec![(
            element.clone(),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed: true,
            },
        )],
    );
    assert!(!supplied.validation.findings.iter().any(|f| f.explanation
        == "The plan does not establish the direct step into the Axel or panel recognition."));
    for report in [&missing, &supplied] {
        let traces: Vec<_> = report
            .applied_rules
            .iter()
            .filter(|r| r.rule_id == "program-sequence-direct-step")
            .collect();
        assert_eq!(traces.len(), 1);
        let trace = traces[0];
        assert_eq!(trace.element_index, Some(0));
        assert_eq!(trace.source.document, "ISU Communication 2788");
        assert_eq!(trace.source.version, "2026-05-19");
        assert_eq!(trace.source.location, "p. 3, jump sequences");
        assert!(report.sources.contains(&trace.source));
        assert!(report.sources.contains(&diagnostic.source));
        assert!(!report
            .sources
            .iter()
            .any(|s| s.document == "ISU Communication 2788" && s.location.contains("Adult")));
    }
    let PlannedElement::Jump(JumpElement::Sequence(mut parts)) = element else {
        unreachable!()
    };
    parts.insert(1, JumpPart::UnspecifiedTransition);
    let unresolved = validate(
        AdultCategory::Silver,
        vec![(
            PlannedElement::Jump(JumpElement::Sequence(parts)),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed: true,
            },
        )],
    );
    assert!(unresolved
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Unknown
            && f.explanation == "The transition and its recognition are not established."));
}

#[test]
fn typed_choreography_evidence_supplements_but_cannot_contradict_the_plan() {
    let entry = |descriptions: Vec<&str>| {
        (
            PlannedElement::ChoreographicSequence {
                movements: Some(descriptions.into_iter().map(str::to_owned).collect()),
            },
            ElementPlanEvidence::ChoreographicSequence {
                movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::InaBauer]),
                clearly_visible: Some(true),
            },
        )
    };
    let correct = validate(
        AdultCategory::Bronze,
        vec![entry(vec!["Spiral", "Ina Bauer"])],
    );
    assert!(!has(
        &correct,
        "program-choreographic-plan-consistency",
        Some(0)
    ));
    assert!(correct
        .applied_rules
        .iter()
        .any(|r| r.rule_id == "program-choreographic-plan-consistency"));
    let conflict = validate(
        AdultCategory::Bronze,
        vec![entry(vec!["Spiral", "Spread Eagle"])],
    );
    assert!(conflict
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-choreographic-plan-consistency"
            && f.kind == FindingKind::Violation));
    assert!(has(
        &validate(AdultCategory::Bronze, vec![entry(vec![])]),
        "program-choreographic-plan-consistency",
        Some(0)
    ));
    let unsupported = validate(
        AdultCategory::Bronze,
        vec![entry(vec!["free-form movement prose"])],
    );
    assert!(unsupported
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-choreographic-plan-consistency"
            && f.kind == FindingKind::Unsupported));
}

#[test]
fn successful_step_checks_retain_family_specific_sources_and_indices() {
    let r = validate(
        AdultCategory::Gold,
        vec![step(Level::Two, Some(SkatingMovement::Spiral), vec![1, 2])],
    );
    for id in ["program-step-recognition", "program-step-level-credit"] {
        let application = r.applied_rules.iter().find(|r| r.rule_id == id).unwrap();
        assert_eq!(application.element_index, Some(0));
        assert!(application.source.location.contains("step sequence"));
        assert!(!application.source.location.contains("spins and level cap"));
        assert!(r.sources.contains(&application.source));
        assert!(!r.validation.findings.iter().any(|f| f.rule_id == id));
    }
}
