#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::{collections::BTreeSet, time::Duration};

fn pos(position: SpinPosition, n: u16) -> SpinStep {
    SpinStep::Position {
        position: Some(position),
        revolutions: Some(n),
        feature_attempts: Some(vec![]),
    }
}
fn jump(kind: JumpKind) -> ProgramVariant {
    ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(kind),
            rotations: Some(1),
        })),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
        recognition_assumed: true,
    }
}
fn spin(
    kind: RegularSpinKind,
    entry: SpinEntry,
    changed: bool,
    target: Level,
    features: Vec<SpinFeatureAttempt>,
) -> ProgramVariant {
    let steps = if kind == RegularSpinKind::Combination {
        vec![
            pos(SpinPosition::Camel, 3),
            pos(SpinPosition::Sit, 3),
            SpinStep::ChangeFoot,
            pos(SpinPosition::Upright, 3),
        ]
    } else {
        vec![pos(
            if kind == RegularSpinKind::Camel {
                SpinPosition::Camel
            } else {
                SpinPosition::Sit
            },
            8,
        )]
    };
    ProgramVariant {
        element: PlannedElement::RegularSpin(RegularSpin {
            entry: Some(entry),
            steps: Some(steps),
        }),
        evidence: ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry,
                    change_foot: changed,
                }),
                planned_level: Some(target),
                feature_attempts: Some(features),
                panel: None,
                final_upright_wind_up: None,
            },
            local_feature_qualification_assumed: true,
        },
        recognition_assumed: true,
    }
}
fn feature(f: SpinFeature, outcome: FeatureAttemptOutcome) -> SpinFeatureAttempt {
    SpinFeatureAttempt {
        feature: f,
        position_step: 0,
        outcome,
    }
}
fn chsp() -> ProgramVariant {
    ProgramVariant {
        element: PlannedElement::ChoreographicSpin,
        evidence: ElementPlanEvidence::ChoreographicSpin {
            consecutive_revolutions_per_foot: Some(vec![3]),
        },
        recognition_assumed: true,
    }
}
fn request(
    category: AdultCategory,
    variants: Vec<ProgramVariant>,
    n: usize,
) -> ProgramOptimizationRequest {
    let profile = adult_profile(category);
    ProgramOptimizationRequest {
        season: profile.season,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        policy: adult_program_policy(category),
        variants,
        exclusions: vec![],
        must_have: vec![],
        min_elements: n,
        max_elements: n,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        equivalence: ProgramEquivalence::OrderedVariants,
        max_proposals: 100_000,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    }
}
fn result(req: &ProgramOptimizationRequest) -> ProgramOptimizationResult {
    optimize_program(&ProfileRegistry::adult(), req).unwrap()
}
fn ids(r: &ProgramOptimizationResult) -> BTreeSet<Vec<usize>> {
    r.candidates
        .iter()
        .map(|c| c.variant_indices.clone())
        .collect()
}

// A source-independent exhaustive oracle: finite words, fixture eligibility and
// literal objective coefficients. It does not call any production rule/scorer.
fn oracle(
    n: usize,
    values: &[u32],
    valid: impl Fn(&[usize]) -> bool,
) -> (u32, BTreeSet<Vec<usize>>) {
    let mut best = 0;
    let mut ties = BTreeSet::new();
    for mut code in 0..values.len().pow(n as u32) {
        let word: Vec<_> = (0..n)
            .map(|_| {
                let digit = code % values.len();
                code /= values.len();
                digit
            })
            .collect();
        if !valid(&word) {
            continue;
        }
        let value = word.iter().map(|i| values[*i]).sum();
        if value > best {
            best = value;
            ties.clear();
        }
        if value == best {
            ties.insert(word);
        }
    }
    (best, ties)
}

#[test]
fn competing_feature_mixed_tradeoff_and_all_ordered_ties_match_independent_oracle() {
    let variants = vec![
        jump(JumpKind::ToeLoop),
        spin(
            RegularSpinKind::Camel,
            SpinEntry::NonFlying,
            false,
            Level::One,
            vec![feature(
                SpinFeature::IncreaseOfSpeed,
                FeatureAttemptOutcome::Successful,
            )],
        ),
        spin(
            RegularSpinKind::Sit,
            SpinEntry::NonFlying,
            false,
            Level::One,
            vec![feature(
                SpinFeature::IncreaseOfSpeed,
                FeatureAttemptOutcome::Successful,
            )],
        ),
        spin(
            RegularSpinKind::Sit,
            SpinEntry::NonFlying,
            false,
            Level::Basic,
            vec![],
        ),
    ];
    let mut req = request(AdultCategory::Bronze, variants, 3);
    req.must_have = vec![vec![0]];
    let r = result(&req);
    // 1T .40, CSp1 1.70, SSp1 1.60, SSpB 1.30 (2786 fixture).
    let expected = oracle(3, &[40, 170, 160, 130], |w| {
        w.contains(&0)
            && w.iter().filter(|i| **i == 0).count() <= 2
            && w.iter().filter(|i| **i == 1).count() <= 1
            && w.iter().filter(|i| **i == 2 || **i == 3).count() <= 1
            && !(w.contains(&1) && w.contains(&2))
    });
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    assert_eq!(r.candidates[0].base_hundredths, expected.0);
    assert_eq!(ids(&r), expected.1);
    for c in &r.candidates {
        assert_eq!(
            c.base_hundredths,
            c.per_element
                .iter()
                .map(|v| u32::from(v.hundredths))
                .sum::<u32>()
        );
        assert!(c
            .validation
            .validation
            .findings
            .iter()
            .all(|f| f.rule_id == "spin-execution-recognition"));
    }
    req.exclusions = vec![PersonalExclusion::SpinFeature(SpinFeature::IncreaseOfSpeed)];
    let r = result(&req);
    assert_eq!(r.excluded_variants, vec![1, 2]);
    assert_eq!(r.candidates[0].base_hundredths, 210);
}

#[test]
fn all_four_profiles_handle_actual_mixed_programs_and_mandatory_roles() {
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let advanced = matches!(category, AdultCategory::Gold | AdultCategory::Masters);
        let variants = if advanced {
            vec![
                jump(JumpKind::Axel),
                spin(
                    RegularSpinKind::Combination,
                    SpinEntry::NonFlying,
                    true,
                    Level::Basic,
                    vec![],
                ),
                spin(
                    RegularSpinKind::Camel,
                    SpinEntry::Flying,
                    false,
                    Level::Basic,
                    vec![],
                ),
                chsp(),
            ]
        } else {
            vec![
                jump(JumpKind::ToeLoop),
                spin(
                    RegularSpinKind::Sit,
                    SpinEntry::NonFlying,
                    false,
                    Level::Basic,
                    vec![],
                ),
            ]
        };
        let n = variants.len();
        let mut req = request(category, variants, n);
        req.must_have = (0..n).map(|i| vec![i]).collect();
        let r = result(&req);
        assert_eq!(r.status, OptimizerStatus::OptimalProven, "{category:?}");
        assert!(r.enumeration_complete);
        let expected = oracle(n, &vec![1; n], |w| (0..n).all(|i| w.contains(&i)));
        assert_eq!(ids(&r), expected.1);
        assert_eq!(
            r.candidates[0].base_hundredths,
            if advanced { 850 } else { 170 }
        );
        if advanced {
            req.exclusions = vec![PersonalExclusion::FlyingEntry];
            assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
        }
    }
}

#[test]
fn steps_and_choreography_share_the_actual_ordered_optimizer_and_catalog() {
    for category in [AdultCategory::Bronze, AdultCategory::Gold] {
        let advanced = category == AdultCategory::Gold;
        let variants = if advanced {
            vec![
                jump(JumpKind::Axel),
                spin(
                    RegularSpinKind::Combination,
                    SpinEntry::NonFlying,
                    true,
                    Level::Basic,
                    vec![],
                ),
                spin(
                    RegularSpinKind::Camel,
                    SpinEntry::Flying,
                    false,
                    Level::Basic,
                    vec![],
                ),
                chsp(),
                ProgramVariant {
                    element: PlannedElement::StepSequence,
                    evidence: ElementPlanEvidence::Step {
                        level: Level::Basic,
                        clean_difficult_turns: Some(2),
                        full_ice_coverage: Some(true),
                        movement: Some(SkatingMovement::Spiral),
                        qualified_features: Some(vec![]),
                        qualification_assumed: true,
                    },
                    recognition_assumed: true,
                },
            ]
        } else {
            vec![
                jump(JumpKind::ToeLoop),
                spin(
                    RegularSpinKind::Sit,
                    SpinEntry::NonFlying,
                    false,
                    Level::Basic,
                    vec![],
                ),
                ProgramVariant {
                    element: PlannedElement::ChoreographicSequence { movements: None },
                    evidence: ElementPlanEvidence::ChoreographicSequence {
                        movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::InaBauer]),
                        clearly_visible: Some(true),
                    },
                    recognition_assumed: true,
                },
            ]
        };
        let n = variants.len();
        let mut req = request(category, variants, n);
        req.must_have = (0..n).map(|i| vec![i]).collect();
        let r = result(&req);
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(r.enumeration_complete);
        let values = if advanced {
            vec![110, 200, 190, 350, 160]
        } else {
            vec![40, 130, 350]
        };
        let reference = oracle(n, &values, |w| (0..n).all(|i| w.contains(&i)));
        assert_eq!(r.candidates[0].base_hundredths, reference.0);
        assert_eq!(ids(&r), reference.1);
    }
}

#[test]
fn finite_infeasibility_interruptions_caps_and_unsupported_inputs_are_conservative() {
    let mut req = request(AdultCategory::Bronze, vec![jump(JumpKind::ToeLoop)], 3);
    assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
    req.max_elements = 2;
    req.min_elements = 1;
    req.max_solutions = 1;
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(!r.enumeration_complete);
    req.time_limit = Some(Duration::ZERO);
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::Incomplete);
    assert!(!r.enumeration_complete);
    req.time_limit = None;
    req.max_proposals = 1;
    req.max_elements = 3;
    req.min_elements = 3;
    assert_eq!(result(&req).status, OptimizerStatus::Incomplete);
    req.exclusions = vec![PersonalExclusion::Unsupported("no back pain".into())];
    assert_eq!(
        optimize_program(&ProfileRegistry::adult(), &req).unwrap_err(),
        ProgramOptimizationError::UnsupportedRestriction
    );
    req.exclusions.clear();
    req.variants[0].recognition_assumed = false;
    assert!(matches!(
        optimize_program(&ProfileRegistry::adult(), &req),
        Err(ProgramOptimizationError::UnsupportedVariant(0))
    ));
}

#[test]
fn synthetic_profile_reuses_solver_and_explicit_shared_policies() {
    let mut profile = adult_profile(AdultCategory::Silver);
    profile.id.name = "synthetic-mixed".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.jumps.max_jumps = 1;
    let mut registry = ProfileRegistry::adult();
    registry.register(profile.clone()).unwrap();
    let mut req = request(
        AdultCategory::Silver,
        vec![
            jump(JumpKind::ToeLoop),
            spin(
                RegularSpinKind::Sit,
                SpinEntry::NonFlying,
                false,
                Level::Basic,
                vec![],
            ),
        ],
        2,
    );
    req.profile = profile.id.clone();
    req.policy.spins.profile = profile.id;
    req.must_have = vec![vec![0], vec![1]];
    let r = optimize_program(&registry, &req).unwrap();
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert_eq!(ids(&r), BTreeSet::from([vec![0, 1], vec![1, 0]]));
    req.policy.max_steps = 1;
    req.profile = adult_profile(AdultCategory::Silver).id.clone();
    req.policy.spins.profile = req.profile.clone();
    assert!(matches!(
        optimize_program(&registry, &req),
        Err(ProgramOptimizationError::Profile(_))
    ));
}

#[test]
fn assumed_silver_axel_sequence_reaches_optimizer_scoring_and_shared_validation() {
    let sequence = ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Sequence(vec![
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::ToeLoop),
                rotations: Some(1),
            }),
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::Axel),
                rotations: Some(1),
            }),
        ])),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: true,
        },
        recognition_assumed: true,
    };
    let mut req = request(
        AdultCategory::Silver,
        vec![
            sequence,
            spin(
                RegularSpinKind::Sit,
                SpinEntry::NonFlying,
                false,
                Level::Basic,
                vec![],
            ),
        ],
        2,
    );
    req.must_have = vec![vec![0], vec![1]];
    let simple = optimize_program_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &req,
        &["1T+1A+SEQ", "SSpB"].map(AssumedRecognition::new),
    )
    .unwrap();
    assert_eq!(simple.status, OptimizerStatus::OptimalProven);
    assert_eq!(simple.candidates[0].base_hundredths, 280);
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    assert_eq!(ids(&r), BTreeSet::from([vec![0, 1], vec![1, 0]]));
    assert_eq!(r.candidates[0].base_hundredths, 280); // 1T .40 + 1A 1.10 + SSpB 1.30.
    for c in &r.candidates {
        assert!(c
            .validation
            .validation
            .findings
            .iter()
            .all(|f| f.rule_id == "spin-execution-recognition"));
        assert!(c
            .validation
            .applied_rules
            .iter()
            .any(|r| r.rule_id == "program-sequence-direct-step"));
    }
    req.variants[0].evidence = ElementPlanEvidence::Jump {
        direct_transitions_assumed: false,
    };
    assert!(matches!(
        optimize_program(&ProfileRegistry::adult(), &req),
        Err(ProgramOptimizationError::UnsupportedVariant(0))
    ));
}

#[test]
fn actual_rule_trace_is_source_bound_and_excludes_absent_element_checks() {
    let r = result(&request(
        AdultCategory::Bronze,
        vec![jump(JumpKind::ToeLoop)],
        1,
    ));
    assert!(r.supported_rules.contains(&"program-step-recognition"));
    assert!(!r
        .applied_rules
        .iter()
        .any(|r| r.rule_id == "program-step-recognition"
            || r.rule_id == "program-choreographic-sequence"
            || r.rule_id == "program-spin-level-credit"));
    assert!(r
        .applied_rules
        .iter()
        .any(|r| r.rule_id == "catalog-unfactored-base-value"
            && r.element_index == Some(0)
            && r.source.document == "ISU Communication 2786"));
    for candidate in &r.candidates {
        for applied in &candidate.validation.applied_rules {
            assert!(r.applied_rules.contains(applied));
            assert!(r.supported_rules.contains(&applied.rule_id));
            assert!(r.sources.contains(&applied.source));
        }
    }
}

#[test]
fn proposal_cap_stops_a_large_finite_domain_without_claiming_a_node_budget() {
    let variants = [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
    ]
    .into_iter()
    .flat_map(|kind| {
        [1, 2].map(move |n| {
            let mut v = jump(kind);
            if let PlannedElement::Jump(JumpElement::Solo(j)) = &mut v.element {
                j.rotations = Some(n);
            }
            v
        })
    })
    .collect();
    // Ten slots and ten variants: 10^10 nonempty ordered assignments before rules.
    // The first highest-BV repeated double-Lutz proposal is ineligible for Bronze.
    let mut req = request(AdultCategory::Bronze, variants, 10);
    req.max_proposals = 1;
    let r = result(&req);
    assert_eq!(r.examined_programs, 1);
    assert_eq!(r.status, OptimizerStatus::Incomplete);
    assert_eq!(r.termination_reason, JumpTerminationReason::SearchLimit);
    assert!(!r.enumeration_complete);
}

#[test]
fn optimize_gold_sov_selections_without_spin_or_step_proof() {
    let category = AdultCategory::Gold;
    let mut variants = vec![];
    for _ in 0..2 {
        variants.push(ProgramVariant {
            element: PlannedElement::RegularSpin(RegularSpin {
                entry: None,
                steps: None,
            }),
            evidence: ElementPlanEvidence::Spin {
                spin: RegularSpinEvidence::default(),
                local_feature_qualification_assumed: false,
            },
            recognition_assumed: false,
        });
    }
    variants.push(ProgramVariant {
        element: PlannedElement::StepSequence,
        evidence: ElementPlanEvidence::Step {
            level: Level::Two,
            clean_difficult_turns: None,
            full_ice_coverage: None,
            movement: None,
            qualified_features: None,
            qualification_assumed: false,
        },
        recognition_assumed: false,
    });
    variants.push(chsp());
    let q = request(category, variants, 4);
    let choices = ["CCoSp3", "FSSp2", "StSq2", "ChSp1"].map(AssumedRecognition::new);
    let result =
        optimize_program_with_assumed_recognition(&ProfileRegistry::adult(), &q, &choices).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(result.candidates[0].base_hundredths, 1260);
    assert!(result.supported_rules.contains(&"program-step-level-cap"));
    assert!(result
        .supported_rules
        .contains(&"program-choreographic-plan-consistency"));
    assert!(!result.supported_rules.contains(&"spin-minimum-revolutions"));
    assert!(!result
        .supported_rules
        .contains(&"program-spin-level-credit"));
    for (value, &index) in result.candidates[0]
        .per_element
        .iter()
        .zip(&result.candidates[0].variant_indices)
    {
        assert_eq!(value.notation, choices[index].code);
    }
}

#[test]
fn assumed_optimizer_rejects_senior_repeat_credit_before_search() {
    let category = SeniorCategory::Women;
    let profile = senior_profile(category);
    let mut axel = jump(JumpKind::Axel);
    let PlannedElement::Jump(JumpElement::Solo(jump)) = &mut axel.element else {
        unreachable!()
    };
    jump.rotations = Some(3);
    let mut q = request(AdultCategory::Gold, vec![axel.clone(), axel], 2);
    q.profile = profile.id;
    q.policy = senior_program_policy(category);
    let choices = ["3A", "3A"].map(AssumedRecognition::new);
    assert!(matches!(
        optimize_program_with_assumed_recognition(&ProfileRegistry::senior(), &q, &choices),
        Err(ProgramOptimizationError::InvalidRequest(reason)) if reason.contains("contextual BV")
    ));
}
