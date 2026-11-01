use skating_program_lab::*;

fn lutz() -> PlannedElement {
    PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Lutz),
        rotations: Some(1),
    }))
}
fn recognition() -> PanelRecognition {
    PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from("1Lz").unwrap()],
                None,
                Some(vec![]),
            )
            .unwrap(),
        ),
    }
}

#[test]
fn exhaustive_reference_matches_independent_eight_case_table() {
    let result = enumerate_bronze_reference(false).unwrap();
    assert_eq!(result.domain.slot_start_seconds, [10, 20, 30]);
    if let ReferenceOutcome::Optimal { candidates, .. } = &result.outcome {
        for candidate in candidates {
            assert!(candidate
                .program
                .sources
                .iter()
                .any(|s| s.document.contains("Adult Figure Skating")));
            assert!(candidate
                .program
                .sources
                .iter()
                .any(|s| s.document == "ISU Communication 2786" && s.location.contains("row 1Lz")));
        }
    } else {
        panic!("expected feasible reference domain");
    }
    assert_eq!(result.domain.duration_seconds, 100);
    assert!(
        result.domain.uninterrupted_timing_assumed
            && result.domain.no_event_exception_assumed
            && result.domain.clean_recognition_assumed
    );
    assert!(!result.domain.exclude_lutz);
    let ReferenceOutcome::Optimal {
        base_hundredths,
        candidates,
    } = result.outcome
    else {
        panic!("expected optimum")
    };
    assert_eq!(base_hundredths, 120);
    // Adult p. 11 limits 1Lz to two; Communication 2786 p. 2 gives 0.60.
    let cases = [
        ([false, false, false], None),
        ([true, false, false], Some(60)),
        ([false, true, false], Some(60)),
        ([true, true, false], Some(120)),
        ([false, false, true], Some(60)),
        ([true, false, true], Some(120)),
        ([false, true, true], Some(120)),
        ([true, true, true], None),
    ];
    let mut expected = Vec::new();
    for (slots, value) in cases {
        let elements = slots
            .iter()
            .filter(|&&v| v)
            .map(|_| lutz())
            .collect::<Vec<_>>();
        let program = Program {
            season: Season {
                start_year: 2026,
                end_year: 2027,
            },
            category: AdultCategory::Bronze,
            ruleset: RulesetId::IsuAdult2026_27July16,
            scoring_table: ScoringTableId::Isu2786June2_2026,
            sources: vec![],
            elements,
        };
        let validation = validate_program(&program);
        if slots == [true; 3] {
            assert!(validation
                .findings
                .iter()
                .any(|f| f.rule_id.ends_with("jump-repetition")));
        } else {
            assert!(validation.findings.is_empty());
        }
        if let Some(score) = value {
            let per_jump =
                match lookup_base_value(program.scoring_table, &recognition(), &[]).unwrap() {
                    BaseValueOutcome::Value(v) => v.hundredths,
                    _ => panic!("expected value"),
                };
            assert_eq!(score, per_jump * program.elements.len() as u16);
            if score == 120 {
                expected.push(slots);
            }
        }
    }
    assert_eq!(
        candidates.iter().map(|c| c.slots).collect::<Vec<_>>(),
        expected
    );
    for candidate in candidates {
        assert_eq!(candidate.coverage, Coverage::Partial);
        assert!(validate_program(&candidate.program).findings.is_empty());
        let starts = candidate
            .slots
            .iter()
            .zip(result.domain.slot_start_seconds)
            .filter_map(|(&occupied, time)| occupied.then_some(time))
            .collect::<Vec<_>>();
        assert_eq!(starts.len(), candidate.program.elements.len());
        assert!(starts
            .iter()
            .all(|&start| start < result.domain.duration_seconds / 2));
        let mut independently_computed = 0;
        for element in &candidate.program.elements {
            assert_eq!(*element, lutz());
            let BaseValueOutcome::Value(value) =
                lookup_base_value(candidate.program.scoring_table, &recognition(), &[]).unwrap()
            else {
                panic!("expected value")
            };
            independently_computed += value.hundredths;
        }
        assert_eq!(candidate.base_hundredths, independently_computed);
    }
}

#[test]
fn excluding_the_only_variant_is_infeasible() {
    let result = enumerate_bronze_reference(true).unwrap();
    assert!(result.domain.exclude_lutz);
    assert_eq!(result.outcome, ReferenceOutcome::Infeasible);
}

#[cfg(feature = "optimizer")]
#[test]
fn pumpkin_matches_reference_and_respects_limits() {
    use std::time::Duration;
    for exclude in [false, true] {
        let request = OptimizerRequest {
            exclude_lutz: exclude,
            enabled_slots: [true; 3],
            ruleset: RulesetId::IsuAdult2026_27July16,
            scoring_table: ScoringTableId::Isu2786June2_2026,
            enumerate_equal_optima: true,
            max_solutions: 8,
            time_limit: None,
        };
        let result = optimize_bronze(&request).unwrap();
        match enumerate_bronze_reference(exclude).unwrap().outcome {
            ReferenceOutcome::Infeasible => {
                assert_eq!(result.status, OptimizerStatus::InfeasibleProven)
            }
            ReferenceOutcome::Optimal {
                base_hundredths,
                candidates,
            } => {
                assert_eq!(result.status, OptimizerStatus::OptimalProven);
                assert!(result.enumeration_complete);
                let mut expected = candidates.iter().map(|c| c.slots).collect::<Vec<_>>();
                expected.sort();
                assert_eq!(
                    result
                        .candidates
                        .iter()
                        .map(|c| c.slots)
                        .collect::<Vec<_>>(),
                    expected
                );
                assert!(result
                    .candidates
                    .iter()
                    .all(|c| c.base_hundredths == base_hundredths));
            }
        }
    }
    let interrupted = optimize_bronze(&OptimizerRequest {
        exclude_lutz: false,
        enabled_slots: [true; 3],
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        enumerate_equal_optima: true,
        max_solutions: 8,
        time_limit: Some(Duration::ZERO),
    })
    .unwrap();
    assert!(matches!(
        interrupted.status,
        OptimizerStatus::Incomplete | OptimizerStatus::FeasibleUnproven
    ));
    assert!(!interrupted.enumeration_complete);
    assert_eq!(
        interrupted.termination_reason,
        TerminationReason::OptimizationInterrupted
    );
    assert_eq!(interrupted.time_limit, Some(Duration::ZERO));
    let limited = optimize_bronze(&OptimizerRequest {
        exclude_lutz: false,
        enabled_slots: [true; 3],
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        enumerate_equal_optima: true,
        max_solutions: 2,
        time_limit: None,
    })
    .unwrap();
    assert_eq!(limited.status, OptimizerStatus::OptimalProven);
    assert_eq!(limited.candidates.len(), 2);
    assert!(!limited.enumeration_complete);
    assert_eq!(limited.termination_reason, TerminationReason::SolutionLimit);
    assert_eq!(limited.max_solutions, 2);
    assert!(limited.enumerate_equal_optima);
}

#[cfg(feature = "optimizer")]
#[test]
fn candidate_slots_narrow_the_domain() {
    let request = OptimizerRequest {
        exclude_lutz: false,
        enabled_slots: [true, false, true],
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        enumerate_equal_optima: true,
        max_solutions: 8,
        time_limit: None,
    };
    let result = optimize_bronze(&request).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].slots, [true, false, true]);
    assert_eq!(result.candidates[0].base_hundredths, 120);
    let no_slots = OptimizerRequest {
        enabled_slots: [false; 3],
        ..request.clone()
    };
    assert_eq!(
        optimize_bronze(&no_slots).unwrap().status,
        OptimizerStatus::InfeasibleProven
    );
    let no_results = OptimizerRequest {
        max_solutions: 0,
        ..request
    };
    assert!(matches!(
        optimize_bronze(&no_results),
        Err(ReferenceError::InvalidLimit)
    ));
}

#[cfg(feature = "optimizer")]
#[test]
fn all_slot_subsets_match_independent_enumeration() {
    for enabled_mask in 0u8..8 {
        for exclude in [false, true] {
            let enabled_slots = std::array::from_fn(|i| enabled_mask & (1 << i) != 0);
            let request = OptimizerRequest {
                exclude_lutz: exclude,
                enabled_slots,
                ruleset: RulesetId::IsuAdult2026_27July16,
                scoring_table: ScoringTableId::Isu2786June2_2026,
                enumerate_equal_optima: true,
                max_solutions: 8,
                time_limit: None,
            };
            let result = optimize_bronze(&request).unwrap();
            let mut choices = Vec::new();
            for mask in 1u8..8 {
                if exclude || mask & !enabled_mask != 0 || mask.count_ones() > 2 {
                    continue;
                }
                choices.push((
                    mask.count_ones() * 60,
                    std::array::from_fn(|i| mask & (1 << i) != 0),
                ));
            }
            if let Some(maximum) = choices.iter().map(|c| c.0).max() {
                let mut expected = choices
                    .into_iter()
                    .filter_map(|(score, slots)| (score == maximum).then_some(slots))
                    .collect::<Vec<_>>();
                expected.sort();
                assert_eq!(result.status, OptimizerStatus::OptimalProven);
                assert!(result.enumeration_complete);
                assert_eq!(
                    result
                        .candidates
                        .iter()
                        .map(|c| c.slots)
                        .collect::<Vec<_>>(),
                    expected
                );
                assert!(result
                    .candidates
                    .iter()
                    .all(|c| u32::from(c.base_hundredths) == maximum
                        && c.program.sources == result.sources));
            } else {
                assert_eq!(result.status, OptimizerStatus::InfeasibleProven);
                assert!(result.candidates.is_empty());
            }
            assert_eq!(
                result.objective,
                OptimizerObjective::MaximizeUnfactoredBaseValueHundredths
            );
            assert!(result
                .implemented_rules
                .contains(&"isu-adult-2026-27-bronze-jump-repetition"));
            assert!(result
                .sources
                .iter()
                .any(|s| s.document == "ISU Communication 2786" && s.location.contains("row 1Lz")));
            assert!(result.sources.iter().any(
                |s| s.document.contains("Adult Figure Skating") && s.location.contains("p. 11")
            ));
        }
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn result_caps_and_disabled_enumeration_are_conservative() {
    let request = OptimizerRequest {
        exclude_lutz: false,
        enabled_slots: [true; 3],
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        enumerate_equal_optima: true,
        max_solutions: 1,
        time_limit: None,
    };
    for limit in [1, 2, 8] {
        let result = optimize_bronze(&OptimizerRequest {
            max_solutions: limit,
            ..request.clone()
        })
        .unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert_eq!(result.candidates.len(), limit.min(3));
        assert_eq!(result.enumeration_complete, limit > 3);
        assert_eq!(
            result.termination_reason,
            if limit > 3 {
                TerminationReason::EnumerationExhausted
            } else {
                TerminationReason::SolutionLimit
            }
        );
    }
    let result = optimize_bronze(&OptimizerRequest {
        enumerate_equal_optima: false,
        max_solutions: 8,
        ..request
    })
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(result.candidates.len(), 1);
    assert!(!result.enumeration_complete);
    assert_eq!(
        result.termination_reason,
        TerminationReason::EnumerationDisabled
    );
    assert!(!result.enumerate_equal_optima);
}
