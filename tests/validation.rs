use skating_program_lab::*;

fn program(category: AdultCategory, elements: Vec<PlannedElement>) -> Program {
    Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category,
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        sources: vec![],
        elements,
    }
}
fn jump(kind: Option<JumpKind>, rotations: Option<u8>) -> ListedJump {
    ListedJump { kind, rotations }
}
fn solo(kind: Option<JumpKind>, rotations: Option<u8>) -> PlannedElement {
    PlannedElement::Jump(JumpElement::Solo(jump(kind, rotations)))
}

#[test]
fn bronze_jump_boundaries_and_source() {
    // Adult announcement 2026-07-16, p. 11: singles except Axel; doubles forbidden.
    let result = validate_program(&program(
        AdultCategory::Bronze,
        vec![
            solo(Some(JumpKind::Lutz), Some(1)),
            solo(Some(JumpKind::Axel), Some(1)),
            solo(Some(JumpKind::ToeLoop), Some(2)),
        ],
    ));
    assert_eq!(result.findings.len(), 2);
    for (finding, index) in result.findings.iter().zip([1, 2]) {
        assert_eq!(finding.kind, FindingKind::Violation);
        assert_eq!(finding.element_index, Some(index));
        assert_eq!(finding.rule_id, "isu-adult-2026-27-bronze-jump-eligibility");
        assert_eq!(finding.source.version, "2026-07-16");
        assert_eq!(finding.source.location, "p. 11, Bronze Free Skating, jumps");
    }
    assert_eq!(result.coverage, Coverage::Partial);
}

#[test]
fn missing_fields_and_multiple_combination_findings_are_preserved() {
    let result = validate_program(&program(
        AdultCategory::Bronze,
        vec![
            solo(None, Some(1)),
            solo(Some(JumpKind::Lutz), None),
            solo(Some(JumpKind::Lutz), Some(0)),
            PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(Some(JumpKind::Axel), Some(1))),
                JumpPart::Euler,
                JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(2))),
            ])),
            PlannedElement::Jump(JumpElement::Sequence(vec![JumpPart::Jump(jump(
                None,
                Some(2),
            ))])),
        ],
    ));
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|f| f.kind == FindingKind::Unknown && f.rule_id.ends_with("jump-eligibility"))
            .count(),
        3
    );
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|f| f.kind == FindingKind::Violation
                && (f.rule_id.ends_with("jump-eligibility")
                    || f.rule_id.ends_with("jump-element-count")))
            .map(|f| f.element_index)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(3), Some(4), None]
    );
}

#[test]
fn bronze_counts_jump_elements_independently_of_local_findings() {
    // Adult announcement 2026-07-16, p. 11: a maximum of four jump elements.
    let single = || solo(Some(JumpKind::Lutz), Some(1));
    let combination = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(1))),
        JumpPart::Jump(jump(Some(JumpKind::Salchow), Some(1))),
    ]));
    let mut elements = vec![
        single(),
        solo(Some(JumpKind::Flip), Some(1)),
        solo(Some(JumpKind::Loop), Some(1)),
        combination,
    ];
    elements.push(PlannedElement::ChoreographicSequence {
        movements: Some(vec!["five jumps".into()]),
    });
    let count_rule = "isu-adult-2026-27-bronze-jump-element-count";
    assert!(
        !validate_program(&program(AdultCategory::Bronze, elements.clone()))
            .findings
            .iter()
            .any(|finding| finding.rule_id == count_rule)
    );
    elements.push(solo(Some(JumpKind::Axel), Some(1)));
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    assert!(result
        .findings
        .iter()
        .any(|finding| finding.element_index == Some(5)
            && finding.rule_id == "isu-adult-2026-27-bronze-jump-eligibility"));
    let counts: Vec<_> = result
        .findings
        .iter()
        .filter(|finding| finding.rule_id == count_rule)
        .collect();
    assert_eq!(counts.len(), 1);
    let count = counts[0];
    assert_eq!(count.kind, FindingKind::Violation);
    assert_eq!(count.rule_id, "isu-adult-2026-27-bronze-jump-element-count");
    assert_eq!(count.element_index, None);
    assert_eq!(count.component_index, None);
    assert_eq!(
        count.source.document,
        "International Adult Figure Skating Competitions Technical Announcement"
    );
    assert_eq!(count.source.version, "2026-07-16");
    assert_eq!(count.source.location, "p. 11, Bronze Free Skating, jumps");
    assert_eq!(result.coverage, Coverage::Partial);
    let silver = validate_program(&program(AdultCategory::Silver, vec![single(); 5]));
    assert!(silver
        .findings
        .iter()
        .any(|f| f.rule_id.ends_with("jump-element-count") && f.kind == FindingKind::Violation));
}

#[test]
fn five_eligible_jumps_produce_one_count_finding() {
    let elements = [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
    ]
    .into_iter()
    .map(|kind| solo(Some(kind), Some(1)))
    .collect();
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|finding| finding.rule_id == "isu-adult-2026-27-bronze-jump-element-count")
            .count(),
        1
    );
}

#[test]
fn sequence_occupies_one_jump_element_slot() {
    let sequence = PlannedElement::Jump(JumpElement::Sequence(vec![
        JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(1))),
        JumpPart::Jump(jump(Some(JumpKind::Salchow), Some(1))),
    ]));
    let elements = vec![
        solo(Some(JumpKind::Loop), Some(1)),
        solo(Some(JumpKind::Flip), Some(1)),
        solo(Some(JumpKind::Lutz), Some(1)),
        sequence,
    ];
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    assert!(!result
        .findings
        .iter()
        .any(|finding| finding.rule_id == "isu-adult-2026-27-bronze-jump-element-count"));
}

#[test]
fn unsupported_category_and_season_are_explicit() {
    let mut other_season = program(
        AdultCategory::Bronze,
        vec![solo(Some(JumpKind::Axel), Some(1))],
    );
    other_season.season = Season {
        start_year: 2027,
        end_year: 2028,
    };
    assert_eq!(
        validate_program(&other_season).findings[0].kind,
        FindingKind::Unsupported
    );
}

#[test]
fn incomplete_axel_and_components_have_distinct_explainable_findings() {
    let result = validate_program(&program(
        AdultCategory::Bronze,
        vec![
            solo(Some(JumpKind::Axel), None),
            PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(Some(JumpKind::Axel), Some(1))),
                JumpPart::Euler,
                JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(2))),
            ])),
        ],
    ));
    let source = SourceReference {
        document: "International Adult Figure Skating Competitions Technical Announcement".into(),
        version: "2026-07-16".into(),
        location: "p. 11, Bronze Free Skating, jumps".into(),
    };
    let expected = |kind, element_index, component_index, explanation| Finding {
        kind,
        rule_id: "isu-adult-2026-27-bronze-jump-eligibility",
        element_index,
        component_index,
        explanation,
        source: source.clone(),
    };
    assert_eq!(
        result
            .findings
            .into_iter()
            .filter(|f| f.rule_id.ends_with("jump-eligibility"))
            .collect::<Vec<_>>(),
        vec![
            expected(
                FindingKind::Unknown,
                Some(0),
                None,
                "Jump kind or a positive rotation count is missing."
            ),
            expected(
                FindingKind::Violation,
                Some(0),
                None,
                "Axel jumps are prohibited in Bronze."
            ),
            expected(
                FindingKind::Violation,
                Some(1),
                Some(0),
                "Axel jumps are prohibited in Bronze."
            ),
            expected(
                FindingKind::Violation,
                Some(1),
                Some(2),
                "Multi-rotation jumps are prohibited in Bronze."
            ),
        ]
    );
}

#[test]
fn bronze_combination_and_repetition_boundaries() {
    let lutz = || JumpPart::Jump(jump(Some(JumpKind::Lutz), Some(1)));
    let toe = || JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(1)));
    let combo = |parts| PlannedElement::Jump(JumpElement::Combination(parts));
    let valid = vec![
        solo(Some(JumpKind::Lutz), Some(1)),
        combo(vec![lutz(), toe()]),
    ];
    assert!(
        validate_program(&program(AdultCategory::Bronze, valid.clone()))
            .findings
            .is_empty()
    );
    let mut invalid = valid;
    invalid.push(combo(vec![lutz(), toe(), JumpPart::Euler, toe()]));
    let result = validate_program(&program(AdultCategory::Bronze, invalid));
    for rule in [
        "combination-count",
        "combination-structure",
        "combination-euler",
        "jump-repetition",
    ] {
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::Violation
                    && f.rule_id.ends_with(rule)
                    && f.source.location == "p. 11, Bronze Free Skating, jumps"),
            "{rule}"
        );
    }
    assert!(result
        .findings
        .iter()
        .any(|f| f.rule_id.ends_with("jump-repetition")
            && f.element_index == Some(2)
            && f.component_index == Some(0)));
}

#[test]
fn incomplete_structures_and_repetitions_are_unknown_and_sequences_reported() {
    let result = validate_program(&program(
        AdultCategory::Bronze,
        vec![
            PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(None, Some(1))),
                JumpPart::UnspecifiedTransition,
            ])),
            PlannedElement::Jump(JumpElement::Sequence(vec![JumpPart::Jump(jump(
                Some(JumpKind::Lutz),
                Some(1),
            ))])),
            PlannedElement::Unspecified,
        ],
    ));
    for rule in [
        "combination-structure",
        "combination-euler",
        "jump-repetition",
        "combination-count",
    ] {
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::Unknown && f.rule_id.ends_with(rule)),
            "{rule}"
        );
    }
    assert!(result
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation
            && f.rule_id.ends_with("combination-structure")
            && f.element_index == Some(1)));
}

#[test]
fn combination_rules_have_independent_boundaries() {
    let part = |kind| JumpPart::Jump(jump(Some(kind), Some(1)));
    let combo = |parts| PlannedElement::Jump(JumpElement::Combination(parts));
    let pair = || vec![part(JumpKind::Lutz), part(JumpKind::ToeLoop)];
    let cases = [
        (vec![combo(pair())], None),
        (
            vec![combo(pair()), combo(pair())],
            Some("combination-count"),
        ),
        (
            vec![combo(vec![
                part(JumpKind::Lutz),
                part(JumpKind::ToeLoop),
                part(JumpKind::Loop),
            ])],
            Some("combination-structure"),
        ),
        (
            vec![combo(vec![
                part(JumpKind::Lutz),
                JumpPart::Euler,
                part(JumpKind::ToeLoop),
            ])],
            Some("combination-euler"),
        ),
    ];
    for (elements, expected) in cases {
        let result = validate_program(&program(AdultCategory::Bronze, elements));
        assert_eq!(
            result.findings.len(),
            usize::from(expected.is_some()),
            "{:?}",
            result.findings
        );
        if let Some(rule) = expected {
            assert!(result.findings[0].rule_id.ends_with(rule));
        }
    }
}

#[test]
fn incomplete_jump_does_not_make_known_combination_length_unknown() {
    let elements = vec![PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(jump(None, Some(1))),
        JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(1))),
    ]))];
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    assert_eq!(result.findings.len(), 2);
    assert!(result
        .findings
        .iter()
        .all(|f| f.kind == FindingKind::Unknown));
    assert!(result
        .findings
        .iter()
        .any(|f| f.rule_id.ends_with("jump-eligibility")));
    assert!(result
        .findings
        .iter()
        .any(|f| f.rule_id.ends_with("jump-repetition")));
}

#[test]
fn each_excess_repetition_has_a_location_for_each_jump_type() {
    for count in 2..=4 {
        let result = validate_program(&program(
            AdultCategory::Bronze,
            vec![solo(Some(JumpKind::Lutz), Some(1)); count],
        ));
        let indices: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.rule_id.ends_with("jump-repetition"))
            .map(|f| f.element_index)
            .collect();
        assert_eq!(indices, (2..count).map(Some).collect::<Vec<_>>());
    }
    let mut elements = vec![solo(Some(JumpKind::Lutz), Some(1)); 3];
    elements.extend(vec![solo(Some(JumpKind::Flip), Some(1)); 3]);
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    let indices: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.rule_id.ends_with("jump-repetition"))
        .map(|f| f.element_index)
        .collect();
    assert_eq!(indices, vec![Some(2), Some(5)]);
}

#[test]
fn category_jump_types_and_element_boundaries() {
    for (category, kind, rotations, forbidden) in [
        (AdultCategory::Bronze, JumpKind::Axel, 1, true),
        (AdultCategory::Silver, JumpKind::Axel, 1, false),
        (AdultCategory::Silver, JumpKind::ToeLoop, 2, true),
        (AdultCategory::Gold, JumpKind::ToeLoop, 2, false),
        (AdultCategory::Gold, JumpKind::Loop, 2, false),
        (AdultCategory::Gold, JumpKind::Flip, 2, true),
        (AdultCategory::Gold, JumpKind::Lutz, 2, true),
    ] {
        let result = validate_program(&program(category, vec![solo(Some(kind), Some(rotations))]));
        assert_eq!(
            result.findings.iter().any(
                |f| f.rule_id.ends_with("jump-eligibility") && f.kind == FindingKind::Violation
            ),
            forbidden
        );
    }
    for (category, limit) in [
        (AdultCategory::Bronze, 4),
        (AdultCategory::Silver, 4),
        (AdultCategory::Gold, 5),
        (AdultCategory::Masters, 5),
    ] {
        for count in [limit, limit + 1] {
            let result = validate_program(&program(
                category,
                vec![solo(Some(JumpKind::ToeLoop), Some(1)); count],
            ));
            assert_eq!(
                result
                    .findings
                    .iter()
                    .any(|f| f.rule_id.ends_with("jump-element-count")
                        && f.kind == FindingKind::Violation),
                count > limit
            );
        }
    }
    let result = validate_program(&program(AdultCategory::Masters, vec![]));
    assert!(result
        .findings
        .iter()
        .any(|f| f.rule_id.ends_with("axel-required") && f.kind == FindingKind::Violation));
}

#[test]
fn compound_limits_repetitions_and_uncertainty() {
    let part = |kind| JumpPart::Jump(jump(Some(kind), Some(1)));
    let combo = |parts| PlannedElement::Jump(JumpElement::Combination(parts));
    let seq = |parts| PlannedElement::Jump(JumpElement::Sequence(parts));
    let result = validate_program(&program(
        AdultCategory::Silver,
        vec![
            solo(Some(JumpKind::Lutz), Some(1)),
            combo(vec![
                part(JumpKind::Lutz),
                part(JumpKind::ToeLoop),
                part(JumpKind::Loop),
            ]),
            seq(vec![
                part(JumpKind::Lutz),
                part(JumpKind::Axel),
                part(JumpKind::ToeLoop),
            ]),
        ],
    ));
    for rule in ["jump-repetition", "combination-structure"] {
        assert!(result
            .findings
            .iter()
            .any(|f| f.rule_id.ends_with(rule) && f.kind == FindingKind::Violation));
    }
    assert!(result
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Unknown && f.explanation.contains("direct step")));
    let result = validate_program(&program(
        AdultCategory::Gold,
        vec![
            combo(vec![
                part(JumpKind::Lutz),
                JumpPart::Euler,
                part(JumpKind::Salchow),
            ]),
            seq(vec![part(JumpKind::ToeLoop), part(JumpKind::Axel)]),
        ],
    ));
    assert!(result
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation && f.explanation.contains("must not use Euler")));
    assert!(result
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Unknown && f.explanation.contains("foot transition")));
    let result = validate_program(&program(
        AdultCategory::Silver,
        vec![combo(vec![part(JumpKind::Lutz), part(JumpKind::Flip)])],
    ));
    assert!(result
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Unknown
            && f.explanation.contains("verified combination subset")));
}

#[test]
fn explicit_transitions_are_unknown_with_sports_rule_source_in_all_categories() {
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let result = validate_program(&program(
            category,
            vec![PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(jump(Some(JumpKind::Lutz), Some(1))),
                JumpPart::OtherTransition("three-turn".into()),
                JumpPart::Jump(jump(Some(JumpKind::ToeLoop), Some(1))),
            ]))],
        ));
        let finding = result
            .findings
            .iter()
            .find(|f| f.rule_id == "isu-sports-2026-jump-transition")
            .unwrap();
        assert_eq!(finding.kind, FindingKind::Unknown);
        assert_eq!(finding.component_index, Some(1));
        assert_eq!(finding.source.version, "2026-06-12");
        assert!(finding.source.location.contains("Rule 610"));
    }
}

#[test]
fn masters_axel_requirement_distinguishes_missing_type_from_rotation() {
    for (kind, rotations, expected) in [
        (Some(JumpKind::ToeLoop), None, FindingKind::Violation),
        (None, Some(1), FindingKind::Unknown),
        (Some(JumpKind::Axel), None, FindingKind::Unknown),
    ] {
        let result = validate_program(&program(
            AdultCategory::Masters,
            vec![solo(kind, rotations)],
        ));
        assert_eq!(
            result
                .findings
                .iter()
                .find(|f| f.rule_id.ends_with("axel-required"))
                .unwrap()
                .kind,
            expected
        );
    }
}

#[test]
fn missing_and_invalid_rotations_and_unspecified_element_limits_are_distinct() {
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        for (rotations, expected) in [
            (None, FindingKind::Unknown),
            (Some(0), FindingKind::Violation),
        ] {
            let result = validate_program(&program(
                category,
                vec![solo(Some(JumpKind::ToeLoop), rotations)],
            ));
            let rule = if rotations.is_none() {
                "jump-eligibility"
            } else {
                "invalid-rotations"
            };
            assert!(result
                .findings
                .iter()
                .any(|f| f.rule_id.ends_with(rule) && f.kind == expected));
        }
        let limit = if matches!(category, AdultCategory::Bronze | AdultCategory::Silver) {
            4
        } else {
            5
        };
        let mut elements = vec![solo(Some(JumpKind::ToeLoop), Some(1)); limit];
        elements.push(PlannedElement::Unspecified);
        let result = validate_program(&program(category, elements));
        assert!(result
            .findings
            .iter()
            .any(|f| f.rule_id.ends_with("jump-element-count") && f.kind == FindingKind::Unknown));
    }
}
