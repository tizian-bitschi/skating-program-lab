use skating_program_lab::*;

#[cfg(feature = "optimizer")]
#[path = "support/generation_defaults.rs"]
#[allow(dead_code)]
mod generation_defaults;

fn silver(codes: &[&str], preference: RotationPreference) -> ManualBvRequest {
    let profile = adult_profile(AdultCategory::Silver);
    ManualBvRequest {
        profile: profile.id,
        scoring_table: profile.scoring_table,
        personal_limits: PersonalLimits::default(),
        rotation_preference: preference,
        submission: SubmissionState::Complete,
        elements: codes.iter().map(|c| AssumedRecognition::new(*c)).collect(),
    }
}

#[test]
fn default_selection_checks_every_gap_and_sequence_step() {
    let profile = adult_profile(AdultCategory::Silver);
    let registry = ProfileRegistry::adult();
    let default = selectable_jump_elements(
        &registry,
        &profile.id,
        profile.scoring_table,
        RotationPreference::default(),
    )
    .unwrap();
    let broad = selectable_jump_elements(
        &registry,
        &profile.id,
        profile.scoring_table,
        RotationPreference::AllowMixedDirections,
    )
    .unwrap();
    for code in [
        "1Lz+1T",
        "1F+1Lo",
        "1Lz+Eu+1F",
        "1Lz+Eu+1A",
        "1Lz+1A+1A+SEQ",
        "1Lz+1A+1T+SEQ",
        "1Lz+Eu+1F+1A+SEQ",
    ] {
        assert!(default.iter().any(|c| c.code == code), "missing {code}");
    }
    for code in [
        "1Lz+1Lz",
        "1F+1Lz",
        "1A+1A+1Lz",
        "1Lz+1Lz+1A+SEQ",
        "1Lz+Eu+1A+SEQ",
    ] {
        assert!(!default.iter().any(|c| c.code == code), "unexpected {code}");
    }
    assert!(default.iter().all(|c| broad.contains(c)));
    assert!(broad.iter().any(|c| c.code == "1Lz+1Lz"));
}

#[test]
fn manual_conflicts_preserve_values_and_rule_findings() {
    let registry = ProfileRegistry::adult();
    let codes = [
        "1A+1A+1Lz",
        "1F",
        "1F+1Lz",
        "1Lo",
        "CCoSp2",
        "CLSp2",
        "ChSq1",
    ];
    let standard =
        assess_manual_bv(&registry, &silver(&codes, RotationPreference::default())).unwrap();
    let broad = assess_manual_bv(
        &registry,
        &silver(&codes, RotationPreference::AllowMixedDirections),
    )
    .unwrap();
    assert_eq!(standard.total.exact_hundredths, Some(1430));
    assert_eq!(standard.total, broad.total);
    assert_eq!(standard.legality, broad.legality);
    assert_eq!(
        standard
            .rotation_conflicts
            .iter()
            .map(|c| c.element_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(standard.rotation_conflicts[0].component_indices, [0, 1, 2]);
    assert!(broad.rotation_conflicts.is_empty());
}

#[test]
fn detailed_manual_assessment_keeps_personal_conflicts_separate() {
    let p = adult_profile(AdultCategory::Silver);
    let pair = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1),
        }),
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1),
        }),
    ]));
    let mut r = ManualProgramRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        policy: adult_program_policy(AdultCategory::Silver),
        submission: SubmissionState::Draft,
        entries: vec![ManualProgramEntry {
            planned: pair,
            evidence: ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
            intended_recognition: None,
            scenario: None,
        }],
        request_hypothetical_total: false,
        second_half_boundary: None,
    };
    let registry = ProfileRegistry::adult();
    let standard = assess_manual_program(&registry, &r).unwrap();
    r.rotation_preference = RotationPreference::AllowMixedDirections;
    let broad = assess_manual_program(&registry, &r).unwrap();
    assert_eq!(standard.intended_base, broad.intended_base);
    assert_eq!(standard.validation, broad.validation);
    assert_eq!(standard.status, broad.status);
    assert_eq!(
        standard.rotation_conflicts,
        [RotationPreferenceConflict {
            element_index: 0,
            component_indices: vec![0, 1]
        }]
    );
    assert!(broad.rotation_conflicts.is_empty());
}

#[cfg(feature = "optimizer")]
mod automatic {
    use super::*;

    struct Continue;
    impl PlanningSearchControl for Continue {
        fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
            false
        }
    }
    fn request(preference: RotationPreference) -> PlanningSearchRequest {
        let p = adult_profile(AdultCategory::Silver);
        PlanningSearchRequest {
            profile: p.id,
            scoring_table: p.scoring_table,
            personal_limits: PersonalLimits::default(),
            rotation_preference: preference,
            required: vec![],
            exclusions: vec![],
        }
    }
    fn lutz_pair() -> ProgramVariant {
        ProgramVariant {
            element: PlannedElement::Jump(JumpElement::Combination(vec![
                JumpPart::Jump(ListedJump {
                    kind: Some(JumpKind::Lutz),
                    rotations: Some(1),
                }),
                JumpPart::Jump(ListedJump {
                    kind: Some(JumpKind::Lutz),
                    rotations: Some(1),
                }),
            ])),
            evidence: ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
            recognition_assumed: true,
        }
    }

    #[test]
    fn silver_proves_1480_in_both_modes_and_preserves_required_conflicts() {
        let registry = ProfileRegistry::adult();
        for preference in [
            RotationPreference::default(),
            RotationPreference::AllowMixedDirections,
        ] {
            let r = request(preference);
            let PlanningSearchOutcome::ProvenMaximum(result) =
                search_maximum_bv(&registry, &r, &Continue).unwrap()
            else {
                panic!("expected maximum")
            };
            assert_eq!(result.maximum_base_hundredths, 1480);
            assert_eq!(result.alternatives.len(), 5);
            for candidate in result.alternatives {
                let assessment = assess_manual_bv(
                    &registry,
                    &ManualBvRequest {
                        profile: r.profile.clone(),
                        scoring_table: r.scoring_table,
                        personal_limits: PersonalLimits::default(),
                        rotation_preference: preference,
                        submission: SubmissionState::Complete,
                        elements: candidate.elements,
                    },
                )
                .unwrap();
                assert!(assessment.rotation_conflicts.is_empty());
                assert_eq!(assessment.total.exact_hundredths, Some(1480));
            }
        }
        let mut r = request(RotationPreference::default());
        r.required = vec![RequiredPlanningElement {
            id: "pair".into(),
            element: AssumedRecognition::new("1Lz+1Lz"),
        }];
        let Err(PlanningSearchError::RotationPreferenceConflict(conflicts)) =
            search_maximum_bv(&registry, &r, &Continue)
        else {
            panic!("expected preference conflict")
        };
        assert_eq!(
            conflicts,
            [RotationPreferenceConflict {
                element_index: 0,
                component_indices: vec![0, 1]
            }]
        );
        let mut session = PlanningSearchSession::new(&registry, r.clone(), None);
        let Err(PlanningSessionError::Search(PlanningSearchError::RotationPreferenceConflict(
            indexed,
        ))) = session.prove(&Continue)
        else {
            panic!("rotation conflict")
        };
        assert_eq!(indexed, conflicts);
        assert_eq!(session.state(), PlanningSessionState::Failed);
        assert_eq!(r.required[0].element.code, "1Lz+1Lz");
        r.rotation_preference = RotationPreference::AllowMixedDirections;
        let mut session = PlanningSearchSession::new(&registry, r.clone(), None);
        assert_eq!(
            session.prove(&Continue).unwrap(),
            PlanningSessionState::Collecting
        );
        assert!(matches!(
            search_maximum_bv(&registry, &r, &Continue),
            Ok(PlanningSearchOutcome::ProvenMaximum(_))
        ));
    }

    #[test]
    fn generation_filters_optional_manual_variants_and_records_preference() {
        let registry = ProfileRegistry::adult();
        let mut r = generation_defaults::request(adult_profile(AdultCategory::Silver));
        r.manual_variants = vec![lutz_pair()];
        let recognition = [AssumedRecognition::new("1Lz+1Lz")];
        let standard =
            generate_program_candidates_with_assumed_recognition(&registry, &r, &recognition)
                .unwrap();
        assert!(standard.domain.variants.is_empty());
        assert_eq!(standard.domain.audit.omissions.len(), 1);
        assert!(standard.domain.audit.omissions[0]
            .reason
            .contains("personal"));
        assert_eq!(
            standard.domain.audit.rotation_preference,
            RotationPreference::SameDirection
        );
        r.rotation_preference = RotationPreference::AllowMixedDirections;
        let broad =
            generate_program_candidates_with_assumed_recognition(&registry, &r, &recognition)
                .unwrap();
        assert_eq!(broad.domain.variants.len(), 1);
    }

    fn completion(original: Vec<CompletionEntry>) -> ProgramCompletionRequest {
        let p = adult_profile(AdultCategory::Silver);
        ProgramCompletionRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            season: p.season,
            profile: p.id,
            scoring_table: p.scoring_table,
            policy: adult_program_policy(AdultCategory::Silver),
            original,
            variants: vec![],
            generated_domain: None,
            exclusions: vec![],
            global_exclusions: vec![],
            insertion: CompletionInsertionPolicy {
                boundaries: vec![0],
                absolute_positions: vec![],
            },
            min_additions: 0,
            max_additions: 0,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            max_proposals: 100,
            max_solutions: 1,
            enumerate_equal_optima: false,
            time_limit: None,
        }
    }

    #[test]
    fn completion_preserves_conflicting_original_and_checks_domain_preference() {
        let registry = ProfileRegistry::adult();
        let variant = lutz_pair();
        for locked in [true, false] {
            let original = CompletionEntry {
                id: "original-pair".into(),
                locked,
                entry: ManualProgramEntry {
                    planned: variant.element.clone(),
                    evidence: variant.evidence.clone(),
                    intended_recognition: None,
                    scenario: None,
                },
            };
            let r = completion(vec![original.clone()]);
            let result = complete_program_with_assumed_recognition(
                &registry,
                &r,
                &[AssumedRecognition::new("1Lz+1Lz")],
                &[],
            );
            assert!(matches!(
                result,
                Err(ProgramCompletionError::RotationPreferenceConflict(_))
            ));
            assert_eq!(r.original, [original]);
            let mut opted_in = r.clone();
            opted_in.rotation_preference = RotationPreference::AllowMixedDirections;
            let broad = complete_program_with_assumed_recognition(
                &registry,
                &opted_in,
                &[AssumedRecognition::new("1Lz+1Lz")],
                &[],
            )
            .unwrap();
            assert!(broad.original_assessment.rotation_conflicts.is_empty());
            assert_eq!(broad.request.original, r.original);
        }
        let mut g = generation_defaults::request(adult_profile(AdultCategory::Silver));
        g.rotation_preference = RotationPreference::AllowMixedDirections;
        let domain = generate_program_candidates(&registry, &g).unwrap();
        let mut r = completion(vec![]);
        r.max_additions = 1;
        r.generated_domain = Some(domain);
        assert!(matches!(
            complete_program(&registry, &r),
            Err(ProgramCompletionError::InvalidRequest(
                "generated domain does not match request"
            ))
        ));
    }

    #[test]
    fn finite_optimization_filters_optional_variants_and_reports_required_conflicts() {
        let registry = ProfileRegistry::adult();
        let p = adult_profile(AdultCategory::Silver);
        let mut r = ProgramOptimizationRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            season: p.season,
            profile: p.id,
            scoring_table: p.scoring_table,
            policy: adult_program_policy(AdultCategory::Silver),
            variants: vec![lutz_pair()],
            exclusions: vec![],
            must_have: vec![],
            min_elements: 1,
            max_elements: 1,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 100,
            max_solutions: 1,
            enumerate_equal_optima: false,
            time_limit: None,
        };
        let recognition = [AssumedRecognition::new("1Lz+1Lz")];
        let standard =
            optimize_program_with_assumed_recognition(&registry, &r, &recognition).unwrap();
        assert!(standard.universe.is_empty());
        assert_eq!(standard.excluded_variants, [0]);
        r.must_have = vec![vec![0]];
        assert!(matches!(
            optimize_program_with_assumed_recognition(&registry, &r, &recognition),
            Err(ProgramOptimizationError::RotationPreferenceConflict(_))
        ));
        r.rotation_preference = RotationPreference::AllowMixedDirections;
        let broad = optimize_program_with_assumed_recognition(&registry, &r, &recognition).unwrap();
        assert_eq!(broad.universe, [0]);
    }
}
