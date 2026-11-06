#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

#[path = "support/generation_defaults.rs"]
#[allow(dead_code)]
mod generation_defaults;

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

#[test]
fn empty_spin_role_universe_is_infeasible_without_a_proposal() {
    let mut q = request(AdultCategory::Gold, vec![jump(JumpKind::Axel)], 1);
    q.min_elements = 0;
    q.max_proposals = 1;
    q.exclusions = vec![PersonalExclusion::Variant(0)];
    let excluded = result(&q);
    assert_eq!(excluded.status, OptimizerStatus::InfeasibleProven);
    assert_eq!(excluded.examined_programs, 0);
    q.variants.clear();
    q.exclusions.clear();
    let absent = result(&q);
    assert_eq!(absent.status, OptimizerStatus::InfeasibleProven);
    assert_eq!(absent.examined_programs, 0);
    q.policy.spins.required.push(vec![]);
    assert!(matches!(
        optimize_program(&ProfileRegistry::adult(), &q),
        Err(ProgramOptimizationError::Profile(_))
    ));
}

// With one choice from each disjoint group and exactly that many slots,
// every word belongs to exactly one fixed-choice stratum. The fixtures have
// order-independent BV, so the optimizer returns every feasible word in a
// stratum as an optimum. Compare the union before testing the open choice.
fn assert_spin_role_oracle(
    registry: &ProfileRegistry,
    mut q: ProgramOptimizationRequest,
    codes: &[AssumedRecognition],
    choices: &[Option<u32>],
) {
    let fixed = 0..codes.len() - choices.len();
    let n = fixed.len() + 1;
    let option_group: Vec<_> = (fixed.end..codes.len()).collect();
    let mut expected_all = BTreeMap::new();
    let mut actual_all = BTreeMap::new();
    for (&option, &score) in option_group.iter().zip(choices) {
        let mut required: Vec<_> = fixed.clone().collect();
        required.push(option);
        q.must_have = required.iter().map(|&i| vec![i]).collect();
        let result = optimize_program_with_assumed_recognition(registry, &q, codes).unwrap();
        if let Some(score) = score {
            assert_eq!(
                result.status,
                OptimizerStatus::OptimalProven,
                "{:?}",
                q.profile
            );
            assert!(result.enumeration_complete);
            let (_, words) = word_oracle(n, codes.len(), |word| {
                required.iter().all(|i| word.contains(i)).then_some(score)
            });
            expected_all.extend(words.into_iter().map(|word| (word, score)));
            for candidate in result.candidates {
                assert!(actual_all
                    .insert(candidate.variant_indices, candidate.base_hundredths)
                    .is_none());
            }
        } else {
            assert_eq!(
                result.status,
                OptimizerStatus::InfeasibleProven,
                "{:?}",
                q.profile
            );
            assert!(result.candidates.is_empty());
            assert!(!result.enumeration_complete);
        }
    }
    assert_eq!(actual_all, expected_all, "{:?}", q.profile);

    q.must_have = fixed.map(|i| vec![i]).chain([option_group]).collect();
    let result = optimize_program_with_assumed_recognition(registry, &q, codes).unwrap();
    let best = expected_all.values().copied().max().unwrap();
    expected_all.retain(|_, value| *value == best);
    assert_eq!(
        result.status,
        OptimizerStatus::OptimalProven,
        "{:?}",
        q.profile
    );
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), expected_all.len());
    if q.policy.require_choreographic_spin {
        assert!(result.applied_rules.iter().any(|rule| {
            rule.rule_id == "program-required-choreographic-spin"
                && rule.source == q.policy.choreography_source
        }));
    }
    let actual: BTreeMap<_, _> = result
        .candidates
        .into_iter()
        .map(|c| (c.variant_indices, c.base_hundredths))
        .collect();
    assert_eq!(actual, expected_all, "{:?}", q.profile);
}

#[test]
fn adult_spin_role_optimizer_matches_complete_fixture_sets_and_bv() {
    use AdultCategory::{Bronze, Gold, Masters, Silver};
    let registry = ProfileRegistry::adult();
    // Fixed codes first, then alternate choices for the remaining slot.
    // Values are literal Communication 2786 totals. CSp1 with CSpB
    // repeats an abbreviation, and FSSpB alone misses Silver's nonflying role.
    type Case = (
        AdultCategory,
        &'static [&'static str],
        &'static [Option<u32>],
    );
    let cases: [Case; 6] = [
        (
            Bronze,
            &["1T", "CSpB", "SSpB", "SSp1", "CSp1"],
            &[Some(300), Some(330), None],
        ),
        (
            Silver,
            &["1T", "SSpB", "SSp1", "FSSpB"],
            &[Some(280), Some(310), None],
        ),
        (
            Gold,
            &["1A", "FSSp2", "StSq2", "ChSp1", "CCoSp2", "CCoSp3"],
            &[Some(1310), Some(1370)],
        ),
        (
            Masters,
            &["1A", "FSSp2", "StSq2", "ChSp1", "CCoSp2", "CCoSp3"],
            &[Some(1310), Some(1370)],
        ),
        // The remaining choice is a choreographic spin or a second solo jump.
        // The role groups admit both choices. The Adult policy requires ChSp1.
        (
            Gold,
            &["1A", "FSSp2", "StSq2", "CCoSp3", "ChSp1", "1T"],
            &[Some(1370), None],
        ),
        (
            Masters,
            &["1A", "FSSp2", "StSq2", "CCoSp3", "ChSp1", "1T"],
            &[Some(1370), None],
        ),
    ];
    for (category, codes, scores) in cases {
        let slots = codes.len() - scores.len() + 1;
        let mut codes: Vec<_> = codes.iter().copied().map(AssumedRecognition::new).collect();
        let generated = generate_program_candidates_with_assumed_recognition(
            &registry,
            &generation_defaults::request(adult_profile(category)),
            &codes,
        )
        .unwrap();
        assert_eq!(generated.recognition, codes);
        let mut variants = generated.domain.variants;
        if category == Silver {
            // The compound is supplied explicitly rather than inferred from solos.
            variants[0] = axel_sequence();
            codes[0] = AssumedRecognition::new("1T+1A+SEQ");
        }
        let q = request(category, variants, slots);
        if matches!(category, Gold | Masters) {
            let mut excluded = q.clone();
            let choreographic = codes.iter().position(|code| code.code == "ChSp1").unwrap();
            excluded
                .exclusions
                .push(PersonalExclusion::Variant(choreographic));
            excluded.max_proposals = 1;
            let absent =
                optimize_program_with_assumed_recognition(&registry, &excluded, &codes).unwrap();
            assert_eq!(absent.status, OptimizerStatus::InfeasibleProven);
            assert_eq!(absent.examined_programs, 0);
        }
        assert_spin_role_oracle(&registry, q, &codes, scores);
    }
}

#[test]
fn senior_spin_role_optimizer_matches_complete_fixture_sets_and_bv() {
    let registry = ProfileRegistry::senior();
    let codes =
        ["2A", "FCSp4", "StSq4", "ChSq1", "ChSp1", "CCoSp3", "CCoSp4"].map(AssumedRecognition::new);
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let profile = senior_profile(category);
        let generated = generate_program_candidates_with_assumed_recognition(
            &registry,
            &generation_defaults::request(profile.clone()),
            &codes,
        )
        .unwrap();
        assert_eq!(generated.recognition, codes);
        let mut q = request(AdultCategory::Gold, generated.domain.variants, 6);
        q.profile = profile.id;
        q.scoring_table = profile.scoring_table;
        q.policy = senior_program_policy(category);
        assert_spin_role_oracle(&registry, q, &codes, &[Some(2180), Some(2240)]);
    }
}

// A source-independent exhaustive oracle: finite words, fixture eligibility and
// literal objective coefficients. It does not call any production rule/scorer.
fn oracle(
    n: usize,
    values: &[u32],
    valid: impl Fn(&[usize]) -> bool,
) -> (u32, BTreeSet<Vec<usize>>) {
    word_oracle(n, values.len(), |word| {
        valid(word).then(|| word.iter().map(|&i| values[i]).sum())
    })
}

fn word_oracle(
    n: usize,
    domain_size: usize,
    score: impl Fn(&[usize]) -> Option<u32>,
) -> (u32, BTreeSet<Vec<usize>>) {
    let mut best = 0;
    let mut ties = BTreeSet::new();
    for mut code in 0..domain_size.pow(n as u32) {
        let word: Vec<_> = (0..n)
            .map(|_| {
                let digit = code % domain_size;
                code /= domain_size;
                digit
            })
            .collect();
        let Some(value) = score(&word) else { continue };
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
            assert!(r.applied_rules.iter().any(|rule| {
                rule.rule_id == "spin-required-role" && rule.source == req.policy.spins.source
            }));
            req.must_have.clear();
            req.exclusions = vec![PersonalExclusion::FlyingEntry];
            let impossible = result(&req);
            assert_eq!(impossible.status, OptimizerStatus::InfeasibleProven);
            assert_eq!(impossible.examined_programs, 0);
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

fn axel_sequence() -> ProgramVariant {
    ProgramVariant {
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
    }
}

#[test]
fn assumed_silver_axel_sequence_reaches_optimizer_scoring_and_shared_validation() {
    let mut req = request(
        AdultCategory::Silver,
        vec![
            axel_sequence(),
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
    assert_eq!(
        r.effective_objective,
        ProgramSearchObjective::UnfactoredBaseValueHundredths
    );
    assert_eq!(
        r.candidates[0].listed_per_element,
        r.candidates[0].per_element
    );
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
    assert!(matches!(
        optimize_program_with_assumed_recognition(
            &ProfileRegistry::adult(),
            &req,
            &["1T+1A+SEQ", "SSpB"].map(AssumedRecognition::new),
        ),
        Err(ProgramOptimizationError::UnsupportedVariant(0)
            | ProgramOptimizationError::UnresolvedValidation(_)
            | ProgramOptimizationError::Scoring(BaseValueError::Unsupported))
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
    // No ten-jump program is eligible for Bronze.
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
    let registry = ProfileRegistry::adult();
    let generation = generation_defaults::request(adult_profile(category));
    let choices =
        ["CCoSp2", "CCoSp3", "FSSp2", "StSq2", "ChSp1", "1A"].map(AssumedRecognition::new);
    let generated =
        generate_program_candidates_with_assumed_recognition(&registry, &generation, &choices)
            .unwrap();
    let q = request(category, generated.domain.variants, 5);
    let result =
        optimize_program_with_assumed_recognition(&registry, &q, &generated.recognition).unwrap();
    // Communication 2786, 2 June 2026. Gold requires the Axel, both spin roles,
    // the step sequence and the choreographic spin in this five-slot fixture.
    let expected = oracle(5, &[300, 360, 280, 270, 350, 110], |word| {
        (2..6).all(|i| word.contains(&i)) && (word.contains(&0) || word.contains(&1))
    });
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(ids(&result), expected.1);
    assert!(result.supported_rules.contains(&"program-step-level-cap"));
    assert!(result
        .supported_rules
        .contains(&"program-choreographic-plan-consistency"));
    assert!(!result.supported_rules.contains(&"spin-minimum-revolutions"));
    assert!(!result
        .supported_rules
        .contains(&"program-spin-level-credit"));
    assert_eq!(ids(&result).len(), result.candidates.len());
    for candidate in &result.candidates {
        let elements = candidate
            .variant_indices
            .iter()
            .map(|&index| choices[index].clone())
            .collect::<Vec<_>>();
        assert!(elements.iter().all(|selection| selection.code != "CCoSp2"));
        let manual = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                profile: q.profile.clone(),
                scoring_table: q.scoring_table,
                submission: SubmissionState::Complete,
                elements,
            },
        )
        .unwrap();
        assert_eq!(candidate.base_hundredths, expected.0);
        assert_eq!(manual.total.exact_hundredths, Some(i64::from(expected.0)));
    }
}

#[test]
fn assumed_senior_full_program_matches_manual_bv() {
    let category = SeniorCategory::Women;
    let profile = senior_profile(category);
    let registry = ProfileRegistry::senior();
    let choices =
        ["2A", "CCoSp4", "FCSp4", "StSq4", "ChSq1", "ChSp1", "FLSp4"].map(AssumedRecognition::new);
    let generation = generation_defaults::request(profile.clone());
    let generated =
        generate_program_candidates_with_assumed_recognition(&registry, &generation, &choices)
            .unwrap();
    assert_eq!(generated.recognition, choices);
    let mut q = request(AdultCategory::Gold, generated.domain.variants, 6);
    q.profile = profile.id;
    q.scoring_table = profile.scoring_table;
    q.policy = senior_program_policy(category);
    q.must_have = vec![vec![0], vec![1], vec![2, 6], vec![3], vec![4], vec![5]];
    q.max_solutions = 1500;
    let result = optimize_program_with_assumed_recognition(&registry, &q, &choices).unwrap();
    // Communication 2786: the two flying-spin choices share BV 3.80.
    let expected = oracle(6, &[330, 420, 380, 410, 350, 350, 380], |word| {
        [0, 1, 3, 4, 5].iter().all(|i| word.contains(i)) && (word.contains(&2) || word.contains(&6))
    });
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(ids(&result).len(), result.candidates.len());
    assert_eq!(ids(&result), expected.1);
    let candidate = &result.candidates[0];
    assert_eq!(candidate.base_hundredths, expected.0);
    let manual = assess_manual_bv(
        &registry,
        &ManualBvRequest {
            profile: q.profile,
            scoring_table: q.scoring_table,
            submission: SubmissionState::Complete,
            elements: candidate
                .variant_indices
                .iter()
                .map(|&i| choices[i].clone())
                .collect(),
        },
    )
    .unwrap();
    assert_eq!(
        manual.total.exact_hundredths,
        Some(i64::from(candidate.base_hundredths))
    );
}

#[test]
fn assumed_senior_credited_optima_match_literal_ordered_oracle() {
    let category = SeniorCategory::Women;
    let profile = senior_profile(category);
    let registry = ProfileRegistry::senior();
    let mut choices =
        ["3A", "3Lz", "CCoSp4", "FCSp4", "StSq4", "ChSq1", "ChSp1"].map(AssumedRecognition::new);
    let generated = generate_program_candidates_with_assumed_recognition(
        &registry,
        &generation_defaults::request(profile.clone()),
        &choices,
    )
    .unwrap();
    let mut q = request(AdultCategory::Gold, generated.domain.variants, 7);
    choices[1] = AssumedRecognition::new("3Lz+2T");
    q.variants[1].element = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(3),
        }),
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(2),
        }),
    ]));
    q.profile = profile.id;
    q.scoring_table = profile.scoring_table;
    q.policy = senior_program_policy(category);
    q.must_have = (2..7).map(|i| vec![i]).collect();
    q.max_solutions = 6000;
    for (excluded, jump_total) in [(false, 1520), (true, 1440)] {
        q.exclusions = if excluded {
            vec![PersonalExclusion::Variant(1)]
        } else {
            vec![]
        };
        let valid = |word: &[usize]| {
            (2..7).all(|i| word.iter().filter(|&&j| j == i).count() == 1)
                && word.contains(&0)
                && (!excluded || !word.contains(&1))
        };
        let raw = oracle(7, &[800, 720, 420, 380, 410, 350, 350], valid);
        let (best, ties) = word_oracle(7, 7, |word| {
            valid(word).then(|| {
                let jump_score = if word.contains(&1) {
                    800 + 720
                } else {
                    800 + 640
                };
                jump_score + 1910
            })
        });
        // The best raw word repeats 3A; the credited winner can differ.
        assert_eq!(raw.0, 1600 + 1910);
        assert!(raw.1.iter().all(|word| !word.contains(&1)));
        assert_eq!(raw.1 == ties, excluded);
        assert_eq!(best, jump_total + 1910);
        assert_ne!(raw.0, best);
        if excluded {
            let mut bounded = q.clone();
            // The first two deterministic proposals fail whole-program validation.
            bounded.max_proposals = 3;
            let stopped =
                optimize_program_with_assumed_recognition(&registry, &bounded, &choices).unwrap();
            assert_eq!(stopped.status, OptimizerStatus::FeasibleUnproven);
            assert_eq!(
                stopped.termination_reason,
                JumpTerminationReason::SearchLimit
            );
            assert!(!stopped.enumeration_complete);
            assert_eq!(stopped.candidates.len(), 1);
            let candidate = &stopped.candidates[0];
            assert_eq!(candidate.base_hundredths, best);
            assert!(ties.contains(&candidate.variant_indices));
        }
        let result = optimize_program_with_assumed_recognition(&registry, &q, &choices).unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert_eq!(
            result.effective_objective,
            ProgramSearchObjective::CreditedBaseValueHundredths
        );
        assert!(result.enumeration_complete);
        assert_eq!(ids(&result), ties);
        assert_eq!(result.candidates.len(), ties.len());
        for candidate in result.candidates {
            assert_eq!(candidate.base_hundredths, best);
            let listed_axels: Vec<_> = candidate
                .listed_per_element
                .iter()
                .filter(|v| v.notation == "3A")
                .map(|v| v.hundredths)
                .collect();
            assert_eq!(
                listed_axels,
                if excluded { vec![800, 800] } else { vec![800] }
            );
            let axels = candidate
                .per_element
                .iter()
                .filter(|v| v.notation.starts_with("3A"))
                .map(|v| v.hundredths)
                .collect::<Vec<_>>();
            assert_eq!(axels, if excluded { vec![800, 640] } else { vec![800] });
        }
    }
}
