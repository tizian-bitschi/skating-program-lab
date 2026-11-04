use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

#[path = "support/protocol_recognition.rs"]
mod protocol_recognition;
use protocol_recognition::recognition;

fn panel(values: &[i8]) -> JudgePanelGrades {
    JudgePanelGrades {
        grades: values.iter().copied().map(Some).collect(),
    }
}
fn source(location: &str) -> SourceReference {
    SourceReference {
        document: "independent test/protocol".into(),
        version: "2026-11-04".into(),
        location: location.into(),
    }
}
fn input(code: &str) -> ProtocolElementInput {
    ProtocolElementInput {
        recognition: recognition(code),
        annotations: vec![],
        judges: Some(panel(&[0; 9])),
        printed_second_half: false,
        source: source(code),
        execution: None,
    }
}
fn replay(elements: &[ProtocolElementInput]) -> ProtocolProgramScore {
    score_protocol_program(
        &ProfileRegistry::senior(),
        &senior_profile(SeniorCategory::Men).id,
        TABLE,
        elements,
    )
    .unwrap()
}

#[test]
fn numerical_mapping_precedes_mean_and_extremes_are_removed_once() {
    // 3Lz<: grade columns -5=-2.36, -4=-1.89, -3=-1.42, -2=-0.94, -1=-0.47.
    let r = recognition("3Lz<");
    let result =
        aggregate_panel_goe(TABLE, &r, &[], &panel(&[-5, -4, -3, -2, -1, 0, 1, 2, 3])).unwrap();
    assert_eq!(result.goe_hundredths, -47);
    assert_eq!(result.retained_positions.len(), 7);
    // Averaging the grades then mapping/rounding gives a different answer here.
    assert_eq!(
        aggregate_panel_goe(TABLE, &r, &[], &panel(&[-4, -3, 0]))
            .unwrap()
            .goe_hundredths,
        -110
    );
    let ties =
        aggregate_panel_goe(TABLE, &recognition("4T"), &[], &panel(&[1, 1, 1, 2, 2])).unwrap();
    assert_eq!(ties.goe_hundredths, 127);
    assert_eq!(ties.retained_positions, vec![1, 2, 3]);
    assert_eq!(
        aggregate_panel_goe(TABLE, &recognition("4T"), &[], &panel(&[-1, -1, 0, 0]))
            .unwrap()
            .goe_hundredths,
        -48
    );
    assert_eq!(
        aggregate_panel_goe(TABLE, &recognition("4T"), &[], &panel(&[1, 1, 0, 0]))
            .unwrap()
            .goe_hundredths,
        48
    );
}

#[test]
fn missing_and_invalid_grades_never_become_zero_or_a_reduced_panel() {
    let r = recognition("3A");
    for grades in [
        vec![],
        vec![Some(0); 10],
        vec![None; 9],
        vec![Some(0), None, Some(1)],
    ] {
        assert_eq!(
            aggregate_panel_goe(TABLE, &r, &[], &JudgePanelGrades { grades }),
            Err(PanelScoreError::UnsupportedPanelConfiguration)
        );
    }
    for grade in [-128, -6, 6, 127] {
        assert_eq!(
            aggregate_panel_goe(TABLE, &r, &[], &panel(&[grade; 9])),
            Err(PanelScoreError::InvalidGrade)
        );
    }
    for count in 1..=9 {
        let s = aggregate_panel_goe(TABLE, &r, &[], &panel(&vec![-2; count])).unwrap();
        assert_eq!(s.goe_hundredths, -160);
        assert_eq!(
            s.retained_positions.len(),
            if count >= 5 { count - 2 } else { count }
        );
    }
}

#[test]
fn supplied_credit_is_separate_from_table_base_and_from_timing_verification() {
    let mut row = input("3Lz<<+REP");
    row.printed_second_half = true;
    row.judges = Some(panel(&[-5; 9]));
    let report = replay(&[row]);
    let score = report.elements[0].as_ref().unwrap();
    assert!(matches!(&score.table_base, BaseValueOutcome::Value(v) if v.hundredths == 210));
    assert_eq!(score.credited_base_hundredths, 185);
    assert_eq!(score.score_hundredths, Some(80));
    assert!(score
        .credit_sources
        .iter()
        .any(|s| s.location.contains("start time unverified")));
    let mut row = input("4T");
    row.judges = None;
    let report = replay(&[row]);
    assert_eq!(
        report.elements[0]
            .as_ref()
            .unwrap()
            .credited_base_hundredths,
        950
    );
    assert_eq!(report.tes_hundredths, None);
    let mut invalid = input("StSq4");
    invalid.printed_second_half = true;
    assert_eq!(
        replay(&[invalid]).elements[0],
        Err(PanelScoreError::InvalidInput)
    );
    let mut unknown = input("4T");
    unknown.recognition.outcome = RecognitionOutcome::Unknown;
    let report = replay(&[unknown]);
    assert_eq!(report.tes_hundredths, None);
    assert_eq!(report.sources[0].location, "4T");
}

fn contextual(codes: &[&str]) -> ProfileProgramScore {
    let elements: Vec<_> = codes
        .iter()
        .map(|code| ProgramElement {
            recognition: recognition(code),
            annotations: vec![],
            goe: -5,
            fall: Some(false),
            start_ms: Some(10_000),
        })
        .collect();
    score_profile_program(
        &ProfileRegistry::senior(),
        &senior_profile(SeniorCategory::Men).id,
        TABLE,
        &elements,
        Some(0),
        Some(ProgramTiming {
            duration_seconds: 240,
            ordinary_timing_confirmed: true,
            second_half_default_confirmed: true,
        }),
        None,
    )
    .unwrap()
}

#[test]
fn marked_second_solo_uses_context_and_original_goe_reference() {
    for (codes, base, goe) in [
        (vec!["4T<", "4T"], 760, -475),
        (vec!["4T<", "4T<"], 608, -380),
        (vec!["3Lz", "3Lz<<"], 168, -105),
        (vec!["3F!", "3F!"], 424, -265),
    ] {
        let report = contextual(&codes);
        let last = report.score.elements[1].as_ref().unwrap();
        assert_eq!(last.credited_base_hundredths, base);
        assert_eq!(last.unfactored.goe_hundredths, goe);
    }
    for codes in [["4T", "4T+2T"], ["4T+2T", "4T"]] {
        let report = contextual(&codes);
        assert_eq!(
            report.score.elements[0]
                .as_ref()
                .unwrap()
                .credited_base_hundredths
                + report.score.elements[1]
                    .as_ref()
                    .unwrap()
                    .credited_base_hundredths,
            2030
        );
    }
    for codes in [
        vec!["2T", "2T", "2T"],
        vec!["4T", "4T", "4T"],
        vec!["4T", "4T", "4S", "4S"],
        vec!["3T", "3T", "3S", "3S", "3Lo", "3Lo"],
        vec!["3T", "2T", "1T", "4T"],
    ] {
        let report = contextual(&codes);
        assert_eq!(report.score.tes_hundredths, None);
        assert!(report.score.elements.iter().any(Result::is_err));
    }
}

#[test]
fn reduced_and_original_references_and_call_applicability_are_catalog_backed() {
    for (code, base, goe) in [
        ("3Lze<", 354, -177),
        ("4Fe<<", 424, -212),
        ("3F!q", 530, -265),
        ("4T+3T<", 1286, -475),
        ("3T+3Lo<", 812, -210),
        ("3F+Eu+3S", 960, -265),
        ("3Lz+3T+2T", 1140, -295),
        ("3F+3A+SEQ", 1330, -400),
    ] {
        let r = recognition(code);
        let s = aggregate_panel_goe(TABLE, &r, &[], &panel(&[-5; 9])).unwrap();
        assert_eq!(s.goe_hundredths, goe, "{code}");
        assert!(
            matches!(lookup_base_value(TABLE, &r, &[]), Ok(BaseValueOutcome::Value(v)) if v.hundredths == base),
            "{code}"
        );
    }
    for code in ["3Te", "3Sq", "3A!"] {
        let result = aggregate_panel_goe(TABLE, &recognition(code), &[], &panel(&[0; 9]));
        if code == "3Sq" {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
    let r = recognition("3Aq");
    assert!(score_element(TABLE, &r, &[], 5, Some(false)).is_err());
    assert_eq!(
        aggregate_panel_goe(TABLE, &r, &[], &panel(&[5; 9]))
            .unwrap()
            .goe_hundredths,
        400
    );
}

#[test]
fn senior_solo_rep_rejects_single_double_credit_but_preserves_legacy_and_special_compounds() {
    for code in ["1A+REP", "2T+REP", "2Aq+REP"] {
        let r = recognition(code);
        assert_eq!(
            lookup_base_value(TABLE, &r, &[]),
            Err(BaseValueError::Unsupported)
        );
        assert_eq!(
            aggregate_panel_goe(TABLE, &r, &[], &panel(&[-5; 9])),
            Err(PanelScoreError::BaseValue(BaseValueError::Unsupported))
        );
        let report = replay(&[input(code)]);
        assert_eq!(
            report.elements[0],
            Err(PanelScoreError::BaseValue(BaseValueError::Unsupported))
        );
        assert_eq!(report.tes_hundredths, None);
    }
    // Existing Adult supplied-REP contract is intentionally unchanged by #78.
    let mut adult = recognition("2T+REP");
    adult.ruleset = RulesetId::IsuAdult2026_27July16;
    assert!(
        matches!(lookup_base_value(TABLE, &adult, &[]), Ok(BaseValueOutcome::Value(v)) if v.hundredths == 104)
    );
    // TPH p. 33: excess compounds retain only their first jump with REP;
    // this is separate from Rule 612's second-identical-solo consequence.
    let report = replay(&[input("2T+2Lo*+REP")]);
    assert_eq!(
        report.elements[0]
            .as_ref()
            .unwrap()
            .credited_base_hundredths,
        104
    );
}

#[test]
fn printed_credit_is_validated_for_every_outcome_and_aliases_apply_once() {
    for code in ["CSp4", "StSq4", "ChSp1", "ChSq1"] {
        for outcome in [
            RecognitionOutcome::Recognized,
            RecognitionOutcome::NoValue,
            RecognitionOutcome::Ignored,
        ] {
            let mut row = input(code);
            row.recognition.outcome = outcome;
            row.judges = None;
            row.printed_second_half = true;
            let report = replay(&[row]);
            assert_eq!(
                report.elements[0],
                Err(PanelScoreError::InvalidInput),
                "{code} {outcome:?}"
            );
            assert_eq!(report.tes_hundredths, None);
        }
    }
    for code in ["2A*", "3A<*"] {
        let mut row = input(code);
        row.judges = None;
        row.printed_second_half = true;
        let report = replay(&[row]);
        assert_eq!(report.tes_hundredths, Some(0));
        assert!(report.elements[0].as_ref().unwrap().supplied_second_half);
    }
    let mut scores = vec![];
    for (flag, annotations) in [
        (true, vec![]),
        (false, vec![ScoringAnnotation::SecondHalfCredit]),
        (true, vec![ScoringAnnotation::SecondHalfCredit]),
    ] {
        let mut row = input("4T");
        row.printed_second_half = flag;
        row.annotations = annotations;
        scores.push(replay(&[row]).elements.remove(0).unwrap());
    }
    assert!(scores.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(scores[0].credited_base_hundredths, 1045);
    let mut duplicate = input("4T");
    duplicate.annotations = vec![ScoringAnnotation::SecondHalfCredit; 2];
    assert_eq!(
        replay(&[duplicate]).elements[0],
        Err(PanelScoreError::InvalidInput)
    );
    let mut row = input("CSp4");
    row.annotations.push(ScoringAnnotation::SecondHalfCredit);
    assert_eq!(
        replay(&[row]).elements[0],
        Err(PanelScoreError::InvalidInput)
    );
}

#[test]
fn component_stars_use_only_retained_catalog_rows_for_bv_and_panel_reference() {
    // Independent SOV expectations plus TPH p. 29 / Q&A 3: excluded components
    // do not supply BV or the numerical reference, but their calls remain supplied.
    for (code, base, goe) in [
        ("4T+3T*", 950, -475),
        ("3T*+2T", 130, -65),
        ("3Lz+2T<*", 590, -295),
        ("3Lz+Eu+3S*", 590, -295),
    ] {
        let mut row = input(code);
        row.judges = Some(panel(&[-5; 9]));
        let report = replay(&[row]);
        let score = report.elements[0].as_ref().unwrap();
        assert_eq!(score.credited_base_hundredths, base, "{code}");
        assert_eq!(
            score.panel_goe.as_ref().unwrap().goe_hundredths,
            goe,
            "{code}"
        );
        assert_eq!(
            score.score_hundredths,
            Some(i32::from(base) + i32::from(goe))
        );
    }
    let mut row = input("3T*+2T*");
    row.judges = None;
    assert_eq!(replay(&[row]).tes_hundredths, Some(0));
}
