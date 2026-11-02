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
fn reference_has_one_distinct_unfactored_optimum() {
    let result = enumerate_bronze_reference(false).unwrap();
    assert!(result.domain.clean_recognition_assumed);
    let ReferenceOutcome::Optimal {
        base_hundredths,
        candidates,
    } = result.outcome
    else {
        panic!("expected optimum")
    };
    assert_eq!(base_hundredths, 120);
    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];
    assert_eq!(candidate.program.elements, vec![lutz(), lutz()]);
    assert_eq!(candidate.coverage, Coverage::Partial);
    assert!(validate_program(&candidate.program).findings.is_empty());
    let BaseValueOutcome::Value(value) =
        lookup_base_value(candidate.program.scoring_table, &recognition(), &[]).unwrap()
    else {
        panic!("expected value")
    };
    assert_eq!(value.hundredths, 60);
    assert!(candidate.program.sources.contains(&value.source));
    assert_eq!(
        enumerate_bronze_reference(true).unwrap().outcome,
        ReferenceOutcome::Infeasible
    );
}

#[cfg(feature = "optimizer")]
fn request() -> OptimizerRequest {
    OptimizerRequest {
        exclude_lutz: false,
        max_elements: 3,
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        enumerate_equal_optima: true,
        max_solutions: 8,
        time_limit: None,
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn pumpkin_matches_distinct_reference_for_all_capacities() {
    for max_elements in 0..=3 {
        for exclude_lutz in [false, true] {
            let result = optimize_bronze(&OptimizerRequest {
                max_elements,
                exclude_lutz,
                ..request()
            })
            .unwrap();
            if max_elements == 0 || exclude_lutz {
                assert_eq!(result.status, OptimizerStatus::InfeasibleProven);
                assert!(result.candidates.is_empty());
            } else {
                assert_eq!(result.status, OptimizerStatus::OptimalProven);
                assert!(result.enumeration_complete);
                assert_eq!(result.candidates.len(), 1);
                assert_eq!(
                    result.candidates[0].program.elements,
                    vec![lutz(); max_elements.min(2)]
                );
                assert_eq!(
                    result.candidates[0].base_hundredths,
                    (max_elements.min(2) * 60) as u16
                );
                if max_elements == 3 {
                    let ReferenceOutcome::Optimal { candidates, .. } =
                        enumerate_bronze_reference(false).unwrap().outcome
                    else {
                        panic!()
                    };
                    assert_eq!(
                        result.candidates[0].program.elements,
                        candidates[0].program.elements
                    );
                }
            }
        }
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn limits_and_interruptions_do_not_claim_complete_enumeration() {
    use std::time::Duration;
    let limited = optimize_bronze(&OptimizerRequest {
        max_solutions: 1,
        ..request()
    })
    .unwrap();
    assert_eq!(limited.status, OptimizerStatus::OptimalProven);
    assert_eq!(limited.candidates.len(), 1);
    assert!(!limited.enumeration_complete);
    assert_eq!(limited.termination_reason, TerminationReason::SolutionLimit);
    let disabled = optimize_bronze(&OptimizerRequest {
        enumerate_equal_optima: false,
        ..request()
    })
    .unwrap();
    assert_eq!(
        disabled.termination_reason,
        TerminationReason::EnumerationDisabled
    );
    let interrupted = optimize_bronze(&OptimizerRequest {
        time_limit: Some(Duration::ZERO),
        ..request()
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
    assert!(matches!(
        optimize_bronze(&OptimizerRequest {
            max_solutions: 0,
            ..request()
        }),
        Err(ReferenceError::InvalidLimit)
    ));
    assert!(matches!(
        optimize_bronze(&OptimizerRequest {
            max_elements: 4,
            ..request()
        }),
        Err(ReferenceError::InvalidLimit)
    ));
}
