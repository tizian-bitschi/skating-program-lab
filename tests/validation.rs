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
            .filter(|f| f.kind == FindingKind::Unknown)
            .count(),
        4
    );
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|f| f.kind == FindingKind::Violation)
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
    let mut elements = vec![single(), single(), single(), combination];
    elements.push(PlannedElement::ChoreographicSequence {
        movements: Some(vec!["five jumps".into()]),
    });
    assert!(
        validate_program(&program(AdultCategory::Bronze, elements.clone()))
            .findings
            .is_empty()
    );
    elements.push(solo(Some(JumpKind::Axel), Some(1)));
    let result = validate_program(&program(AdultCategory::Bronze, elements));
    assert_eq!(result.findings.len(), 2);
    assert_eq!(result.findings[0].element_index, Some(5));
    assert_eq!(
        result.findings[0].rule_id,
        "isu-adult-2026-27-bronze-jump-eligibility"
    );
    let count = &result.findings[1];
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
    let unsupported = validate_program(&program(AdultCategory::Silver, vec![single(); 5]));
    assert_eq!(unsupported.findings.len(), 1);
    assert_eq!(unsupported.findings[0].kind, FindingKind::Unsupported);
}

#[test]
fn unsupported_category_and_season_are_explicit() {
    for category in [
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let result = validate_program(&program(
            category,
            vec![solo(Some(JumpKind::Axel), Some(1))],
        ));
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].kind, FindingKind::Unsupported);
        assert_eq!(result.findings[0].rule_id, "validator-scope");
        assert_eq!(
            result.findings[0].source,
            SourceReference {
                document: "Skating Program Lab architecture".into(),
                version: "issue-13".into(),
                location: "Current state, validator scope".into()
            }
        );
        assert_eq!(result.coverage, Coverage::Partial);
    }
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
        result.findings,
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
