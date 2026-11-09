use skating_program_lab::*;

#[test]
fn manual_step_cap_preserves_credit_and_category_findings() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let request = ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        submission: SubmissionState::Draft,
        elements: ["ChSq1", "StSq3"].map(AssumedRecognition::new).to_vec(),
    };
    let baseline = assess_manual_bv(&registry, &request).unwrap();
    let limited = assess_manual_bv_with_step_limit(&registry, &request, Some(Level::One)).unwrap();
    assert_eq!(limited.total, baseline.total);
    assert_eq!(limited.legality, baseline.legality);
    assert_eq!(
        limited.elements[1].table_base.as_ref().unwrap().hundredths,
        350
    );
    assert_eq!(
        limited.personal_conflicts,
        vec![PersonalLimitConflict {
            element_index: 1,
            component_index: None,
            kind: PersonalLimitConflictKind::StepLevel {
                requested: Level::Three,
                maximum: Level::One
            },
        }]
    );
}

#[test]
fn selection_retains_lower_steps_and_intersects_category_ceiling() {
    for (registry, p, ceiling) in [
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
            4,
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Gold),
            2,
        ),
    ] {
        for (cap, maximum) in [
            Level::Basic,
            Level::One,
            Level::Two,
            Level::Three,
            Level::Four,
        ]
        .into_iter()
        .enumerate()
        {
            let choices = selectable_elements_with_step_limit(
                &registry,
                &p.id,
                p.scoring_table,
                &PersonalLimits::default(),
                Some(maximum),
            )
            .unwrap();
            let steps: Vec<_> = choices
                .iter()
                .filter(|s| s.kind == SelectableElementKind::StepSequence)
                .map(|s| s.row.notation)
                .collect();
            assert_eq!(
                steps,
                ["StSqB", "StSq1", "StSq2", "StSq3", "StSq4"][..=cap.min(ceiling)]
            );
        }
    }
}

#[cfg(feature = "optimizer")]
struct Continue;
#[cfg(feature = "optimizer")]
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn search_caps_steps_with_other_limits_exclusions_and_exact_requirements() {
    let registry = ProfileRegistry::adult();
    let p = adult_profile(AdultCategory::Gold);
    let mut request = PlanningSearchRequest {
        personal_limits: PersonalLimits {
            jump_rotations: vec![JumpRotationLimit {
                kind: JumpKind::Axel,
                maximum: JumpRotationClass::Single,
            }],
            spin_levels: vec![SpinLevelLimit {
                abbreviation: SpinAbbreviation {
                    kind: RegularSpinKind::Combination,
                    entry: SpinEntry::NonFlying,
                    change_foot: true,
                },
                maximum: Level::Basic,
            }],
        },
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        required: vec![RequiredPlanningElement {
            id: "step".into(),
            element: AssumedRecognition::new("StSq1"),
        }],
        exclusions: vec![PlanningExclusion::JumpKind(JumpKind::Lutz)],
    };
    let PlanningSearchOutcome::ProvenMaximum(maximum) =
        search_maximum_bv_with_step_limit(&registry, &request, Some(Level::One), &Continue)
            .unwrap()
    else {
        panic!("maximum");
    };
    for alternative in maximum.alternatives {
        assert!(alternative.elements.iter().any(|e| e.code == "StSq1"));
        let step = alternative
            .elements
            .iter()
            .position(|e| e.code == "StSq1")
            .unwrap();
        assert_eq!(
            alternative.assessment.elements[step]
                .table_base
                .as_ref()
                .unwrap()
                .hundredths,
            190
        );
        assert!(alternative.assessment.personal_conflicts.is_empty());
    }
    request
        .exclusions
        .push(PlanningExclusion::SpinKind(RegularSpinKind::Combination));
    assert!(matches!(
        search_maximum_bv_with_step_limit(&registry, &request, Some(Level::One), &Continue)
            .unwrap(),
        PlanningSearchOutcome::Infeasible { .. }
    ));
    request.exclusions.pop();
    request.required.insert(
        0,
        RequiredPlanningElement {
            id: "jump".into(),
            element: AssumedRecognition::new("1T"),
        },
    );
    let Err(PlanningSearchError::PersonalLimitConflict(conflicts)) =
        search_maximum_bv_with_step_limit(&registry, &request, Some(Level::Basic), &Continue)
    else {
        panic!("required conflict");
    };
    assert_eq!(conflicts[0].element_index, 1);
    assert_eq!(request.required[1].element.code, "StSq1");
}

#[cfg(feature = "optimizer")]
#[test]
fn automatic_step_selection_obeys_caps_without_changing_other_credit() {
    let registry = ProfileRegistry::adult();
    let p = adult_profile(AdultCategory::Gold);
    let request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        required: vec![],
        exclusions: vec![],
    };
    let mut values = vec![];
    for (cap, code) in [
        (Some(Level::Basic), "StSqB"),
        (Some(Level::One), "StSq1"),
        (Some(Level::Four), "StSq2"),
        (None, "StSq2"),
    ] {
        let PlanningSearchOutcome::ProvenMaximum(maximum) =
            search_maximum_bv_with_step_limit(&registry, &request, cap, &Continue).unwrap()
        else {
            panic!("maximum");
        };
        values.push(maximum.maximum_base_hundredths);
        assert!(maximum
            .alternatives
            .iter()
            .all(|a| a.elements.iter().any(|e| e.code == code)));
    }
    assert_eq!(values[1] - values[0], 30);
    assert_eq!(values[2] - values[1], 80);
    assert_eq!(values[2], values[3]);
}
