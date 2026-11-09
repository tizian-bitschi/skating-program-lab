use skating_program_lab::*;

#[test]
fn bronze_waltz_is_ignored_and_retains_planned_identity() {
    let registry = ProfileRegistry::adult();
    let request = ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: adult_profile(AdultCategory::Bronze).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        submission: SubmissionState::Draft,
        elements: ["W", "W", "W", "W", "W", "1Lz", "1Lz"]
            .map(AssumedRecognition::new)
            .to_vec(),
    };
    let assessment = assess_manual_bv(&registry, &request).unwrap();
    assert_eq!(assessment.total.exact_hundredths, Some(120));
    assert!(
        assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty(),
        "{:?}",
        assessment.legality.validation.validation.findings
    );
    assert!(selectable_jump_elements(
        &registry,
        &request.profile,
        request.scoring_table,
        request.rotation_preference
    )
    .unwrap()
    .contains(&AssumedRecognition::new("W")));
}
#[test]
fn silver_waltz_repeats_as_single_axel_and_consumes_a_jump_box() {
    let request = ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: adult_profile(AdultCategory::Silver).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        submission: SubmissionState::Draft,
        elements: ["W", "W", "1A"].map(AssumedRecognition::new).to_vec(),
    };
    let a = assess_manual_bv(&ProfileRegistry::adult(), &request).unwrap();
    assert!(a
        .legality
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation && f.element_index == Some(2)));
    assert_eq!(a.elements[0].table_base.as_ref().unwrap().hundredths, 0);
}

#[cfg(feature = "optimizer")]
#[test]
fn ignored_waltz_preserves_exact_required_multiplicity_and_identity() {
    struct Continue;
    impl PlanningSearchControl for Continue {
        fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
            false
        }
    }
    let request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: adult_profile(AdultCategory::Bronze).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        required: (0..7)
            .map(|i| RequiredPlanningElement {
                id: format!("w{i}"),
                element: AssumedRecognition::new("W"),
            })
            .collect(),
        exclusions: vec![],
    };
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::adult(), &request, &Continue).unwrap()
    else {
        panic!("maximum expected")
    };
    assert_eq!(result.maximum_base_hundredths, 1100);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
    assert_eq!(result.alternatives.len(), 5);
    let mut identities = std::collections::BTreeSet::new();
    for program in result.alternatives {
        assert_eq!(program.elements.iter().filter(|e| e.code == "W").count(), 7);
        let mut identity: Vec<_> = program.elements.iter().map(|e| e.code.clone()).collect();
        identity.sort();
        assert!(identities.insert(identity));
        for (input_index, required) in request.required.iter().enumerate() {
            let index = program
                .provenance
                .iter()
                .position(|p| {
                    *p == PlanningProvenance::Required {
                        id: required.id.clone(),
                        input_index,
                    }
                })
                .unwrap();
            assert_eq!(program.elements[index], required.element);
        }
        assert_eq!(
            program
                .provenance
                .iter()
                .filter(|p| matches!(p, PlanningProvenance::Required { .. }))
                .count(),
            7
        );
        assert_eq!(program.assessment.total.exact_hundredths, Some(1100));
        let assessment = assess_manual_bv(
            &ProfileRegistry::adult(),
            &ManualBvRequest {
                personal_limits: request.personal_limits.clone(),
                rotation_preference: request.rotation_preference,
                profile: request.profile.clone(),
                scoring_table: request.scoring_table,
                submission: SubmissionState::Complete,
                elements: program.elements.clone(),
            },
        )
        .unwrap();
        assert_eq!(assessment, program.assessment);
        assert_eq!(
            assessment.planning_compliance(),
            Ok(PlanningCompliance::Compliant)
        );
    }
    assert_eq!(identities.len(), 5);
}
#[test]
fn declared_called_waltz_compounds_have_zero_component_value_and_axel_semantics() {
    let mut registry = ProfileRegistry::adult();
    registry
        .register(senior_profile(SeniorCategory::Women))
        .unwrap();
    for (profile, solo, compound) in [
        (adult_profile(AdultCategory::Silver).id, "W", "W+1T"),
        (
            adult_profile(AdultCategory::Masters).id,
            "W(1A<<)",
            "W(1A<<)+1T",
        ),
        (
            senior_profile(SeniorCategory::Women).id,
            "W(1A<<)",
            "W(1A<<)+1T",
        ),
    ] {
        let request = ManualBvRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile,
            scoring_table: ScoringTableId::Isu2786June2_2026,
            submission: SubmissionState::Draft,
            elements: vec![AssumedRecognition::new(compound)],
        };
        let a = assess_manual_bv(&registry, &request).unwrap();
        assert_eq!(a.total.exact_hundredths, Some(40), "{compound}: {a:?}");
        let choices = selectable_jump_elements_with_exclusions(
            &registry,
            &request.profile,
            request.scoring_table,
            request.rotation_preference,
            &request.personal_limits,
            &[PlanningExclusion::JumpKind(JumpKind::Axel)],
        )
        .unwrap();
        assert!(!choices.iter().any(|c| c.code.contains(solo)));
        let choices = selectable_jump_elements_with_exclusions(
            &registry,
            &request.profile,
            request.scoring_table,
            request.rotation_preference,
            &request.personal_limits,
            &[PlanningExclusion::ExactElement(solo.into())],
        )
        .unwrap();
        assert!(choices.contains(&AssumedRecognition::new("1A")));
        assert!(choices.contains(&AssumedRecognition::new(compound)));
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn ignored_waltz_has_one_empty_zero_optimum() {
    struct Continue;
    impl PlanningSearchControl for Continue {
        fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
            false
        }
    }
    let registry = ProfileRegistry::adult();
    let profile = adult_profile(AdultCategory::Bronze).id;
    let scoring_table = ScoringTableId::Isu2786June2_2026;
    let exclusions = selectable_elements(&registry, &profile, scoring_table)
        .unwrap()
        .iter()
        .map(|e| PlanningExclusion::ExactElement(e.row.notation.into()))
        .chain(
            [
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
                JumpKind::Lutz,
            ]
            .map(PlanningExclusion::JumpKind),
        )
        .collect();
    let request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile,
        scoring_table,
        required: vec![],
        exclusions,
    };
    let PlanningSearchOutcome::ProvenMaximum(maximum) =
        search_maximum_bv(&registry, &request, &Continue).unwrap()
    else {
        panic!("zero maximum")
    };
    assert_eq!(maximum.maximum_base_hundredths, 0);
    assert_eq!(maximum.alternatives.len(), 1);
    assert!(maximum.alternatives[0].elements.is_empty());
    assert_eq!(
        maximum.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    let assessment = assess_manual_bv(
        &registry,
        &ManualBvRequest {
            personal_limits: request.personal_limits.clone(),
            rotation_preference: request.rotation_preference,
            profile: request.profile.clone(),
            scoring_table: request.scoring_table,
            submission: SubmissionState::Complete,
            elements: vec![],
        },
    )
    .unwrap();
    assert_eq!(assessment, maximum.alternatives[0].assessment);
    assert_eq!(
        assessment.planning_compliance(),
        Ok(PlanningCompliance::Compliant)
    );
}
#[cfg(feature = "optimizer")]
#[test]
fn called_and_ignored_waltz_requirements_have_localized_exclusion_conflicts() {
    struct Continue;
    impl PlanningSearchControl for Continue {
        fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
            false
        }
    }
    for (category, code) in [
        (AdultCategory::Bronze, "W"),
        (AdultCategory::Silver, "W"),
        (AdultCategory::Masters, "W(1A<<)"),
    ] {
        let request = PlanningSearchRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile: adult_profile(category).id,
            scoring_table: ScoringTableId::Isu2786June2_2026,
            required: vec![RequiredPlanningElement {
                id: "waltz".into(),
                element: AssumedRecognition::new(code),
            }],
            exclusions: vec![PlanningExclusion::ExactElement(code.into())],
        };
        assert!(
            matches!(search_maximum_bv(&ProfileRegistry::adult(), &request, &Continue), Err(PlanningSearchError::InvalidRequest { diagnostics, .. }) if diagnostics[0].required_input_indices == [0])
        );
    }
}

#[test]
fn called_waltz_satisfies_axel_requirement_and_preserves_clean_axel_identity() {
    let registry = ProfileRegistry::adult();
    let request = ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: adult_profile(AdultCategory::Masters).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        submission: SubmissionState::Draft,
        elements: vec![AssumedRecognition::new("W(1A<<)")],
    };
    let a = assess_manual_bv(&registry, &request).unwrap();
    assert!(!a
        .legality
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id.contains("axel-required")));
    assert_eq!(a.total.exact_hundredths, Some(0));
    let choices = selectable_jump_elements_with_exclusions(
        &registry,
        &request.profile,
        request.scoring_table,
        request.rotation_preference,
        &request.personal_limits,
        &[PlanningExclusion::ExactElement("1A".into())],
    )
    .unwrap();
    assert!(choices.contains(&AssumedRecognition::new("W(1A<<)")));
    assert!(!choices.contains(&AssumedRecognition::new("1A")));
}
