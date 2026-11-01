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
        calls: Some(vec![PanelCall::UnderRotated]),
    };
    assert_eq!(planned, original);
    assert_eq!(observed.fall, Some(true));
    assert_eq!(recognition.calls, Some(vec![PanelCall::UnderRotated]));
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
        calls: None,
    };
    let unknown_panel = panel.clone();
    panel.outcome = RecognitionOutcome::Ignored;
    assert_ne!(panel, unknown_panel);
    panel.outcome = RecognitionOutcome::NoValue;
    assert_ne!(panel.outcome, RecognitionOutcome::Ignored);
    panel.outcome = RecognitionOutcome::Recognized;
    panel.code = Some("1A".into());
    panel.calls = Some(vec![PanelCall::Downgraded]);
    assert_ne!(panel.outcome, RecognitionOutcome::NoValue);
    assert_eq!(panel.calls, Some(vec![PanelCall::Downgraded]));
    panel.calls = Some(vec![]);
    assert_ne!(panel.calls, unknown_panel.calls);
    assert_ne!(Some(false), unknown.fall);
}
