use skating_program_lab::*;
#[path = "generation_defaults.rs"]
mod generation_defaults;

pub fn domain(category: AdultCategory, broad: bool) -> CandidateGenerationRequest {
    let mut g = generation_defaults::request(adult_profile(category));
    g.jump_kinds = if broad {
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
    };
    g.rotations = if broad { vec![1, 2] } else { vec![1] };
    g.solo_jumps = true;
    g.direct_combinations = broad;
    g.base_spins = vec![SpinAbbreviation {
        kind: RegularSpinKind::Sit,
        entry: SpinEntry::NonFlying,
        change_foot: false,
    }];
    g.spin_plans = if broad {
        vec![
            qualified_spin(RegularSpinKind::Camel),
            qualified_spin(RegularSpinKind::Sit),
        ]
    } else {
        vec![]
    };
    g.movement_plans = generation_defaults::basic_movements();
    g
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
