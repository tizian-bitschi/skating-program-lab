use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

#[test]
fn planned_euler_transitions_need_no_performance_observations() {
    // Adult July 16 pp. 8–10; TP Handbook July 15 p. 28 (same-foot turn into Axel).
    // SOV 2786 June 2: 1Lz .60, Eu .00, 1S .40, 1A 1.10.
    for (code, expected) in [("1Lz+Eu+1S", 100), ("1Lz+Eu+1A", 170), ("1Lz+1S", 100)] {
        let result = assess_manual_bv(
            &ProfileRegistry::adult(),
            &request(
                adult_profile(AdultCategory::Silver).id,
                SubmissionState::Draft,
                &[code],
            ),
        )
        .unwrap();
        assert_eq!(result.total.exact_hundredths, Some(expected), "{code}");
        assert!(
            !result
                .legality
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.rule_id.contains("euler") && f.kind == FindingKind::Unknown),
            "{code}"
        );
    }
}

#[test]
fn euler_sequence_requires_a_separate_qualifying_step() {
    let registry = ProfileRegistry::adult();
    let mut input = request(
        adult_profile(AdultCategory::Masters).id,
        SubmissionState::Draft,
        &["2F+Eu+1A+SEQ"],
    );
    input.elements[0].direct_transitions_assumed = true;
    let invalid = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(
        invalid.elements[0].credited_base,
        Err(ManualBvError::ContextUnavailable)
    );
    assert!(invalid
        .legality
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation
            && f.explanation.contains("Axel-type second or third")));
    input.elements[0].code = "2F+Eu+2S+1A+SEQ".into();
    let valid = assess_manual_bv(&registry, &input).unwrap();
    // 2786 June2: 2F1.80 + Eu0 + 2S1.30 + 1A1.10 =4.20.
    assert_eq!(valid.total.exact_hundredths, Some(420));
}

#[test]
fn planned_euler_does_not_bypass_category_or_program_limits() {
    for (category, codes, text) in [
        (
            AdultCategory::Bronze,
            vec!["1Lz+Eu+1S"],
            "Euler is prohibited",
        ),
        (
            AdultCategory::Silver,
            vec!["1Lz+Eu+1S", "1F+Eu+1Lo"],
            "Euler limit",
        ),
    ] {
        let assessment = assess_manual_bv(
            &ProfileRegistry::adult(),
            &request(adult_profile(category).id, SubmissionState::Draft, &codes),
        )
        .unwrap();
        assert!(assessment
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.kind == FindingKind::Violation && f.explanation.contains(text)));
    }
}

fn request(profile: ProfileId, submission: SubmissionState, codes: &[&str]) -> ManualBvRequest {
    ManualBvRequest {
        profile,
        scoring_table: TABLE,
        submission,
        elements: codes
            .iter()
            .map(|code| AssumedRecognition::new(*code))
            .collect(),
    }
}

#[test]
fn gold_spin_draft_has_base_value_before_a_goe_request() {
    let draft = request(
        adult_profile(AdultCategory::Gold).id,
        SubmissionState::Draft,
        &["CCoSp3", "FSSp2"],
    );
    let result = assess_manual_bv(&ProfileRegistry::adult(), &draft).unwrap();
    // Communication 2786 (2 June 2026), CCoSp3 3.60 and FSSp2 2.80.
    assert_eq!(result.total.exact_hundredths, Some(640));
    assert!(!result.legality.repairable_absences.is_empty());
}

#[test]
fn source_values_for_complete_adult_selections() {
    use AdultCategory::*;
    // ISU 2786 (2 June 2026), pp. 2, 6, 7, 9 and 14, BASE column.
    let fixtures: &[(AdultCategory, &[&str], &[u16])] = &[
        (Bronze, &["1Lz", "SSpB", "ChSq1"], &[60, 130, 350]),
        (
            Silver,
            &["1Lz", "CCoSp2", "SSpB", "ChSq1"],
            &[60, 300, 130, 350],
        ),
        (
            Gold,
            &["2T+2Lo", "CCoSp3", "FSSp2", "StSq2", "ChSp1"],
            &[300, 360, 280, 270, 350],
        ),
        (
            Masters,
            &["1A", "2T+2Lo", "CCoSp4", "FSSp2", "StSq4", "ChSp1"],
            &[110, 300, 420, 280, 410, 350],
        ),
    ];
    let registry = ProfileRegistry::adult();
    for &(category, codes, expected) in fixtures {
        let profile = adult_profile(category).id;
        let result = assess_manual_bv(
            &registry,
            &request(profile.clone(), SubmissionState::Complete, codes),
        )
        .unwrap();
        assert_eq!(
            result.total.exact_hundredths,
            Some(expected.iter().map(|&n| i64::from(n)).sum()),
            "{category:?}"
        );
        for (index, (element, &base)) in result.elements.iter().zip(expected).enumerate() {
            let table = element.table_base.as_ref().unwrap();
            assert_eq!(
                (element.element_index, table.table, table.hundredths),
                (index, TABLE, base)
            );
            assert_eq!(
                element.credited_base,
                Ok(BaseValueOutcome::Value(table.clone()))
            );
            if !codes[index].contains('+') {
                let selectable =
                    check_selectable_element(&registry, &profile, TABLE, codes[index]).unwrap();
                assert_eq!(table.source, selectable.row.source.reference(codes[index]));
            }
        }
        assert!(result.legality.repairable_absences.is_empty());
        assert!(
            result.legality.validation.validation.findings.is_empty(),
            "{category:?}: {:?}",
            result.legality.validation.validation.findings
        );
    }
}

#[test]
fn unavailable_selections_keep_indices_values_and_known_violations() {
    let registry = ProfileRegistry::adult();
    let silver = adult_profile(AdultCategory::Silver).id;
    let input = request(
        silver,
        SubmissionState::Complete,
        &["?", "CCoSp3", "1Lz", "1Lz", "1Lz", "ChSq1"],
    );
    let result = assess_manual_bv(&registry, &input).unwrap();
    assert!(matches!(
        result.elements[0].credited_base,
        Err(ManualBvError::Selection {
            error: ElementSelectionError::UnknownCode { .. },
            component_index: None,
        })
    ));
    assert!(matches!(
        result.elements[1].credited_base,
        Err(ManualBvError::Selection {
            error: ElementSelectionError::LevelNotSelectable(_),
            component_index: None,
        })
    ));
    let base = result.elements[1].table_base.as_ref().unwrap();
    assert_eq!((base.notation.as_str(), base.hundredths), ("CCoSp3", 360));
    assert_eq!(result.total.unresolved_indices, [0, 1]);
    assert_eq!(result.total.resolved_hundredths, 530);
    assert_eq!(result.total.exact_hundredths, None);
    assert!(
        result
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|finding| {
                finding.kind == FindingKind::Violation && finding.rule_id.contains("repetition")
            }),
        "{:?}",
        result.legality.validation.validation.findings
    );

    let bronze = request(
        adult_profile(AdultCategory::Bronze).id,
        SubmissionState::Draft,
        &["CCoSp1"],
    );
    let result = assess_manual_bv(&registry, &bronze).unwrap();
    assert!(matches!(
        result.elements[0].credited_base,
        Err(ManualBvError::Selection {
            error: ElementSelectionError::TypeNotSelectable(_),
            component_index: None,
        })
    ));
}

#[test]
fn missing_role_and_sequence_assumption_are_reported_independently_of_table_value() {
    let registry = ProfileRegistry::adult();
    let mut input = request(
        adult_profile(AdultCategory::Silver).id,
        SubmissionState::Draft,
        &["1T+1A+SEQ", "ChSq1"],
    );
    let draft = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(
        draft.elements[0].table_base.as_ref().unwrap().hundredths,
        150
    );
    assert_eq!(
        draft.elements[0].credited_base,
        Err(ManualBvError::ContextUnavailable)
    );
    assert_eq!(draft.total.unresolved_indices, [0]);
    assert!(!draft.legality.repairable_absences.is_empty());

    input.elements[0].direct_transitions_assumed = true;
    input.submission = SubmissionState::Complete;
    let complete = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(complete.total.exact_hundredths, Some(500));
    assert!(complete.legality.repairable_absences.is_empty());
    for index in draft.legality.repairable_absences {
        assert!(complete
            .legality
            .validation
            .validation
            .findings
            .contains(&draft.legality.validation.validation.findings[index]));
    }
}

#[test]
fn senior_uses_the_protocol_scorer_repetition_value_and_keeps_unknown_context_explicit() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Men);
    let mut input = request(profile.id, SubmissionState::Draft, &["4T", "4T", "ChSp1"]);
    let result = assess_manual_bv(&registry, &input).unwrap();
    let scored = |element: &ManualBvElement| match &element.credited_base {
        Ok(BaseValueOutcome::Value(value)) => value.hundredths,
        outcome => panic!("unexpected BV: {outcome:?}"),
    };
    // ISU 2786 pp. 4 and 14, remark 5. Rule 612 gives the second 4T 80%.
    assert_eq!(
        result.elements[1].table_base.as_ref().unwrap().hundredths,
        950
    );
    assert_eq!(
        result.elements.iter().map(scored).collect::<Vec<_>>(),
        [950, 760, 350]
    );
    assert_eq!(result.total.exact_hundredths, Some(2060));

    input.elements[1] = AssumedRecognition::new("4T+2T");
    let compound = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(
        compound.elements.iter().map(scored).collect::<Vec<_>>(),
        [950, 1080, 350]
    );

    input.elements[1] = AssumedRecognition::new("?");
    let unknown = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(
        unknown.elements[0].credited_base,
        Err(ManualBvError::ContextUnavailable)
    );
    assert_eq!(unknown.total.unresolved_indices, [0, 1]);
    assert_eq!(unknown.total.resolved_hundredths, 350);
}

#[test]
fn senior_quint_compounds_keep_the_table_sum_and_block_credit() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Women).id;
    for (code, listed, assume_step, quint_index) in [
        ("5T+2T", 1530, false, 0),
        ("2T+5T", 1530, false, 1),
        ("5T+2A+SEQ", 1730, true, 0),
    ] {
        let mut input = request(profile.clone(), SubmissionState::Draft, &["FCSpB", code]);
        input.elements[1].direct_transitions_assumed = assume_step;
        let result = assess_manual_bv(&registry, &input).unwrap();
        assert_eq!(
            result.elements[1].table_base.as_ref().unwrap().hundredths,
            listed
        );
        assert_eq!(
            result.elements[1].credited_base,
            Err(ManualBvError::ContextUnavailable)
        );
        assert_eq!(result.total.unresolved_indices, [1]);
        assert_eq!(result.total.resolved_hundredths, 190);
        assert!(result
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|f| {
                f.rule_id == "senior-compound-structure"
                    && f.kind == FindingKind::Violation
                    && f.element_index == Some(1)
                    && f.component_index == Some(quint_index)
            }));
    }
}

#[test]
fn senior_unknown_components_only_remove_supported_credit_where_context_is_uncertain() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Women).id;
    for (codes, unresolved, subtotal) in [
        (["?", "2T", "2T"], &[0, 1, 2][..], 0),
        (["2T", "2T", "?"], &[2][..], 260),
        (["2S+?", "2T", "2T"], &[0, 2][..], 130),
        (["2T+?", "2T", "FCSpB"], &[0, 1][..], 190),
    ] {
        let result = assess_manual_bv(
            &registry,
            &request(profile.clone(), SubmissionState::Draft, &codes),
        )
        .unwrap();
        assert_eq!(result.total.unresolved_indices, unresolved, "{codes:?}");
        assert_eq!(result.total.resolved_hundredths, subtotal, "{codes:?}");
        if codes[0].contains('+') {
            let Err(ManualBvError::Selection {
                error: ElementSelectionError::UnknownCode { source },
                component_index: Some(1),
            }) = &result.elements[0].credited_base
            else {
                panic!("{:?}", result.elements[0]);
            };
            assert_eq!(source.document, "ISU Communication 2786");
            assert_eq!(source.version, "2026-06-02");
            assert_eq!(source.location, "Exact notation lookup: ?");
        }
    }
}

#[test]
fn senior_excess_jump_and_compound_slots_do_not_have_exact_credit() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Women).id;
    for (codes, rule, indices) in [
        (
            &["2S", "2T", "2Lo", "2F", "2Lz", "1A", "2A", "FCSp1"][..],
            "senior-jump-count",
            &[0, 1, 2, 3, 4, 5, 6][..],
        ),
        (
            &["2S+2T", "2F+2Lo", "2Lz+2Lo", "FCSp1"][..],
            "senior-compound-count",
            &[0, 1, 2][..],
        ),
    ] {
        let result = assess_manual_bv(
            &registry,
            &request(profile.clone(), SubmissionState::Draft, codes),
        )
        .unwrap();
        assert_eq!(result.total.exact_hundredths, None);
        assert_eq!(result.total.unresolved_indices, indices);
        // ISU 2786 (2 June 2026), p. 6, FCSp1.
        assert_eq!(result.total.resolved_hundredths, 230);
        assert!(result
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|f| { f.rule_id == rule && f.kind == FindingKind::Violation }));
    }
}

#[test]
fn manual_entry_points_agree_on_clean_senior_repetition() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Women);
    let policy = senior_program_policy(SeniorCategory::Women);
    let input = request(profile.id.clone(), SubmissionState::Draft, &["3A", "3A"]);
    let axel = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(3),
    }));
    let program = ProfileProgram {
        profile: input.profile.clone(),
        scoring_table: TABLE,
        elements: vec![axel.clone(); 2],
    };
    let simple = assess_manual_bv(&registry, &input).unwrap();
    let typed = assess_manual_program_with_assumed_recognition(
        &registry,
        &program,
        &input.elements,
        &policy,
        input.submission,
    )
    .unwrap();
    let strict_entry = |index| ManualProgramEntry {
        planned: axel.clone(),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
        intended_recognition: Some(PanelRecognition {
            ruleset: profile.scoring.recognition_ruleset,
            source: None,
            planned_element: Some(index),
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: Some(
                RecognitionDetails::new(
                    vec![ComponentCode::try_from("3A").unwrap()],
                    None,
                    Some(vec![]),
                )
                .unwrap(),
            ),
        }),
        scenario: None,
    };
    let strict = assess_manual_program(
        &registry,
        &ManualProgramRequest {
            profile: input.profile,
            scoring_table: TABLE,
            policy,
            submission: input.submission,
            entries: vec![strict_entry(0), strict_entry(1)],
            request_hypothetical_total: false,
            second_half_boundary: None,
        },
    )
    .unwrap();
    assert_eq!(simple.total.exact_hundredths, Some(1440));
    assert_eq!(typed.total_base_hundredths, Some(1440));
    assert_eq!(strict.intended_base, simple.total);
    assert_eq!(
        strict.elements[1].intended_base.as_ref().unwrap(),
        simple.elements[1].credited_base.as_ref().unwrap()
    );
    assert!(strict.hypothetical_total.is_none());
}
