use skating_program_lab::*;

fn jump(kind: JumpKind) -> ListedJump {
    ListedJump {
        kind: Some(kind),
        rotations: Some(1),
    }
}

#[test]
fn bronze_plan_preserves_order_and_details() {
    // Adult announcement 2026-07-16, p. 11: Bronze singles and a two-jump combination.
    // This is a constructed plan illustrating those provisions, not a published program.
    let program = Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: AdultCategory::Bronze,
        ruleset: RulesetId::IsuAdult2026_27July16,
        elements: vec![
            PlannedElement::Jump(JumpElement::Solo(jump(JumpKind::Lutz))),
            PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(JumpKind::Salchow)),
                JumpPart::Jump(jump(JumpKind::ToeLoop)),
            ])),
            PlannedElement::RegularSpin(RegularSpin {
                steps: Some(vec![
                    SpinStep::Position {
                        position: Some(SpinPosition::Sit),
                        revolutions: Some(3),
                    },
                    SpinStep::ChangeFoot,
                    SpinStep::Position {
                        position: Some(SpinPosition::Sit),
                        revolutions: Some(3),
                    },
                ]),
                feature_attempts: Some(vec!["difficult variation".into()]),
            }),
            PlannedElement::ChoreographicSequence,
        ],
    };
    let copy = program.clone();
    assert_eq!(
        copy.season,
        Season {
            start_year: 2026,
            end_year: 2027
        }
    );
    assert_eq!(copy.category, AdultCategory::Bronze);
    assert_eq!(copy.ruleset, RulesetId::IsuAdult2026_27July16);
    assert_eq!(copy.elements, program.elements);
    assert!(
        matches!(&copy.elements[1], PlannedElement::Jump(JumpElement::Combination(parts))
        if parts == &vec![JumpPart::Jump(jump(JumpKind::Salchow)), JumpPart::Jump(jump(JumpKind::ToeLoop))])
    );
    assert!(
        matches!(&copy.elements[2], PlannedElement::RegularSpin(spin)
        if spin.steps.as_ref().unwrap().len() == 3 && spin.feature_attempts.as_ref().unwrap() == &["difficult variation"])
    );
}

#[test]
fn transitions_and_unspecified_details_remain_distinct() {
    // Adult announcement pp. 8–10: Euler is not a listed jump. No legality is inferred.
    let parts = vec![
        JumpPart::Jump(jump(JumpKind::Flip)),
        JumpPart::Euler,
        JumpPart::Jump(ListedJump {
            kind: None,
            rotations: None,
        }),
    ];
    let combination = JumpElement::Combination(parts.clone());
    let sequence = JumpElement::Sequence(parts.clone());
    assert_ne!(combination, sequence);
    assert_eq!(
        parts
            .iter()
            .filter(|part| matches!(part, JumpPart::Jump(_)))
            .count(),
        2
    );
    assert_ne!(
        RegularSpin {
            steps: None,
            feature_attempts: None
        },
        RegularSpin {
            steps: Some(vec![]),
            feature_attempts: Some(vec![])
        }
    );
    assert_ne!(
        PlannedElement::StepSequence,
        PlannedElement::ChoreographicSequence
    );
}
