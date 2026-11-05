use skating_program_lab::*;

fn assumptions(codes: &[&str]) -> Vec<AssumedRecognition> {
    codes
        .iter()
        .map(|code| AssumedRecognition::new(*code))
        .collect()
}

fn spin() -> PlannedElement {
    PlannedElement::RegularSpin(RegularSpin {
        entry: None,
        steps: None,
    })
}

fn planned(profile: RuleProfile, elements: Vec<PlannedElement>) -> ProfileProgram {
    ProfileProgram {
        profile: profile.id,
        scoring_table: profile.scoring_table,
        elements,
    }
}

#[test]
fn gold_selects_published_levels_without_technical_evidence() {
    let category = AdultCategory::Gold;
    let registry = ProfileRegistry::adult();
    let policy = adult_program_policy(category);
    let plan = planned(
        adult_profile(category),
        vec![spin(), spin(), PlannedElement::StepSequence],
    );
    let choices = assumptions(&["CCoSp3", "FSSp2", "StSq2"]);
    let assessment = assess_manual_program_with_assumed_recognition(
        &registry,
        &plan,
        &choices,
        &policy,
        SubmissionState::Draft,
    )
    .unwrap();
    // ISU 2786 (2 June 2026), pp. 7 and 9, BASE column.
    assert_eq!(
        assessment
            .unfactored_base
            .iter()
            .map(|v| v.hundredths)
            .collect::<Vec<_>>(),
        [360, 280, 270]
    );
    assert_eq!(assessment.total_base_hundredths, Some(910));
    assert!(!assessment.legality.repairable_absences.is_empty());

    let spin_evidence = |kind, entry, change_foot, level| ElementPlanEvidence::Spin {
        spin: RegularSpinEvidence {
            abbreviation: Some(SpinAbbreviation {
                kind,
                entry,
                change_foot,
            }),
            planned_level: Some(level),
            feature_attempts: Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::DifficultVariation,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            }]),
            ..RegularSpinEvidence::default()
        },
        local_feature_qualification_assumed: true,
    };
    let detailed = vec![
        spin_evidence(
            RegularSpinKind::Combination,
            SpinEntry::NonFlying,
            true,
            Level::Three,
        ),
        spin_evidence(RegularSpinKind::Sit, SpinEntry::Flying, false, Level::Two),
        ElementPlanEvidence::Step {
            level: Level::Two,
            clean_difficult_turns: None,
            full_ice_coverage: None,
            movement: None,
            qualified_features: None,
            qualification_assumed: false,
        },
    ];
    let strict =
        assess_program_legality(&registry, &plan, &detailed, &policy, SubmissionState::Draft)
            .unwrap();
    for id in [
        "program-feature-qualification",
        "program-spin-level-credit",
        "program-step-level-credit",
    ] {
        assert!(strict
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == id));
        assert!(!assessment
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == id));
    }
    assert_eq!(plan, assessment.program);
}

#[test]
fn structural_violations_remain_under_assumed_recognition() {
    let category = AdultCategory::Gold;
    let plan = planned(
        adult_profile(category),
        vec![
            spin(),
            spin(),
            spin(),
            spin(),
            PlannedElement::StepSequence,
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(2),
            })),
        ],
    );
    let report = assess_program_legality_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &plan,
        &assumptions(&["CCoSp4", "CCoSp2", "CSpB", "CSp1", "StSq3", "2Lz"]),
        &adult_program_policy(category),
        SubmissionState::Complete,
    )
    .unwrap();
    for id in [
        "spin-count",
        "spin-distinct-abbreviation",
        "spin-required-role",
        "spin-eligibility",
        "spin-level-cap",
        "program-step-level-cap",
        "isu-adult-2026-27-gold-jump-eligibility",
    ] {
        assert!(
            report
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.rule_id == id && f.kind == FindingKind::Violation),
            "missing {id}"
        );
    }
}

#[test]
fn senior_levels_and_fixed_codes_are_edition_bound() {
    let category = SeniorCategory::Women;
    let policy = senior_program_policy(category);
    let registry = ProfileRegistry::senior();
    let plan = planned(
        senior_profile(category),
        vec![
            spin(),
            PlannedElement::ChoreographicSequence { movements: None },
        ],
    );
    for (code, base) in [
        ("CCoSpB", 200),
        ("CCoSp1", 240),
        ("CCoSp2", 300),
        ("CCoSp3", 360),
        ("CCoSp4", 420),
    ] {
        let assessment = assess_manual_program_with_assumed_recognition(
            &registry,
            &plan,
            &assumptions(&[code, "ChSq1"]),
            &policy,
            SubmissionState::Draft,
        )
        .unwrap();
        assert_eq!(assessment.unfactored_base[0].hundredths, base);
        assert_eq!(assessment.unfactored_base[1].hundredths, 350);
        assert_eq!(
            assessment.unfactored_base[0].table,
            ScoringTableId::Isu2786June2_2026
        );
    }
    for code in ["CCoSp5", "CCoSp3V", "ChSp1"] {
        assert!(assess_manual_program_with_assumed_recognition(
            &registry,
            &plan,
            &assumptions(&[code, "ChSq1"]),
            &policy,
            SubmissionState::Draft,
        )
        .is_err());
    }
}

#[test]
fn assumed_mode_skips_variation_qualification_and_keeps_the_shared_level_cap() {
    let category = AdultCategory::Gold;
    let registry = ProfileRegistry::adult();
    let policy = adult_program_policy(category);
    let plan = planned(
        adult_profile(category),
        vec![PlannedElement::RegularSpin(RegularSpin {
            entry: Some(SpinEntry::Flying),
            steps: Some(vec![SpinStep::Position {
                position: Some(SpinPosition::Sit),
                revolutions: Some(8),
                feature_attempts: None,
            }]),
        })],
    );
    let evidence = [ElementPlanEvidence::Spin {
        spin: RegularSpinEvidence {
            abbreviation: Some(SpinAbbreviation {
                kind: RegularSpinKind::Sit,
                entry: SpinEntry::Flying,
                change_foot: false,
            }),
            planned_level: Some(Level::Four),
            feature_attempts: Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::DifficultVariation,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            }]),
            ..RegularSpinEvidence::default()
        },
        local_feature_qualification_assumed: true,
    }];
    let strict = assess_program_legality(
        &registry,
        &plan,
        &evidence,
        &policy,
        SubmissionState::Complete,
    )
    .unwrap();
    let simple = assess_program_legality_with_assumed_recognition(
        &registry,
        &plan,
        &assumptions(&["FSSp4"]),
        &policy,
        SubmissionState::Complete,
    )
    .unwrap();
    let has = |report: &ProgramLegalityReport, rule_id, kind| {
        report
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == rule_id && f.kind == kind)
    };
    assert!(has(
        &strict,
        "program-feature-qualification",
        FindingKind::Unsupported
    ));
    assert!(has(&strict, "spin-level-cap", FindingKind::Unknown));
    for report in [&strict, &simple] {
        assert!(has(report, "spin-required-role", FindingKind::Violation));
    }
    assert!(has(&simple, "spin-level-cap", FindingKind::Violation));
    assert!(!has(
        &simple,
        "program-feature-qualification",
        FindingKind::Unsupported
    ));
}

#[test]
fn assumed_sequence_requires_an_explicit_direct_transition() {
    let category = AdultCategory::Silver;
    let registry = ProfileRegistry::adult();
    let policy = adult_program_policy(category);
    let jump = |kind| {
        JumpPart::Jump(ListedJump {
            kind: Some(kind),
            rotations: Some(1),
        })
    };
    let plan = planned(
        adult_profile(category),
        vec![PlannedElement::Jump(JumpElement::Sequence(vec![
            jump(JumpKind::ToeLoop),
            jump(JumpKind::Axel),
        ]))],
    );
    let mut choice = AssumedRecognition::new("1T+1A+SEQ");
    let assess = |choice| {
        assess_manual_program_with_assumed_recognition(
            &registry,
            &plan,
            &[choice],
            &policy,
            SubmissionState::Draft,
        )
        .unwrap()
    };
    let unresolved = assess(choice.clone());
    assert_eq!(unresolved.total_base_hundredths, None);
    assert!(unresolved
        .legality
        .rules
        .iter()
        .any(|rule| rule.rule_id == "program-sequence-direct-step"
            && rule.state == RuleCoverageState::MissingEvidence));
    choice.direct_transitions_assumed = true;
    let assumed = assess(choice);
    assert_eq!(assumed.total_base_hundredths, Some(150));
    assert!(assumed.legality.rules.iter().any(
        |rule| rule.rule_id.contains("direct") && rule.state == RuleCoverageState::Conditional
    ));
}

#[test]
fn assumed_choreography_reports_unknown_plan_vocabulary() {
    let category = AdultCategory::Silver;
    let plan = planned(
        adult_profile(category),
        vec![PlannedElement::ChoreographicSequence {
            movements: Some(vec!["unmodeled choreography".into()]),
        }],
    );
    let result = assess_manual_program_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &plan,
        &assumptions(&["ChSq1"]),
        &adult_program_policy(category),
        SubmissionState::Draft,
    )
    .unwrap();
    assert!(result
        .legality
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-choreographic-plan-consistency"
            && f.kind == FindingKind::Unsupported));
    assert_eq!(result.total_base_hundredths, None);
}

#[test]
fn senior_manual_total_uses_eighty_percent_for_second_advanced_solo() {
    let category = SeniorCategory::Women;
    let axel = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(3),
    }));
    let plan = planned(senior_profile(category), vec![axel.clone(), axel]);
    let result = assess_manual_program_with_assumed_recognition(
        &ProfileRegistry::senior(),
        &plan,
        &assumptions(&["3A", "3A"]),
        &senior_program_policy(category),
        SubmissionState::Draft,
    )
    .unwrap();
    assert_eq!(result.unfactored_base[1].hundredths, 800);
    assert_eq!(result.total_base_hundredths, Some(1440));
}
