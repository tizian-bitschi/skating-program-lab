use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const RULESET: RulesetId = RulesetId::IsuSenior2026_27June12;

fn listed(kind: JumpKind, rotations: u8) -> ListedJump {
    ListedJump {
        kind: Some(kind),
        rotations: Some(rotations),
    }
}
fn solo(kind: JumpKind, rotations: u8) -> PlannedElement {
    PlannedElement::Jump(JumpElement::Solo(listed(kind, rotations)))
}
fn compound(jumps: &[(JumpKind, u8)], sequence: bool) -> PlannedElement {
    let parts = jumps
        .iter()
        .map(|(k, n)| JumpPart::Jump(listed(*k, *n)))
        .collect();
    PlannedElement::Jump(if sequence {
        JumpElement::Sequence(parts)
    } else {
        JumpElement::Combination(parts)
    })
}
fn program(elements: Vec<PlannedElement>) -> ProfileProgram {
    ProfileProgram {
        profile: senior_profile(SeniorCategory::Women).id,
        scoring_table: TABLE,
        elements,
    }
}
fn findings(elements: Vec<PlannedElement>) -> Vec<Finding> {
    validate_profile_program(&ProfileRegistry::senior(), &program(elements))
        .unwrap()
        .validation
        .findings
}
fn violates(elements: Vec<PlannedElement>, rule: &str) -> bool {
    findings(elements)
        .iter()
        .any(|f| f.rule_id == rule && f.kind == FindingKind::Violation)
}
fn panel(codes: &[&str], level: Option<Level>) -> PanelRecognition {
    PanelRecognition {
        ruleset: RULESET,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                codes
                    .iter()
                    .map(|s| ComponentCode::try_from(*s).unwrap())
                    .collect(),
                level,
                Some(vec![]),
            )
            .unwrap(),
        ),
    }
}
fn scored(codes: &[&[&str]]) -> ProfileProgramScore {
    let elements = codes
        .iter()
        .map(|codes| ProgramElement {
            recognition: panel(codes, None),
            annotations: vec![],
            goe: 0,
            fall: Some(false),
            start_ms: Some(10_000),
        })
        .collect::<Vec<_>>();
    score_profile_program(
        &ProfileRegistry::senior(),
        &senior_profile(SeniorCategory::Women).id,
        TABLE,
        &elements,
        Some(0),
        Some(ProgramTiming {
            duration_seconds: 240,
            ordinary_timing_confirmed: true,
            second_half_default_confirmed: true,
        }),
        None,
    )
    .unwrap()
}

#[test]
fn exact_identities_and_shared_technical_definition() {
    let registry = ProfileRegistry::senior();
    let women = senior_profile(SeniorCategory::Women);
    let men = senior_profile(SeniorCategory::Men);
    assert_ne!(women.id, men.id);
    assert_eq!(women.id.name, "isu-senior-2026-27-women-free");
    assert_eq!(women.id.version, "2026-06-12");
    assert_eq!(women.jumps, men.jumps);
    assert_eq!(women.scoring, men.scoring);
    assert_eq!(women.scoring.duration_seconds, 240);
    assert_eq!(women.scoring.recognition_ruleset, RULESET);
    assert_eq!(RULESET.as_str().parse::<RulesetId>().unwrap(), RULESET);
    assert!("isu-senior-2025-26".parse::<RulesetId>().is_err());
    let mut unknown = women.id.clone();
    unknown.version = "2025-26".into();
    assert!(registry.resolve(&unknown, TABLE).is_err());
    assert!(ProfileRegistry::adult().resolve(&women.id, TABLE).is_err());
    for profile in [women, men] {
        assert_eq!(registry.resolve(&profile.id, TABLE).unwrap(), &profile);
        assert!(profile.sources.iter().all(|s| !s.version.contains("2025")));
    }
}

#[test]
fn six_jump_and_eleven_slot_maxima_do_not_require_exact_counts() {
    use JumpKind::*;
    let six = vec![
        solo(Axel, 2),
        solo(ToeLoop, 3),
        solo(Salchow, 3),
        solo(Loop, 3),
        solo(Flip, 3),
        solo(Lutz, 3),
    ];
    assert!(!violates(six.clone(), "senior-jump-count"));
    let mut seven = six.clone();
    seven.push(solo(Axel, 1));
    assert!(violates(seven, "senior-jump-count"));
    let mut eleven = six;
    eleven.extend([
        PlannedElement::RegularSpin(RegularSpin {
            entry: None,
            steps: None,
        }),
        PlannedElement::RegularSpin(RegularSpin {
            entry: None,
            steps: None,
        }),
        PlannedElement::ChoreographicSpin,
        PlannedElement::StepSequence,
        PlannedElement::ChoreographicSequence { movements: None },
    ]);
    assert!(!violates(eleven.clone(), "senior-jump-count"));
    eleven.push(PlannedElement::Unspecified);
    assert!(violates(eleven, "senior-jump-count"));
    assert!(violates(vec![solo(Lutz, 3)], "senior-axel"));
    assert!(!violates(vec![solo(Axel, 1)], "senior-jump-count"));
}

#[test]
fn senior_compounds_and_euler_have_independent_boundaries() {
    use JumpKind::*;
    let first = compound(&[(Lutz, 3), (ToeLoop, 2), (Loop, 2)], false);
    let second = compound(&[(Flip, 3), (ToeLoop, 3)], false);
    assert!(!violates(
        vec![first.clone(), second.clone()],
        "senior-compound-count"
    ));
    assert!(violates(
        vec![
            first.clone(),
            second,
            compound(&[(Salchow, 3), (Loop, 1)], false)
        ],
        "senior-compound-count"
    ));
    assert!(violates(
        vec![
            first,
            compound(&[(Flip, 3), (ToeLoop, 3), (Loop, 1)], false)
        ],
        "senior-compound-structure"
    ));
    assert!(!findings(vec![solo(Lutz, 5)])
        .iter()
        .any(|f| f.rule_id == "senior-eligibility"
            && f.element_index == Some(0)
            && f.kind == FindingKind::Violation));
    assert!(violates(
        vec![compound(&[(Lutz, 5), (ToeLoop, 2)], false)],
        "senior-compound-structure"
    ));
    let euler = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(listed(Lutz, 3)),
        JumpPart::Euler,
        JumpPart::Jump(listed(Salchow, 3)),
    ]));
    let fs = findings(vec![euler.clone()]);
    assert!(fs
        .iter()
        .any(|f| f.rule_id == "senior-euler" && f.kind == FindingKind::Unknown));
    assert!(!fs
        .iter()
        .any(|f| f.rule_id == "senior-euler" && f.kind == FindingKind::Violation));
    assert!(violates(
        vec![euler.clone(), compound(&[(Flip, 2), (Axel, 2)], true)],
        "senior-euler"
    ));
    assert!(violates(vec![euler.clone(), euler], "senior-euler"));
    assert!(violates(
        vec![PlannedElement::Jump(JumpElement::Combination(vec![
            JumpPart::Euler,
            JumpPart::Jump(listed(Salchow, 3)),
            JumpPart::Jump(listed(ToeLoop, 2))
        ]))],
        "senior-euler"
    ));
    let unknown = findings(vec![PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(listed(Lutz, 3)),
        JumpPart::UnspecifiedTransition,
        JumpPart::Jump(listed(ToeLoop, 2)),
    ]))]);
    assert!(unknown
        .iter()
        .any(|f| f.rule_id == "senior-jump-transition" && f.kind == FindingKind::Unknown));
    assert!(!findings(vec![compound(&[(Lutz, 3), (ToeLoop, 2)], false)])
        .iter()
        .any(|f| f.rule_id == "senior-jump-transition"));
}

#[test]
fn repetition_counts_kinds_doubles_and_advanced_types_separately() {
    use JumpKind::*;
    assert!(!violates(vec![solo(ToeLoop, 1); 3], "senior-repetition"));
    assert!(violates(vec![solo(ToeLoop, 1); 4], "senior-repetition"));
    assert!(!violates(vec![solo(ToeLoop, 2); 2], "senior-repetition"));
    assert!(violates(vec![solo(ToeLoop, 2); 3], "senior-repetition"));
    assert!(!violates(
        vec![solo(Lutz, 1), solo(Lutz, 2), solo(Lutz, 3)],
        "senior-repetition"
    ));
    assert!(violates(
        vec![solo(Lutz, 1), solo(Lutz, 2), solo(Lutz, 3), solo(Lutz, 4)],
        "senior-repetition"
    ));
    assert!(!violates(
        vec![
            solo(Lutz, 3),
            solo(Lutz, 3),
            solo(ToeLoop, 4),
            solo(ToeLoop, 4)
        ],
        "senior-repetition"
    ));
    assert!(violates(
        vec![
            solo(Lutz, 4),
            solo(Lutz, 4),
            solo(ToeLoop, 4),
            solo(ToeLoop, 4)
        ],
        "senior-repetition"
    ));
    assert!(violates(
        vec![
            solo(Lutz, 3),
            solo(Lutz, 3),
            solo(Flip, 3),
            solo(Flip, 3),
            solo(ToeLoop, 3),
            solo(ToeLoop, 3)
        ],
        "senior-repetition"
    ));
    assert!(violates(vec![solo(Lutz, 3); 3], "senior-repetition"));
    assert!(violates(vec![solo(Lutz, 4); 3], "senior-repetition"));
    assert!(violates(
        vec![
            solo(ToeLoop, 2),
            compound(&[(Lutz, 3), (ToeLoop, 2)], false),
            solo(ToeLoop, 2)
        ],
        "senior-repetition"
    ));
}

#[test]
fn second_advanced_solo_receives_eighty_percent_but_compound_occurrence_does_not() {
    // Communication 2786: 3Lz 5.90, 4T 9.50, 2T 1.30. Rule 612 p. 71: 80% second solo.
    let result = scored(&[&["3Lz"], &["3Lz"]]);
    assert_eq!(
        result.score.elements[0]
            .as_ref()
            .unwrap()
            .credited_base_hundredths,
        590
    );
    assert_eq!(
        result.score.elements[1]
            .as_ref()
            .unwrap()
            .credited_base_hundredths,
        472
    );
    assert_eq!(result.score.tes_hundredths, Some(1062));
    let quad = scored(&[&["4T"], &["4T"]]);
    assert_eq!(
        quad.score.elements[1]
            .as_ref()
            .unwrap()
            .credited_base_hundredths,
        760
    );
    for codes in [
        vec![&["3Lz"][..], &["3Lz", "2T"][..]],
        vec![&["3Lz", "2T"][..], &["3Lz"][..]],
    ] {
        let result = scored(&codes);
        assert_eq!(result.score.tes_hundredths, Some(1310));
    }
    assert_eq!(scored(&[&["2T"], &["2T"]]).score.tes_hundredths, Some(260));
    let BaseValueOutcome::Value(raw) =
        lookup_base_value(TABLE, &panel(&["3Lz"], None), &[]).unwrap()
    else {
        panic!()
    };
    assert_eq!(raw.hundredths, 590);
    assert!(result.score.fall_deduction.is_err());
    assert_eq!(result.score.time_deduction.as_ref().unwrap().hundredths, 0);
}

fn spin(kind: RegularSpinKind, entry: SpinEntry, n: u16) -> (PlannedElement, ElementPlanEvidence) {
    let positions = if kind == RegularSpinKind::Combination {
        vec![
            (SpinPosition::Camel, 3),
            (SpinPosition::Sit, 3),
            (SpinPosition::Upright, n - 6),
        ]
    } else {
        vec![(SpinPosition::Camel, n)]
    };
    (
        PlannedElement::RegularSpin(RegularSpin {
            entry: Some(entry),
            steps: Some(
                positions
                    .into_iter()
                    .map(|(position, n)| SpinStep::Position {
                        position: Some(position),
                        revolutions: Some(n),
                        feature_attempts: Some(vec![]),
                    })
                    .collect(),
            ),
        }),
        ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry,
                    change_foot: false,
                }),
                planned_level: Some(Level::Basic),
                feature_attempts: Some(vec![]),
                panel: None,
                final_upright_wind_up: None,
            },
            local_feature_qualification_assumed: true,
        },
    )
}
fn skeleton() -> Vec<(PlannedElement, ElementPlanEvidence)> {
    vec![
        (
            solo(JumpKind::Axel, 1),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
        ),
        spin(RegularSpinKind::Combination, SpinEntry::NonFlying, 10),
        spin(RegularSpinKind::Camel, SpinEntry::Flying, 6),
        (
            PlannedElement::ChoreographicSpin,
            ElementPlanEvidence::ChoreographicSpin {
                consecutive_revolutions_per_foot: Some(vec![3]),
            },
        ),
        (
            PlannedElement::StepSequence,
            ElementPlanEvidence::Step {
                level: Level::Basic,
                clean_difficult_turns: Some(2),
                full_ice_coverage: Some(true),
                movement: None,
                qualified_features: Some(vec![]),
                qualification_assumed: true,
            },
        ),
        (
            PlannedElement::ChoreographicSequence { movements: None },
            ElementPlanEvidence::ChoreographicSequence {
                movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::InaBauer]),
                clearly_visible: Some(true),
            },
        ),
    ]
}
fn legality(
    entries: &[(PlannedElement, ElementPlanEvidence)],
    submission: SubmissionState,
) -> ProgramLegalityReport {
    assess_program_legality(
        &ProfileRegistry::senior(),
        &program(entries.iter().map(|e| e.0.clone()).collect()),
        &entries.iter().map(|e| e.1.clone()).collect::<Vec<_>>(),
        &senior_program_policy(SeniorCategory::Women),
        submission,
    )
    .unwrap()
}

#[test]
fn complete_skeleton_roles_levels_and_draft_obligations() {
    let good = skeleton();
    let complete = legality(&good, SubmissionState::Complete);
    assert!(!complete
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation));
    for index in [0, 1, 2, 3, 4, 5] {
        let mut missing = good.clone();
        missing.remove(index);
        let draft = legality(&missing, SubmissionState::Draft);
        assert!(
            !draft.repairable_absences.is_empty(),
            "missing role {index}"
        );
        assert!(legality(&missing, SubmissionState::Complete)
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.kind == FindingKind::Violation));
    }
    let mut short = good.clone();
    short[1] = spin(RegularSpinKind::Combination, SpinEntry::NonFlying, 9);
    assert!(legality(&short, SubmissionState::Complete)
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-minimum-revolutions" && f.kind == FindingKind::Violation));
    let mut fly_combo = good.clone();
    fly_combo[1] = spin(RegularSpinKind::Combination, SpinEntry::Flying, 10);
    fly_combo.remove(2);
    assert!(legality(&fly_combo, SubmissionState::Complete)
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-required-role" && f.kind == FindingKind::Violation));
    let mut unqualified = good;
    if let ElementPlanEvidence::Step {
        level,
        qualification_assumed,
        ..
    } = &mut unqualified[4].1
    {
        *level = Level::Four;
        *qualification_assumed = false;
    }
    assert!(legality(&unqualified, SubmissionState::Complete)
        .rules
        .iter()
        .any(|r| r.rule_id == "program-step-level-credit"
            && r.state == RuleCoverageState::MissingEvidence));
}

#[test]
fn custom_profiles_and_event_bindings_do_not_fall_back_to_adult() {
    let mut profile = senior_profile(SeniorCategory::Women);
    profile.id.name = "synthetic-senior".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.jumps.max_jumps = 2;
    let mut registry = ProfileRegistry::default();
    registry.register(profile.clone()).unwrap();
    let mut p = program(vec![
        solo(JumpKind::Axel, 1),
        solo(JumpKind::ToeLoop, 1),
        solo(JumpKind::Salchow, 1),
    ]);
    p.profile = profile.id.clone();
    assert!(validate_profile_program(&registry, &p)
        .unwrap()
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "senior-jump-count" && f.kind == FindingKind::Violation));
    // Three singles are permitted under Senior; Adult's identical-jump limit is two.
    let elements = vec![solo(JumpKind::ToeLoop, 1); 3];
    assert!(!violates(elements.clone(), "senior-repetition"));
    let adult = adult_profile(AdultCategory::Masters);
    let adult_findings = validate_profile_program(
        &ProfileRegistry::adult(),
        &ProfileProgram {
            profile: adult.id,
            scoring_table: TABLE,
            elements,
        },
    )
    .unwrap()
    .validation
    .findings;
    assert!(adult_findings
        .iter()
        .any(|f| f.rule_id == "isu-adult-2026-27-jump-repetition"
            && f.kind == FindingKind::Violation));
    let replacement = SourceReference {
        document: "Synthetic event exception".into(),
        version: "1".into(),
        location: "test clause 1".into(),
    };
    profile.sources.push(replacement.clone());
    profile.overrides.push(RuleOverride {
        replaces: "senior-jump-count",
        replacement: RuleBinding {
            rule: ProfileRule::Count,
            id: "event-jump-count",
            mechanism: RuleMechanism::SeniorJumpPlanV1,
            source: replacement.clone(),
        },
        event_scope: profile.event_scope.clone(),
        justification: "synthetic test".into(),
    });
    let mut registry = ProfileRegistry::default();
    registry.register(profile).unwrap();
    let result = validate_profile_program(&registry, &p).unwrap();
    assert!(result
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "event-jump-count" && f.source == replacement));
}

#[test]
fn manual_assessment_uses_context_without_rewriting_supplied_recognition() {
    let mut entries = vec![];
    for index in 0..2 {
        let mut recognition = panel(&["3Lz"], None);
        recognition.planned_element = Some(index);
        entries.push(ManualProgramEntry {
            planned: solo(JumpKind::Lutz, 3),
            evidence: ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
            intended_recognition: Some(recognition.clone()),
            scenario: Some(ManualElementScenario {
                recognition,
                annotations: vec![],
                final_goe: Some(0),
                fall: Some(false),
                execution: None,
            }),
        });
    }
    let request = ManualProgramRequest {
        rotation_preference: RotationPreference::default(),
        profile: senior_profile(SeniorCategory::Women).id,
        scoring_table: TABLE,
        policy: senior_program_policy(SeniorCategory::Women),
        submission: SubmissionState::Draft,
        entries,
        request_hypothetical_total: true,
        second_half_boundary: None,
    };
    let result = assess_manual_program(&ProfileRegistry::senior(), &request).unwrap();
    assert_eq!(result.intended_base.exact_hundredths, Some(1062));
    assert_eq!(result.scenario_base.unwrap().exact_hundredths, Some(1062));
    assert_eq!(
        result.hypothetical_total.unwrap().exact_hundredths,
        Some(1062)
    );
    assert_eq!(result.request, request);
    assert_eq!(
        request.entries[1]
            .intended_recognition
            .as_ref()
            .unwrap()
            .details
            .as_ref()
            .unwrap()
            .notation()
            .unwrap(),
        "3Lz"
    );
}

#[test]
fn missing_evidence_and_spin_caps_remain_visible() {
    let mut entries = skeleton();
    entries.push(spin(RegularSpinKind::Camel, SpinEntry::Flying, 6));
    let report = legality(&entries, SubmissionState::Complete);
    for id in ["spin-count", "spin-distinct-abbreviation"] {
        assert!(report
            .validation
            .validation
            .findings
            .iter()
            .any(|f| f.rule_id == id && f.kind == FindingKind::Violation));
    }
    let mut entries = skeleton();
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entries[1].1 {
        spin.planned_level = Some(Level::Four);
    }
    if let ElementPlanEvidence::Step {
        full_ice_coverage, ..
    } = &mut entries[4].1
    {
        *full_ice_coverage = None;
    }
    if let ElementPlanEvidence::ChoreographicSpin {
        consecutive_revolutions_per_foot,
    } = &mut entries[3].1
    {
        *consecutive_revolutions_per_foot = Some(vec![2, 2]);
    }
    let report = legality(&entries, SubmissionState::Complete);
    for id in [
        "program-spin-level-credit",
        "program-step-recognition",
        "program-choreographic-spin",
    ] {
        assert!(report
            .rules
            .iter()
            .any(|r| r.rule_id == id && r.state == RuleCoverageState::MissingEvidence));
    }
    assert!(report
        .rules
        .iter()
        .any(|r| r.rule_id == "program-variation-categories"
            && r.state == RuleCoverageState::Unsupported));
}

#[test]
fn senior_recognition_and_deduction_uncertainty_never_use_adult_defaults() {
    let profile = senior_profile(SeniorCategory::Women);
    let mut mismatched = profile.clone();
    mismatched.scoring.recognition_ruleset = RulesetId::IsuAdult2026_27July16;
    assert!(ProfileRegistry::default().register(mismatched).is_err());
    let element = ProgramElement {
        recognition: panel(&["1A"], None),
        annotations: vec![],
        goe: 0,
        fall: Some(false),
        start_ms: Some(1000),
    };
    let score = score_profile_program(
        &ProfileRegistry::senior(),
        &profile.id,
        TABLE,
        std::slice::from_ref(&element),
        Some(0),
        None,
        None,
    )
    .unwrap()
    .score;
    assert_eq!(score.time_deduction, Err(ProgramScoreError::TimingUnknown));
    assert_eq!(
        score.fall_deduction,
        Err(ProgramScoreError::UnsupportedCategory)
    );
    let score = score_profile_program(
        &ProfileRegistry::senior(),
        &profile.id,
        TABLE,
        std::slice::from_ref(&element),
        Some(0),
        Some(ProgramTiming {
            duration_seconds: 251,
            ordinary_timing_confirmed: true,
            second_half_default_confirmed: true,
        }),
        None,
    )
    .unwrap()
    .score;
    assert_eq!(score.time_deduction.unwrap().hundredths, 100);
    let mut unknown = element;
    unknown.recognition.details = None;
    let score = score_profile_program(
        &ProfileRegistry::senior(),
        &profile.id,
        TABLE,
        &[unknown],
        Some(0),
        None,
        None,
    )
    .unwrap()
    .score;
    assert_eq!(score.tes_hundredths, None);
    assert!(score.elements[0].is_err());
}

#[cfg(feature = "optimizer")]
#[test]
fn generator_mixed_solver_and_locked_completion_resolve_senior_identity() {
    let profile = senior_profile(SeniorCategory::Women);
    let registry = ProfileRegistry::senior();
    let generated = generate_program_candidates(
        &registry,
        &CandidateGenerationRequest {
            rotation_preference: RotationPreference::default(),
            season: profile.season,
            profile: profile.id.clone(),
            scoring_table: TABLE,
            discipline: ProfileDiscipline::Single,
            segment: ProgramSegment::FreeSkating,
            jump_kinds: vec![JumpKind::Axel],
            rotations: vec![1, 3],
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
            max_candidates: 10,
            max_expansions: 100,
        },
    )
    .unwrap();
    assert_eq!(generated.variants.len(), 2);
    assert_eq!(generated.audit.profile, profile.id);
    let mixed = optimize_program(
        &registry,
        &ProgramOptimizationRequest {
            rotation_preference: RotationPreference::default(),
            season: profile.season,
            profile: profile.id.clone(),
            scoring_table: TABLE,
            policy: senior_program_policy(SeniorCategory::Women),
            variants: generated.variants,
            exclusions: vec![],
            must_have: vec![],
            min_elements: 1,
            max_elements: 1,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 100,
            max_solutions: 10,
            enumerate_equal_optima: true,
            time_limit: None,
        },
    )
    .unwrap();
    assert_eq!(mixed.status, OptimizerStatus::InfeasibleProven); // a jump-only plan lacks required roles
    let mut full_skeleton = skeleton();
    full_skeleton.extend(
        [
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
        ]
        .into_iter()
        .map(|kind| {
            (
                solo(kind, 2),
                ElementPlanEvidence::Jump {
                    direct_transitions_assumed: false,
                },
            )
        }),
    );
    let original = full_skeleton
        .into_iter()
        .enumerate()
        .map(|(i, (planned, evidence))| {
            let (codes, level) = match i {
                0 => (vec!["1A"], None),
                1 => (vec!["CoSp"], Some(Level::Basic)),
                2 => (vec!["FCSp"], Some(Level::Basic)),
                3 => (vec!["ChSp1"], None),
                4 => (vec!["StSq"], Some(Level::Basic)),
                5 => (vec!["ChSq1"], None),
                6 => (vec!["2T"], None),
                7 => (vec!["2S"], None),
                8 => (vec!["2Lo"], None),
                9 => (vec!["2F"], None),
                10 => (vec!["2Lz"], None),
                _ => unreachable!(),
            };
            let mut recognition = panel(&codes, level);
            recognition.planned_element = Some(i);
            CompletionEntry {
                id: format!("original-{i}"),
                locked: true,
                entry: ManualProgramEntry {
                    planned,
                    evidence,
                    intended_recognition: Some(recognition),
                    scenario: None,
                },
            }
        })
        .collect::<Vec<_>>();
    let request = ProgramCompletionRequest {
        rotation_preference: RotationPreference::default(),
        season: profile.season,
        profile: profile.id,
        scoring_table: TABLE,
        policy: senior_program_policy(SeniorCategory::Women),
        original,
        variants: vec![],
        generated_domain: None,
        exclusions: vec![],
        global_exclusions: vec![],
        insertion: CompletionInsertionPolicy {
            boundaries: vec![],
            absolute_positions: vec![],
        },
        min_additions: 0,
        max_additions: 0,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        max_proposals: 10,
        max_solutions: 10,
        enumerate_equal_optima: true,
        time_limit: None,
    };
    let result = complete_program(&registry, &request).unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].program.elements.len(), 11);
    assert_eq!(
        result.candidates[0].assessment.request.entries,
        request
            .original
            .iter()
            .map(|e| e.entry.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn known_zero_spins_preserve_repetition_credit_and_unrelated_values() {
    let profile = senior_profile(SeniorCategory::Women);
    for outcome in [RecognitionOutcome::NoValue, RecognitionOutcome::Ignored] {
        let mut zero = panel(&["ChSp1"], None);
        zero.outcome = outcome;
        let elements = [panel(&["3Lz"], None), panel(&["3Lz"], None), zero]
            .into_iter()
            .map(|recognition| ProgramElement {
                recognition,
                annotations: vec![],
                goe: 0,
                fall: Some(false),
                start_ms: Some(10_000),
            })
            .collect::<Vec<_>>();
        let result = score_profile_program(
            &ProfileRegistry::senior(),
            &profile.id,
            TABLE,
            &elements,
            Some(0),
            Some(ProgramTiming {
                duration_seconds: 240,
                ordinary_timing_confirmed: true,
                second_half_default_confirmed: true,
            }),
            None,
        )
        .unwrap();
        assert_eq!(result.score.tes_hundredths, Some(1062));
        assert_eq!(
            result.score.elements[0]
                .as_ref()
                .unwrap()
                .credited_base_hundredths,
            590
        );
        assert_eq!(
            result.score.elements[1]
                .as_ref()
                .unwrap()
                .credited_base_hundredths,
            472
        );
        assert!(matches!(
            &result.score.elements[2],
            Err(ProgramScoreError::Element(
                ElementScoreError::NoValue | ElementScoreError::Ignored
            ))
        ));
    }
    let mut unknown = panel(&["ChSp1"], None);
    unknown.outcome = RecognitionOutcome::Unknown;
    unknown.details = None;
    let elements = [panel(&["ChSp1"], None), panel(&["3Lz"], None), unknown]
        .into_iter()
        .map(|recognition| ProgramElement {
            recognition,
            annotations: vec![],
            goe: 0,
            fall: Some(false),
            start_ms: Some(1000),
        })
        .collect::<Vec<_>>();
    let result = score_profile_program(
        &ProfileRegistry::senior(),
        &profile.id,
        TABLE,
        &elements,
        Some(0),
        Some(ProgramTiming {
            duration_seconds: 240,
            ordinary_timing_confirmed: true,
            second_half_default_confirmed: true,
        }),
        None,
    )
    .unwrap();
    assert!(result.score.elements[0].is_ok());
    assert!(result.score.elements[1].is_err());
    assert_eq!(result.score.tes_hundredths, None);
}

#[test]
fn official_spin_policy_integrity_is_identical_at_both_public_boundaries() {
    let registry = ProfileRegistry::senior();
    let p = program(vec![]);
    for remove_roles in [false, true] {
        let mut policy = senior_program_policy(SeniorCategory::Women);
        if remove_roles {
            policy.spins.required.clear();
        } else {
            policy.spins.max_spins = 4;
        }
        assert_eq!(
            validate_profile_program_with_evidence(
                &registry,
                &p,
                &ProgramValidationEvidence::default(),
                Some(&policy.spins)
            )
            .unwrap_err()
            .kind,
            ProfileErrorKind::Invalid
        );
        assert_eq!(
            validate_supported_program(&registry, &p, &[], &policy)
                .unwrap_err()
                .kind,
            ProfileErrorKind::Invalid
        );
    }
    let mut custom = senior_profile(SeniorCategory::Women);
    custom.id.name = "custom-spin-policy".into();
    custom.provenance = ProfileProvenance::Synthetic;
    let mut registry = ProfileRegistry::default();
    registry.register(custom.clone()).unwrap();
    let mut p = p;
    p.profile = custom.id.clone();
    let mut policy = senior_program_policy(SeniorCategory::Women);
    policy.spins.profile = custom.id;
    policy.spins.max_spins = 4;
    assert!(validate_profile_program_with_evidence(
        &registry,
        &p,
        &ProgramValidationEvidence::default(),
        Some(&policy.spins)
    )
    .is_ok());
    assert!(validate_supported_program(&registry, &p, &[], &policy).is_ok());
}

#[cfg(feature = "optimizer")]
#[test]
fn mixed_senior_optimizer_produces_a_valid_complete_program() {
    let profile = senior_profile(SeniorCategory::Women);
    let mut entries = skeleton();
    entries[0].0 = compound(&[(JumpKind::Axel, 1), (JumpKind::ToeLoop, 1)], false);
    let variants = entries
        .into_iter()
        .map(|(element, evidence)| ProgramVariant {
            element,
            evidence,
            recognition_assumed: true,
        })
        .collect::<Vec<_>>();
    let result = optimize_program(
        &ProfileRegistry::senior(),
        &ProgramOptimizationRequest {
            rotation_preference: RotationPreference::default(),
            season: profile.season,
            profile: profile.id.clone(),
            scoring_table: TABLE,
            policy: senior_program_policy(SeniorCategory::Women),
            must_have: (0..variants.len()).map(|i| vec![i]).collect(),
            variants,
            exclusions: vec![],
            min_elements: 6,
            max_elements: 6,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 100_000,
            max_solutions: 1,
            enumerate_equal_optima: false,
            time_limit: None,
        },
    )
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    let candidate = &result.candidates[0];
    assert_eq!(candidate.program.profile, profile.id);
    assert_eq!(candidate.base_hundredths, 1380); // 1A+1T 1.50, CoSpB 1.80, FCSpB 1.90, ChSp1 3.50, StSqB 1.60, ChSq1 3.50
    assert_eq!(candidate.program.elements.len(), 6);
    assert_eq!(
        candidate
            .program
            .elements
            .iter()
            .filter(|e| matches!(e, PlannedElement::Jump(_)))
            .count(),
        1
    );
    assert!(candidate
        .validation
        .validation
        .findings
        .iter()
        .all(|f| f.rule_id == "spin-execution-recognition"));
}

#[cfg(feature = "optimizer")]
mod eleven_slots {
    use super::*;
    use std::collections::BTreeSet;

    fn fixture() -> ProgramCompletionRequest {
        let profile = senior_profile(SeniorCategory::Women);
        let mut entries = skeleton();
        entries.extend(
            [
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
            ]
            .map(|kind| {
                (
                    solo(kind, 2),
                    ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false,
                    },
                )
            }),
        );
        let codes = [
            "1A", "CoSp", "FCSp", "ChSp1", "StSq", "ChSq1", "2T", "2S", "2Lo", "2F",
        ];
        let original = entries
            .into_iter()
            .enumerate()
            .map(|(i, (planned, evidence))| {
                let mut recognition =
                    panel(&[codes[i]], matches!(i, 1 | 2 | 4).then_some(Level::Basic));
                recognition.planned_element = Some(i);
                CompletionEntry {
                    id: format!("anchor-{i}"),
                    locked: true,
                    entry: ManualProgramEntry {
                        planned,
                        evidence,
                        intended_recognition: Some(recognition),
                        scenario: None,
                    },
                }
            })
            .collect();
        ProgramCompletionRequest {
            rotation_preference: RotationPreference::default(),
            season: profile.season,
            profile: profile.id,
            scoring_table: TABLE,
            policy: senior_program_policy(SeniorCategory::Women),
            original,
            variants: vec![ProgramVariant {
                element: solo(JumpKind::ToeLoop, 2),
                evidence: ElementPlanEvidence::Jump {
                    direct_transitions_assumed: false,
                },
                recognition_assumed: true,
            }],
            generated_domain: None,
            exclusions: vec![],
            global_exclusions: vec![],
            insertion: CompletionInsertionPolicy {
                boundaries: (0..=10).collect(),
                absolute_positions: vec![],
            },
            min_additions: 1,
            max_additions: 1,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            max_proposals: 100,
            max_solutions: 100,
            enumerate_equal_optima: true,
            time_limit: None,
        }
    }

    #[test]
    fn all_eleven_insertion_ties_preserve_equal_looking_originals_and_recognition() {
        let registry = ProfileRegistry::senior();
        for (boundaries, absolute, expected) in [
            (
                (0..=10).collect(),
                vec![],
                (0..=10).collect::<BTreeSet<_>>(),
            ),
            (vec![0, 5, 10], vec![], BTreeSet::from([0, 5, 10])),
            (
                (0..=10).collect(),
                vec![("anchor-6".into(), 6)],
                (7..=10).collect(),
            ),
        ] {
            let mut req = fixture();
            req.insertion.boundaries = boundaries;
            req.insertion.absolute_positions = absolute;
            let result = complete_program(&registry, &req).unwrap();
            assert_eq!(result.status, OptimizerStatus::OptimalProven);
            assert!(result.enumeration_complete);
            let mut positions = BTreeSet::new();
            for candidate in result.candidates {
                assert_eq!(candidate.total_base_hundredths, 2080);
                assert_eq!(candidate.existing_base_hundredths, 1950);
                assert_eq!(candidate.added_base_hundredths, 130);
                assert_eq!(candidate.program.profile, req.profile);
                assert_eq!(candidate.program.scoring_table, TABLE);
                let added = candidate
                    .provenance
                    .iter()
                    .position(|p| matches!(p, CompletionProvenance::Inserted { .. }))
                    .unwrap();
                positions.insert(added);
                assert_eq!(
                    candidate.provenance[added],
                    CompletionProvenance::Inserted {
                        variant_index: 0,
                        boundary: added,
                        ordinal: 0
                    }
                );
                for (i, original) in req.original.iter().enumerate() {
                    let index = i + usize::from(i >= added);
                    assert_eq!(
                        candidate.provenance[index],
                        CompletionProvenance::Original {
                            id: original.id.clone(),
                            original_index: i,
                            locked: true
                        }
                    );
                    let mut entry = original.entry.clone();
                    entry.intended_recognition.as_mut().unwrap().planned_element = Some(index);
                    assert_eq!(candidate.assessment.request.entries[index], entry);
                    assert_eq!(candidate.evidence[index], original.entry.evidence);
                }
                for (index, value) in candidate.per_element.iter().enumerate() {
                    assert_eq!(
                        BaseValueOutcome::Value(value.clone()),
                        lookup_base_value(
                            TABLE,
                            candidate.assessment.request.entries[index]
                                .intended_recognition
                                .as_ref()
                                .unwrap(),
                            &[]
                        )
                        .unwrap()
                    );
                }
            }
            assert_eq!(positions, expected);
        }
    }

    #[test]
    fn eleven_slot_caps_and_empty_suffix_are_conservative() {
        let registry = ProfileRegistry::senior();
        let mut req = fixture();
        req.max_solutions = 1;
        let result = complete_program(&registry, &req).unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(!result.enumeration_complete);
        assert_eq!(
            result.termination_reason,
            JumpTerminationReason::SolutionLimit
        );
        req.max_solutions = 100;
        req.min_additions = 0;
        req.variants.clear();
        let result = complete_program(&registry, &req).unwrap();
        assert!(result.enumeration_complete);
        assert_eq!(result.candidates[0].program.elements.len(), 10);
        assert_eq!(result.candidates[0].total_base_hundredths, 1950);
        req.max_additions = 2;
        assert!(matches!(
            complete_program(&registry, &req),
            Err(ProgramCompletionError::InvalidRequest(_))
        ));
    }

    #[test]
    fn eleven_slot_original_ruleset_mismatch_is_rejected() {
        let mut req = fixture();
        req.original[0]
            .entry
            .intended_recognition
            .as_mut()
            .unwrap()
            .ruleset = RulesetId::IsuAdult2026_27July16;
        assert!(complete_program(&ProfileRegistry::senior(), &req).is_err());
    }

    #[test]
    fn full_eleven_slot_quad_scenario_is_rescored_and_twelfth_slot_rejected() {
        let registry = ProfileRegistry::senior();
        let req = fixture();
        let mut variants: Vec<_> = req
            .original
            .iter()
            .map(|o| ProgramVariant {
                element: o.entry.planned.clone(),
                evidence: o.entry.evidence.clone(),
                recognition_assumed: true,
            })
            .collect();
        // Independent clean SOV values: 3A=800, 4T=950, 4S=970, 4Lo=1050, 4F=1100, 4Lz=1150.
        for (i, kind, rotations) in [
            (0, JumpKind::Axel, 3),
            (6, JumpKind::ToeLoop, 4),
            (7, JumpKind::Salchow, 4),
            (8, JumpKind::Loop, 4),
            (9, JumpKind::Flip, 4),
        ] {
            variants[i].element = solo(kind, rotations);
        }
        variants.push(ProgramVariant {
            element: solo(JumpKind::Lutz, 4),
            evidence: ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
            recognition_assumed: true,
        });
        let mut full = ProgramOptimizationRequest {
            rotation_preference: RotationPreference::default(),
            season: req.season,
            profile: req.profile,
            scoring_table: TABLE,
            policy: req.policy,
            must_have: (0..11).map(|i| vec![i]).collect(),
            variants,
            exclusions: vec![],
            min_elements: 11,
            max_elements: 11,
            objective: req.objective,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 100,
            max_solutions: 1,
            enumerate_equal_optima: true,
            time_limit: None,
        };
        // Capacity is independent of solving an 11! ordered domain. A zero
        // deadline tests public request acceptance without a latency promise.
        full.time_limit = Some(std::time::Duration::ZERO);
        let result = optimize_program(&registry, &full).unwrap();
        assert_eq!(result.status, OptimizerStatus::Incomplete);
        assert!(!result.enumeration_complete);
        let mut locked = fixture();
        let codes = [
            "3A", "CoSp", "FCSp", "ChSp1", "StSq", "ChSq1", "4T", "4S", "4Lo", "4F", "4Lz",
        ];
        locked.original = full
            .variants
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut recognition =
                    panel(&[codes[i]], matches!(i, 1 | 2 | 4).then_some(Level::Basic));
                recognition.planned_element = Some(i);
                CompletionEntry {
                    id: format!("quad-{i}"),
                    locked: true,
                    entry: ManualProgramEntry {
                        planned: v.element.clone(),
                        evidence: v.evidence.clone(),
                        intended_recognition: Some(recognition),
                        scenario: None,
                    },
                }
            })
            .collect();
        locked.variants.clear();
        locked.insertion.boundaries.clear();
        locked.min_additions = 0;
        locked.max_additions = 0;
        let result = complete_program(&registry, &locked).unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(result.enumeration_complete);
        let candidate = &result.candidates[0];
        assert_eq!(candidate.total_base_hundredths, 7250); // jumps 6020 + other roles 1230
        assert_eq!(
            candidate.program.elements,
            full.variants
                .iter()
                .map(|v| v.element.clone())
                .collect::<Vec<_>>()
        );
        assert!(candidate
            .validation
            .validation
            .findings
            .iter()
            .all(|f| f.rule_id == "spin-execution-recognition"));
        full.max_elements = 12;
        assert!(matches!(
            optimize_program(&registry, &full),
            Err(ProgramOptimizationError::InvalidRequest(_))
        ));
        let mut twelve = candidate.program.clone();
        twelve.elements.push(PlannedElement::Unspecified);
        let report = validate_profile_program(&registry, &twelve).unwrap();
        let finding = report
            .validation
            .findings
            .iter()
            .find(|f| f.rule_id == "senior-jump-count" && f.kind == FindingKind::Violation)
            .unwrap();
        assert!(format!("{finding:?}").contains("612"));
    }
}
