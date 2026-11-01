use skating_program_lab::*;

fn code(value: &str) -> ComponentCode {
    ComponentCode::try_from(value).unwrap()
}
fn call(target: CallTarget, call: PanelCall) -> RecordedCall {
    RecordedCall { target, call }
}

#[test]
fn observed_fall_and_supplied_calls_do_not_change_the_plan() {
    let planned = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Lutz),
        rotations: Some(1),
    }));
    let observed = ObservedElement {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: Some(0),
        movement: Some("Lutz".into()),
        attempted_rotations: Some(1),
        fall: Some(true),
    };
    let details = RecognitionDetails::new(
        vec![code("1Lz")],
        None,
        Some(vec![call(
            CallTarget::Component(0),
            PanelCall::UnderRotated,
        )]),
    )
    .unwrap();
    let recognition = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: Some(0),
        observed_element: Some(0),
        outcome: RecognitionOutcome::Recognized,
        details: Some(details),
    };
    assert_eq!(
        recognition.details.as_ref().unwrap().notation().as_deref(),
        Some("1Lz<")
    );
    assert_eq!(recognition.planned_element, observed.planned_element);
    assert_eq!(recognition.observed_element, Some(0));
    assert_eq!(observed.fall, Some(true));
    let unknown = ObservedElement {
        ruleset: observed.ruleset,
        source: None,
        planned_element: None,
        movement: None,
        attempted_rotations: None,
        fall: None,
    };
    assert_ne!(observed, unknown);
    assert_ne!(unknown.fall, Some(false));
    assert_eq!(
        planned,
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1),
        }))
    );
}

#[test]
fn structure_markers_have_one_valid_position_and_kind() {
    let components = vec![code("3F"), code("2T")];
    assert_eq!(
        RecognitionDetails::new(
            components.clone(),
            None,
            Some(vec![call(
                CallTarget::BetweenComponents(0),
                PanelCall::Sequence
            )])
        ),
        Err(RecognitionError::InvalidTarget)
    );
    assert_eq!(
        RecognitionDetails::new(
            components.clone(),
            None,
            Some(vec![call(CallTarget::Component(0), PanelCall::Combination)])
        ),
        Err(RecognitionError::InvalidTarget)
    );
    assert_eq!(
        RecognitionDetails::new(
            components.clone(),
            None,
            Some(vec![call(CallTarget::Element, PanelCall::Sequence)])
        ),
        Err(RecognitionError::InvalidTarget)
    );
    assert_eq!(
        RecognitionDetails::new(
            components.clone(),
            None,
            Some(vec![
                call(CallTarget::BetweenComponents(1), PanelCall::Combination),
                call(CallTarget::BetweenComponents(1), PanelCall::Sequence)
            ])
        ),
        Err(RecognitionError::ConflictingCalls)
    );
    let final_marker = RecognitionDetails::new(
        components,
        None,
        Some(vec![call(
            CallTarget::BetweenComponents(2),
            PanelCall::Sequence,
        )]),
    )
    .unwrap();
    assert_eq!(final_marker.notation().as_deref(), Some("3F+2T+SEQ"));
}

#[test]
fn notation_orders_marks_independently_of_input_order_and_single_element_alias() {
    // ISU Q&A 3, p. 1, Q2: an under-rotation mark precedes the asterisk.
    let input_orders = [
        vec![
            call(CallTarget::Component(0), PanelCall::NotCounted),
            call(CallTarget::Component(0), PanelCall::UnderRotated),
        ],
        vec![
            call(CallTarget::Component(0), PanelCall::UnderRotated),
            call(CallTarget::Component(0), PanelCall::NotCounted),
        ],
        vec![
            call(CallTarget::Element, PanelCall::UnderRotated),
            call(CallTarget::Component(0), PanelCall::NotCounted),
        ],
        vec![
            call(CallTarget::Element, PanelCall::NotCounted),
            call(CallTarget::Component(0), PanelCall::UnderRotated),
        ],
    ];
    for calls in input_orders {
        let details = RecognitionDetails::new(vec![code("2T")], None, Some(calls)).unwrap();
        assert_eq!(details.notation().as_deref(), Some("2T<*"));
        assert!(details
            .calls()
            .unwrap()
            .iter()
            .all(|entry| entry.target == CallTarget::Component(0)));
    }
    // Communication 2786 p. 2: 1Lze< uses edge before under-rotation.
    let details = RecognitionDetails::new(
        vec![code("1Lz")],
        None,
        Some(vec![
            call(CallTarget::Component(0), PanelCall::UnderRotated),
            call(CallTarget::Element, PanelCall::WrongEdge),
        ]),
    )
    .unwrap();
    assert_eq!(details.notation().as_deref(), Some("1Lze<"));
}

#[test]
fn recognition_retains_source_and_all_component_marks() {
    // ISU Q&A 3, October 2026, p. 1, Q2: 3S+SEQ+2T<*+2Aq*.
    let calls = vec![
        call(CallTarget::BetweenComponents(1), PanelCall::Sequence),
        call(CallTarget::Component(1), PanelCall::UnderRotated),
        call(CallTarget::Component(1), PanelCall::NotCounted),
        call(CallTarget::Component(2), PanelCall::Quarter),
        call(CallTarget::Component(2), PanelCall::NotCounted),
    ];
    let details = RecognitionDetails::new(
        vec![code("3S"), code("2T"), code("2A")],
        None,
        Some(calls.clone()),
    )
    .unwrap();
    let panel = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: Some(SourceReference {
            document: "ISU Technical Committee Q&A 3".into(),
            version: "2026-10".into(),
            location: "p. 1, Q2".into(),
        }),
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(details),
    };
    let detail = panel.details.unwrap();
    assert_eq!(detail.components(), &[code("3S"), code("2T"), code("2A")]);
    assert_eq!(detail.calls(), Some(calls.as_slice()));
    assert_eq!(detail.notation().as_deref(), Some("3S+SEQ+2T<*+2Aq*"));
    assert_eq!(panel.source.unwrap().location, "p. 1, Q2");
}

#[test]
fn notation_preserves_spin_levels_combination_and_unknown_calls() {
    // Adult announcement p. 11 and Communication 2786 p. 8.
    let spin = RecognitionDetails::new(
        vec![code("CSSp")],
        Some(Level::Basic),
        Some(vec![call(CallTarget::Element, PanelCall::ReducedSpinValue)]),
    )
    .unwrap();
    assert_eq!(spin.level(), Some(Level::Basic));
    assert_eq!(spin.notation().as_deref(), Some("CSSpBV"));
    let combo = RecognitionDetails::new(
        vec![code("1Lz"), code("1T")],
        None,
        Some(vec![
            call(CallTarget::BetweenComponents(1), PanelCall::Combination),
            call(CallTarget::Component(1), PanelCall::NotCounted),
        ]),
    )
    .unwrap();
    assert_eq!(combo.notation().as_deref(), Some("1Lz+COMBO+1T*"));
    let unknown = RecognitionDetails::new(vec![code("1A")], None, None).unwrap();
    let empty = RecognitionDetails::new(vec![code("1A")], None, Some(vec![])).unwrap();
    assert_eq!(unknown.notation(), None);
    assert_eq!(empty.notation().as_deref(), Some("1A"));
}

#[test]
fn recognition_rejects_conflicting_or_unrepresentable_input() {
    assert!(ComponentCode::try_from("1A<<").is_err());
    assert!(ComponentCode::try_from("CSSp1V").is_err());
    assert_eq!(
        RecognitionDetails::new(vec![], None, None),
        Err(RecognitionError::EmptyComponents)
    );
    assert_eq!(
        RecognitionDetails::new(
            vec![code("1A")],
            None,
            Some(vec![call(CallTarget::Component(1), PanelCall::Quarter)])
        ),
        Err(RecognitionError::InvalidTarget)
    );
    assert_eq!(
        RecognitionDetails::new(
            vec![code("1A")],
            None,
            Some(vec![
                call(CallTarget::Component(0), PanelCall::Quarter),
                call(CallTarget::Component(0), PanelCall::Downgraded)
            ])
        ),
        Err(RecognitionError::ConflictingCalls)
    );
    assert_eq!(
        RecognitionDetails::new(
            vec![code("1Lz")],
            None,
            Some(vec![
                call(CallTarget::Component(0), PanelCall::WrongEdge),
                call(CallTarget::Component(0), PanelCall::UnclearEdge)
            ])
        ),
        Err(RecognitionError::ConflictingCalls)
    );
    assert_eq!(
        RecognitionDetails::new(
            vec![code("1A")],
            None,
            Some(vec![
                call(CallTarget::Element, PanelCall::Quarter),
                call(CallTarget::Component(0), PanelCall::Downgraded)
            ])
        ),
        Err(RecognitionError::ConflictingCalls)
    );
    assert_eq!(
        RecognitionDetails::new(vec![code("1A")], Some(Level::One), None),
        Err(RecognitionError::InvalidTarget)
    );
}

#[test]
fn level_and_scoring_annotation_are_distinct() {
    assert_eq!(Level::try_from(1), Ok(Level::One));
    assert_eq!(Level::try_from(4), Ok(Level::Four));
    assert_eq!(Level::try_from(0), Err(0));
    assert_eq!(Level::try_from(5), Err(5));
    let annotation = ScoringAnnotations {
        recognized_element: 0,
        annotations: vec![ScoringAnnotation::SecondHalfCredit],
    };
    assert_eq!(
        annotation.annotations,
        vec![ScoringAnnotation::SecondHalfCredit]
    );
}

#[test]
fn ignored_and_recognized_zero_value_remain_distinct() {
    // Adult announcement pp. 9-11: Bronze ignores a Waltz jump, while Silver
    // and Gold call it as 1A<<, which has zero base value in 2786 p. 3.
    let mut panel = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Ignored,
        details: None,
    };
    let ignored = panel.clone();
    panel.outcome = RecognitionOutcome::Recognized;
    panel.details = Some(
        RecognitionDetails::new(
            vec![code("1A")],
            None,
            Some(vec![call(CallTarget::Component(0), PanelCall::Downgraded)]),
        )
        .unwrap(),
    );
    assert_ne!(panel, ignored);
    assert_eq!(panel.details.unwrap().notation().as_deref(), Some("1A<<"));
    assert_ne!(RecognitionOutcome::NoValue, RecognitionOutcome::Ignored);
    assert_ne!(RecognitionOutcome::Unknown, RecognitionOutcome::Ignored);
}
