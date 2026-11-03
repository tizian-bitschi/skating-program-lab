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
        for e in a.elements {
            let BaseValueOutcome::Value(v) = e.intended_base.unwrap() else {
                panic!()
            };
            assert_eq!(v.source.document, "ISU Communication 2786");
            assert_eq!(v.source.version, "2026-06-02");
            assert!(!v.source.location.is_empty());
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
    for (call, goe, fall) in [
        (PanelCall::Quarter, -2, false),
        (PanelCall::UnderRotated, -2, false),
        (PanelCall::WrongEdge, -3, false),
        (PanelCall::Quarter, -5, true),
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
        assert!(
            a.elements[1].scenario_score.as_ref().unwrap().is_ok(),
            "{:?}",
            a.elements[1]
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
