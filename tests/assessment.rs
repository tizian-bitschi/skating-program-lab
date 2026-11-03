use skating_program_lab::*;

fn recognition(codes: &[&str], level: Option<Level>) -> PanelRecognition {
    PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                codes
                    .iter()
                    .map(|c| ComponentCode::try_from(*c).unwrap())
                    .collect(),
                level,
                Some(vec![]),
            )
            .unwrap(),
        ),
    }
}
fn jump(kind: JumpKind, code: &str) -> ManualProgramEntry {
    ManualProgramEntry {
        planned: PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(kind),
            rotations: Some(1),
        })),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: true,
        },
        intended_recognition: Some(recognition(&[code], None)),
        scenario: None,
    }
}
fn spin(kind: RegularSpinKind, code: &str, position: SpinPosition) -> ManualProgramEntry {
    ManualProgramEntry {
        planned: PlannedElement::RegularSpin(RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: Some(vec![SpinStep::Position {
                position: Some(position),
                revolutions: Some(8),
                feature_attempts: Some(vec![]),
            }]),
        }),
        evidence: ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry: SpinEntry::NonFlying,
                    change_foot: false,
                }),
                planned_level: Some(Level::Basic),
                feature_attempts: Some(vec![]),
                panel: None,
                final_upright_wind_up: None,
            },
            local_feature_qualification_assumed: true,
        },
        intended_recognition: Some(recognition(&[code], Some(Level::Basic))),
        scenario: None,
    }
}
fn fixture(category: AdultCategory) -> ManualProgramRequest {
    let mut combo = jump(JumpKind::ToeLoop, "1T");
    combo.planned = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(1),
        }),
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Loop),
            rotations: Some(1),
        }),
    ]));
    combo.intended_recognition = Some(recognition(&["1T", "1Lo"], None));
    let mut entries = vec![
        jump(JumpKind::Salchow, "1S"),
        jump(JumpKind::Lutz, "1Lz"),
        combo,
        spin(RegularSpinKind::Camel, "CSp", SpinPosition::Camel),
    ];
    if category == AdultCategory::Silver {
        entries.push(spin(RegularSpinKind::Sit, "SSp", SpinPosition::Sit));
    }
    entries.push(ManualProgramEntry {
        planned: PlannedElement::ChoreographicSequence {
            movements: Some(vec!["spiral".into(), "spread eagle".into()]),
        },
        evidence: ElementPlanEvidence::ChoreographicSequence {
            movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::SpreadEagle]),
            clearly_visible: Some(true),
        },
        intended_recognition: Some(recognition(&["ChSq1"], None)),
        scenario: None,
    });
    ManualProgramRequest {
        profile: adult_profile(category).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        policy: adult_program_policy(category),
        submission: SubmissionState::Complete,
        entries,
        request_hypothetical_total: true,
        second_half_boundary: None,
    }
}
fn assess(r: &ManualProgramRequest) -> ManualProgramAssessment {
    assess_manual_program(&ProfileRegistry::adult(), r).unwrap()
}
fn add_goe(e: &mut ManualProgramEntry, goe: Option<i8>) {
    e.scenario = Some(ManualElementScenario {
        recognition: e.intended_recognition.clone().unwrap(),
        annotations: vec![],
        final_goe: goe,
        fall: Some(false),
        execution: None,
    });
}

#[test]
fn bronze_and_silver_source_checked_bv_and_missing_goe() {
    // Communication 2786, 2026-06-02, pp. 2, 6, 14: 1S .40,
    // 1Lz .60, 1T+1Lo .40+.50, CSpB 1.30, SSpB 1.30, ChSq1 3.50.
    for (category, expected) in [(AdultCategory::Bronze, 670), (AdultCategory::Silver, 800)] {
        let r = fixture(category);
        let a = assess(&r);
        assert_eq!(
            a.intended_base.exact_hundredths,
            Some(expected),
            "{:?}",
            a.elements
        );
        assert!(a.goe_adjustment.is_none());
        assert!(a.hypothetical_total.is_none());
        let mut expected_rows = vec![
            (40, "1S", vec!["p. 2", "row 1S"]),
            (60, "1Lz", vec!["p. 2", "row 1Lz"]),
            (90, "1T+1Lo", vec!["p. 14, remark 4", "code 1T", "code 1Lo"]),
            (130, "CSpB", vec!["p. 6", "value row 11, code CSpB"]),
        ];
        if category == AdultCategory::Silver {
            expected_rows.push((130, "SSpB", vec!["p. 6", "value row 16, code SSpB"]));
        }
        expected_rows.push((350, "ChSq1", vec!["p. 14", "value row 11, code ChSq1"]));
        assert_eq!(a.elements.len(), expected_rows.len());
        for (e, (hundredths, notation, source_locations)) in
            a.elements.into_iter().zip(expected_rows)
        {
            let BaseValueOutcome::Value(v) = e.intended_base.unwrap() else {
                panic!()
            };
            assert_eq!(v.source.document, "ISU Communication 2786");
            assert_eq!(v.source.version, "2026-06-02");
            assert_eq!(v.hundredths, hundredths);
            assert_eq!(v.notation, notation);
            for location in source_locations {
                assert!(
                    v.source.location.contains(location),
                    "{}",
                    v.source.location
                );
            }
        }
        assert_eq!(a.validation.validation.coverage, Coverage::Partial);
    }
}

#[test]
fn absent_zero_and_mixed_goe_are_distinct() {
    let mut r = fixture(AdultCategory::Bronze);
    for e in &mut r.entries {
        add_goe(e, None);
    }
    let a = assess(&r);
    assert_eq!(a.scenario_base.unwrap().exact_hundredths, Some(670));
    assert!(a.goe_adjustment.is_none());
    assert!(a.hypothetical_total.is_none());
    r.entries[0].scenario.as_mut().unwrap().final_goe = Some(0);
    let partial = assess(&r);
    assert_eq!(
        partial.goe_adjustment.as_ref().unwrap().unresolved_indices,
        vec![1, 2, 3, 4]
    );
    assert_eq!(partial.hypothetical_total.unwrap().exact_hundredths, None);
    for e in &mut r.entries {
        e.scenario.as_mut().unwrap().final_goe = Some(0);
    }
    let complete = assess(&r);
    assert_eq!(complete.goe_adjustment.unwrap().exact_hundredths, Some(0));
    assert_eq!(
        complete.hypothetical_total.unwrap().exact_hundredths,
        Some(670)
    );
    r.request_hypothetical_total = false;
    assert!(assess(&r).hypothetical_total.is_none());
}

#[test]
fn draft_missing_roles_are_completion_diagnostics() {
    let mut r = fixture(AdultCategory::Silver);
    r.entries.clear();
    r.submission = SubmissionState::Draft;
    let draft = assess(&r);
    assert!(!draft.completion_findings.is_empty());
    assert_eq!(draft.status, AssessmentStatus::Indeterminate);
    r.submission = SubmissionState::Complete;
    assert_eq!(assess(&r).status, AssessmentStatus::ConfirmedViolation);
}

#[test]
fn unknown_rotations_and_missing_spin_credit_remain_unresolved_and_ordered() {
    let mut r = fixture(AdultCategory::Bronze);
    if let PlannedElement::Jump(JumpElement::Solo(j)) = &mut r.entries[0].planned {
        j.rotations = None;
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
        spin.planned_level = Some(Level::One);
    }
    let a = assess(&r);
    assert_eq!(a.intended_base.exact_hundredths, None);
    assert_eq!(a.intended_base.unresolved_indices, vec![0, 3]);
    r.entries.swap(0, 3);
    let b = assess(&r);
    assert!(b
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-spin-level-credit" && f.element_index == Some(0)));
    assert_eq!(b.elements[0].element_index, 0);
    assert!(b.elements[0].intended_base.is_err());
}

#[test]
fn invalid_identity_boundary_target_and_plan_association() {
    let mut r = fixture(AdultCategory::Bronze);
    r.profile.version = "unknown".into();
    assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
    r = fixture(AdultCategory::Bronze);
    r.second_half_boundary = Some(99);
    assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
    r.second_half_boundary = None;
    r.entries[0]
        .intended_recognition
        .as_mut()
        .unwrap()
        .planned_element = Some(1);
    assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
    r.entries[0].intended_recognition = Some(recognition(&["1A"], None));
    assert_eq!(
        assess(&r).elements[0].intended_base,
        Err(AssessmentValueError::PlanRecognitionMismatch)
    );
}

#[test]
fn supplied_calls_falls_and_unsupported_outcomes_stay_separate() {
    // Communication 2786 p. 2: explicit 1Lzq, 1Lz<, 1Lze rows.
    // Communication 2788 pp. 14, 16: final GOE limits, including falls.
    for (call, goe, fall, base, adjustment, score, total) in [
        (PanelCall::Quarter, -2, false, 60, -12, 48, 658),
        (PanelCall::UnderRotated, -2, false, 48, -10, 38, 648),
        (PanelCall::WrongEdge, -3, false, 48, -14, 34, 644),
        (PanelCall::Quarter, -5, true, 60, -30, 30, 640),
    ] {
        let mut r = fixture(AdultCategory::Bronze);
        for e in &mut r.entries {
            add_goe(e, Some(0));
        }
        let s = r.entries[1].scenario.as_mut().unwrap();
        s.recognition.details = Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from("1Lz").unwrap()],
                None,
                Some(vec![RecordedCall {
                    target: CallTarget::Component(0),
                    call,
                }]),
            )
            .unwrap(),
        );
        s.final_goe = Some(goe);
        s.fall = Some(fall);
        let a = assess(&r);
        assert_eq!(a.intended_base.exact_hundredths, Some(670));
        let scored = a.elements[1]
            .scenario_score
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        assert_eq!(scored.base.hundredths, base);
        assert_eq!(scored.goe_hundredths, adjustment);
        assert_eq!(scored.score_hundredths, score);
        assert_eq!(scored.goe, goe);
        assert_eq!(
            a.hypothetical_total.as_ref().unwrap().exact_hundredths,
            Some(total)
        );
        assert_eq!(
            a.request.entries[1]
                .scenario
                .as_ref()
                .unwrap()
                .recognition
                .details
                .as_ref()
                .unwrap()
                .calls(),
            Some(
                [RecordedCall {
                    target: CallTarget::Component(0),
                    call
                }]
                .as_slice()
            )
        );
        assert_eq!(
            a.request.entries[1].scenario.as_ref().unwrap().fall,
            Some(fall)
        );
    }
    for outcome in [
        RecognitionOutcome::Unknown,
        RecognitionOutcome::NoValue,
        RecognitionOutcome::Ignored,
    ] {
        let mut r = fixture(AdultCategory::Bronze);
        for e in &mut r.entries {
            add_goe(e, Some(0));
        }
        r.entries[0].scenario.as_mut().unwrap().recognition.outcome = outcome;
        let a = assess(&r);
        assert_eq!(a.hypothetical_total.unwrap().exact_hundredths, None);
        assert_eq!(a.scenario_base.unwrap().exact_hundredths, None);
    }
}

#[test]
fn boundary_is_explicit_and_requires_complete_goe() {
    let mut r = fixture(AdultCategory::Bronze);
    for e in &mut r.entries {
        add_goe(e, Some(0));
    }
    r.second_half_boundary = Some(0);
    let a = assess(&r);
    // Last three jumps: (.40 + .60 + .90) * .1 = .19 additional BV.
    assert_eq!(a.intended_base.exact_hundredths, Some(670));
    assert_eq!(a.hypothetical_total.unwrap().exact_hundredths, Some(689));
    assert!(
        a.planning_score.unwrap().unwrap().elements[0]
            .as_ref()
            .unwrap()
            .second_half
    );
    r.entries[0].scenario.as_mut().unwrap().final_goe = None;
    let partial = assess(&r);
    assert!(partial.planning_score.is_none());
    assert_eq!(
        partial.hypothetical_total.unwrap().unresolved_indices,
        vec![0, 1, 2, 3, 4]
    );
}

#[test]
fn unknown_calls_and_unsupported_combinations_prevent_exact_scenarios() {
    let mut r = fixture(AdultCategory::Bronze);
    for e in &mut r.entries {
        add_goe(e, Some(0));
    }
    r.entries[0].scenario.as_mut().unwrap().recognition.details = Some(
        RecognitionDetails::new(vec![ComponentCode::try_from("1S").unwrap()], None, None).unwrap(),
    );
    let a = assess(&r);
    assert_eq!(
        a.elements[0].scenario_base,
        Some(Err(BaseValueError::UnknownCalls))
    );
    assert_eq!(a.hypothetical_total.unwrap().unresolved_indices, vec![0]);
    r.entries[0].scenario.as_mut().unwrap().recognition =
        recognition(&["1T", "1S", "1Lo", "1F"], None);
    assert_eq!(
        assess(&r).hypothetical_total.unwrap().exact_hundredths,
        None
    );
}

#[test]
fn shared_spin_feature_consumption_uses_current_order() {
    let mut r = fixture(AdultCategory::Silver);
    for i in [3, 4] {
        if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[i].evidence {
            spin.planned_level = Some(Level::One);
            spin.feature_attempts = Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::IncreaseOfSpeed,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            }]);
        }
        r.entries[i].intended_recognition.as_mut().unwrap().details = Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from(if i == 3 { "CSp" } else { "SSp" }).unwrap()],
                Some(Level::One),
                Some(vec![]),
            )
            .unwrap(),
        );
    }
    let a = assess(&r);
    assert!(a.elements[3].intended_base.is_ok());
    assert!(a.elements[4].intended_base.is_err());
    r.entries.swap(3, 4);
    let b = assess(&r);
    assert!(b.elements[3].intended_base.is_ok());
    assert!(b.elements[4].intended_base.is_err());
    assert!(b
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-spin-level-credit" && f.element_index == Some(4)));
}

#[test]
fn silver_role_and_jump_eligibility_are_checked_independently_of_bv() {
    let mut r = fixture(AdultCategory::Silver);
    r.entries
        .retain(|e| !matches!(e.planned, PlannedElement::RegularSpin(_)));
    assert!(assess(&r).completion_findings.iter().any(|i| assess(&r)
        .validation
        .validation
        .findings[*i]
        .rule_id
        == "spin-required-role"));
    r.entries[0] = jump(JumpKind::Axel, "1A");
    // Silver permits Axel; Bronze prohibits it even with a supported 1.10 BV.
    assert!(!assess(&r)
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.element_index == Some(0) && f.kind == FindingKind::Violation));
    r.profile = adult_profile(AdultCategory::Bronze).id;
    r.policy = adult_program_policy(AdultCategory::Bronze);
    let a = assess(&r);
    assert_eq!(a.status, AssessmentStatus::ConfirmedViolation);
    assert!(a
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.element_index == Some(0) && f.kind == FindingKind::Violation));
    assert!(a.elements[0].intended_base.is_ok());
}

#[test]
fn explicit_recognition_does_not_fill_missing_plan_qualification() {
    let mut r = fixture(AdultCategory::Bronze);
    if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
        spin.planned_level = Some(Level::One);
    }
    r.entries[3].intended_recognition = Some(recognition(&["CSp"], Some(Level::One)));
    assert_eq!(assess(&r).intended_base.exact_hundredths, None);
    assert!(assess(&r).hypothetical_total.is_none());
    // A separately supplied panel outcome is a conditional scenario, never proof
    // that the planned feature earned credit.
    for e in &mut r.entries {
        add_goe(e, Some(0));
    }
    let a = assess(&r);
    assert_eq!(a.intended_base.exact_hundredths, None);
    assert_eq!(a.hypothetical_total.unwrap().exact_hundredths, Some(710));
    assert!(a
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-spin-level-credit"));
}

#[test]
fn invalid_feature_targets_cannot_qualify_exact_planned_levels() {
    for target in [1, 99] {
        let mut r = fixture(AdultCategory::Bronze);
        if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
            spin.planned_level = Some(Level::One);
            spin.feature_attempts = Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::IncreaseOfSpeed,
                position_step: target,
                outcome: FeatureAttemptOutcome::Successful,
            }]);
        }
        r.entries[3].intended_recognition = Some(recognition(&["CSp"], Some(Level::One)));
        let a = assess(&r);
        assert!(matches!(
            a.elements[3].intended_base,
            Err(AssessmentValueError::UnresolvedPlanEvidence(_))
        ));
        assert_eq!(a.intended_base.exact_hundredths, None);
        assert!(a
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == "spin-feature-target"
                && f.kind == FindingKind::Violation
                && f.element_index == Some(3)));
        assert!(a
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == "program-spin-level-credit" && f.element_index == Some(3)));
    }
    // A target must also be a Position step, not a ChangeFoot marker.
    let mut r = fixture(AdultCategory::Silver);
    if let PlannedElement::RegularSpin(spin) = &mut r.entries[3].planned {
        spin.steps.as_mut().unwrap().push(SpinStep::ChangeFoot);
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
        spin.feature_attempts = Some(vec![SpinFeatureAttempt {
            feature: SpinFeature::IncreaseOfSpeed,
            position_step: 1,
            outcome: FeatureAttemptOutcome::Successful,
        }]);
    }
    assert_eq!(assess(&r).intended_base.exact_hundredths, None);
}

#[test]
fn invalid_feature_evidence_does_not_consume_later_valid_credit() {
    let mut r = fixture(AdultCategory::Silver);
    for (i, target) in [(3, 99), (4, 0)] {
        if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[i].evidence {
            spin.planned_level = Some(Level::One);
            spin.feature_attempts = Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::IncreaseOfSpeed,
                position_step: target,
                outcome: FeatureAttemptOutcome::Successful,
            }]);
        }
        r.entries[i].intended_recognition = Some(recognition(
            &[if i == 3 { "CSp" } else { "SSp" }],
            Some(Level::One),
        ));
    }
    let a = assess(&r);
    assert!(a.elements[3].intended_base.is_err());
    assert!(a.elements[4].intended_base.is_ok());
    assert!(!a
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "program-spin-level-credit" && f.element_index == Some(4)));
}

#[test]
fn observation_references_are_rejected_in_every_recognition_stage() {
    for observed in [0, 999] {
        let mut r = fixture(AdultCategory::Bronze);
        r.entries[0]
            .intended_recognition
            .as_mut()
            .unwrap()
            .observed_element = Some(observed);
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
        r = fixture(AdultCategory::Bronze);
        add_goe(&mut r.entries[0], Some(0));
        r.entries[0]
            .scenario
            .as_mut()
            .unwrap()
            .recognition
            .observed_element = Some(observed);
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
        r = fixture(AdultCategory::Bronze);
        let mut panel = recognition(&["CSp"], Some(Level::Basic));
        panel.planned_element = Some(3);
        panel.observed_element = Some(observed);
        if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
            spin.panel = Some(panel);
        }
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
    }
}

#[test]
fn supplied_plan_indices_must_follow_reordering_in_both_stages() {
    for scenario in [false, true] {
        let mut r = fixture(AdultCategory::Bronze);
        if scenario {
            add_goe(&mut r.entries[0], Some(0));
            r.entries[0]
                .scenario
                .as_mut()
                .unwrap()
                .recognition
                .planned_element = Some(0);
        } else {
            r.entries[0]
                .intended_recognition
                .as_mut()
                .unwrap()
                .planned_element = Some(0);
        }
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_ok());
        r.entries.swap(0, 1);
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_err());
        if scenario {
            r.entries[1]
                .scenario
                .as_mut()
                .unwrap()
                .recognition
                .planned_element = Some(1);
        } else {
            r.entries[1]
                .intended_recognition
                .as_mut()
                .unwrap()
                .planned_element = Some(1);
        }
        assert!(assess_manual_program(&ProfileRegistry::adult(), &r).is_ok());
    }
}

#[test]
fn independent_spin_panel_findings_do_not_change_intended_bv() {
    for index in [Some(3), None] {
        for code in ["CSp", "SSp"] {
            let mut r = fixture(AdultCategory::Bronze);
            let before = assess(&r);
            let mut panel = recognition(&[code], Some(Level::Basic));
            panel.planned_element = index;
            if let ElementPlanEvidence::Spin { spin, .. } = &mut r.entries[3].evidence {
                spin.panel = Some(panel);
            }
            let after = assess(&r);
            assert_eq!(
                after.elements[3].intended_base,
                before.elements[3].intended_base
            );
            assert_eq!(after.intended_base.exact_hundredths, Some(670));
            let Ok(BaseValueOutcome::Value(value)) = &after.elements[3].intended_base else {
                panic!()
            };
            assert_eq!(value.hundredths, 130);
            assert_eq!(value.notation, "CSpB");
            assert!(after
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.rule_id == "spin-panel-reconciliation"
                    && f.kind == FindingKind::Unsupported
                    && f.element_index == Some(3)));
            assert_eq!(
                after
                    .validation
                    .validation
                    .findings
                    .iter()
                    .any(|f| f.rule_id == "spin-panel-target"
                        && f.kind == FindingKind::Violation
                        && f.element_index == Some(3)),
                index.is_none()
            );
        }
    }
}
