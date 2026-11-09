#![cfg(feature = "optimizer")]
use skating_program_lab::*;

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[test]
fn registered_category_supports_all_four_planning_workflows() {
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "synthetic-two-jump-category".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.category = "Synthetic planning fixture; no official category".into();
    profile.jumps.max_jumps = 2;
    profile.jumps.max_compounds = 0;
    profile.jumps.max_combinations = 0;
    profile.jumps.max_three_jump_compounds = 0;
    profile
        .jumps
        .allowed
        .retain(|p| p.kind == JumpKind::Lutz || p.kind == JumpKind::ToeLoop);
    let source = SourceReference {
        document: "Synthetic planning policy".into(),
        version: "test-v1".into(),
        location: "two solo jumps and one spin; not official rules".into(),
    };
    profile.sources.push(source.clone());
    for (rule, id) in [
        (ProfileRule::Count, "synthetic-count"),
        (ProfileRule::Eligibility, "synthetic-eligibility"),
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
            justification: "Synthetic category policy".into(),
        });
    }
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.max_spins = 1;
    policy.spins.source = source.clone();
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    profile.sources.extend([
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    let mut registry = ProfileRegistry::adult();
    registry
        .register_with_program_policy(profile.clone(), policy)
        .unwrap();
    let mut request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![],
    };
    let prove = |r: &PlanningSearchRequest| {
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&registry, r, &Continue).unwrap()
        else {
            panic!("maximum expected")
        };
        assert!(result.alternatives.len() <= 5);
        for alternative in &result.alternatives {
            assert_eq!(
                alternative.assessment.total.exact_hundredths,
                Some(i64::from(result.maximum_base_hundredths))
            );
            assert_eq!(
                alternative.assessment.planning_compliance(),
                Ok(PlanningCompliance::Compliant)
            );
            assert_eq!(alternative.assessment.planning_rule_inventory().count(), 8);
        }
        result
    };
    // Two Lutz solos (120), one level-1 spin (240), ChSq1 (350).
    assert_eq!(prove(&request).maximum_base_hundredths, 710);
    request
        .exclusions
        .push(PlanningExclusion::JumpKind(JumpKind::Lutz));
    assert_eq!(prove(&request).maximum_base_hundredths, 670);
    request.exclusions.clear();
    request.required.push(RequiredPlanningElement {
        id: "toe".into(),
        element: AssumedRecognition::new("1T"),
    });
    assert_eq!(prove(&request).maximum_base_hundredths, 690);
    request
        .exclusions
        .push(PlanningExclusion::JumpKind(JumpKind::Lutz));
    let restricted = prove(&request);
    assert_eq!(restricted.maximum_base_hundredths, 670);
    let assessed = assess_manual_bv(
        &registry,
        &ManualBvRequest {
            personal_limits: request.personal_limits,
            rotation_preference: request.rotation_preference,
            profile: request.profile.clone(),
            scoring_table: request.scoring_table,
            submission: SubmissionState::Complete,
            elements: restricted.alternatives[0].elements.clone(),
        },
    )
    .unwrap();
    assert_eq!(assessed.total.exact_hundredths, Some(670));
    assert_eq!(
        assessed.planning_compliance(),
        Ok(PlanningCompliance::Compliant)
    );
    // Independent manual input, not a solver reconstruction.
    let manual = assess_manual_bv(
        &registry,
        &ManualBvRequest {
            profile: request.profile.clone(),
            scoring_table: request.scoring_table,
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            submission: SubmissionState::Complete,
            elements: ["1T", "1T", "CLSp1", "ChSq1"]
                .map(AssumedRecognition::new)
                .to_vec(),
        },
    )
    .unwrap();
    assert_eq!(manual.total.exact_hundredths, Some(670));
    assert_eq!(
        manual.planning_compliance(),
        Ok(PlanningCompliance::Compliant)
    );
}

#[test]
fn invalid_registered_policy_does_not_publish_profile() {
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "invalid-synthetic-policy".into();
    profile.provenance = ProfileProvenance::Synthetic;
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.required.push(vec![]);
    profile.sources.extend([
        policy.spins.source.clone(),
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    let mut registry = ProfileRegistry::default();
    assert_eq!(
        registry
            .register_with_program_policy(profile.clone(), policy)
            .unwrap_err()
            .kind,
        ProfileErrorKind::Invalid
    );
    assert_eq!(
        registry.get(&profile.id).unwrap_err().kind,
        ProfileErrorKind::Unknown
    );
}

#[test]
fn unsupported_registered_mechanism_never_becomes_infeasibility_or_compliance() {
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "unsupported-construction-mechanism".into();
    profile.bindings[0].mechanism = RuleMechanism::Unsupported("future".into());
    let mut registry = ProfileRegistry::adult();
    registry.register(profile.clone()).unwrap();
    let search = PlanningSearchRequest {
        profile: profile.id.clone(),
        scoring_table: profile.scoring_table,
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        required: vec![],
        exclusions: vec![],
    };
    assert!(matches!(
        search_maximum_bv(&registry, &search, &Continue),
        Err(PlanningSearchError::Profile(ProfileSelectionError {
            kind: ProfileErrorKind::Unsupported,
            ..
        }))
    ));
    assert!(matches!(
        assess_manual_bv(
            &registry,
            &ManualBvRequest {
                profile: profile.id,
                scoring_table: profile.scoring_table,
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::default(),
                submission: SubmissionState::Complete,
                elements: vec![AssumedRecognition::new("1Lz")],
            }
        ),
        Err(ProfileSelectionError {
            kind: ProfileErrorKind::Unsupported,
            ..
        })
    ));
}

#[test]
fn registered_twelve_jump_policy_has_consistent_search_and_manual_capability() {
    // Independent synthetic policy: 12 slots, each exact single <=2, kinds<=3.
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "synthetic-twelve-slots".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.category = "Synthetic twelve-slot fixture".into();
    profile.jumps.max_jumps = 12;
    let source = SourceReference {
        document: "Synthetic twelve-slot specification".into(),
        version: "v1".into(),
        location: "12 jump slots; remaining Bronze mechanism limits retained".into(),
    };
    profile.sources.push(source.clone());
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.source = source.clone();
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    profile.sources.extend([
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    let mut registry = ProfileRegistry::adult();
    registry
        .register_with_program_policy(profile.clone(), policy)
        .unwrap();
    let r = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id.clone(),
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![PlanningExclusion::JumpKind(JumpKind::Waltz)],
    };
    let PlanningSearchOutcome::ProvenMaximum(maximum) =
        search_maximum_bv(&registry, &r, &Continue).unwrap()
    else {
        panic!("expected maximum")
    };
    // Two each Lz(.60), F(.50), Lo(.50), S(.40), T(.40) =4.80; spins4.80 +ChSq3.50.
    assert_eq!(maximum.maximum_base_hundredths, 1310);
    for alternative in maximum.alternatives {
        assert_eq!(
            alternative.assessment.planning_compliance(),
            Ok(PlanningCompliance::Compliant)
        );
        let manual = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::default(),
                profile: profile.id.clone(),
                scoring_table: profile.scoring_table,
                submission: SubmissionState::Complete,
                elements: alternative.elements,
            },
        )
        .unwrap();
        assert_eq!(
            manual.planning_compliance(),
            Ok(PlanningCompliance::Compliant)
        );
    }
}
