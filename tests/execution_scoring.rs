use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const RULESET: RulesetId = RulesetId::IsuAdult2026_27July16;

fn unit(codes: &[&str], calls: Vec<RecordedCall>) -> PanelRecognition {
    PanelRecognition {
        ruleset: RULESET,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                codes
                    .iter()
                    .map(|code| ComponentCode::try_from(*code).unwrap())
                    .collect(),
                None,
                Some(calls),
            )
            .unwrap(),
        ),
    }
}

fn component(index: usize, call: PanelCall) -> RecordedCall {
    RecordedCall {
        target: CallTarget::Component(index),
        call,
    }
}

fn structure(before: usize, call: PanelCall) -> RecordedCall {
    RecordedCall {
        target: CallTarget::BetweenComponents(before),
        call,
    }
}

fn evidence(errors: Vec<JumpExecutionError>) -> ElementExecutionEvidence {
    ElementExecutionEvidence {
        errors: Some(errors),
    }
}

fn interrupted() -> PanelRecognition {
    unit(
        &["2Lz", "2Lo"],
        vec![
            structure(1, PanelCall::Combination),
            component(1, PanelCall::NotCounted),
        ],
    )
}

#[test]
fn step_out_evidence_enables_interrupted_score_without_changing_legacy_contract() {
    let r = interrupted();
    let observed = evidence(vec![JumpExecutionError::StepOut { component: 0 }]);
    let original = (r.clone(), observed.clone());
    // 2788 p. 14: step out -3..-4, starting cap +2 => final at most -1.
    // SOV p. 2: retained 2Lz BV 210, GOE -1 = -21; deleted 2Lo contributes zero.
    let score = score_element_with_execution(TABLE, &r, &[], -1, None, &observed).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (210, -21, 189)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], 0, Some(false), &observed),
        Err(ElementScoreError::UnsupportedMarks)
    );
    assert_eq!(
        score_element(TABLE, &r, &[], -1, Some(false)),
        Err(ElementScoreError::UnsupportedMarks)
    );
    for unknown in [ElementExecutionEvidence::default(), evidence(vec![])] {
        assert_eq!(
            score_element_with_execution(TABLE, &r, &[], -1, Some(false), &unknown),
            Err(ElementScoreError::UnsupportedMarks)
        );
    }
    assert_eq!((r, observed), original);
}

#[test]
fn fall_location_and_deleted_calls_have_cumulative_consequences() {
    // Single/double analogue of October Q&A 3: tail calls remain assessed.
    let r = unit(
        &["2S", "2T", "2A"],
        vec![
            structure(1, PanelCall::Sequence),
            component(1, PanelCall::UnderRotated),
            component(1, PanelCall::NotCounted),
            component(2, PanelCall::Quarter),
            component(2, PanelCall::NotCounted),
        ],
    );
    let observed = evidence(vec![JumpExecutionError::Fall { component: 0 }]);
    // Minimum reductions 2+2+5 with starting cap +2 => only final -5 after clamping.
    let score = score_element_with_execution(TABLE, &r, &[], -5, None, &observed).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (130, -65, 65)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], -4, Some(true), &observed),
        Err(ElementScoreError::InconsistentFall)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], -5, Some(false), &observed),
        Err(ElementScoreError::InconsistentFall)
    );
    let clean = interrupted();
    let twice = evidence(vec![
        JumpExecutionError::Fall { component: 0 },
        JumpExecutionError::Fall { component: 1 },
    ]);
    assert!(score_element_with_execution(TABLE, &clean, &[], -5, Some(true), &twice).is_ok());
    assert_eq!(
        score_element_with_execution(TABLE, &clean, &[], -4, Some(true), &twice),
        Err(ElementScoreError::InconsistentFall)
    );
}

#[test]
fn full_revolutions_and_euler_step_over_use_verified_distinct_rules() {
    let full = evidence(vec![JumpExecutionError::FullRevolutionBetween {
        before_component: 1,
        full_revolutions: 1,
    }]);
    let r = unit(&["2Lz", "2T"], vec![]);
    // Judges Handbook p. 5: one full revolution / two three turns, -3..-4, cap +2.
    let score = score_element_with_execution(TABLE, &r, &[], -1, Some(false), &full).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (340, -21, 319)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], 0, Some(false), &full),
        Err(ElementScoreError::UnsupportedMarks)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &interrupted(), &[], -1, Some(false), &full),
        Err(ElementScoreError::UnsupportedMarks)
    );
    let excess = evidence(vec![JumpExecutionError::FullRevolutionBetween {
        before_component: 1,
        full_revolutions: 2,
    }]);
    assert!(
        score_element_with_execution(TABLE, &interrupted(), &[], -1, Some(false), &excess).is_ok()
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], -1, Some(false), &excess),
        Err(ElementScoreError::UnsupportedMarks)
    );
    let euler = unit(&["1Lz", "Eu", "1S"], vec![]);
    let step = evidence(vec![JumpExecutionError::EulerStepOver { component: 1 }]);
    // 2788 p. 16 / Judges Handbook p. 5: -2..-3; one error starts at most +4.
    let score = score_element_with_execution(TABLE, &euler, &[], 2, Some(false), &step).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (100, 12, 112)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &euler, &[], 3, Some(false), &step),
        Err(ElementScoreError::UnsupportedMarks)
    );
    let q = unit(&["1Lz", "Eu", "1S"], vec![component(0, PanelCall::Quarter)]);
    let score = score_element_with_execution(TABLE, &q, &[], -2, Some(false), &step).unwrap();
    assert_eq!(score.score_hundredths, 88); // 60+0+40 minus reference GOE 12.
    assert_eq!(
        score_element_with_execution(TABLE, &q, &[], -1, Some(false), &step),
        Err(ElementScoreError::UnsupportedMarks)
    );
}

#[test]
fn invalid_or_unsupported_evidence_is_rejected_without_inference() {
    use JumpExecutionError::*;
    for errors in [
        vec![StepOut { component: 9 }],
        vec![StepOut { component: 0 }, StepOut { component: 0 }],
        vec![Fall { component: 0 }, StepOut { component: 0 }],
        vec![FullRevolutionBetween {
            before_component: 0,
            full_revolutions: 1,
        }],
        vec![FullRevolutionBetween {
            before_component: 2,
            full_revolutions: 1,
        }],
        vec![FullRevolutionBetween {
            before_component: 1,
            full_revolutions: 0,
        }],
        vec![
            FullRevolutionBetween {
                before_component: 1,
                full_revolutions: 2,
            },
            FullRevolutionBetween {
                before_component: 1,
                full_revolutions: 3,
            },
        ],
        vec![EulerStepOver { component: 0 }],
        vec![TouchDownWithWeightTransfer { component: 0 }],
        vec![StepOut { component: 1 }], // Does not explain the supplied interruption before component 1.
    ] {
        assert_eq!(
            score_element_with_execution(TABLE, &interrupted(), &[], -5, None, &evidence(errors)),
            Err(ElementScoreError::UnsupportedMarks)
        );
    }
    let clean = unit(&["2Lz", "2T"], vec![]);
    assert_eq!(
        score_element_with_execution(
            TABLE,
            &clean,
            &[],
            -1,
            Some(false),
            &evidence(vec![StepOut { component: 0 }])
        ),
        Err(ElementScoreError::UnsupportedMarks)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &clean, &[], 0, Some(true), &evidence(vec![])),
        Err(ElementScoreError::InconsistentFall)
    );
}

#[test]
fn nonterminal_sequence_before_deleted_axel_and_rep_needs_explicit_cause() {
    let r = unit(
        &["1Lz", "1A"],
        vec![
            structure(1, PanelCall::Sequence),
            component(1, PanelCall::NotCounted),
            RecordedCall {
                target: CallTarget::Element,
                call: PanelCall::Repetition,
            },
        ],
    );
    assert_eq!(
        score_element(TABLE, &r, &[], -1, Some(false)),
        Err(ElementScoreError::UnsupportedMarks)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], -1, Some(false), &evidence(vec![])),
        Err(ElementScoreError::UnsupportedMarks)
    );
    let step = evidence(vec![JumpExecutionError::StepOut { component: 0 }]);
    let score = score_element_with_execution(TABLE, &r, &[], -1, Some(false), &step).unwrap();
    assert_eq!(
        (
            score.base.hundredths,
            score.goe_hundredths,
            score.score_hundredths
        ),
        (48, -6, 42)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &r, &[], 0, Some(false), &step),
        Err(ElementScoreError::UnsupportedMarks)
    );
}

fn ordinary() -> Option<ProgramTiming> {
    Some(ProgramTiming {
        duration_seconds: 100,
        ordinary_timing_confirmed: true,
        second_half_default_confirmed: true,
    })
}

#[test]
fn execution_program_and_planning_keep_credit_and_deductions_separate() {
    let r = unit(
        &["1Lz", "1T"],
        vec![
            structure(1, PanelCall::Combination),
            component(1, PanelCall::NotCounted),
        ],
    );
    let element = ProgramElement {
        recognition: r,
        annotations: vec![],
        goe: -3,
        fall: None,
        start_ms: Some(50_000),
    };
    let observations = [evidence(vec![JumpExecutionError::Fall { component: 0 }])];
    let timed = score_program_with_execution(
        RULESET,
        AdultCategory::Bronze,
        TABLE,
        std::slice::from_ref(&element),
        Some(2),
        ordinary(),
        &observations,
    );
    // SOV 1Lz: 60; timing credit 66; unfactored -3 GOE -18 => TES 48.
    assert_eq!(timed.tes_hundredths, Some(48));
    assert_eq!(
        timed.elements[0].as_ref().unwrap().credited_base_hundredths,
        66
    );
    assert_eq!(
        timed.elements[0]
            .as_ref()
            .unwrap()
            .unfactored
            .goe_hundredths,
        -18
    );
    assert_eq!(timed.fall_deduction.unwrap().hundredths, 100); // Two separately supplied outside falls.
    assert_eq!(timed.time_deduction.unwrap().hundredths, 0);
    let planned = score_planning_scenario_with_execution(
        RULESET,
        TABLE,
        std::slice::from_ref(&element),
        Some(0),
        &observations,
    )
    .unwrap();
    assert_eq!(planned.hypothetical_hundredths, Some(48));
    assert_eq!(
        score_program(
            RULESET,
            AdultCategory::Bronze,
            TABLE,
            std::slice::from_ref(&element),
            Some(2),
            ordinary()
        )
        .tes_hundredths,
        None
    );
    assert_eq!(
        score_program_with_execution(
            RULESET,
            AdultCategory::Bronze,
            TABLE,
            std::slice::from_ref(&element),
            None,
            ordinary(),
            &observations
        )
        .fall_deduction,
        Err(ProgramScoreError::UnknownFall)
    );
}

#[test]
fn execution_alignment_and_partial_errors_do_not_produce_exact_totals() {
    let element = ProgramElement {
        recognition: interrupted(),
        annotations: vec![],
        goe: -1,
        fall: Some(false),
        start_ms: Some(20_000),
    };
    let wrong_length = score_program_with_execution(
        RULESET,
        AdultCategory::Bronze,
        TABLE,
        std::slice::from_ref(&element),
        Some(0),
        ordinary(),
        &[],
    );
    assert_eq!(wrong_length.tes_hundredths, None);
    assert_eq!(
        wrong_length.elements,
        vec![Err(ProgramScoreError::InvalidInput)]
    );
    assert_eq!(
        wrong_length.fall_deduction,
        Err(ProgramScoreError::InvalidInput)
    );
    assert_eq!(
        score_planning_scenario_with_execution(
            RULESET,
            TABLE,
            std::slice::from_ref(&element),
            None,
            &[]
        ),
        Err(ProgramScoreError::InvalidInput)
    );
    let spin = ProgramElement {
        recognition: unit(&["ChSp1"], vec![]),
        annotations: vec![],
        goe: 1,
        fall: Some(false),
        start_ms: Some(10_000),
    };
    let invalid = [
        evidence(vec![]),
        evidence(vec![JumpExecutionError::StepOut { component: 9 }]),
    ];
    let elements = [spin, element];
    let partial = score_program_with_execution(
        RULESET,
        AdultCategory::Bronze,
        TABLE,
        &elements,
        Some(0),
        ordinary(),
        &invalid,
    );
    assert_eq!(partial.tes_hundredths, None);
    assert_eq!(partial.elements[0].as_ref().unwrap().score_hundredths, 400);
    assert_eq!(
        partial.elements[1],
        Err(ProgramScoreError::Element(
            ElementScoreError::UnsupportedMarks
        ))
    );
    let planned =
        score_planning_scenario_with_execution(RULESET, TABLE, &elements, None, &invalid).unwrap();
    assert_eq!(planned.hypothetical_hundredths, None);
    assert_eq!(planned.elements[0].as_ref().unwrap().score_hundredths, 400);
}

#[test]
fn unknown_execution_is_compatible_and_explicit_no_errors_resolves_unknown_fall() {
    let element = ProgramElement {
        recognition: unit(&["1Lz"], vec![]),
        annotations: vec![],
        goe: 1,
        fall: Some(false),
        start_ms: Some(20_000),
    };
    let unknown = [ElementExecutionEvidence::default()];
    assert_eq!(
        score_program_with_execution(
            RULESET,
            AdultCategory::Bronze,
            TABLE,
            std::slice::from_ref(&element),
            Some(0),
            ordinary(),
            &unknown
        ),
        score_program(
            RULESET,
            AdultCategory::Bronze,
            TABLE,
            std::slice::from_ref(&element),
            Some(0),
            ordinary()
        )
    );
    assert_eq!(
        score_planning_scenario_with_execution(
            RULESET,
            TABLE,
            std::slice::from_ref(&element),
            None,
            &unknown
        ),
        score_planning_scenario(RULESET, TABLE, std::slice::from_ref(&element), None)
    );
    assert_eq!(
        score_element_with_execution(
            TABLE,
            &element.recognition,
            &[],
            1,
            Some(false),
            &unknown[0]
        ),
        score_element(TABLE, &element.recognition, &[], 1, Some(false))
    );
    assert_eq!(
        score_element(TABLE, &element.recognition, &[], 1, None),
        Err(ElementScoreError::UnknownFall)
    );
    assert_eq!(
        score_element_with_execution(TABLE, &element.recognition, &[], 1, None, &evidence(vec![]))
            .unwrap()
            .score_hundredths,
        66
    );
}
