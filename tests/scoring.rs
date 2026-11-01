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
        lookup_base_value(TABLE, &recognition("1T", None, Some(vec![])), &[]),
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
    assert_eq!(
        lookup_base_value(TABLE, &combination, &[]),
        Err(BaseValueError::Unsupported)
    );
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
        score_element(TABLE, &marked, &[], 1, Some(false)),
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
        score_element(TABLE, &r, &[], -3, Some(false)),
        Err(ElementScoreError::UnsupportedMarks)
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
        score_element(TABLE, &combo, &[], 0, Some(false)),
        Err(ElementScoreError::BaseValue(BaseValueError::Unsupported))
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
