use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
fn solo(kind: JumpKind, rotations: u8) -> PlannedElement {
    PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(kind),
        rotations: Some(rotations),
    }))
}
fn plan(id: ProfileId, elements: Vec<PlannedElement>) -> ProfileProgram {
    ProfileProgram {
        profile: id,
        scoring_table: TABLE,
        elements,
    }
}
fn synthetic() -> RuleProfile {
    let mut profile = adult_profile(AdultCategory::Silver);
    profile.id = ProfileId {
        name: "synthetic-one-jump".into(),
        version: "test-v1".into(),
    };
    profile.category = "synthetic fixture; no official category".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.event_scope = "synthetic integration test".into();
    profile.precedence = "explicit test overrides of count and eligibility".into();
    profile.jumps.max_jumps = 1;
    profile
        .jumps
        .allowed
        .retain(|p| p.kind == JumpKind::ToeLoop);
    let source = SourceReference {
        document: "Synthetic profile fixture".into(),
        version: "test-v1".into(),
        location: "one solo toe loop; not an official rule".into(),
    };
    profile.sources.push(source.clone());
    for (rule, id) in [
        (ProfileRule::Count, "synthetic-count-v1"),
        (ProfileRule::Eligibility, "synthetic-eligibility-v1"),
    ] {
        let mut replacement = profile
            .bindings
            .iter()
            .find(|b| b.rule == rule)
            .unwrap()
            .clone();
        let replaces = replacement.id;
        replacement.id = id;
        replacement.source = source.clone();
        profile.overrides.push(RuleOverride {
            replaces,
            replacement,
            event_scope: profile.event_scope.clone(),
            justification: "synthetic test policy replaces this single binding".into(),
        });
    }
    profile
}

#[test]
fn configured_synthetic_profile_changes_count_and_eligibility_with_its_own_sources() {
    let mut registry = ProfileRegistry::adult();
    let synthetic = synthetic();
    registry.register(synthetic.clone()).unwrap();
    let elements = vec![solo(JumpKind::ToeLoop, 1), solo(JumpKind::Axel, 1)];
    let result =
        validate_profile_program(&registry, &plan(synthetic.id.clone(), elements.clone())).unwrap();
    assert_eq!(result.profile, synthetic.id);
    assert_eq!(result.validation.coverage, Coverage::Partial);
    assert_eq!(result.validation.findings.len(), 2);
    for (finding, id) in result
        .validation
        .findings
        .iter()
        .zip(["synthetic-eligibility-v1", "synthetic-count-v1"])
    {
        assert_eq!(finding.rule_id, id);
        assert_eq!(finding.kind, FindingKind::Violation);
        assert_eq!(finding.source.document, "Synthetic profile fixture");
        assert!(finding.explanation.contains("profile"));
    }
    let silver = adult_profile(AdultCategory::Silver);
    assert!(
        validate_profile_program(&registry, &plan(silver.id, elements))
            .unwrap()
            .validation
            .findings
            .is_empty()
    );
}

#[test]
fn adult_source_backed_eligibility_and_caps_survive_profile_selection() {
    let registry = ProfileRegistry::adult();
    // Announcement July 16 pp. 8-11: Bronze excludes Axel; Silver excludes
    // doubles; Gold permits 2Lo, Masters requires Axel and permits doubles.
    let fixtures = [
        (
            AdultCategory::Bronze,
            vec![true, false, false],
            4,
            "p. 11, Bronze Free Skating, jumps",
        ),
        (
            AdultCategory::Silver,
            vec![true, true, false],
            4,
            "p. 10, Silver Free Skating, jumps",
        ),
        (
            AdultCategory::Gold,
            vec![true, true, true],
            5,
            "p. 9, Gold Free Skating, jumps",
        ),
        (
            AdultCategory::Masters,
            vec![true, true, true],
            5,
            "p. 8, Masters Free Skating, jumps",
        ),
    ];
    for (category, eligibility, cap, location) in fixtures {
        let profile = adult_profile(category);
        assert_eq!(profile.jumps.max_jumps, cap);
        assert_eq!(
            profile.season,
            Season {
                start_year: 2026,
                end_year: 2027
            }
        );
        assert_eq!(
            profile
                .bindings
                .iter()
                .find(|b| b.rule == ProfileRule::Eligibility)
                .unwrap()
                .source
                .location,
            location
        );
        for (element, allowed) in [
            solo(JumpKind::ToeLoop, 1),
            solo(JumpKind::Axel, 1),
            solo(JumpKind::Loop, 2),
        ]
        .into_iter()
        .zip(eligibility)
        {
            let elements = vec![element];
            let new =
                validate_profile_program(&registry, &plan(profile.id.clone(), elements.clone()))
                    .unwrap();
            assert_eq!(
                !new.validation
                    .findings
                    .iter()
                    .any(|f| f.kind == FindingKind::Violation
                        && f.rule_id.ends_with("jump-eligibility")),
                allowed
            );
            let legacy = Program {
                season: profile.season,
                category,
                ruleset: RulesetId::IsuAdult2026_27July16,
                scoring_table: TABLE,
                sources: vec![],
                elements,
            };
            assert_eq!(new.validation, validate_program(&legacy));
        }
    }
}

#[test]
fn identity_errors_retain_requested_version_and_registry_never_overwrites() {
    let mut registry = ProfileRegistry::adult();
    let baseline = adult_profile(AdultCategory::Bronze);
    let mut changed = baseline.clone();
    changed.jumps.max_jumps = 1;
    assert_eq!(
        registry.register(changed).unwrap_err().kind,
        ProfileErrorKind::Invalid
    );
    assert_eq!(registry.get(&baseline.id).unwrap(), &baseline);
    let mut unknown = baseline.id;
    unknown.version = "unresearched-2027".into();
    let error = registry.resolve(&unknown, TABLE).unwrap_err();
    assert_eq!(error.profile, unknown);
    assert_eq!(error.kind, ProfileErrorKind::Unknown);
    assert!(error.reason.contains("unknown"));
    let mut next = synthetic();
    next.id.version = "test-v2".into();
    registry.register(synthetic()).unwrap();
    registry.register(next.clone()).unwrap();
    assert_ne!(
        registry.get(&synthetic().id).unwrap().id,
        registry.get(&next.id).unwrap().id
    );
}

#[test]
fn overrides_are_order_independent_and_conflicts_are_rejected() {
    let profile = synthetic();
    let mut reversed = profile.clone();
    reversed.overrides.reverse();
    let check = |p: RuleProfile| {
        let mut registry = ProfileRegistry::default();
        registry.register(p.clone()).unwrap();
        validate_profile_program(
            &registry,
            &plan(
                p.id,
                vec![solo(JumpKind::Axel, 1), solo(JumpKind::ToeLoop, 1)],
            ),
        )
        .unwrap()
    };
    assert_eq!(check(profile.clone()), check(reversed));
    let mut variants = vec![];
    let mut duplicate = profile.clone();
    duplicate.overrides.push(duplicate.overrides[0].clone());
    variants.push(duplicate);
    let mut absent = profile.clone();
    absent.overrides[0].replaces = "absent";
    variants.push(absent);
    let mut cycle = profile.clone();
    cycle.overrides[0].replaces = cycle.overrides[0].replacement.id;
    variants.push(cycle);
    let mut scope = profile.clone();
    scope.overrides[0].event_scope = "other event".into();
    variants.push(scope);
    let mut uncited = profile.clone();
    uncited.overrides[0].replacement.source.version.clear();
    variants.push(uncited);
    let mut wrong_rule = profile.clone();
    wrong_rule.overrides[0].replacement.rule = ProfileRule::Euler;
    variants.push(wrong_rule);
    let mut no_reason = profile.clone();
    no_reason.overrides[0].justification.clear();
    variants.push(no_reason);
    for invalid in variants {
        assert_eq!(
            ProfileRegistry::default()
                .register(invalid)
                .unwrap_err()
                .kind,
            ProfileErrorKind::Invalid
        );
    }
}

#[test]
fn incomplete_sources_invalid_bounds_and_duplicate_bindings_are_rejected() {
    let mut variants = vec![];
    let mut missing = synthetic();
    missing.sources.clear();
    variants.push(missing);
    let mut invalid = synthetic();
    invalid.jumps.allowed[0].min_rotations = 0;
    variants.push(invalid);
    let mut duplicate = synthetic();
    duplicate.bindings.push(duplicate.bindings[0].clone());
    variants.push(duplicate);
    let mut mechanism = synthetic();
    mechanism.bindings[0].mechanism = RuleMechanism::AdultScoringV1;
    variants.push(mechanism);
    let mut duration = synthetic();
    duration.scoring.duration_seconds = 0;
    variants.push(duration);
    let mut family = synthetic();
    family.families.allowed.clear();
    variants.push(family);
    for invalid in variants {
        let id = invalid.id.clone();
        let error = ProfileRegistry::default().register(invalid).unwrap_err();
        assert_eq!(error.profile, id);
        assert_eq!(error.kind, ProfileErrorKind::Invalid);
    }
}

#[test]
fn unsupported_mechanisms_and_pair_scope_never_evaluate_as_allowed() {
    let mut variants = vec![];
    let mut pair = synthetic();
    pair.discipline = ProfileDiscipline::Pair;
    variants.push(pair);
    let mut segment = synthetic();
    segment.segment = ProgramSegment::ShortProgram;
    variants.push(segment);
    let mut mechanism = synthetic();
    mechanism.bindings[0].mechanism = RuleMechanism::Unsupported("first-success-v2".into());
    variants.push(mechanism);
    let mut feature = synthetic();
    feature.unsupported_policies.push(UnsupportedPolicy {
        family: ProfileElementFamily::RegularSpin,
        min_level: Some(1),
        max_level: Some(2),
        permitted_features: vec!["first successful use".into()],
    });
    variants.push(feature);
    for profile in variants {
        let mut registry = ProfileRegistry::default();
        registry.register(profile.clone()).unwrap();
        let error =
            validate_profile_program(&registry, &plan(profile.id.clone(), vec![])).unwrap_err();
        assert_eq!(error.profile, profile.id);
        assert_eq!(error.kind, ProfileErrorKind::Unsupported);
    }
}

#[test]
fn typed_family_variants_and_repetitions_are_shared_policy() {
    let mut profile = synthetic();
    profile.jumps.max_jumps = 4;
    profile.jumps.max_repetitions = 1;
    profile.families.allowed = vec![
        ProfileElementFamily::SoloJump,
        ProfileElementFamily::StepSequence,
    ];
    profile.families.required = vec![ProfileElementFamily::StepSequence];
    let mut registry = ProfileRegistry::default();
    registry.register(profile.clone()).unwrap();
    let result = validate_profile_program(
        &registry,
        &plan(
            profile.id.clone(),
            vec![
                solo(JumpKind::ToeLoop, 1),
                solo(JumpKind::ToeLoop, 1),
                PlannedElement::ChoreographicSpin,
            ],
        ),
    )
    .unwrap();
    assert_eq!(
        result
            .validation
            .findings
            .iter()
            .filter(|f| f.kind == FindingKind::Violation)
            .count(),
        3
    );
    let unknown = validate_profile_program(
        &registry,
        &plan(profile.id, vec![PlannedElement::Unspecified]),
    )
    .unwrap();
    assert!(unknown
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Unknown && f.explanation.contains("required")));
}

fn recognized() -> ProgramElement {
    ProgramElement {
        recognition: PanelRecognition {
            ruleset: RulesetId::IsuAdult2026_27July16,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: Some(
                RecognitionDetails::new(
                    vec![ComponentCode::try_from("1T").unwrap()],
                    None,
                    Some(vec![]),
                )
                .unwrap(),
            ),
        },
        annotations: vec![],
        goe: 0,
        fall: Some(false),
        start_ms: Some(1000),
    }
}

#[test]
fn scoring_preserves_catalog_values_and_uses_explicit_profile_duration_and_deductions() {
    let mut registry = ProfileRegistry::adult();
    let mut custom = synthetic();
    custom.scoring.duration_seconds = 60;
    custom.scoring.fall_hundredths = Some(25);
    let fixture_source = custom.sources.last().unwrap().clone();
    custom.scoring.duration_source = fixture_source.clone();
    custom.scoring.time_deduction_source = fixture_source.clone();
    custom.scoring.fall_source = fixture_source.clone();
    registry.register(custom.clone()).unwrap();
    let elements = [recognized()];
    let timing = Some(ProgramTiming {
        duration_seconds: 60,
        ordinary_timing_confirmed: true,
        second_half_default_confirmed: true,
    });
    let score = score_profile_program(
        &registry,
        &custom.id,
        TABLE,
        &elements,
        Some(2),
        timing,
        None,
    )
    .unwrap();
    assert_eq!(score.score.time_deduction.as_ref().unwrap().hundredths, 0);
    assert_eq!(score.score.fall_deduction.as_ref().unwrap().hundredths, 50);
    assert_eq!(
        score.score.fall_deduction.as_ref().unwrap().source,
        fixture_source
    );
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let adult = adult_profile(category);
        let adult_timing = Some(ProgramTiming {
            duration_seconds: adult.scoring.duration_seconds,
            ordinary_timing_confirmed: true,
            second_half_default_confirmed: true,
        });
        let new = score_profile_program(
            &registry,
            &adult.id,
            TABLE,
            &elements,
            Some(2),
            adult_timing,
            None,
        )
        .unwrap();
        assert_eq!(
            new.score,
            score_program(
                RulesetId::IsuAdult2026_27July16,
                category,
                TABLE,
                &elements,
                Some(2),
                adult_timing
            )
        );
        assert_eq!(
            new.score.elements[0].as_ref().unwrap().unfactored.base,
            score.score.elements[0].as_ref().unwrap().unfactored.base
        );
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn synthetic_profile_uses_existing_solver_and_preserves_identity_and_bounds() {
    let custom = synthetic();
    let mut registry = ProfileRegistry::adult();
    registry.register(custom.clone()).unwrap();
    let request = ProfileJumpOptimizerRequest {
        profile: custom.id.clone(),
        scoring_table: TABLE,
        allowed_jumps: vec![
            ListedJump {
                kind: Some(JumpKind::ToeLoop),
                rotations: Some(1),
            },
            ListedJump {
                kind: Some(JumpKind::Axel),
                rotations: Some(1),
            },
        ],
        combinations: vec![],
        exclusions: vec![],
        excluded_kinds: vec![],
        max_elements: 2,
        require_nonempty: true,
        max_search_nodes: 1000,
        enumerate_equal_optima: true,
        max_solutions: 10,
        time_limit: None,
    };
    let result = optimize_profile_jumps(&registry, &request).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].base_hundredths, 40); // Communication 2786 p. 2, 1T.
    assert_eq!(result.candidates[0].program.profile, custom.id);
    assert_eq!(
        result.candidates[0].program.elements,
        vec![solo(JumpKind::ToeLoop, 1)]
    );
    let adult_request = ProfileJumpOptimizerRequest {
        profile: adult_profile(AdultCategory::Silver).id,
        ..request.clone()
    };
    let adult = optimize_profile_jumps(&registry, &adult_request).unwrap();
    assert_eq!(adult.candidates[0].base_hundredths, 220); // Two 1A, 1.10 each.
    let mut unsupported = custom;
    unsupported.id.version = "unsupported".into();
    unsupported.bindings[0].mechanism = RuleMechanism::Unsupported("future".into());
    registry.register(unsupported.clone()).unwrap();
    let rejected = ProfileJumpOptimizerRequest {
        profile: unsupported.id.clone(),
        ..request
    };
    assert!(
        matches!(optimize_profile_jumps(&registry, &rejected), Err(ProfileJumpOptimizerError::Selection(ProfileSelectionError { kind: ProfileErrorKind::Unsupported, profile, .. })) if profile == unsupported.id)
    );
}
