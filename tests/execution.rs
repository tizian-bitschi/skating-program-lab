use skating_program_lab::*;

#[test]
fn observed_fall_and_supplied_calls_do_not_change_the_plan() {
    // Adult announcement pp. 9–11 and Communication 2786 p. 2 distinguish
    // the attempted movement from the supplied under-rotation call.
    let planned = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Lutz),
        rotations: Some(1),
    }));
    let original = planned.clone();
    let observed = ObservedElement {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: Some(0),
        movement: Some("Lutz".into()),
        attempted_rotations: Some(1),
        fall: Some(true),
    };
    let recognition = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: Some(0),
        observed_element: Some(0),
        outcome: RecognitionOutcome::Recognized,
        code: Some("1Lz".into()),
        level: None,
        components: None,
        calls: Some(vec![RecordedCall {
            target: CallTarget::Element,
            call: PanelCall::UnderRotated,
        }]),
    };
    assert_eq!(planned, original);
    assert_eq!(observed.fall, Some(true));
    assert_eq!(
        recognition.calls,
        Some(vec![RecordedCall {
            target: CallTarget::Element,
            call: PanelCall::UnderRotated
        }])
    );
    assert_eq!(recognition.planned_element, observed.planned_element);
    assert_eq!(recognition.observed_element, Some(0));
}

#[test]
fn unknown_and_explicit_outcomes_remain_distinct() {
    // Adult announcement pp. 9–11: an ignored Bronze Waltz jump differs
    // from a recognized zero-value Silver/Gold 1A<<. No decision is inferred.
    let unknown = ObservedElement {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: None,
        movement: None,
        attempted_rotations: None,
        fall: None,
    };
    assert_eq!(unknown.fall, None);
    let mut panel = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: Some(SourceReference {
            document: "Adult announcement".into(),
            version: "2026-07-16".into(),
            location: "pp. 9–11".into(),
        }),
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Unknown,
        code: None,
        level: None,
        components: None,
        calls: None,
    };
    let unknown_panel = panel.clone();
    panel.outcome = RecognitionOutcome::Ignored;
    assert_ne!(panel, unknown_panel);
    panel.outcome = RecognitionOutcome::NoValue;
    assert_ne!(panel.outcome, RecognitionOutcome::Ignored);
    panel.outcome = RecognitionOutcome::Recognized;
    panel.code = Some("1A".into());
    panel.calls = Some(vec![RecordedCall {
        target: CallTarget::Element,
        call: PanelCall::Downgraded,
    }]);
    assert_ne!(panel.outcome, RecognitionOutcome::NoValue);
    assert_eq!(
        panel.calls,
        Some(vec![RecordedCall {
            target: CallTarget::Element,
            call: PanelCall::Downgraded
        }])
    );
    panel.calls = Some(vec![]);
    assert_ne!(panel.calls, unknown_panel.calls);
    assert_ne!(Some(false), unknown.fall);
}

#[test]
fn marks_retain_their_component_and_element_targets() {
    // ISU Q&A 3, October 2026, Q2: 3S+SEQ+2T<*+2Aq*.
    let recognition = PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: Some(SourceReference {
            document: "ISU Technical Committee Q&A 3".into(),
            version: "2026-10".into(),
            location: "p. 1, Q2".into(),
        }),
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        code: None,
        level: None,
        components: Some(vec!["3S".into(), "2T".into(), "2A".into()]),
        calls: Some(vec![
            RecordedCall {
                target: CallTarget::BetweenComponents(1),
                call: PanelCall::Sequence,
            },
            RecordedCall {
                target: CallTarget::Component(1),
                call: PanelCall::UnderRotated,
            },
            RecordedCall {
                target: CallTarget::Component(1),
                call: PanelCall::NotCounted,
            },
            RecordedCall {
                target: CallTarget::Component(2),
                call: PanelCall::Quarter,
            },
            RecordedCall {
                target: CallTarget::Component(2),
                call: PanelCall::NotCounted,
            },
        ]),
    };
    assert_eq!(
        recognition.calls.as_ref().unwrap()[1].target,
        CallTarget::Component(1)
    );
    assert_ne!(
        recognition.calls.as_ref().unwrap()[1].target,
        recognition.calls.as_ref().unwrap()[3].target
    );

    // Adult announcement p. 11: V on a change-foot spin with too few revolutions.
    let spin_mark = RecordedCall {
        target: CallTarget::Element,
        call: PanelCall::ReducedSpinValue,
    };
    assert_ne!(spin_mark.call, PanelCall::NotCounted);
    // Handbook p. 33: +REP applies to the jump element, separately from *.
    let repetition = RecordedCall {
        target: CallTarget::Element,
        call: PanelCall::Repetition,
    };
    assert_ne!(repetition.call, spin_mark.call);
}

#[test]
fn level_and_scoring_annotation_are_distinct() {
    assert_eq!(Level::try_from(1), Ok(Level::One));
    assert_eq!(Level::try_from(4), Ok(Level::Four));
    assert_eq!(Level::try_from(0), Err(0));
    assert_eq!(Level::try_from(5), Err(5));
    assert_ne!(Some(Level::Basic), None);

    let annotation = ScoringAnnotations {
        recognized_element: 0,
        annotations: vec![ScoringAnnotation::SecondHalfCredit],
    };
    assert_eq!(
        annotation.annotations,
        vec![ScoringAnnotation::SecondHalfCredit]
    );
}
