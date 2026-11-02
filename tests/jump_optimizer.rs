#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::time::Duration;

fn jump(kind: JumpKind, rotations: u8) -> ListedJump {
    ListedJump {
        kind: Some(kind),
        rotations: Some(rotations),
    }
}
fn solo(jump: &ListedJump) -> JumpElement {
    JumpElement::Solo(jump.clone())
}
fn combo(first: &ListedJump, second: &ListedJump) -> JumpElement {
    JumpElement::Combination(vec![
        JumpPart::Jump(first.clone()),
        JumpPart::Jump(second.clone()),
    ])
}
fn request(category: AdultCategory, jumps: Vec<ListedJump>) -> JumpOptimizerRequest {
    JumpOptimizerRequest {
        category,
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        allowed_jumps: jumps,
        combinations: vec![],
        exclusions: vec![],
        excluded_kinds: vec![],
        max_elements: 2,
        require_nonempty: false,
        max_search_nodes: 1000,
        enumerate_equal_optima: true,
        max_solutions: 20,
        time_limit: None,
    }
}
fn sequences(result: &JumpOptimizerResult) -> Vec<Vec<JumpElement>> {
    result
        .candidates
        .iter()
        .map(|candidate| {
            candidate
                .program
                .elements
                .iter()
                .map(|e| {
                    if let PlannedElement::Jump(j) = e {
                        j.clone()
                    } else {
                        panic!("not a jump")
                    }
                })
                .collect()
        })
        .collect()
}

#[test]
fn independently_enumerated_small_domains_match_for_every_category() {
    // Communication 2786, p. 2: 1T=0.40, 1S=0.40, 1A=1.10, 2Lo=1.70.
    let t = jump(JumpKind::ToeLoop, 1);
    let s = jump(JumpKind::Salchow, 1);
    let a = jump(JumpKind::Axel, 1);
    let lo = jump(JumpKind::Loop, 2);
    for category in [AdultCategory::Bronze, AdultCategory::Silver] {
        let result = optimize_jumps(&request(category, vec![t.clone(), s.clone()])).unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(result.enumeration_complete);
        assert_eq!(result.feasible_subsections, 7);
        assert!(result.candidates.iter().all(|c| c.base_hundredths == 80));
        let found = sequences(&result);
        assert_eq!(found.len(), 4);
        for first in [&t, &s] {
            for second in [&t, &s] {
                assert!(found.contains(&vec![solo(first), solo(second)]));
            }
        }
    }
    let silver =
        optimize_jumps(&request(AdultCategory::Silver, vec![a.clone(), lo.clone()])).unwrap();
    assert_eq!(silver.feasible_subsections, 3);
    assert_eq!(silver.candidates[0].base_hundredths, 220);
    assert_eq!(sequences(&silver), vec![vec![solo(&a), solo(&a)]]);
    let gold = optimize_jumps(&request(AdultCategory::Gold, vec![a.clone(), lo.clone()])).unwrap();
    assert_eq!(gold.feasible_subsections, 7);
    assert_eq!(gold.candidates[0].base_hundredths, 340);
    assert_eq!(sequences(&gold), vec![vec![solo(&lo), solo(&lo)]]);
    let masters = optimize_jumps(&request(
        AdultCategory::Masters,
        vec![a.clone(), lo.clone()],
    ))
    .unwrap();
    assert_eq!(masters.feasible_subsections, 4);
    assert!(masters.enumeration_complete);
    assert_eq!(masters.candidates.len(), 2);
    assert!(masters.candidates.iter().all(|c| c.base_hundredths == 280));
    assert!(sequences(&masters).contains(&vec![solo(&a), solo(&lo)]));
    assert!(sequences(&masters).contains(&vec![solo(&lo), solo(&a)]));
}

#[test]
fn exclusions_remove_solos_and_compounds_and_masters_requires_axel() {
    let lz = jump(JumpKind::Lutz, 1);
    let t = jump(JumpKind::ToeLoop, 1);
    let a = jump(JumpKind::Axel, 1);
    for excluded in [&lz, &t] {
        let mut req = request(AdultCategory::Gold, vec![lz.clone(), t.clone(), a.clone()]);
        req.combinations = vec![combo(&lz, &t)];
        req.exclusions = vec![excluded.clone()];
        let result = optimize_jumps(&req).unwrap();
        assert!(!result.universe.contains(&solo(excluded)));
        assert!(!result.universe.contains(&combo(&lz, &t)));
        assert!(result.candidates.iter().flat_map(|c| &c.program.elements).all(|e| {
            !matches!(e, PlannedElement::Jump(j) if j == &solo(excluded) || j == &combo(&lz, &t))
        }));
    }
    let mut req = request(AdultCategory::Masters, vec![a.clone(), t]);
    req.excluded_kinds.push(JumpKind::Axel);
    let result = optimize_jumps(&req).unwrap();
    assert_eq!(result.status, OptimizerStatus::InfeasibleProven);
    assert!(result.candidates.is_empty());
}

#[test]
fn combinations_count_as_one_and_components_count_for_repetition() {
    let lz = jump(JumpKind::Lutz, 1);
    let t = jump(JumpKind::ToeLoop, 1);
    let mut req = request(AdultCategory::Bronze, vec![lz.clone(), t.clone()]);
    req.combinations.push(combo(&lz, &t));
    let bronze = optimize_jumps(&req).unwrap();
    assert_eq!(bronze.status, OptimizerStatus::OptimalProven);
    assert!(bronze.enumeration_complete);
    assert!(bronze.candidates.iter().all(|c| c.base_hundredths == 160));
    assert_eq!(bronze.candidates.len(), 2);
    assert!(sequences(&bronze).contains(&vec![solo(&lz), combo(&lz, &t)]));
    assert!(sequences(&bronze).contains(&vec![combo(&lz, &t), solo(&lz)]));
    req.category = AdultCategory::Gold;
    let gold = optimize_jumps(&req).unwrap();
    assert_eq!(gold.candidates[0].base_hundredths, 200);
    assert_eq!(sequences(&gold), vec![vec![combo(&lz, &t), combo(&lz, &t)]]);
    req.max_elements = 3;
    let gold = optimize_jumps(&req).unwrap();
    assert_eq!(gold.candidates[0].base_hundredths, 200);
    assert!(gold
        .candidates
        .iter()
        .all(|c| validate_program(&c.program).findings.is_empty()));
}

#[test]
fn category_element_limits_apply_to_the_same_api() {
    let jumps = vec![
        jump(JumpKind::ToeLoop, 1),
        jump(JumpKind::Salchow, 1),
        jump(JumpKind::Loop, 1),
        jump(JumpKind::Flip, 1),
        jump(JumpKind::Lutz, 1),
    ];
    for (category, limit) in [
        (AdultCategory::Bronze, 4),
        (AdultCategory::Silver, 4),
        (AdultCategory::Gold, 5),
    ] {
        let mut req = request(category, jumps.clone());
        req.max_elements = 5;
        req.max_search_nodes = 5000;
        req.max_solutions = 500;
        let result = optimize_jumps(&req).unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(result
            .candidates
            .iter()
            .all(|c| c.program.elements.len() == limit));
    }
}

#[test]
fn empty_domain_limits_and_unsupported_variants_are_explicit() {
    let mut req = request(AdultCategory::Bronze, vec![]);
    let result = optimize_jumps(&req).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(result.candidates[0].base_hundredths, 0);
    assert!(result.candidates[0].program.elements.is_empty());
    req.require_nonempty = true;
    assert_eq!(
        optimize_jumps(&req).unwrap().status,
        OptimizerStatus::InfeasibleProven
    );
    let t = jump(JumpKind::ToeLoop, 1);
    req.allowed_jumps = vec![t.clone()];
    req.require_nonempty = false;
    req.max_search_nodes = 1;
    let limited = optimize_jumps(&req).unwrap();
    assert_eq!(limited.status, OptimizerStatus::FeasibleUnproven);
    assert_eq!(
        limited.termination_reason,
        JumpTerminationReason::SearchLimit
    );
    assert!(!limited.enumeration_complete);
    req.time_limit = Some(Duration::ZERO);
    assert_eq!(
        optimize_jumps(&req).unwrap().status,
        OptimizerStatus::Incomplete
    );
    req.time_limit = None;
    req.max_search_nodes = 100;
    req.max_solutions = 1;
    let limited = optimize_jumps(&req).unwrap();
    assert_eq!(limited.status, OptimizerStatus::OptimalProven);
    assert!(!limited.enumeration_complete);
    assert_eq!(
        limited.termination_reason,
        JumpTerminationReason::SolutionLimit
    );
    req.combinations = vec![JumpElement::Sequence(vec![JumpPart::Jump(t)])];
    assert!(matches!(
        optimize_jumps(&req),
        Err(JumpOptimizerError::UnsupportedVariant)
    ));
}
