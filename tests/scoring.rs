use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

fn recognition(
    code: &str,
    level: Option<Level>,
    calls: Option<Vec<PanelCall>>,
) -> PanelRecognition {
    PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from(code).unwrap()],
                level,
                calls.map(|calls| {
                    calls
                        .into_iter()
                        .map(|call| RecordedCall {
                            target: CallTarget::Component(0),
                            call,
                        })
                        .collect()
                }),
            )
            .unwrap(),
        ),
    }
}

#[test]
fn source_table_fixtures() {
    use PanelCall::*;
    // Communication 2786, 2 June 2026: Jumps p. 2, BASE column, each named row.
    let fixtures = [
        (vec![], 60, "1Lz"),
        (vec![Quarter], 60, "1Lzq"),
        (vec![UnderRotated], 48, "1Lz<"),
        (vec![Downgraded], 0, "1Lz<<"),
        (vec![UnclearEdge], 60, "1Lz!"),
        (vec![WrongEdge], 48, "1Lze"),
        (vec![WrongEdge, UnderRotated], 36, "1Lze<"),
    ];
    for (calls, expected, notation) in fixtures {
        let result = lookup_base_value(TABLE, &recognition("1Lz", None, Some(calls)), &[]).unwrap();
        let BaseValueOutcome::Value(value) = result else {
            panic!("expected value")
        };
        assert_eq!(
            (value.hundredths, value.notation.as_str(), value.table),
            (expected, notation, TABLE)
        );
        assert!(value.source.location.contains("p. 2"));
        assert!(value.source.location.contains(notation));
        assert_eq!(value.source.version, "2026-06-02");
    }
    // Communication 2786: 1A<< p. 3, CSSp1 p. 8, ChSp1 p. 14, BASE column.
    for (code, level, calls, expected, page) in [
        ("1A", None, vec![Downgraded], 0, "p. 3"),
        ("CSSp", Some(Level::One), vec![], 230, "p. 8"),
        ("ChSp1", None, vec![], 350, "p. 14"),
    ] {
        let BaseValueOutcome::Value(value) =
            lookup_base_value(TABLE, &recognition(code, level, Some(calls)), &[]).unwrap()
        else {
            panic!("expected value")
        };
        assert_eq!(value.hundredths, expected);
        assert!(value.source.location.contains(page));
    }
}

#[test]
fn unknown_and_unsupported_do_not_become_zero() {
    let mut r = recognition("1Lz", None, Some(vec![PanelCall::Downgraded]));
    assert!(matches!(
        lookup_base_value(TABLE, &r, &[]),
        Ok(BaseValueOutcome::Value(BaseValue { hundredths: 0, .. }))
    ));
    r.outcome = RecognitionOutcome::Ignored;
    assert_eq!(
        lookup_base_value(TABLE, &r, &[]),
        Ok(BaseValueOutcome::Ignored)
    );
    r.outcome = RecognitionOutcome::NoValue;
    assert_eq!(
        lookup_base_value(TABLE, &r, &[]),
        Ok(BaseValueOutcome::NoValue)
    );
    r.outcome = RecognitionOutcome::Unknown;
    assert_eq!(
        lookup_base_value(TABLE, &r, &[]),
        Err(BaseValueError::UnknownRecognition)
    );
    r.outcome = RecognitionOutcome::Recognized;
    r.details = None;
    assert_eq!(
        lookup_base_value(TABLE, &r, &[]),
        Err(BaseValueError::UnknownDetails)
    );
    assert_eq!(
        lookup_base_value(TABLE, &recognition("1Lz", None, None), &[]),
        Err(BaseValueError::UnknownCalls)
    );
    assert_eq!(
        lookup_base_value(TABLE, &recognition("3T", None, Some(vec![])), &[]),
        Err(BaseValueError::Unsupported)
    );
    assert_eq!(
        lookup_base_value(
            TABLE,
            &recognition("1Lz", None, Some(vec![PanelCall::NotCounted])),
            &[]
        ),
        Err(BaseValueError::Unsupported)
    );
    assert_eq!(
        lookup_base_value(
            TABLE,
            &recognition("1Lz", None, Some(vec![])),
            &[ScoringAnnotation::SecondHalfCredit]
        ),
        Err(BaseValueError::Unsupported)
    );
    assert!("unknown-table".parse::<ScoringTableId>().is_err());
    let mut combination = recognition("1Lz", None, Some(vec![]));
    combination.details = Some(
        RecognitionDetails::new(
            vec![
                ComponentCode::try_from("1Lz").unwrap(),
                ComponentCode::try_from("1T").unwrap(),
            ],
            None,
            Some(vec![]),
        )
        .unwrap(),
    );
    assert!(matches!(
        lookup_base_value(TABLE, &combination, &[]),
        Ok(BaseValueOutcome::Value(BaseValue {
            hundredths: 100,
            ..
        }))
    ));
}

#[test]
fn single_judge_scores_follow_source_rows() {
    // Communication 2786, updated 2 June: 1Lz p. 2, CSSp1 p. 8, ChSp1 p. 14.
    for (code, level, base, adjustment, total) in [
        ("1Lz", None, 60, 6, 66),
        ("CSSp", Some(Level::One), 230, 23, 253),
        ("ChSp1", None, 350, 50, 400),
    ] {
        let r = recognition(code, level, Some(vec![]));
        let score = score_element(TABLE, &r, &[], 1, Some(false)).unwrap();
        assert_eq!(
            (
                score.base.hundredths,
                score.goe_hundredths,
                score.score_hundredths
            ),
            (base, adjustment, total)
        );
        assert_eq!(score.base.table, TABLE);
        assert!(score.goe_source.location.contains("GOE column +1"));
        assert_eq!(score.consistency_source.version, "2026-05-19");
        let (values, section, notation) = match code {
            "1Lz" => (
                [-30, -24, -18, -12, -6, 0, 6, 12, 18, 24, 30],
                "p. 2, Jumps",
                "1Lz",
            ),
            "CSSp" => (
                [-115, -92, -69, -46, -23, 0, 23, 46, 69, 92, 115],
                "p. 8, Spins",
                "CSSp1",
            ),
            _ => (
                [-250, -200, -150, -100, -50, 0, 50, 100, 150, 200, 250],
                "p. 14, Choreographic Elements",
                "ChSp1",
            ),
        };
        for (grade, expected) in (-5..=5).zip(values) {
            let score = score_element(TABLE, &r, &[], grade, Some(false)).unwrap();
            assert_eq!(score.goe_hundredths, expected);
            let column = if grade == 0 {
                "BASE (zero adjustment)".to_string()
            } else {
                format!("GOE column {grade:+}")
            };
            assert_eq!(
                score.goe_source,
                SourceReference {
                    document: "ISU Communication 2786".into(),
                    version: "2026-06-02".into(),
                    location: format!("{section}, {column}, row {notation}"),
                }
            );
        }
        assert_eq!(
            score_element(TABLE, &r, &[], -2, Some(true)),
            Err(ElementScoreError::InconsistentFall)
        );
        assert!(score_element(TABLE, &r, &[], -3, Some(true)).is_ok());
    }
}

#[test]
fn scoring_rejects_incomplete_or_inconsistent_inputs() {
    let r = recognition("1Lz", None, Some(vec![]));
    assert_eq!(
        score_element(TABLE, &r, &[], 6, Some(false)),
        Err(ElementScoreError::InvalidGoe)
    );
    assert_eq!(
        score_element(TABLE, &r, &[], -6, Some(false)),
        Err(ElementScoreError::InvalidGoe)
    );
    assert_eq!(
        score_element(TABLE, &r, &[], 0, None),
        Err(ElementScoreError::UnknownFall)
    );
    // Communication 2788 pp. 14, 16: -5 reduction and starting GOE at most +2.
    assert_eq!(
        score_element(TABLE, &r, &[], -2, Some(true)),
        Err(ElementScoreError::InconsistentFall)
    );
    assert_eq!(
        score_element(TABLE, &r, &[], -3, Some(true))
            .unwrap()
            .score_hundredths,
        42
    );
    assert_eq!(
        score_element(TABLE, &r, &[], -5, Some(true))
            .unwrap()
            .score_hundredths,
        30
    );
    let marked = recognition("1Lz", None, Some(vec![PanelCall::UnderRotated]));
    assert_eq!(
        score_element(TABLE, &marked, &[], 4, Some(false)),
        Err(ElementScoreError::UnsupportedMarks)
    );
    assert_eq!(
        score_element(
            TABLE,
            &r,
            &[ScoringAnnotation::SecondHalfCredit],
            1,
            Some(false)
        ),
        Err(ElementScoreError::BaseValue(BaseValueError::Unsupported))
    );
}

#[test]
fn score_preserves_outcomes_and_rejects_unknowns_and_combinations() {
    let mut r = recognition("1Lz", None, Some(vec![PanelCall::Downgraded]));
    assert!(matches!(
        lookup_base_value(TABLE, &r, &[]),
        Ok(BaseValueOutcome::Value(BaseValue { hundredths: 0, .. }))
    ));
    assert_eq!(
        score_element(TABLE, &r, &[], -3, Some(false))
            .unwrap()
            .score_hundredths,
        0
    );
    r.outcome = RecognitionOutcome::Ignored;
    assert_eq!(
        score_element(TABLE, &r, &[], 0, Some(false)),
        Err(ElementScoreError::Ignored)
    );
    r.outcome = RecognitionOutcome::NoValue;
    assert_eq!(
        score_element(TABLE, &r, &[], 0, Some(false)),
        Err(ElementScoreError::NoValue)
    );
    let unknown = recognition("1Lz", None, None);
    assert_eq!(
        score_element(TABLE, &unknown, &[], 0, Some(false)),
        Err(ElementScoreError::BaseValue(BaseValueError::UnknownCalls))
    );
    let mut combo = recognition("1Lz", None, Some(vec![]));
    combo.details = Some(
        RecognitionDetails::new(
            vec![
                ComponentCode::try_from("1Lz").unwrap(),
                ComponentCode::try_from("1T").unwrap(),
            ],
            None,
            Some(vec![]),
        )
        .unwrap(),
    );
    assert_eq!(
        score_element(TABLE, &combo, &[], 0, Some(false))
            .unwrap()
            .score_hundredths,
        100
    );
    assert_eq!(
        RecognitionDetails::new(
            vec![ComponentCode::try_from("1Lz").unwrap()],
            None,
            Some(vec![
                RecordedCall {
                    target: CallTarget::Component(0),
                    call: PanelCall::Quarter
                },
                RecordedCall {
                    target: CallTarget::Component(0),
                    call: PanelCall::UnderRotated
                }
            ])
        ),
        Err(RecognitionError::ConflictingCalls)
    );
}

fn program_element(
    code: &str,
    level: Option<Level>,
    goe: i8,
    fall: bool,
    start_ms: Option<u32>,
) -> ProgramElement {
    ProgramElement {
        recognition: recognition(code, level, Some(vec![])),
        annotations: vec![],
        goe,
        fall: Some(fall),
        start_ms,
    }
}

fn ordinary(seconds: u32) -> Option<ProgramTiming> {
    Some(ProgramTiming {
        duration_seconds: seconds,
        ordinary_timing_confirmed: true,
        second_half_default_confirmed: true,
    })
}

#[test]
fn planning_boundary_is_optional_and_never_timing_evidence() {
    let elements = vec![
        program_element("1Lz", None, 0, false, None),
        program_element("1Lz", None, 0, false, None),
    ];
    let plain =
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &elements, None).unwrap();
    assert_eq!(plain.hypothetical_hundredths, Some(120));
    assert!(plain
        .elements
        .iter()
        .all(|result| !result.as_ref().unwrap().second_half));
    let assumed =
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &elements, Some(1))
            .unwrap();
    assert_eq!(assumed.second_half_boundary, Some(1));
    assert_eq!(assumed.hypothetical_hundredths, Some(126));
    assert!(!assumed.elements[0].as_ref().unwrap().second_half);
    assert!(assumed.elements[1].as_ref().unwrap().second_half);
    assert_eq!(
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &elements,
            Some(0),
            None
        )
        .tes_hundredths,
        None
    );
    assert_eq!(
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &elements, Some(3)),
        Err(ProgramScoreError::InvalidInput)
    );
}

#[test]
fn planning_preserves_zero_outcomes_and_annotation_semantics() {
    let mut ignored = program_element("1Lz", None, 0, false, None);
    ignored.recognition.outcome = RecognitionOutcome::Ignored;
    let mut no_value = ignored.clone();
    no_value.recognition.outcome = RecognitionOutcome::NoValue;
    let mut jump = program_element("1Lz", None, 0, false, None);
    jump.annotations.push(ScoringAnnotation::SecondHalfCredit);
    let elements = vec![ignored, no_value, jump];
    let plain =
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &elements, None).unwrap();
    assert_eq!(plain.hypothetical_hundredths, Some(60));
    assert_eq!(
        plain.elements[0],
        Err(ProgramScoreError::Element(ElementScoreError::Ignored))
    );
    assert_eq!(
        plain.elements[1],
        Err(ProgramScoreError::Element(ElementScoreError::NoValue))
    );
    let assumed =
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &elements, Some(2))
            .unwrap();
    assert_eq!(assumed.hypothetical_hundredths, Some(66));
    let mut timed = elements[2].clone();
    timed.start_ms = Some(50_000);
    let actual = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &[timed],
        Some(0),
        ordinary(100),
    );
    assert_eq!(actual.tes_hundredths, Some(66));
}

#[test]
fn planning_boundary_and_uncertain_selection_are_local_to_affected_jumps() {
    let jump = program_element("1Lz", None, 0, false, None);
    let jumps = vec![jump.clone(); 5];
    let all =
        score_planning_scenario(RulesetId::IsuAdult2026_27July16, TABLE, &jumps, Some(0)).unwrap();
    assert_eq!(
        all.elements
            .iter()
            .map(|s| s.as_ref().unwrap().second_half)
            .collect::<Vec<_>>(),
        vec![false, false, true, true, true]
    );
    let none = score_planning_scenario(
        RulesetId::IsuAdult2026_27July16,
        TABLE,
        &jumps,
        Some(jumps.len()),
    )
    .unwrap();
    assert!(none
        .elements
        .iter()
        .all(|s| !s.as_ref().unwrap().second_half));
    let mut unknown = jump.clone();
    unknown.recognition.outcome = RecognitionOutcome::Unknown;
    let before = score_planning_scenario(
        RulesetId::IsuAdult2026_27July16,
        TABLE,
        &[unknown.clone(), jump.clone()],
        Some(1),
    )
    .unwrap();
    assert!(before.elements[1].as_ref().unwrap().second_half);
    assert_eq!(before.hypothetical_hundredths, None);
    let after = score_planning_scenario(
        RulesetId::IsuAdult2026_27July16,
        TABLE,
        &[jump.clone(), unknown, jump.clone(), jump],
        Some(0),
    )
    .unwrap();
    assert_eq!(after.elements[0], Err(ProgramScoreError::TimingUnknown));
    assert!(after.elements[2].as_ref().unwrap().second_half);
    assert!(after.elements[3].as_ref().unwrap().second_half);
    assert_eq!(after.hypothetical_hundredths, None);
}

#[test]
fn program_totals_keep_deductions_separate() {
    // Adult announcement pp. 6, 10, 11 and Communication 2786 pp. 2, 8.
    let elements = vec![
        program_element("1Lz", None, -3, true, Some(10_000)),
        program_element("CSSp", Some(Level::One), 1, false, None),
    ];
    let result = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &elements,
        Some(1),
        ordinary(110),
    );
    assert_eq!(result.tes_hundredths, Some(42 + 253));
    assert_eq!(result.fall_deduction.as_ref().unwrap().hundredths, 50);
    assert_eq!(result.time_deduction.as_ref().unwrap().hundredths, 0);
    assert_eq!(result.fall_deduction.unwrap().source.version, "2026-07-16");
    for (category, seconds, expected) in [
        (AdultCategory::Bronze, 111, 100),
        (AdultCategory::Bronze, 116, 200),
        (AdultCategory::Silver, 130, 0),
        (AdultCategory::Silver, 131, 100),
    ] {
        assert_eq!(
            score_program(
                RulesetId::IsuAdult2026_27July16,
                category,
                TABLE,
                &[],
                Some(0),
                ordinary(seconds)
            )
            .time_deduction
            .unwrap()
            .hundredths,
            expected
        );
    }
}

#[test]
fn second_half_uses_required_duration_last_three_and_unfactored_goe() {
    // Synthetic selection test only: five repeated jumps do not form a legal Bronze program.
    // Rule 353.1(h)(iii), p. 9: last three jumps starting in second half.
    let elements: Vec<_> = [49_999, 50_000, 60_000, 70_000, 80_000]
        .into_iter()
        .map(|start| program_element("1Lz", None, 1, false, Some(start)))
        .collect();
    let result = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &elements,
        Some(0),
        ordinary(110),
    );
    assert_eq!(result.tes_hundredths, Some(2 * 66 + 3 * 72));
    let scores: Vec<_> = result
        .elements
        .iter()
        .map(|x| x.as_ref().unwrap())
        .collect();
    assert_eq!(
        scores.iter().map(|s| s.second_half).collect::<Vec<_>>(),
        vec![false, false, true, true, true]
    );
    assert_eq!(
        (
            scores[2].credited_base_hundredths,
            scores[2].unfactored.goe_hundredths
        ),
        (66, 6)
    );
    assert_eq!(
        scores[2].second_half_source.as_ref().unwrap().version,
        "2026-06-12"
    );
}

#[test]
fn unresolved_inputs_do_not_yield_exact_tes() {
    let mut element = program_element("1Lz", None, 1, false, None);
    element
        .annotations
        .push(ScoringAnnotation::SecondHalfCredit);
    assert_eq!(
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &[element.clone()],
            Some(0),
            ordinary(100)
        )
        .tes_hundredths,
        None
    );
    element.start_ms = Some(60_000);
    let mut t = ordinary(100).unwrap();
    t.second_half_default_confirmed = false;
    assert_eq!(
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &[element.clone()],
            Some(0),
            Some(t)
        )
        .tes_hundredths,
        None
    );
    let unsupported = program_element("3T", None, 0, false, Some(70_000));
    assert_eq!(
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &[element, unsupported],
            Some(0),
            ordinary(100)
        )
        .tes_hundredths,
        None
    );
    assert!(score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &[],
        Some(0),
        None
    )
    .time_deduction
    .is_err());
}

#[test]
fn invalid_timing_and_falls_remain_unresolved() {
    let later = program_element("1Lz", None, 0, false, Some(80_000));
    let earlier = program_element("1Lz", None, 0, false, Some(60_000));
    assert_eq!(
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &[later, earlier],
            Some(0),
            ordinary(100)
        )
        .tes_hundredths,
        None
    );
    assert!(score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &[],
        Some(0),
        ordinary(u32::MAX)
    )
    .time_deduction
    .is_err());
    let mut unknown = program_element("1Lz", None, -3, true, Some(30_000));
    unknown.recognition.outcome = RecognitionOutcome::Unknown;
    assert!(score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &[unknown],
        Some(0),
        ordinary(100)
    )
    .fall_deduction
    .is_err());
    for (category, seconds) in [(AdultCategory::Gold, 181), (AdultCategory::Masters, 191)] {
        assert_eq!(
            score_program(
                RulesetId::IsuAdult2026_27July16,
                category,
                TABLE,
                &[],
                None,
                ordinary(seconds)
            )
            .time_deduction
            .unwrap()
            .hundredths,
            100
        );
    }
}

#[test]
fn timing_rule_502_prevents_false_exact_scores() {
    let rules = RulesetId::IsuAdult2026_27July16;
    for (duration, start, expected_exact) in [
        (112, 110_000, true),
        (112, 111_000, false),
        (60, 10_000, false),
        (61, 10_000, true),
    ] {
        let jump = program_element("1Lz", None, 1, false, Some(start));
        let result = score_program(
            rules,
            AdultCategory::Bronze,
            TABLE,
            &[jump],
            Some(0),
            ordinary(duration),
        );
        assert_eq!(
            result.tes_hundredths.is_some(),
            expected_exact,
            "{duration} {start}"
        );
    }
    let spin = program_element("CSSp", Some(Level::One), 1, false, None);
    assert_eq!(
        score_program(
            rules,
            AdultCategory::Bronze,
            TABLE,
            &[spin],
            Some(0),
            ordinary(112)
        )
        .tes_hundredths,
        None
    );
    assert_eq!(
        score_program(
            rules,
            AdultCategory::Bronze,
            TABLE,
            &[],
            Some(0),
            ordinary(60)
        )
        .tes_hundredths,
        None
    );
}

#[test]
fn annotations_and_unresolved_nonjumps_are_isolated() {
    let rules = RulesetId::IsuAdult2026_27July16;
    let mut spin = program_element("CSSp", Some(Level::One), 1, false, None);
    spin.annotations.push(ScoringAnnotation::SecondHalfCredit);
    assert!(score_program(
        rules,
        AdultCategory::Bronze,
        TABLE,
        &[spin],
        Some(0),
        ordinary(100)
    )
    .elements[0]
        .is_err());
    let mut jump = program_element("1Lz", None, 1, false, Some(50_000));
    jump.annotations.push(ScoringAnnotation::SecondHalfCredit);
    let unsupported = program_element("CSSp", Some(Level::Two), 0, false, None);
    let result = score_program(
        rules,
        AdultCategory::Bronze,
        TABLE,
        &[jump.clone(), unsupported],
        Some(0),
        ordinary(100),
    );
    assert_eq!(result.elements[0].as_ref().unwrap().score_hundredths, 72);
    assert!(result.elements[1].is_err());
    assert_eq!(result.tes_hundredths, None);
    jump.annotations.push(ScoringAnnotation::SecondHalfCredit);
    assert!(score_program(
        rules,
        AdultCategory::Bronze,
        TABLE,
        &[jump],
        Some(0),
        ordinary(100)
    )
    .elements[0]
        .is_err());
}

#[test]
fn bronze_boundary_fixture_uses_required_duration_and_unfactored_goe() {
    // Two single Lutz jumps are within Bronze's jump-element and repetition limits.
    let jumps = [
        program_element("1Lz", None, 1, false, Some(49_999)),
        program_element("1Lz", None, 1, false, Some(50_000)),
    ];
    let result = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &jumps,
        Some(0),
        ordinary(110),
    );
    let first = result.elements[0].as_ref().unwrap();
    let second = result.elements[1].as_ref().unwrap();
    assert_eq!(
        (
            first.score_hundredths,
            second.credited_base_hundredths,
            second.unfactored.goe_hundredths,
            second.score_hundredths
        ),
        (66, 66, 6, 72)
    );
    assert_eq!(result.tes_hundredths, Some(138));
}

#[test]
fn timestamps_follow_program_order_across_element_types() {
    let rules = RulesetId::IsuAdult2026_27July16;
    let spin = program_element("CSSp", Some(Level::One), 1, false, Some(80_000));
    let jump = program_element("1Lz", None, 1, false, Some(50_000));
    let result = score_program(
        rules,
        AdultCategory::Bronze,
        TABLE,
        &[spin, jump],
        Some(0),
        ordinary(100),
    );
    assert_eq!(result.elements[0].as_ref().unwrap().score_hundredths, 253);
    assert_eq!(result.elements[1], Err(ProgramScoreError::InvalidTiming));
    assert_eq!(result.tes_hundredths, None);
    let spin = program_element("CSSp", Some(Level::One), 1, false, Some(40_000));
    let jump = program_element("1Lz", None, 1, false, Some(50_000));
    let result = score_program(
        rules,
        AdultCategory::Bronze,
        TABLE,
        &[spin, jump],
        Some(0),
        ordinary(100),
    );
    assert_eq!(result.tes_hundredths, Some(325));
}

#[test]
fn resolved_zero_and_ignored_elements_do_not_make_tes_incomplete() {
    // Communication 2786 p. 2: 1Lz<< has zero in every GOE column.
    // Adult announcement p. 11: the Bronze Waltz jump is ignored.
    let mut zero = program_element("1Lz", None, -1, false, Some(10_000));
    zero.recognition = recognition("1Lz", None, Some(vec![PanelCall::Downgraded]));
    let mut ignored = program_element("1A", None, 0, false, Some(20_000));
    ignored.recognition.outcome = RecognitionOutcome::Ignored;
    let jump = program_element("1Lz", None, 1, false, Some(60_000));
    let result = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &[zero, ignored, jump],
        Some(0),
        ordinary(100),
    );
    assert_eq!(result.elements[0].as_ref().unwrap().score_hundredths, 0);
    assert_eq!(
        result.elements[1],
        Err(ProgramScoreError::Element(ElementScoreError::Ignored))
    );
    assert_eq!(result.elements[2].as_ref().unwrap().score_hundredths, 72);
    assert_eq!(result.tes_hundredths, Some(72));
}

#[test]
fn no_value_is_zero_but_invalid_or_unknown_outcomes_are_not() {
    let mut element = program_element("1A", None, 0, false, Some(20_000));
    element.recognition.outcome = RecognitionOutcome::NoValue;
    let score = |element| {
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &[element],
            Some(0),
            ordinary(100),
        )
    };
    let result = score(element.clone());
    assert_eq!(
        result.elements[0],
        Err(ProgramScoreError::Element(ElementScoreError::NoValue))
    );
    assert_eq!(result.tes_hundredths, Some(0));
    element
        .annotations
        .push(ScoringAnnotation::SecondHalfCredit);
    assert_eq!(score(element.clone()).tes_hundredths, None);
    element.annotations.clear();
    element.recognition.outcome = RecognitionOutcome::Unknown;
    assert_eq!(score(element).tes_hundredths, None);
}

#[test]
fn downgraded_zero_rows_require_consistent_goe() {
    // Communication 2788 pp. 14 and 16: downgrade reduction and starting limit.
    for code in ["1Lz", "1A"] {
        let r = recognition(code, None, Some(vec![PanelCall::Downgraded]));
        assert_eq!(
            score_element(TABLE, &r, &[], -1, Some(false))
                .unwrap()
                .score_hundredths,
            0
        );
        for goe in [0, 5] {
            assert_eq!(
                score_element(TABLE, &r, &[], goe, Some(false)),
                Err(ElementScoreError::UnsupportedMarks)
            );
        }
    }
}

#[test]
fn no_value_jump_does_not_claim_a_bonus_when_eligibility_is_unknown() {
    let mut no_value = program_element("1S", None, 0, false, Some(70_000));
    no_value.recognition.outcome = RecognitionOutcome::NoValue;
    let jumps = [
        program_element("1Lz", None, 1, false, Some(51_000)),
        program_element("1Lz", None, 1, false, Some(60_000)),
        no_value.clone(),
        {
            no_value.start_ms = Some(80_000);
            no_value
        },
    ];
    let result = score_program(
        RulesetId::IsuAdult2026_27July16,
        AdultCategory::Bronze,
        TABLE,
        &jumps,
        Some(0),
        ordinary(100),
    );
    assert_eq!(result.tes_hundredths, None);
}

#[test]
fn repeated_program_scores_and_sources_match() {
    let elements = [program_element("1Lz", None, 1, false, Some(51_000))];
    let score = || {
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            &elements,
            Some(0),
            ordinary(100),
        )
    };
    assert_eq!(score(), score());
}

#[test]
fn downgrade_and_fall_require_minimum_final_goe() {
    for code in ["1Lz", "1A"] {
        let r = recognition(code, None, Some(vec![PanelCall::Downgraded]));
        for goe in [-3, -4] {
            assert_eq!(
                score_element(TABLE, &r, &[], goe, Some(true)),
                Err(ElementScoreError::InconsistentFall)
            );
        }
        assert_eq!(
            score_element(TABLE, &r, &[], -5, Some(true))
                .unwrap()
                .score_hundredths,
            0
        );
    }
}

#[test]
fn no_value_before_half_or_known_nonjump_does_not_block_bonus() {
    let mut zero = program_element("1S", None, 0, false, Some(20_000));
    zero.recognition.outcome = RecognitionOutcome::NoValue;
    let jump = program_element("1Lz", None, 1, false, Some(30_000));
    let score = |elements: &[ProgramElement]| {
        score_program(
            RulesetId::IsuAdult2026_27July16,
            AdultCategory::Bronze,
            TABLE,
            elements,
            Some(0),
            ordinary(100),
        )
    };
    assert_eq!(score(&[zero, jump]).tes_hundredths, Some(66));
    let mut spin = program_element("CSSp", Some(Level::One), 0, false, Some(60_000));
    spin.recognition.outcome = RecognitionOutcome::NoValue;
    let late_jump = program_element("1Lz", None, 1, false, Some(70_000));
    assert_eq!(score(&[spin, late_jump]).tes_hundredths, Some(72));
}

#[test]
fn clean_jump_columns_match_june_table() {
    // Communication 2786, 2 June 2026, pp. 2–3: BASE and each GOE column.
    let fixtures = [
        ("1T", 40, [-20, -16, -12, -8, -4, 0, 4, 8, 12, 16, 20]),
        ("1S", 40, [-20, -16, -12, -8, -4, 0, 4, 8, 12, 16, 20]),
        ("1Lo", 50, [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25]),
        ("1F", 50, [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25]),
        ("1Lz", 60, [-30, -24, -18, -12, -6, 0, 6, 12, 18, 24, 30]),
        ("1A", 110, [-55, -44, -33, -22, -11, 0, 11, 22, 33, 44, 55]),
        ("2T", 130, [-65, -52, -39, -26, -13, 0, 13, 26, 39, 52, 65]),
        ("2S", 130, [-65, -52, -39, -26, -13, 0, 13, 26, 39, 52, 65]),
        ("2Lo", 170, [-85, -68, -51, -34, -17, 0, 17, 34, 51, 68, 85]),
        ("2F", 180, [-90, -72, -54, -36, -18, 0, 18, 36, 54, 72, 90]),
        (
            "2Lz",
            210,
            [-105, -84, -63, -42, -21, 0, 21, 42, 63, 84, 105],
        ),
        (
            "2A",
            330,
            [-165, -132, -99, -66, -33, 0, 33, 66, 99, 132, 165],
        ),
    ];
    for (code, base, expected_values) in fixtures {
        let r = recognition(code, None, Some(vec![]));
        for (grade, expected) in (-5i8..=5).zip(expected_values) {
            let score = score_element(TABLE, &r, &[], grade, Some(false)).unwrap();
            assert_eq!(
                (score.base.hundredths, score.goe_hundredths),
                (base, expected),
                "{code} {grade}"
            );
        }
    }
}

#[test]
fn marked_rows_and_call_boundaries() {
    use PanelCall::*;
    // Communication 2786 pp. 2–4 and 2788 pp. 14, 16.
    for (code, call, grade, base, adjustment) in [
        ("2Lo", Quarter, 3, 170, 51),
        ("2T", UnderRotated, -3, 104, -31),
        ("2A", Downgraded, -3, 110, -33),
        ("2F", WrongEdge, 0, 144, 0),
        ("1Lz", UnclearEdge, 4, 60, 24),
    ] {
        let r = recognition(code, None, Some(vec![call]));
        let score = score_element(TABLE, &r, &[], grade, Some(false)).unwrap();
        assert_eq!(
            (score.base.hundredths, score.goe_hundredths),
            (base, adjustment)
        );
    }
    for call in [WrongEdge, UnclearEdge] {
        assert_eq!(
            lookup_base_value(TABLE, &recognition("2T", None, Some(vec![call])), &[]),
            Err(BaseValueError::Unsupported)
        );
    }
    let combined = recognition("1Lz", None, Some(vec![WrongEdge, UnderRotated]));
    assert_eq!(
        score_element(TABLE, &combined, &[], -5, Some(false)),
        Err(ElementScoreError::UnsupportedMarks)
    );
}

#[test]
fn jump_value_does_not_depend_on_adult_category() {
    let jump = program_element("2Lo", None, 1, false, Some(10_000));
    for (category, duration) in [
        (AdultCategory::Bronze, 100),
        (AdultCategory::Silver, 120),
        (AdultCategory::Gold, 170),
        (AdultCategory::Masters, 180),
    ] {
        let result = score_program(
            RulesetId::IsuAdult2026_27July16,
            category,
            TABLE,
            std::slice::from_ref(&jump),
            Some(0),
            ordinary(duration),
        );
        let score = result.elements[0].as_ref().unwrap();
        assert_eq!(
            (
                score.unfactored.base.hundredths,
                score.unfactored.goe_hundredths
            ),
            (170, 17)
        );
    }
}

#[test]
fn call_and_fall_limits_are_enforced() {
    use PanelCall::*;
    for (call, maximum, fall_maximum) in [
        (Quarter, 3, -5),
        (UnderRotated, 3, -5),
        (Downgraded, -1, -5),
        (UnclearEdge, 4, -4),
        (WrongEdge, 0, -5),
    ] {
        let r = recognition("2F", None, Some(vec![call]));
        assert!(score_element(TABLE, &r, &[], maximum, Some(false)).is_ok());
        assert_eq!(
            score_element(TABLE, &r, &[], maximum + 1, Some(false)),
            Err(ElementScoreError::UnsupportedMarks)
        );
        assert!(score_element(TABLE, &r, &[], fall_maximum, Some(true)).is_ok());
        assert_eq!(
            score_element(TABLE, &r, &[], fall_maximum + 1, Some(true)),
            Err(ElementScoreError::InconsistentFall)
        );
    }
}

#[test]
fn recognized_pair_sums_base_and_uses_one_reference_goe() {
    let mut combo = recognition("1Lz", None, Some(vec![]));
    combo.details = Some(
        RecognitionDetails::new(
            vec![
                ComponentCode::try_from("1Lz").unwrap(),
                ComponentCode::try_from("1T").unwrap(),
            ],
            None,
            Some(vec![]),
        )
        .unwrap(),
    );
    // ISU 2786 p. 2: 0.60 + 0.40; Rule 353.1(h)(i): GOE +2 of 1Lz is +0.12 once.
    let score = score_element(TABLE, &combo, &[], 2, Some(false)).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (100, 12, 112)
    );
    assert!(score.goe_source.location.contains("1Lz"));
    assert_eq!(score.base.source.version, "2026-06-02");
    combo.details = Some(
        RecognitionDetails::new(
            vec![
                ComponentCode::try_from("1Lz").unwrap(),
                ComponentCode::try_from("1T").unwrap(),
            ],
            None,
            Some(vec![RecordedCall {
                target: CallTarget::Component(0),
                call: PanelCall::UnderRotated,
            }]),
        )
        .unwrap(),
    );
    let score = score_element(TABLE, &combo, &[], 2, Some(false)).unwrap();
    assert_eq!((score.base.hundredths, score.goe_hundredths), (88, 10));
    combo.details = Some(
        RecognitionDetails::new(
            vec![
                ComponentCode::try_from("1Lz").unwrap(),
                ComponentCode::try_from("1F").unwrap(),
            ],
            None,
            Some(vec![]),
        )
        .unwrap(),
    );
    assert_eq!(
        score_element(TABLE, &combo, &[], 0, Some(false)),
        Err(ElementScoreError::BaseValue(BaseValueError::Unsupported))
    );
}
