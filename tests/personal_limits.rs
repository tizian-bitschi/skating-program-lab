use skating_program_lab::*;

#[test]
fn selection_caps_exact_spin_abbreviation_including_reduced_variants() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let limits = PersonalLimits {
        spin_levels: vec![SpinLevelLimit {
            abbreviation: SpinAbbreviation {
                kind: RegularSpinKind::Combination,
                entry: SpinEntry::NonFlying,
                change_foot: true,
            },
            maximum: Level::One,
        }],
        ..PersonalLimits::default()
    };
    let choices =
        selectable_elements_with_limits(&registry, &p.id, p.scoring_table, &limits).unwrap();
    for code in ["CCoSpB", "CCoSp1", "CCoSpBV", "CCoSp1V", "FCCoSp2", "CoSp2"] {
        assert!(
            choices.iter().any(|c| c.row.notation == code),
            "missing {code}"
        );
    }
    for code in [
        "CCoSp2", "CCoSp3", "CCoSp4", "CCoSp2V", "CCoSp3V", "CCoSp4V",
    ] {
        assert!(
            !choices.iter().any(|c| c.row.notation == code),
            "unexpected {code}"
        );
    }
}

#[test]
fn manual_limits_report_components_without_changing_values_or_legality() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let mut request = ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: p.id,
        scoring_table: p.scoring_table,
        submission: SubmissionState::Draft,
        elements: ["3Lz+2T", "2T+3Lz", "2T+2A+SEQ", "CCoSp2V"]
            .iter()
            .map(|c| AssumedRecognition {
                code: (*c).into(),
                direct_transitions_assumed: c.ends_with("SEQ"),
            })
            .collect(),
    };
    let baseline = assess_manual_bv(&registry, &request).unwrap();
    request.personal_limits = PersonalLimits {
        jump_rotations: vec![
            JumpRotationLimit {
                kind: JumpKind::Lutz,
                maximum: JumpRotationClass::Double,
            },
            JumpRotationLimit {
                kind: JumpKind::Axel,
                maximum: JumpRotationClass::Single,
            },
        ],
        spin_levels: vec![SpinLevelLimit {
            abbreviation: SpinAbbreviation {
                kind: RegularSpinKind::Combination,
                entry: SpinEntry::NonFlying,
                change_foot: true,
            },
            maximum: Level::One,
        }],
    };
    let limited = assess_manual_bv(&registry, &request).unwrap();
    assert_eq!(limited.total, baseline.total);
    assert_eq!(limited.legality, baseline.legality);
    assert_eq!(
        limited
            .personal_conflicts
            .iter()
            .map(|c| (c.element_index, c.component_index))
            .collect::<Vec<_>>(),
        [(0, Some(0)), (1, Some(1)), (2, Some(1)), (3, None)]
    );
    assert_eq!(
        limited.personal_conflicts[1].kind,
        PersonalLimitConflictKind::JumpRotation {
            kind: JumpKind::Lutz,
            requested: 3,
            maximum: JumpRotationClass::Double
        }
    );
    assert_eq!(
        limited.personal_conflicts[3].kind,
        PersonalLimitConflictKind::SpinLevel {
            abbreviation: request.personal_limits.spin_levels[0].abbreviation,
            requested: Level::Two,
            maximum: Level::One
        }
    );
}

#[test]
fn jump_selection_applies_listed_classes_at_every_position() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let limits = PersonalLimits {
        jump_rotations: vec![
            JumpRotationLimit {
                kind: JumpKind::Lutz,
                maximum: JumpRotationClass::Double,
            },
            JumpRotationLimit {
                kind: JumpKind::Axel,
                maximum: JumpRotationClass::Single,
            },
        ],
        ..PersonalLimits::default()
    };
    let choices = selectable_jump_elements_with_limits(
        &registry,
        &p.id,
        p.scoring_table,
        RotationPreference::AllowMixedDirections,
        &limits,
    )
    .unwrap();
    for code in ["2Lz+2T", "2T+2Lz", "2Lz+1A+SEQ", "2T+Eu+2F"] {
        assert!(choices.iter().any(|c| c.code == code), "missing {code}");
    }
    for code in ["3Lz+2T", "2T+3Lz", "2T+2A+SEQ", "2T+1A+2A+SEQ"] {
        assert!(!choices.iter().any(|c| c.code == code), "unexpected {code}");
    }
    assert_eq!(JumpRotationClass::try_from(0), Err(0));
    assert_eq!(JumpRotationClass::try_from(6), Err(6));
}

#[cfg(feature = "optimizer")]
#[path = "support/generation_defaults.rs"]
#[allow(dead_code)]
mod generation_defaults;

#[cfg(feature = "optimizer")]
#[test]
fn generation_filters_supplied_candidates_and_records_limits() {
    let registry = ProfileRegistry::adult();
    let p = adult_profile(AdultCategory::Gold);
    let mut request = generation_defaults::request(p);
    request.jump_kinds = vec![JumpKind::Lutz, JumpKind::ToeLoop];
    request.rotations = vec![1, 2];
    request.solo_jumps = true;
    request.direct_combinations = true;
    request
        .personal_limits
        .jump_rotations
        .push(JumpRotationLimit {
            kind: JumpKind::Lutz,
            maximum: JumpRotationClass::Single,
        });
    let generated = generate_program_candidates(&registry, &request).unwrap();
    assert_eq!(generated.audit.personal_limits, request.personal_limits);
    assert!(generated.variants.iter().any(|v| matches!(
        &v.element,
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1)
        }))
    )));
    assert!(!generated.variants.iter().any(|v| match &v.element {
        PlannedElement::Jump(JumpElement::Solo(j)) =>
            j.kind == Some(JumpKind::Lutz) && j.rotations == Some(2),
        PlannedElement::Jump(JumpElement::Combination(parts)) => parts.iter().any(|p| matches!(
            p,
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(2)
            })
        )),
        _ => false,
    }));
    let baseline = generate_program_candidates(
        &registry,
        &generation_defaults::request(adult_profile(AdultCategory::Gold)),
    )
    .unwrap();
    assert!(baseline.variants.is_empty());
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
fn maximum_proof_carries_limits_and_required_conflicts_are_localized() {
    let registry = ProfileRegistry::adult();
    let p = adult_profile(AdultCategory::Silver);
    let mut request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        required: vec![],
        exclusions: vec![],
    };
    request
        .personal_limits
        .jump_rotations
        .push(JumpRotationLimit {
            kind: JumpKind::Axel,
            maximum: JumpRotationClass::Single,
        });
    let PlanningSearchOutcome::ProvenMaximum(maximum) =
        search_maximum_bv(&registry, &request, &Continue).unwrap()
    else {
        panic!("maximum");
    };
    assert_eq!(maximum.maximum_base_hundredths, 1430);
    assert_eq!(maximum.personal_limits, request.personal_limits);
    request
        .exclusions
        .push(PlanningExclusion::JumpKind(JumpKind::Axel));
    let PlanningSearchOutcome::ProvenMaximum(maximum) =
        search_maximum_bv(&registry, &request, &Continue).unwrap()
    else {
        panic!("maximum");
    };
    assert_eq!(maximum.maximum_base_hundredths, 1300);
    request.personal_limits.spin_levels.push(SpinLevelLimit {
        abbreviation: SpinAbbreviation {
            kind: RegularSpinKind::Combination,
            entry: SpinEntry::NonFlying,
            change_foot: true,
        },
        maximum: Level::One,
    });
    request.required.push(RequiredPlanningElement {
        id: "original".into(),
        element: AssumedRecognition::new("CCoSp2"),
    });
    let Err(PlanningSearchError::PersonalLimitConflict(conflicts)) =
        search_maximum_bv(&registry, &request, &Continue)
    else {
        panic!("required conflict");
    };
    assert_eq!(conflicts[0].element_index, 0);
}

#[test]
fn every_spin_maximum_retains_all_lower_levels_and_choreography() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let abbreviation = SpinAbbreviation {
        kind: RegularSpinKind::Combination,
        entry: SpinEntry::NonFlying,
        change_foot: true,
    };
    let levels = [
        Level::Basic,
        Level::One,
        Level::Two,
        Level::Three,
        Level::Four,
    ];
    let codes = ["CCoSpB", "CCoSp1", "CCoSp2", "CCoSp3", "CCoSp4"];
    for (boundary, maximum) in levels.into_iter().enumerate() {
        let limits = PersonalLimits {
            spin_levels: vec![SpinLevelLimit {
                abbreviation,
                maximum,
            }],
            ..PersonalLimits::default()
        };
        let choices =
            selectable_elements_with_limits(&registry, &p.id, p.scoring_table, &limits).unwrap();
        for (i, code) in codes.iter().enumerate() {
            assert_eq!(
                choices.iter().any(|c| c.row.notation == *code),
                i <= boundary,
                "{maximum:?} {code}"
            );
            assert_eq!(
                choices.iter().any(|c| c.row.notation == format!("{code}V")),
                i <= boundary,
                "{maximum:?} {code}V"
            );
        }
        for code in ["FCCoSp4", "CoSp4", "ChSp1", "ChSq1", "StSq4"] {
            assert!(
                choices.iter().any(|c| c.row.notation == code),
                "unrelated {code}"
            );
        }
    }
}

#[test]
fn every_jump_kind_uses_listed_classes_and_category_ceiling() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    for (kind, suffix) in [
        (JumpKind::ToeLoop, "T"),
        (JumpKind::Salchow, "S"),
        (JumpKind::Loop, "Lo"),
        (JumpKind::Flip, "F"),
        (JumpKind::Lutz, "Lz"),
        (JumpKind::Axel, "A"),
    ] {
        for maximum in [
            JumpRotationClass::Single,
            JumpRotationClass::Double,
            JumpRotationClass::Triple,
            JumpRotationClass::Quadruple,
            JumpRotationClass::Quintuple,
        ] {
            let limits = PersonalLimits {
                jump_rotations: vec![JumpRotationLimit { kind, maximum }],
                ..PersonalLimits::default()
            };
            let choices =
                selectable_elements_with_limits(&registry, &p.id, p.scoring_table, &limits)
                    .unwrap();
            // Published selection supports five listed classes; 5A is absent from this SOV.
            for rotation in 1..=5 {
                assert_eq!(
                    choices
                        .iter()
                        .any(|c| c.row.notation == format!("{rotation}{suffix}")),
                    rotation <= maximum as u8 && !(kind == JumpKind::Axel && rotation == 5),
                    "{kind:?} {maximum:?} {rotation}"
                );
            }
            let silver = adult_profile(AdultCategory::Silver);
            let adult_choices = selectable_elements_with_limits(
                &ProfileRegistry::adult(),
                &silver.id,
                silver.scoring_table,
                &limits,
            )
            .unwrap();
            assert!(!adult_choices.iter().any(|c| matches!(
                c.kind,
                SelectableElementKind::SoloJump { rotations: 2.., .. }
            )));
        }
    }
}

#[test]
fn detailed_manual_assessment_keeps_triple_lutz_value_and_legality() {
    let registry = ProfileRegistry::senior();
    let p = senior_profile(SeniorCategory::Women);
    let mut request = ManualProgramRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        policy: senior_program_policy(SeniorCategory::Women),
        submission: SubmissionState::Draft,
        entries: vec![ManualProgramEntry {
            planned: PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(3),
            })),
            evidence: ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            },
            intended_recognition: Some(PanelRecognition {
                ruleset: RulesetId::IsuSenior2026_27June12,
                source: None,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Recognized,
                details: Some(
                    RecognitionDetails::new(
                        vec![ComponentCode::try_from("3Lz").unwrap()],
                        None,
                        Some(vec![]),
                    )
                    .unwrap(),
                ),
            }),
            scenario: None,
        }],
        request_hypothetical_total: false,
        second_half_boundary: None,
    };
    let baseline = assess_manual_program(&registry, &request).unwrap();
    request
        .personal_limits
        .jump_rotations
        .push(JumpRotationLimit {
            kind: JumpKind::Lutz,
            maximum: JumpRotationClass::Double,
        });
    let limited = assess_manual_program(&registry, &request).unwrap();
    assert_eq!(limited.intended_base.exact_hundredths, Some(590));
    assert_eq!(limited.intended_base, baseline.intended_base);
    assert_eq!(limited.validation, baseline.validation);
    assert_eq!(limited.status, baseline.status);
    assert_eq!(
        limited.personal_conflicts,
        [PersonalLimitConflict {
            element_index: 0,
            component_index: None,
            kind: PersonalLimitConflictKind::JumpRotation {
                kind: JumpKind::Lutz,
                requested: 3,
                maximum: JumpRotationClass::Double
            }
        }]
    );
}

#[cfg(feature = "optimizer")]
fn basic_spin_limits() -> Vec<SpinLevelLimit> {
    [
        RegularSpinKind::Camel,
        RegularSpinKind::Sit,
        RegularSpinKind::Upright,
        RegularSpinKind::Layback,
        RegularSpinKind::Combination,
    ]
    .into_iter()
    .flat_map(|kind| {
        [SpinEntry::Flying, SpinEntry::NonFlying]
            .into_iter()
            .flat_map(move |entry| {
                [false, true]
                    .into_iter()
                    .map(move |change_foot| SpinLevelLimit {
                        abbreviation: SpinAbbreviation {
                            kind,
                            entry,
                            change_foot,
                        },
                        maximum: Level::Basic,
                    })
            })
    })
    .collect()
}

#[cfg(feature = "optimizer")]
#[test]
fn genuinely_limited_adult_and_senior_maxima_match_independent_bounds() {
    let mut limits = PersonalLimits {
        spin_levels: basic_spin_limits(),
        ..PersonalLimits::default()
    };
    for kind in [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
        JumpKind::Axel,
    ] {
        limits.jump_rotations.push(JumpRotationLimit {
            kind,
            maximum: JumpRotationClass::Single,
        });
    }
    for (registry, p, expected) in [
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Bronze),
            1020,
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
            2170,
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Men),
            2170,
        ),
    ] {
        let request = PlanningSearchRequest {
            personal_limits: limits.clone(),
            rotation_preference: RotationPreference::default(),
            profile: p.id,
            scoring_table: p.scoring_table,
            required: vec![],
            exclusions: vec![],
        };
        let PlanningSearchOutcome::ProvenMaximum(maximum) =
            search_maximum_bv(&registry, &request, &Continue).unwrap()
        else {
            panic!("maximum");
        };
        // Bronze: five jump components = 2*60+3*50, two Basic spins = 400, ChSq = 350.
        // Senior: nine listed singles at most: 3*110+3*60+3*50=660.
        // ChSp reserves one of three spin slots: two Basic spins (400), StSq4 (410),
        // ChSp1+ChSq1 (700). A sequence and a combination pack all nine into six slots.
        assert_eq!(
            maximum.maximum_base_hundredths, expected,
            "{}",
            request.profile.name
        );
        assert!(maximum
            .alternatives
            .iter()
            .all(|a| a.assessment.personal_conflicts.is_empty()));
        assert_eq!(maximum.personal_limits, limits);
    }
}

#[cfg(feature = "optimizer")]
#[test]
fn impossible_complete_repertoire_is_not_relaxed() {
    let p = senior_profile(SeniorCategory::Women);
    let request = PlanningSearchRequest {
        personal_limits: PersonalLimits {
            spin_levels: basic_spin_limits(),
            ..PersonalLimits::default()
        },
        rotation_preference: RotationPreference::default(),
        profile: p.id,
        scoring_table: p.scoring_table,
        required: vec![],
        exclusions: vec![PlanningExclusion::SpinKind(RegularSpinKind::Combination)],
    };
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap(),
        PlanningSearchOutcome::Infeasible { .. }
    ));
}

#[cfg(feature = "optimizer")]
#[test]
fn generator_omits_supplied_over_limit_variants_and_intersects_legacy_limits() {
    let registry = ProfileRegistry::adult();
    let mut q = generation_defaults::request(adult_profile(AdultCategory::Gold));
    q.manual_variants.push(ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(3),
        })),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
        recognition_assumed: true,
    });
    q.jump_kinds = vec![JumpKind::ToeLoop, JumpKind::Lutz];
    q.rotations = vec![1, 2];
    q.solo_jumps = true;
    q.personal_limits.jump_rotations.push(JumpRotationLimit {
        kind: JumpKind::Lutz,
        maximum: JumpRotationClass::Double,
    });
    q.skill_limits.max_jump_rotations = Some(1);
    let generated = generate_program_candidates(&registry, &q).unwrap();
    assert_eq!(generated.audit.manual_definitions_checked, 1);
    assert!(generated.variants.iter().all(|v| matches!(
        v.element,
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            rotations: Some(1),
            ..
        }))
    )));
    assert_eq!(generated.variants.len(), 2);
    assert!(generated
        .audit
        .omissions
        .iter()
        .any(|o| o.variant == q.manual_variants[0]));
}

#[cfg(feature = "optimizer")]
#[test]
fn assumed_generation_filters_all_supported_v_levels_and_preserves_input_indices() {
    let registry = ProfileRegistry::senior();
    let mut q = generation_defaults::request(senior_profile(SeniorCategory::Women));
    q.personal_limits.spin_levels.push(SpinLevelLimit {
        abbreviation: SpinAbbreviation {
            kind: RegularSpinKind::Combination,
            entry: SpinEntry::NonFlying,
            change_foot: true,
        },
        maximum: Level::One,
    });
    let codes: Vec<_> = ["CCoSpB", "CCoSp1V", "CCoSp2V", "CCoSp4", "FCCoSp4"]
        .into_iter()
        .map(AssumedRecognition::new)
        .collect();
    let generated =
        generate_program_candidates_with_assumed_recognition(&registry, &q, &codes).unwrap();
    assert_eq!(generated.input_indices, [0, 1, 4]);
    assert_eq!(
        generated.recognition,
        [codes[0].clone(), codes[1].clone(), codes[4].clone()]
    );
    assert_eq!(generated.domain.audit.personal_limits, q.personal_limits);
}
