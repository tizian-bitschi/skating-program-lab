use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

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
        Err(ManualBvError::Selection(
            ElementSelectionError::UnknownCode { .. }
        ))
    ));
    assert!(matches!(
        result.elements[1].credited_base,
        Err(ManualBvError::Selection(
            ElementSelectionError::LevelNotSelectable(_)
        ))
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
        Err(ManualBvError::Selection(
            ElementSelectionError::TypeNotSelectable(_)
        ))
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
