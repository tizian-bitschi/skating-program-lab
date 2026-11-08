#![cfg(feature = "optimizer")]
use skating_program_lab::*;

fn request(c: AdultCategory) -> CandidateGenerationRequest {
    let p = adult_profile(c);
    CandidateGenerationRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        season: p.season,
        profile: p.id,
        scoring_table: p.scoring_table,
        discipline: ProfileDiscipline::Single,
        segment: ProgramSegment::FreeSkating,
        jump_kinds: vec![
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
            JumpKind::Axel,
        ],
        rotations: vec![1, 2],
        solo_jumps: true,
        direct_combinations: false,
        axel_sequences: false,
        confirmed_sequence_transitions: vec![],
        max_compound_length: 3,
        base_spins: vec![],
        spin_plans: vec![],
        movement_plans: vec![],
        manual_variants: vec![],
        exclusions: vec![],
        excluded_spin_positions: vec![],
        excluded_spin_entries: vec![],
        skill_limits: PersonalSkillLimits::default(),
        recognition_assumed: true,
        max_candidates: 1000,
        max_expansions: 100_000,
    }
}
fn generate(r: &CandidateGenerationRequest) -> CandidateGenerationResult {
    generate_program_candidates(&ProfileRegistry::adult(), r).unwrap()
}
fn codes(r: &CandidateGenerationResult) -> Vec<String> {
    r.variants
        .iter()
        .map(|v| match &v.element {
            PlannedElement::Jump(j) => {
                let one = |j: &ListedJump| {
                    format!(
                        "{}{}",
                        j.rotations.unwrap(),
                        match j.kind.unwrap() {
                            JumpKind::ToeLoop => "T",
                            JumpKind::Salchow => "S",
                            JumpKind::Loop => "Lo",
                            JumpKind::Flip => "F",
                            JumpKind::Lutz => "Lz",
                            JumpKind::Axel => "A",
                        }
                    )
                };
                match j {
                    JumpElement::Solo(j) => one(j),
                    JumpElement::Combination(p) | JumpElement::Sequence(p) => format!(
                        "{}{}",
                        p.iter()
                            .map(|p| if let JumpPart::Jump(j) = p {
                                one(j)
                            } else {
                                panic!()
                            })
                            .collect::<Vec<_>>()
                            .join("+"),
                        if matches!(j, JumpElement::Sequence(_)) {
                            "+SEQ"
                        } else {
                            ""
                        }
                    ),
                }
            }
            _ => format!("{:?}", v.element),
        })
        .collect()
}
#[test]
fn four_profiles_match_independent_solo_oracle_in_order() {
    for (c, expected) in [
        (AdultCategory::Bronze, vec!["1T", "1S", "1Lo", "1F", "1Lz"]),
        (
            AdultCategory::Silver,
            vec!["1T", "1S", "1Lo", "1F", "1Lz", "1A"],
        ),
        (
            AdultCategory::Gold,
            vec!["1T", "1S", "1Lo", "1F", "1Lz", "1A", "2T", "2S", "2Lo"],
        ),
        (
            AdultCategory::Masters,
            vec![
                "1T", "1S", "1Lo", "1F", "1Lz", "1A", "2T", "2S", "2Lo", "2F", "2Lz", "2A",
            ],
        ),
    ] {
        let r = generate(&request(c));
        assert_eq!(codes(&r), expected);
        assert_eq!(
            r.audit.completeness,
            CandidateDomainCompleteness::CompleteDeclaredDomain
        );
        assert!(!r.audit.sources.is_empty());
    }
}
#[test]
fn tiny_compound_domain_has_all_equal_value_alternatives_and_stable_order() {
    let mut q = request(AdultCategory::Silver);
    q.jump_kinds = vec![JumpKind::Loop, JumpKind::ToeLoop];
    q.rotations = vec![1];
    q.direct_combinations = true;
    q.max_compound_length = 2;
    assert_eq!(
        codes(&generate(&q)),
        ["1T", "1Lo", "1T+1T", "1T+1Lo", "1Lo+1T", "1Lo+1Lo"]
    );
    q.max_compound_length = 3;
    // Three identical listed jumps violate the shared repetition limit.
    assert_eq!(generate(&q).variants.len(), 12);
    q.profile = adult_profile(AdultCategory::Masters).id;
    assert_eq!(
        codes(&generate(&q)),
        [
            "1T",
            "1Lo",
            "1T+1T",
            "1T+1Lo",
            "1Lo+1T",
            "1Lo+1Lo",
            "1T+1T+1Lo",
            "1T+1Lo+1T",
            "1T+1Lo+1Lo",
            "1Lo+1T+1T",
            "1Lo+1T+1Lo",
            "1Lo+1Lo+1T",
        ]
    );
}
#[test]
fn component_filters_cover_combinations_and_sequences() {
    let mut q = request(AdultCategory::Masters);
    q.direct_combinations = true;
    q.axel_sequences = true;
    q.confirmed_sequence_transitions = vec![vec![
        ListedJump {
            kind: Some(JumpKind::Salchow),
            rotations: Some(1),
        },
        ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        },
    ]];
    q.exclusions = vec![
        PersonalExclusion::JumpKind(JumpKind::ToeLoop),
        PersonalExclusion::JumpKind(JumpKind::Lutz),
    ];
    let r = generate(&q);
    assert!(codes(&r)
        .iter()
        .all(|s| !s.contains('T') && !s.contains("Lz")));
    assert!(codes(&r).iter().any(|s| s == "1S+1A+SEQ"));
    q.exclusions.push(PersonalExclusion::Jump(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(1),
    }));
    assert!(codes(&generate(&q)).iter().all(|s| !s.contains("1A")));
}
fn abbreviation(kind: RegularSpinKind, entry: SpinEntry) -> SpinAbbreviation {
    SpinAbbreviation {
        kind,
        entry,
        change_foot: false,
    }
}
#[test]
fn automatic_spins_filters_and_missing_qualification_remain_explicit() {
    let mut q = request(AdultCategory::Silver);
    q.jump_kinds.clear();
    q.base_spins = vec![
        abbreviation(RegularSpinKind::Camel, SpinEntry::NonFlying),
        abbreviation(RegularSpinKind::Sit, SpinEntry::Flying),
        abbreviation(RegularSpinKind::Layback, SpinEntry::NonFlying),
    ];
    let r = generate(&q);
    assert_eq!(r.variants.len(), 2);
    assert!(r
        .audit
        .omissions
        .iter()
        .any(|o| o.reason.contains("spin-layback-position")));
    q.exclusions = vec![
        PersonalExclusion::FlyingEntry,
        PersonalExclusion::SpinKind(RegularSpinKind::Layback),
    ];
    assert_eq!(generate(&q).variants.len(), 1);
    let ProgramVariant {
        element: PlannedElement::RegularSpin(plan),
        evidence: ElementPlanEvidence::Spin { mut spin, .. },
        ..
    } = r.variants[0].clone()
    else {
        panic!()
    };
    spin.planned_level = Some(Level::One);
    q.base_spins.clear();
    q.spin_plans = vec![SpinGenerationPlan {
        plan: plan.clone(),
        evidence: spin.clone(),
        local_feature_qualification_assumed: false,
    }];
    assert!(generate(&q).variants.is_empty());
    spin.feature_attempts = Some(vec![SpinFeatureAttempt {
        feature: SpinFeature::IncreaseOfSpeed,
        position_step: 0,
        outcome: FeatureAttemptOutcome::Successful,
    }]);
    q.spin_plans[0].evidence = spin;
    assert!(generate(&q).variants.is_empty());
    q.spin_plans[0].local_feature_qualification_assumed = true;
    assert_eq!(generate(&q).variants.len(), 1);
    q.exclusions
        .push(PersonalExclusion::SpinFeature(SpinFeature::IncreaseOfSpeed));
    assert!(generate(&q).variants.is_empty());
    q.exclusions.clear();
    q.spin_plans[0].evidence.feature_attempts.as_mut().unwrap()[0].feature =
        SpinFeature::DifficultVariation;
    assert!(generate(&q).audit.omissions[0]
        .reason
        .contains("qualification"));
}
#[test]
fn caps_empty_domains_errors_and_manual_dedup_are_distinct() {
    let mut q = request(AdultCategory::Bronze);
    q.max_candidates = 2;
    assert_eq!(
        generate(&q).audit.completeness,
        CandidateDomainCompleteness::IncompleteCandidateCap
    );
    q.max_candidates = 1000;
    q.max_expansions = 2;
    assert_eq!(
        generate(&q).audit.completeness,
        CandidateDomainCompleteness::IncompleteExpansionCap
    );
    q.max_expansions = 100_000;
    q.jump_kinds.clear();
    let empty = generate(&q);
    assert!(empty.variants.is_empty());
    assert_eq!(
        empty.audit.completeness,
        CandidateDomainCompleteness::CompleteDeclaredDomain
    );
    q.jump_kinds = vec![JumpKind::ToeLoop];
    q.rotations = vec![1];
    let original = generate(&q).variants[0].clone();
    q.manual_variants = vec![original.clone(), original];
    assert_eq!(generate(&q).variants.len(), 1);
    q.manual_variants[0].recognition_assumed = false;
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::InvalidManualVariant { index: 0, .. })
    ));
    q.manual_variants.clear();
    q.discipline = ProfileDiscipline::Pair;
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::UnsupportedGenerator(_))
    ));
    q.discipline = ProfileDiscipline::Single;
    q.profile.name = "unknown".into();
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::Profile(_))
    ));
}
#[test]
fn generated_variants_feed_existing_optimizer_unchanged() {
    let mut q = request(AdultCategory::Bronze);
    q.jump_kinds = vec![JumpKind::ToeLoop, JumpKind::Salchow];
    q.rotations = vec![1];
    q.base_spins = vec![abbreviation(RegularSpinKind::Sit, SpinEntry::NonFlying)];
    let generated = generate(&q);
    let result = optimize_program(
        &ProfileRegistry::adult(),
        &ProgramOptimizationRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            season: q.season,
            profile: q.profile,
            scoring_table: q.scoring_table,
            policy: adult_program_policy(AdultCategory::Bronze),
            variants: generated.variants,
            exclusions: vec![],
            must_have: vec![],
            min_elements: 2,
            max_elements: 2,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 1000,
            max_solutions: 100,
            enumerate_equal_optima: true,
            time_limit: None,
        },
    )
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(result.candidates[0].base_hundredths, 170);
    assert_eq!(result.candidates.len(), 4); // .40 T/S + 1.30 SSpB, both orders.
}
#[test]
fn movement_templates_are_profile_checked_and_evidence_bound() {
    let mut q = request(AdultCategory::Gold);
    q.jump_kinds.clear();
    q.movement_plans = vec![
        ElementPlanEvidence::Step {
            level: Level::Basic,
            clean_difficult_turns: Some(2),
            full_ice_coverage: Some(true),
            movement: Some(SkatingMovement::Spiral),
            qualified_features: Some(vec![]),
            qualification_assumed: true,
        },
        ElementPlanEvidence::ChoreographicSpin {
            consecutive_revolutions_per_foot: Some(vec![3]),
        },
        ElementPlanEvidence::ChoreographicSequence {
            movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::InaBauer]),
            clearly_visible: Some(true),
        },
    ];
    let r = generate(&q);
    assert_eq!(r.variants.len(), 2); // Gold StSqB and ChSp1, not ChSq1.
    assert!(matches!(
        r.variants[0].element,
        PlannedElement::StepSequence
    ));
    assert!(matches!(
        r.variants[1].element,
        PlannedElement::ChoreographicSpin
    ));
    q.profile = adult_profile(AdultCategory::Bronze).id;
    let r = generate(&q);
    assert_eq!(r.variants.len(), 1);
    assert!(matches!(
        r.variants[0].element,
        PlannedElement::ChoreographicSequence { .. }
    ));
    q.movement_plans = vec![ElementPlanEvidence::ChoreographicSequence {
        movements: None,
        clearly_visible: None,
    }];
    assert!(generate(&q).variants.is_empty());
    q.movement_plans = vec![ElementPlanEvidence::Jump {
        direct_transitions_assumed: false,
    }];
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::InvalidRequest(_))
    ));
}

#[test]
fn manual_unknown_transitions_and_skill_limits_do_not_establish_feasibility() {
    let mut q = request(AdultCategory::Silver);
    q.direct_combinations = true;
    q.rotations = vec![1];
    q.skill_limits.max_jump_rotations = Some(1);
    q.base_spins = vec![abbreviation(RegularSpinKind::Camel, SpinEntry::NonFlying)];
    q.skill_limits.max_position_revolutions = Some(7);
    assert!(generate(&q)
        .variants
        .iter()
        .all(|v| matches!(v.element, PlannedElement::Jump(_))));
    q.manual_variants = vec![ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Combination(vec![
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::ToeLoop),
                rotations: Some(1),
            }),
            JumpPart::Euler,
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::Salchow),
                rotations: Some(1),
            }),
        ])),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
        recognition_assumed: true,
    }];
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::InvalidManualVariant { .. })
    ));
    q.manual_variants.clear();
    q.max_candidates = 1001;
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::InvalidRequest(_))
    ));
}

#[test]
fn sequence_generation_requires_exact_transition_confirmation() {
    let mut q = request(AdultCategory::Silver);
    q.jump_kinds = vec![JumpKind::Salchow, JumpKind::ToeLoop, JumpKind::Axel];
    q.rotations = vec![1];
    q.solo_jumps = false;
    q.axel_sequences = true;
    q.max_compound_length = 2;
    let r = generate(&q);
    assert!(r.variants.is_empty());
    assert!(!r.audit.omissions.is_empty());
    q.confirmed_sequence_transitions = vec![vec![
        ListedJump {
            kind: Some(JumpKind::Salchow),
            rotations: Some(1),
        },
        ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        },
    ]];
    assert_eq!(codes(&generate(&q)), ["1S+1A+SEQ"]);
}

#[test]
fn position_and_entry_filters_cover_automatic_and_manual_combination_spins() {
    let mut q = request(AdultCategory::Gold);
    q.jump_kinds.clear();
    q.base_spins = vec![
        SpinAbbreviation {
            kind: RegularSpinKind::Combination,
            entry: SpinEntry::NonFlying,
            change_foot: true,
        },
        abbreviation(RegularSpinKind::Sit, SpinEntry::Flying),
    ];
    let variants = generate(&q).variants;
    assert_eq!(variants.len(), 2);
    q.manual_variants = variants;
    q.excluded_spin_positions = vec![SpinPosition::Camel];
    let r = generate(&q);
    assert_eq!(r.variants.len(), 1);
    assert_eq!(r.audit.excluded_spin_positions, [SpinPosition::Camel]);
    assert!(
        matches!(&r.variants[0].element, PlannedElement::RegularSpin(s) if s.entry == Some(SpinEntry::Flying))
    );
    q.excluded_spin_positions.clear();
    q.excluded_spin_entries = vec![SpinEntry::NonFlying];
    assert_eq!(generate(&q).variants.len(), 1);
    q.excluded_spin_entries = vec![SpinEntry::Flying];
    let r = generate(&q);
    assert_eq!(r.variants.len(), 1);
    assert!(
        matches!(&r.variants[0].element, PlannedElement::RegularSpin(s) if s.entry == Some(SpinEntry::NonFlying))
    );
}

#[test]
fn manual_inputs_are_validated_and_prioritized_independently_of_caps() {
    let mut q = request(AdultCategory::Bronze);
    q.jump_kinds = vec![JumpKind::ToeLoop, JumpKind::Salchow];
    q.rotations = vec![1];
    let mut manual = generate(&q).variants[1].clone();
    q.manual_variants = vec![manual.clone()];
    q.max_candidates = 1;
    q.max_expansions = 1;
    let r = generate(&q);
    assert_eq!(codes(&r), ["1S"]);
    assert_eq!(r.audit.manual_definitions_checked, 1);
    assert_ne!(
        r.audit.completeness,
        CandidateDomainCompleteness::CompleteDeclaredDomain
    );
    manual.recognition_assumed = false;
    q.manual_variants.push(manual);
    assert!(matches!(
        generate_program_candidates(&ProfileRegistry::adult(), &q),
        Err(CandidateGenerationError::InvalidManualVariant { index: 1, .. })
    ));
    q.manual_variants = generate(&request(AdultCategory::Bronze)).variants[..2].to_vec();
    let r = generate(&q);
    assert_eq!(r.audit.manual_definitions_checked, 2);
    assert!(r
        .audit
        .omissions
        .iter()
        .any(|o| o.reason == "manual variant omitted by candidate cap"));
}

#[test]
fn equivalent_step_feature_sets_deduplicate_before_consuming_capacity() {
    let mut q = request(AdultCategory::Masters);
    q.jump_kinds.clear();
    q.max_candidates = 1;
    let step = |features| ElementPlanEvidence::Step {
        level: Level::Two,
        clean_difficult_turns: Some(2),
        full_ice_coverage: Some(true),
        movement: Some(SkatingMovement::Spiral),
        qualified_features: Some(features),
        qualification_assumed: true,
    };
    q.movement_plans = vec![step(vec![1, 2]), step(vec![2, 1, 1])];
    let r = generate(&q);
    assert_eq!(r.variants.len(), 1);
    assert_eq!(
        r.audit.completeness,
        CandidateDomainCompleteness::CompleteDeclaredDomain
    );
    assert!(
        matches!(&r.variants[0].evidence, ElementPlanEvidence::Step { qualified_features: Some(f), .. } if f == &[1, 2])
    );
    q.manual_variants = vec![ProgramVariant {
        element: PlannedElement::StepSequence,
        evidence: step(vec![2, 1]),
        recognition_assumed: true,
    }];
    assert_eq!(generate(&q).variants.len(), 1);
}

#[test]
fn generation_admits_assumed_gold_levels_and_applies_semantic_filters() {
    let mut q = request(AdultCategory::Gold);
    q.solo_jumps = false;
    q.manual_variants = vec![ProgramVariant {
        element: PlannedElement::RegularSpin(RegularSpin {
            entry: None,
            steps: None,
        }),
        evidence: ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence::default(),
            local_feature_qualification_assumed: false,
        },
        recognition_assumed: false,
    }];
    let choices = [AssumedRecognition::new("FSSp2")];
    let registry = ProfileRegistry::adult();
    let result =
        generate_program_candidates_with_assumed_recognition(&registry, &q, &choices).unwrap();
    assert_eq!(result.domain.variants.len(), 1);
    assert_eq!(result.input_indices, [0]);
    assert_eq!(result.recognition, choices);
    assert_eq!(
        result.domain.variants[0].element,
        q.manual_variants[0].element
    );
    q.exclusions.push(PersonalExclusion::FlyingEntry);
    let result =
        generate_program_candidates_with_assumed_recognition(&registry, &q, &choices).unwrap();
    assert!(result.domain.variants.is_empty());
    assert_eq!(result.domain.audit.omissions.len(), 1);
}

#[test]
fn exact_codes_generate_eligible_levels_without_templates() {
    let registry = ProfileRegistry::adult();
    let choices = [
        "CCoSpB", "CCoSp2", "CCoSp3", "FSSp2", "StSq2", "ChSp1", "1A",
    ]
    .map(AssumedRecognition::new);
    let mut gold = request(AdultCategory::Gold);
    gold.solo_jumps = false;
    let generated =
        generate_program_candidates_with_assumed_recognition(&registry, &gold, &choices).unwrap();
    assert_eq!(generated.recognition, choices);
    assert!(generated.domain.variants[..4]
        .iter()
        .all(|variant| matches!(
            (&variant.element, &variant.evidence),
            (
                PlannedElement::RegularSpin(RegularSpin {
                    entry: None,
                    steps: None
                }),
                ElementPlanEvidence::Spin {
                    spin: RegularSpinEvidence {
                        feature_attempts: None,
                        ..
                    },
                    ..
                }
            )
        )));
    gold.exclusions.push(PersonalExclusion::FlyingEntry);
    let filtered =
        generate_program_candidates_with_assumed_recognition(&registry, &gold, &choices).unwrap();
    assert_eq!(filtered.input_indices, [0, 1, 2, 4, 5, 6]);

    let selected = ["CCoSp2", "CCoSp3", "1T"].map(AssumedRecognition::new);
    for (category, expected) in [
        (AdultCategory::Silver, vec![0, 2]),
        (AdultCategory::Bronze, vec![2]),
    ] {
        let mut q = request(category);
        q.solo_jumps = false;
        let result =
            generate_program_candidates_with_assumed_recognition(&registry, &q, &selected).unwrap();
        assert_eq!(result.input_indices, expected);
        assert_eq!(
            result.domain.audit.omissions.len(),
            selected.len() - expected.len()
        );
    }
}

#[test]
fn exact_codes_generate_eligible_masters_level_four() {
    let mut q = request(AdultCategory::Masters);
    q.solo_jumps = false;
    let choices = ["CCoSp4", "FSSp4", "StSq4"].map(AssumedRecognition::new);
    let generated = generate_program_candidates_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &q,
        &choices,
    )
    .unwrap();
    assert_eq!(generated.recognition, choices);
    assert_eq!(generated.domain.variants.len(), choices.len());
}

#[test]
fn generation_preserves_explicit_axel_transition_assumptions() {
    let mut q = request(AdultCategory::Silver);
    q.solo_jumps = false;
    let jump = |kind| {
        JumpPart::Jump(ListedJump {
            kind: Some(kind),
            rotations: Some(1),
        })
    };
    q.manual_variants.push(ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Sequence(vec![
            jump(JumpKind::ToeLoop),
            jump(JumpKind::Axel),
        ])),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: true,
        },
        recognition_assumed: false,
    });
    let result = generate_program_candidates_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &q,
        &[AssumedRecognition::new("1T+1A+SEQ")],
    )
    .unwrap();
    assert_eq!(
        result.domain.variants[0].evidence,
        q.manual_variants[0].evidence
    );
}

#[test]
fn generation_rejects_saved_credit_above_the_profile_cap() {
    let mut q = request(AdultCategory::Silver);
    q.solo_jumps = false;
    q.manual_variants.push(ProgramVariant {
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
    let result = generate_program_candidates_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &q,
        &[AssumedRecognition::new("CCoSp3")],
    );
    assert!(matches!(
        result,
        Err(CandidateGenerationError::InvalidManualVariant { index: 0, reason })
            if reason.contains("CCoSp3") && reason.contains("exceeds the profile cap")
    ));
}
