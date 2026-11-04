use skating_program_lab::*;
use std::collections::BTreeMap;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

// Fixture adapter only: production callers supply typed recognition, not strings.
fn recognition(code: &str) -> PanelRecognition {
    let mut calls = vec![];
    let mut components = vec![];
    let mut level = None;
    for token in code.split('+') {
        if token == "SEQ" {
            calls.push(RecordedCall {
                target: CallTarget::BetweenComponents(components.len()),
                call: PanelCall::Sequence,
            });
            continue;
        }
        if token == "REP" {
            calls.push(RecordedCall {
                target: CallTarget::Element,
                call: PanelCall::Repetition,
            });
            continue;
        }
        let mut stem = token.to_string();
        let target = CallTarget::Component(components.len());
        for (suffix, call) in [
            ("*", PanelCall::NotCounted),
            ("<<", PanelCall::Downgraded),
            ("<", PanelCall::UnderRotated),
            ("q", PanelCall::Quarter),
            ("!", PanelCall::UnclearEdge),
            ("e", PanelCall::WrongEdge),
            ("V", PanelCall::ReducedSpinValue),
        ] {
            if let Some(value) = stem.strip_suffix(suffix) {
                stem = value.into();
                calls.push(RecordedCall { target, call });
            }
        }
        if !stem.starts_with(char::is_numeric) && !matches!(stem.as_str(), "Eu" | "ChSq1" | "ChSp1")
        {
            level = Some(match stem.pop().unwrap() {
                'B' => Level::Basic,
                '1' => Level::One,
                '2' => Level::Two,
                '3' => Level::Three,
                '4' => Level::Four,
                _ => panic!("{code}"),
            });
        }
        components.push(ComponentCode::try_from(stem.as_str()).unwrap());
    }
    PanelRecognition {
        ruleset: RulesetId::IsuSenior2026_27June12,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(RecognitionDetails::new(components, level, Some(calls)).unwrap()),
    }
}

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

fn cents(text: &str) -> i32 {
    let (a, b) = text.trim_start_matches('-').split_once('.').unwrap();
    let value = a.parse::<i32>().unwrap() * 100 + b.parse::<i32>().unwrap();
    if text.starts_with('-') {
        -value
    } else {
        value
    }
}

#[test]
fn all_independently_transcribed_protocol_rows_replay_to_printed_hundredths() {
    let mut programs: BTreeMap<String, Vec<(ProtocolElementInput, Vec<String>)>> = BTreeMap::new();
    for line in include_str!("fixtures/grand-prix-2026-27/elements.tsv")
        .lines()
        .skip(1)
    {
        let f: Vec<_> = line.split('\t').collect();
        let mut row = input(f[4]);
        row.source = source(&format!(
            "{} page {} rank {} row {}",
            f[0], f[1], f[2], f[3]
        ));
        row.printed_second_half = f[7] == "x";
        row.judges = Some(JudgePanelGrades {
            grades: f[9..18]
                .iter()
                .map(|g| {
                    if *g == "-" {
                        None
                    } else {
                        Some(g.parse().unwrap())
                    }
                })
                .collect(),
        });
        programs
            .entry(format!("{} {} {}", f[0], f[1], f[2]))
            .or_default()
            .push((row, f.iter().map(|s| s.to_string()).collect()));
    }
    let mut checked = 0;
    for rows in programs.values() {
        let inputs: Vec<_> = rows.iter().map(|(r, _)| r.clone()).collect();
        let report = replay(&inputs);
        let mut expected_tes = 0;
        for ((input, fields), scored) in rows.iter().zip(report.elements) {
            let scored =
                scored.unwrap_or_else(|e| panic!("{} {}: {e:?}", input.source.location, fields[4]));
            assert_eq!(
                i32::from(scored.credited_base_hundredths),
                cents(&fields[6]),
                "{} BV",
                input.source.location
            );
            assert_eq!(
                scored
                    .panel_goe
                    .as_ref()
                    .map_or(0, |g| i32::from(g.goe_hundredths)),
                cents(&fields[8]),
                "{} GOE",
                input.source.location
            );
            assert_eq!(
                scored.score_hundredths,
                Some(cents(&fields[19])),
                "{} total",
                input.source.location
            );
            expected_tes += i64::from(cents(&fields[19]));
            checked += 1;
        }
        assert_eq!(report.tes_hundredths, Some(expected_tes));
    }
    assert_eq!(checked, 132);
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
