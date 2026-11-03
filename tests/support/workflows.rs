use skating_program_lab::*;

pub fn domain(category: AdultCategory, broad: bool) -> CandidateGenerationRequest {
    let p = adult_profile(category);
    CandidateGenerationRequest {
        season: p.season,
        profile: p.id,
        scoring_table: p.scoring_table,
        discipline: ProfileDiscipline::Single,
        segment: ProgramSegment::FreeSkating,
        jump_kinds: if broad {
            vec![
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
                JumpKind::Lutz,
                JumpKind::Axel,
            ]
        } else {
            vec![JumpKind::ToeLoop, JumpKind::Salchow]
        },
        rotations: if broad { vec![1, 2] } else { vec![1] },
        solo_jumps: true,
        direct_combinations: broad,
        axel_sequences: false,
        confirmed_sequence_transitions: vec![],
        max_compound_length: 2,
        base_spins: vec![SpinAbbreviation {
            kind: RegularSpinKind::Sit,
            entry: SpinEntry::NonFlying,
            change_foot: false,
        }],
        spin_plans: if broad {
            vec![
                qualified_spin(RegularSpinKind::Camel),
                qualified_spin(RegularSpinKind::Sit),
            ]
        } else {
            vec![]
        },
        movement_plans: vec![
            ElementPlanEvidence::ChoreographicSequence {
                movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::SpreadEagle]),
                clearly_visible: Some(true),
            },
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
        ],
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

fn qualified_spin(kind: RegularSpinKind) -> SpinGenerationPlan {
    SpinGenerationPlan {
        plan: RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: Some(vec![SpinStep::Position {
                position: Some(if kind == RegularSpinKind::Camel {
                    SpinPosition::Camel
                } else {
                    SpinPosition::Sit
                }),
                revolutions: Some(8),
                feature_attempts: Some(vec![]),
            }]),
        },
        evidence: RegularSpinEvidence {
            abbreviation: Some(SpinAbbreviation {
                kind,
                entry: SpinEntry::NonFlying,
                change_foot: false,
            }),
            planned_level: Some(Level::One),
            feature_attempts: Some(vec![SpinFeatureAttempt {
                feature: SpinFeature::IncreaseOfSpeed,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            }]),
            ..Default::default()
        },
        local_feature_qualification_assumed: true,
    }
}

pub fn optimization(
    q: &CandidateGenerationRequest,
    variants: Vec<ProgramVariant>,
    slots: usize,
) -> ProgramOptimizationRequest {
    ProgramOptimizationRequest {
        season: q.season,
        profile: q.profile.clone(),
        scoring_table: q.scoring_table,
        policy: adult_program_policy(if q.profile == adult_profile(AdultCategory::Bronze).id {
            AdultCategory::Bronze
        } else if q.profile == adult_profile(AdultCategory::Silver).id {
            AdultCategory::Silver
        } else if q.profile == adult_profile(AdultCategory::Gold).id {
            AdultCategory::Gold
        } else {
            AdultCategory::Masters
        }),
        variants,
        exclusions: vec![],
        must_have: vec![],
        min_elements: slots,
        max_elements: slots,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        equivalence: ProgramEquivalence::OrderedVariants,
        max_proposals: 100_000,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    }
}
