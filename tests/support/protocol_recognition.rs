use skating_program_lab::*;

// Fixture adapter only: production callers supply typed recognition, not strings.
pub fn recognition(code: &str) -> PanelRecognition {
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
