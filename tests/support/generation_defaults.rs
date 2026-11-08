use skating_program_lab::*;

// Empty finite selections: callers must choose every candidate family explicitly.
pub fn request(profile: RuleProfile) -> CandidateGenerationRequest {
    CandidateGenerationRequest {
        rotation_preference: RotationPreference::default(),
        season: profile.season,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        discipline: ProfileDiscipline::Single,
        segment: ProgramSegment::FreeSkating,
        jump_kinds: vec![],
        rotations: vec![],
        solo_jumps: false,
        direct_combinations: false,
        axel_sequences: false,
        confirmed_sequence_transitions: vec![],
        max_compound_length: 2,
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

pub fn basic_movements() -> Vec<ElementPlanEvidence> {
    vec![
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
    ]
}
