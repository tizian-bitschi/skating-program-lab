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
        scoring_table: ScoringTableId::Isu2786June2_2026,
        sources: vec![
            SourceReference {
                document: "International Adult Figure Skating Competitions Technical Announcement"
                    .into(),
                version: "2026-07-16".into(),
                location: "p. 11, Bronze Free Skating".into(),
            },
            SourceReference {
                document: "ISU Communication 2786, Scale of Values".into(),
                version: "2026-06-02".into(),
                location: "pp. 2, 8, 14".into(),
            },
        ],
        elements: vec![
            PlannedElement::Jump(JumpElement::Solo(jump(JumpKind::Lutz))),
            PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(JumpKind::Salchow)),
                JumpPart::Jump(jump(JumpKind::ToeLoop)),
            ])),
            PlannedElement::RegularSpin(RegularSpin {
                entry: Some(SpinEntry::NonFlying),
                steps: Some(vec![
                    SpinStep::Position {
                        position: Some(SpinPosition::Sit),
                        revolutions: Some(3),
                        feature_attempts: Some(vec!["difficult variation".into()]),
                    },
                    SpinStep::ChangeFoot,
                    SpinStep::Position {
                        position: Some(SpinPosition::Sit),
                        revolutions: Some(3),
                        feature_attempts: Some(vec![]),
                    },
                ]),
            }),
            PlannedElement::ChoreographicSequence {
                movements: Some(vec!["spiral".into(), "spread eagle".into()]),
            },
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
    assert_eq!(copy.scoring_table, ScoringTableId::Isu2786June2_2026);
    assert_eq!(copy.sources, program.sources);
    assert_eq!(copy.sources[0].location, "p. 11, Bronze Free Skating");
    assert_eq!(copy.sources[1].version, "2026-06-02");
    assert_eq!(
        copy.elements[3],
        PlannedElement::ChoreographicSequence {
            movements: Some(vec!["spiral".into(), "spread eagle".into()]),
        }
    );
    assert_eq!(copy.elements, program.elements);
    assert!(
        matches!(&copy.elements[1], PlannedElement::Jump(JumpElement::Combination(parts))
        if parts == &vec![JumpPart::Jump(jump(JumpKind::Salchow)), JumpPart::Jump(jump(JumpKind::ToeLoop))])
    );
    assert_eq!(
        copy.elements[2],
        PlannedElement::RegularSpin(RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: Some(vec![
                SpinStep::Position {
                    position: Some(SpinPosition::Sit),
                    revolutions: Some(3),
                    feature_attempts: Some(vec!["difficult variation".into()])
                },
                SpinStep::ChangeFoot,
                SpinStep::Position {
                    position: Some(SpinPosition::Sit),
                    revolutions: Some(3),
                    feature_attempts: Some(vec![])
                },
            ]),
        })
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
            entry: None,
            steps: None,
        },
        RegularSpin {
            entry: None,
            steps: Some(vec![]),
        }
    );
    assert_ne!(
        RegularSpin {
            entry: Some(SpinEntry::Flying),
            steps: None
        },
        RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: None
        }
    );
    assert_ne!(
        PlannedElement::ChoreographicSpin,
        PlannedElement::ChoreographicSequence { movements: None }
    );
    let position = |feature_attempts| SpinStep::Position {
        position: Some(SpinPosition::Sit),
        revolutions: Some(3),
        feature_attempts,
    };
    assert_ne!(position(None), position(Some(vec![])));
    assert_ne!(
        vec![
            position(Some(vec!["variation".into()])),
            SpinStep::ChangeFoot,
            position(Some(vec![]))
        ],
        vec![
            position(Some(vec![])),
            SpinStep::ChangeFoot,
            position(Some(vec!["variation".into()]))
        ]
    );
    assert_ne!(
        PlannedElement::StepSequence,
        PlannedElement::ChoreographicSequence { movements: None }
    );
}
