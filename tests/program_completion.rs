#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::{collections::BTreeSet, time::Duration};

#[path = "support/generation_defaults.rs"]
#[allow(dead_code)]
mod generation_defaults;

fn pos(position: SpinPosition, n: u16) -> SpinStep {
    SpinStep::Position {
        position: Some(position),
        revolutions: Some(n),
        feature_attempts: Some(vec![]),
    }
}
fn jump(kind: JumpKind) -> ProgramVariant {
    ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(kind),
            rotations: Some(1),
        })),
        evidence: ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
        recognition_assumed: true,
    }
}
fn spin(
    kind: RegularSpinKind,
    entry: SpinEntry,
    changed: bool,
    target: Level,
    features: Vec<SpinFeatureAttempt>,
) -> ProgramVariant {
    let steps = if kind == RegularSpinKind::Combination {
        vec![
            pos(SpinPosition::Camel, 3),
            pos(SpinPosition::Sit, 3),
            SpinStep::ChangeFoot,
            pos(SpinPosition::Upright, 3),
        ]
    } else {
        vec![pos(
            if kind == RegularSpinKind::Camel {
                SpinPosition::Camel
            } else {
                SpinPosition::Sit
            },
            8,
        )]
    };
    ProgramVariant {
        element: PlannedElement::RegularSpin(RegularSpin {
            entry: Some(entry),
            steps: Some(steps),
        }),
        evidence: ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry,
                    change_foot: changed,
                }),
                planned_level: Some(target),
                feature_attempts: Some(features),
                panel: None,
                final_upright_wind_up: None,
            },
            local_feature_qualification_assumed: true,
        },
        recognition_assumed: true,
    }
}
fn feature(f: SpinFeature, outcome: FeatureAttemptOutcome) -> SpinFeatureAttempt {
    SpinFeatureAttempt {
        feature: f,
        position_step: 0,
        outcome,
    }
}
fn chsp() -> ProgramVariant {
    ProgramVariant {
        element: PlannedElement::ChoreographicSpin,
        evidence: ElementPlanEvidence::ChoreographicSpin {
            consecutive_revolutions_per_foot: Some(vec![3]),
        },
        recognition_assumed: true,
    }
}

fn entry(id: &str, v: ProgramVariant) -> CompletionEntry {
    let details = match &v.element {
        PlannedElement::Jump(JumpElement::Solo(j)) => RecognitionDetails::new(
            vec![ComponentCode::try_from(match j.kind.unwrap() {
                JumpKind::Axel => "1A",
                JumpKind::ToeLoop => "1T",
                JumpKind::Salchow => "1S",
                _ => panic!("fixture jump"),
            })
            .unwrap()],
            None,
            Some(vec![]),
        )
        .unwrap(),
        PlannedElement::RegularSpin(_) => {
            let ElementPlanEvidence::Spin { spin, .. } = &v.evidence else {
                panic!()
            };
            RecognitionDetails::new(
                vec![
                    ComponentCode::try_from(match spin.abbreviation.unwrap().kind {
                        RegularSpinKind::Sit => "SSp",
                        RegularSpinKind::Camel => "CSp",
                        RegularSpinKind::Combination => "CCoSp",
                        _ => panic!(),
                    })
                    .unwrap(),
                ],
                spin.planned_level,
                Some(vec![]),
            )
            .unwrap()
        }
        _ => panic!("fixture element"),
    };
    CompletionEntry {
        id: id.into(),
        locked: true,
        entry: ManualProgramEntry {
            planned: v.element,
            evidence: v.evidence,
            scenario: None,
            intended_recognition: Some(PanelRecognition {
                ruleset: RulesetId::IsuAdult2026_27July16,
                source: None,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Recognized,
                details: Some(details),
            }),
        },
    }
}
fn sit() -> ProgramVariant {
    spin(
        RegularSpinKind::Sit,
        SpinEntry::NonFlying,
        false,
        Level::One,
        vec![feature(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Successful,
        )],
    )
}
fn request() -> ProgramCompletionRequest {
    let p = adult_profile(AdultCategory::Silver);
    ProgramCompletionRequest {
        season: p.season,
        profile: p.id,
        scoring_table: p.scoring_table,
        policy: adult_program_policy(AdultCategory::Silver),
        original: vec![entry("axel", jump(JumpKind::Axel)), entry("sit", sit())],
        variants: vec![jump(JumpKind::ToeLoop), jump(JumpKind::Salchow)],
        generated_domain: None,
        exclusions: vec![],
        global_exclusions: vec![],
        insertion: CompletionInsertionPolicy {
            boundaries: vec![0, 1, 2],
            absolute_positions: vec![],
        },
        min_additions: 2,
        max_additions: 2,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        max_proposals: 100_000,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    }
}
fn result(r: &ProgramCompletionRequest) -> ProgramCompletionResult {
    complete_program(&ProfileRegistry::adult(), r).unwrap()
}
fn words(r: &ProgramCompletionResult) -> BTreeSet<Vec<usize>> {
    r.candidates
        .iter()
        .map(|c| {
            c.provenance
                .iter()
                .map(|p| match p {
                    CompletionProvenance::Original { original_index, .. } => 2 + original_index,
                    CompletionProvenance::Inserted { variant_index, .. } => *variant_index,
                })
                .collect()
        })
        .collect()
}

#[test]
fn silver_all_optimal_words_match_independent_oracle_and_assessment() {
    let req = request();
    let r = result(&req);
    // Independent literal coefficients: 1T=.40, 1S=.40, 1A=1.10, SSp1=1.60.
    // Enumerate length-four words, exactly one of each immutable original,
    // axel before sit, at most two identical inserted jumps.
    let mut expected = BTreeSet::new();
    for mut code in 0..4usize.pow(4) {
        let w: Vec<_> = (0..4)
            .map(|_| {
                let v = code % 4;
                code /= 4;
                v
            })
            .collect();
        if w.iter().filter(|i| **i == 2).count() == 1
            && w.iter().filter(|i| **i == 3).count() == 1
            && w.iter().position(|i| *i == 2) < w.iter().position(|i| *i == 3)
        {
            expected.insert(w);
        }
    }
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    assert_eq!(words(&r), expected);
    assert_eq!(r.candidates.len(), 24);
    for c in &r.candidates {
        assert_eq!(
            (
                c.existing_base_hundredths,
                c.added_base_hundredths,
                c.total_base_hundredths
            ),
            (270, 80, 350)
        );
        assert_eq!(c.assessment.intended_base.exact_hundredths, Some(350));
        assert_eq!(
            assess_manual_program(&ProfileRegistry::adult(), &c.assessment.request).unwrap(),
            c.assessment
        );
        assert!(!c.validation.applied_rules.is_empty());
        for (i, p) in c.provenance.iter().enumerate() {
            if let CompletionProvenance::Original {
                original_index, id, ..
            } = p
            {
                assert_eq!(id, &req.original[*original_index].id);
                assert_eq!(
                    c.assessment.request.entries[i],
                    req.original[*original_index].entry
                );
            }
        }
    }
}

#[test]
fn boundaries_absolute_positions_and_rebinding_are_zero_based() {
    let mut req = request();
    req.insertion.boundaries = vec![1];
    req.insertion.absolute_positions = vec![("axel".into(), 0), ("sit".into(), 3)];
    req.original[1]
        .entry
        .intended_recognition
        .as_mut()
        .unwrap()
        .planned_element = Some(1);
    req.original[1].entry.scenario = Some(ManualElementScenario {
        recognition: req.original[1].entry.intended_recognition.clone().unwrap(),
        annotations: vec![],
        final_goe: None,
        fall: None,
        execution: None,
    });
    let r = result(&req);
    assert_eq!(r.candidates.len(), 4);
    for c in &r.candidates {
        assert_eq!(
            c.assessment.request.entries[3]
                .scenario
                .as_ref()
                .unwrap()
                .recognition
                .planned_element,
            Some(3)
        );
        assert_eq!(
            c.assessment.request.entries[3]
                .intended_recognition
                .as_ref()
                .unwrap()
                .planned_element,
            Some(3)
        );
        for (ordinal, p) in c.provenance[1..3].iter().enumerate() {
            assert!(
                matches!(p, CompletionProvenance::Inserted { boundary: 1, ordinal: o, .. } if *o == ordinal)
            );
        }
    }
    req.insertion.boundaries.clear();
    assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
}

#[test]
fn exclusions_only_filter_additions_and_global_conflicts_name_originals() {
    let mut req = request();
    req.exclusions = vec![
        PersonalExclusion::JumpKind(JumpKind::Axel),
        PersonalExclusion::SpinKind(RegularSpinKind::Sit),
        PersonalExclusion::FlyingEntry,
    ];
    assert_eq!(result(&req).candidates.len(), 24);
    req.global_exclusions = vec![PersonalExclusion::JumpKind(JumpKind::Axel)];
    assert!(
        matches!(complete_program(&ProfileRegistry::adult(), &req), Err(ProgramCompletionError::GlobalExclusionConflict(ids)) if ids == vec!["axel"])
    );
}

#[test]
fn draft_missing_spin_is_repairable_but_illegal_and_unknown_originals_are_distinct() {
    let mut req = request();
    req.original.pop();
    req.insertion.boundaries = vec![0, 1];
    req.variants = vec![sit()];
    req.min_additions = 1;
    req.max_additions = 1;
    assert_eq!(result(&req).status, OptimizerStatus::OptimalProven);
    req.original[0].entry.planned = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(2),
    }));
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::ImmutableViolation(_))
    ));
    req.original[0].entry.planned = PlannedElement::Jump(JumpElement::Solo(ListedJump {
        kind: None,
        rotations: Some(1),
    }));
    req.original[0].entry.intended_recognition = None;
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::IndeterminateOriginal(_))
    ));
}

#[test]
fn repetitions_include_compound_components_and_spin_caps_include_originals() {
    let mut req = request();
    let mut combo = jump(JumpKind::ToeLoop);
    combo.element = PlannedElement::Jump(JumpElement::Combination(vec![
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(1),
        }),
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::Loop),
            rotations: Some(1),
        }),
    ]));
    req.variants = vec![combo, jump(JumpKind::ToeLoop), sit()];
    let mut toe = entry("toe", jump(JumpKind::ToeLoop));
    toe.locked = false;
    req.original.insert(1, toe);
    req.insertion.boundaries = vec![0, 1, 2, 3];
    req.min_additions = 1;
    req.max_additions = 1;
    let r = result(&req);
    assert!(r.candidates.iter().all(|c| c.added_base_hundredths == 90));
    req.min_additions = 2;
    req.max_additions = 2;
    assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
    req.global_exclusions = vec![PersonalExclusion::JumpKind(JumpKind::Loop)];
    req.min_additions = 1;
    req.max_additions = 1;
    assert!(result(&req)
        .candidates
        .iter()
        .all(|c| c.added_base_hundredths == 40));
}

#[test]
fn order_dependent_first_attempt_feature_is_validated_across_original_and_insert() {
    let mut req = request();
    req.original[1] = entry(
        "sit",
        spin(
            RegularSpinKind::Sit,
            SpinEntry::NonFlying,
            false,
            Level::Basic,
            vec![feature(
                SpinFeature::IncreaseOfSpeed,
                FeatureAttemptOutcome::Unsuccessful,
            )],
        ),
    );
    req.variants = vec![spin(
        RegularSpinKind::Camel,
        SpinEntry::NonFlying,
        false,
        Level::One,
        vec![feature(
            SpinFeature::IncreaseOfSpeed,
            FeatureAttemptOutcome::Successful,
        )],
    )];
    req.min_additions = 1;
    req.max_additions = 1;
    req.insertion.boundaries = vec![0, 1];
    assert_eq!(result(&req).status, OptimizerStatus::OptimalProven);
    req.insertion.boundaries = vec![2];
    assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
}

#[test]
fn budgets_caps_unsupported_domains_and_visible_solution_identity() {
    let mut req = request();
    req.time_limit = Some(Duration::ZERO);
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::Incomplete);
    assert!(!r.enumeration_complete);
    req.time_limit = None;
    req.max_solutions = 1;
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(!r.enumeration_complete);
    req.max_solutions = 1000;
    req.max_proposals = 1;
    assert!(!result(&req).enumeration_complete);
    req.variants[0].recognition_assumed = false;
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::Optimization(
            ProgramOptimizationError::UnsupportedVariant(0)
        ))
    ));
    let mut req = request();
    req.variants = vec![jump(JumpKind::Axel)];
    req.min_additions = 1;
    req.max_additions = 1;
    let r = result(&req);
    // Original Axel provenance distinguishes all three insertion boundaries.
    assert_eq!(r.candidates.len(), 3);
    assert!(r.enumeration_complete);
}

#[test]
fn gold_and_masters_required_roles_ties_and_impossible_restrictions() {
    for category in [AdultCategory::Gold, AdultCategory::Masters] {
        let mut req = request();
        req.profile = adult_profile(category).id;
        req.policy = adult_program_policy(category);
        req.original = vec![entry("axel", jump(JumpKind::Axel))];
        req.variants = vec![
            spin(
                RegularSpinKind::Combination,
                SpinEntry::NonFlying,
                true,
                Level::Basic,
                vec![],
            ),
            spin(
                RegularSpinKind::Camel,
                SpinEntry::Flying,
                false,
                Level::Basic,
                vec![],
            ),
            chsp(),
        ];
        req.insertion.boundaries = vec![0, 1];
        req.min_additions = 3;
        req.max_additions = 3;
        let r = result(&req);
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(r.enumeration_complete);
        assert_eq!(r.candidates.len(), 24);
        req.exclusions.push(PersonalExclusion::FlyingEntry);
        assert_eq!(result(&req).status, OptimizerStatus::InfeasibleProven);
    }
}

#[test]
fn malformed_anchor_contracts_are_rejected_before_search() {
    let mut req = request();
    req.original[1].id = "axel".into();
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::InvalidRequest(_))
    ));
    let mut req = request();
    req.insertion.boundaries = vec![3];
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::InvalidRequest(_))
    ));
    let mut req = request();
    req.insertion.absolute_positions = vec![("sit".into(), 0), ("axel".into(), 1)];
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::InvalidRequest(_))
    ));
    let mut req = request();
    req.global_exclusions = vec![PersonalExclusion::Variant(0)];
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::InvalidRequest(_))
    ));
}

#[test]
fn different_original_recognition_evidence_remains_distinct_in_visible_ties() {
    let mut req = request();
    req.variants = vec![jump(JumpKind::Axel)];
    req.min_additions = 1;
    req.max_additions = 1;
    // The original has a caller-supplied target reference, unlike the inserted Axel.
    req.original[0]
        .entry
        .intended_recognition
        .as_mut()
        .unwrap()
        .planned_element = Some(0);
    let r = result(&req);
    assert_eq!(r.candidates.len(), 3);
    assert!(r.enumeration_complete);
}

#[test]
fn generated_domain_audit_is_preserved_and_bound_to_supplied_variants() {
    let mut req = request();
    let generator = CandidateGenerationRequest {
        season: req.season,
        profile: req.profile.clone(),
        scoring_table: req.scoring_table,
        discipline: ProfileDiscipline::Single,
        segment: ProgramSegment::FreeSkating,
        jump_kinds: vec![JumpKind::ToeLoop, JumpKind::Salchow],
        rotations: vec![1],
        solo_jumps: true,
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
        max_candidates: 1,
        max_expansions: 100,
    };
    let domain = generate_program_candidates(&ProfileRegistry::adult(), &generator).unwrap();
    assert_eq!(
        domain.audit.completeness,
        CandidateDomainCompleteness::IncompleteCandidateCap
    );
    req.variants = domain.variants.clone();
    req.generated_domain = Some(domain);
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete); // Only over supplied finite variants.
    assert_eq!(
        r.domain_completeness,
        Some(CandidateDomainCompleteness::IncompleteCandidateCap)
    );
    assert!(!r.assumptions.is_empty());
    assert!(!r.unmodeled_variants.is_empty());
    assert!(!r.supported_rules.is_empty());
    assert_eq!(
        r.request.generated_domain.unwrap().audit.completeness,
        CandidateDomainCompleteness::IncompleteCandidateCap
    );
    req.variants = vec![jump(JumpKind::Salchow)];
    assert!(matches!(
        complete_program(&ProfileRegistry::adult(), &req),
        Err(ProgramCompletionError::InvalidRequest(_))
    ));
}

fn fixed_call_case(call: PanelCall, target: CallTarget, original_bv: u32) {
    let mut req = request();
    let recognition = req.original[0].entry.intended_recognition.as_mut().unwrap();
    recognition.details = Some(
        RecognitionDetails::new(
            vec![ComponentCode::try_from("1A").unwrap()],
            None,
            Some(vec![RecordedCall { target, call }]),
        )
        .unwrap(),
    );
    let r = result(&req);
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    assert_eq!(r.candidates.len(), 24);
    assert_eq!(
        r.original_assessment.intended_base.exact_hundredths,
        Some(i64::from(original_bv + 160))
    );
    for c in &r.candidates {
        assert_eq!(c.existing_base_hundredths, original_bv + 160);
        assert_eq!(c.added_base_hundredths, 80);
        assert_eq!(c.total_base_hundredths, original_bv + 240);
        assert_eq!(
            c.assessment.intended_base.exact_hundredths,
            Some(i64::from(original_bv + 240))
        );
        for (index, p) in c.provenance.iter().enumerate() {
            if matches!(
                p,
                CompletionProvenance::Original {
                    original_index: 0,
                    ..
                }
            ) {
                assert_eq!(c.per_element[index].hundredths, original_bv as u16);
                assert_eq!(c.assessment.request.entries[index], req.original[0].entry);
                assert_eq!(
                    c.assessment.elements[index].intended_base,
                    Ok(BaseValueOutcome::Value(c.per_element[index].clone()))
                );
                assert_eq!(
                    r.original_assessment.elements[0].intended_base,
                    Ok(BaseValueOutcome::Value(c.per_element[index].clone()))
                );
            }
        }
    }
}

#[test]
fn fixed_underrotated_axel_preserves_recognition_and_reduced_objective() {
    // Literal SOV fixture: 1A< = .88, SSp1 = 1.60, additions = .80.
    fixed_call_case(PanelCall::UnderRotated, CallTarget::Component(0), 88);
}

#[test]
fn fixed_repetition_call_preserves_recognition_and_reduced_objective() {
    // Supplied solo +REP uses the existing 80% BV path, not an inferred call.
    fixed_call_case(PanelCall::Repetition, CallTarget::Element, 88);
}

#[test]
fn assumed_completion_keeps_original_plan_and_uses_no_panel_outcomes() {
    let mut q = request();
    for original in &mut q.original {
        original.entry.intended_recognition = None;
    }
    let originals = ["1A", "SSp2"].map(AssumedRecognition::new);
    let additions = ["1T", "1S"].map(AssumedRecognition::new);
    let result = complete_program_with_assumed_recognition(
        &ProfileRegistry::adult(),
        &q,
        &originals,
        &additions,
    )
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(result.request.original[1].entry, q.original[1].entry);
    assert!(matches!(
        &result.request.original[1].entry.evidence,
        ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence { feature_attempts: Some(attempts), .. },
            local_feature_qualification_assumed: true,
        } if !attempts.is_empty()
    ));
    let resolved_spin = &result.original_assessment.request.entries[1].evidence;
    assert_ne!(resolved_spin, &q.original[1].entry.evidence);
    assert!(matches!(
        resolved_spin,
        ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                planned_level: Some(Level::Two),
                ..
            },
            ..
        }
    ));
    let candidate = &result.candidates[0];
    for (index, provenance) in candidate.provenance.iter().enumerate() {
        let entry = &candidate.assessment.request.entries[index];
        assert!(entry.intended_recognition.is_none() && entry.scenario.is_none());
        if let CompletionProvenance::Original {
            id,
            original_index,
            locked,
        } = provenance
        {
            let original = &result.request.original[*original_index];
            assert_eq!(id, &original.id);
            assert!(*locked && original.locked);
            assert_eq!(entry.planned, original.entry.planned);
            assert_eq!(entry.evidence, candidate.evidence[index]);
            assert_eq!(
                entry,
                &result.original_assessment.request.entries[*original_index]
            );
            assert_eq!(
                candidate.per_element[index].notation,
                originals[*original_index].code
            );
        }
    }
    assert_eq!(
        candidate.assessment.intended_base.exact_hundredths,
        Some(i64::from(candidate.total_base_hundredths))
    );
}

#[test]
fn gold_code_domain_completion_matches_manual_bv_and_order_oracle() {
    let registry = ProfileRegistry::adult();
    let category = AdultCategory::Gold;
    let generation = generation_defaults::request(adult_profile(category));
    let original_codes = ["1A", "CCoSp3"].map(AssumedRecognition::new);
    let addition_codes =
        ["CCoSp2", "FSSp2", "StSq2", "ChSp1", "FCSp3"].map(AssumedRecognition::new);
    let originals = generate_program_candidates_with_assumed_recognition(
        &registry,
        &generation,
        &original_codes,
    )
    .unwrap();
    let additions = generate_program_candidates_with_assumed_recognition(
        &registry,
        &generation,
        &addition_codes,
    )
    .unwrap();
    let mut q = request();
    q.profile = generation.profile;
    q.policy = adult_program_policy(category);
    q.original = ["axel", "combination"]
        .into_iter()
        .zip(originals.domain.variants)
        .map(|(id, variant)| {
            let mut original = entry(id, variant);
            original.entry.intended_recognition = None;
            original
        })
        .collect();
    q.variants = additions.domain.variants.clone();
    q.generated_domain = Some(additions.domain);
    q.exclusions = vec![PersonalExclusion::Variant(4)];
    q.min_additions = 3;
    q.max_additions = 3;

    let result = complete_program_with_assumed_recognition(
        &registry,
        &q,
        &original_codes,
        &additions.recognition,
    )
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    let mut actual = BTreeSet::new();
    for candidate in &result.candidates {
        let mut word = Vec::new();
        for (index, provenance) in candidate.provenance.iter().enumerate() {
            match provenance {
                CompletionProvenance::Original {
                    original_index, id, ..
                } => {
                    assert_eq!(id, &q.original[*original_index].id);
                    assert_eq!(
                        candidate.assessment.request.entries[index],
                        q.original[*original_index].entry
                    );
                }
                CompletionProvenance::Inserted { boundary, .. } => {
                    assert_eq!(
                        *boundary,
                        word.iter()
                            .filter(|&&code| matches!(code, "1A" | "CCoSp3"))
                            .count()
                    );
                }
            }
            word.push(candidate.per_element[index].notation.as_str());
        }
        let manual = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                profile: q.profile.clone(),
                scoring_table: q.scoring_table,
                submission: SubmissionState::Complete,
                elements: word.iter().copied().map(AssumedRecognition::new).collect(),
            },
        )
        .unwrap();
        assert_eq!(candidate.total_base_hundredths, 1370);
        assert_eq!(manual.total.exact_hundredths, Some(1370));
        assert!(actual.insert(word));
    }
    let codes = ["1A", "CCoSp3", "FSSp2", "StSq2", "ChSp1"];
    let expected = (0..5usize.pow(5))
        .filter_map(|encoded| {
            let indices: Vec<_> = (0..5)
                .map(|place| encoded / 5usize.pow(place) % 5)
                .collect();
            let axel = indices.iter().position(|&i| i == 0)?;
            let spin = indices.iter().position(|&i| i == 1)?;
            (axel < spin && (2..5).all(|i| indices.contains(&i)))
                .then(|| indices.iter().map(|&i| codes[i]).collect::<Vec<_>>())
        })
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn assumed_senior_completion_keeps_distinct_equal_bv_choices() {
    let category = SeniorCategory::Women;
    let profile = senior_profile(category);
    let registry = ProfileRegistry::senior();
    let generation = generation_defaults::request(profile.clone());
    let original_codes =
        ["2A", "3T", "CCoSp4", "StSq4", "ChSq1", "ChSp1"].map(AssumedRecognition::new);
    let addition_codes = ["FCSp4", "FLSp4"].map(AssumedRecognition::new);
    let originals = generate_program_candidates_with_assumed_recognition(
        &registry,
        &generation,
        &original_codes,
    )
    .unwrap();
    let additions = generate_program_candidates_with_assumed_recognition(
        &registry,
        &generation,
        &addition_codes,
    )
    .unwrap();
    let mut q = request();
    q.profile = profile.id;
    q.scoring_table = profile.scoring_table;
    q.policy = senior_program_policy(category);
    q.original = original_codes
        .iter()
        .zip(originals.domain.variants)
        .map(|(code, variant)| CompletionEntry {
            id: code.code.clone(),
            locked: true,
            entry: ManualProgramEntry {
                planned: variant.element,
                evidence: variant.evidence,
                intended_recognition: None,
                scenario: None,
            },
        })
        .collect();
    q.variants = additions.domain.variants;
    q.insertion.boundaries = vec![q.original.len()];
    q.min_additions = 1;
    q.max_additions = 1;
    let result = complete_program_with_assumed_recognition(
        &registry,
        &q,
        &original_codes,
        &additions.recognition,
    )
    .unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), 2);
    let mut found = BTreeSet::new();
    for candidate in &result.candidates {
        for (index, original) in q.original.iter().enumerate() {
            assert!(matches!(
                &candidate.provenance[index],
                CompletionProvenance::Original { original_index, id, locked, .. }
                    if *original_index == index && id == &original.id && *locked
            ));
            assert_eq!(candidate.assessment.request.entries[index], original.entry);
        }
        // Communication 2786: 2A, 3T, CCoSp4, StSq4, ChSq1, ChSp1 and either flying spin.
        assert_eq!(candidate.total_base_hundredths, 2660);
        let manual = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                profile: q.profile.clone(),
                scoring_table: q.scoring_table,
                submission: SubmissionState::Complete,
                elements: candidate
                    .per_element
                    .iter()
                    .map(|value| AssumedRecognition::new(&value.notation))
                    .collect(),
            },
        )
        .unwrap();
        assert_eq!(
            manual.total.exact_hundredths,
            Some(i64::from(candidate.total_base_hundredths))
        );
        found.insert(candidate.per_element.last().unwrap().notation.as_str());
    }
    assert_eq!(found, BTreeSet::from(["FCSp4", "FLSp4"]));
}

// Build locked originals from exactly the supplied recognition selections.
fn senior_locked_request(
    category: SeniorCategory,
    codes: &[AssumedRecognition],
) -> ProgramCompletionRequest {
    let profile = senior_profile(category);
    let domain = generate_program_candidates_with_assumed_recognition(
        &ProfileRegistry::senior(),
        &generation_defaults::request(profile.clone()),
        codes,
    )
    .unwrap();
    let mut q = request();
    q.profile = profile.id;
    q.scoring_table = profile.scoring_table;
    q.policy = senior_program_policy(category);
    q.original = codes
        .iter()
        .zip(domain.domain.variants)
        .map(|(code, variant)| CompletionEntry {
            id: code.code.clone(),
            locked: true,
            entry: ManualProgramEntry {
                planned: variant.element,
                evidence: variant.evidence,
                intended_recognition: None,
                scenario: None,
            },
        })
        .collect();
    q
}

#[test]
fn assumed_senior_completion_credits_the_second_axel_in_each_allowed_gap() {
    let category = SeniorCategory::Women;
    let registry = ProfileRegistry::senior();
    let codes = ["3A", "CCoSp4", "FCSp4", "StSq4", "ChSq1", "ChSp1"].map(AssumedRecognition::new);
    let mut q = senior_locked_request(category, &codes);
    q.variants = vec![ProgramVariant {
        element: q.original[0].entry.planned.clone(),
        evidence: q.original[0].entry.evidence.clone(),
        recognition_assumed: true,
    }];
    q.insertion.boundaries = (0..=codes.len()).collect();
    q.min_additions = 1;
    q.max_additions = 1;
    let result =
        complete_program_with_assumed_recognition(&registry, &q, &codes, &codes[..1]).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert_eq!(
        result.effective_objective,
        ProgramSearchObjective::CreditedBaseValueHundredths
    );
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), codes.len() + 1);
    let mut gaps = BTreeSet::new();
    for candidate in result.candidates {
        let mut selected = vec![];
        let mut original_indices = vec![];
        let mut axels = vec![];
        for (index, provenance) in candidate.provenance.iter().enumerate() {
            let code = match provenance {
                CompletionProvenance::Inserted {
                    variant_index: 0,
                    boundary,
                    ordinal: 0,
                } => {
                    gaps.insert(*boundary);
                    assert_eq!(
                        candidate.added_base_hundredths,
                        if *boundary == 0 { 800 } else { 640 }
                    );
                    &codes[0]
                }
                CompletionProvenance::Original {
                    id,
                    original_index,
                    locked: true,
                } => {
                    assert_eq!(id, &q.original[*original_index].id);
                    assert_eq!(
                        candidate.assessment.request.entries[index],
                        q.original[*original_index].entry
                    );
                    original_indices.push(*original_index);
                    &codes[*original_index]
                }
                other => panic!("unexpected placement: {other:?}"),
            };
            selected.push(code.clone());
            if code.code == "3A" {
                assert_eq!(candidate.listed_per_element[index].notation, "3A");
                assert_eq!(candidate.listed_per_element[index].hundredths, 800);
                axels.push((
                    candidate.per_element[index].notation.as_str(),
                    candidate.per_element[index].hundredths,
                ));
            }
        }
        assert_eq!(original_indices, (0..codes.len()).collect::<Vec<_>>());
        assert_eq!(axels, [("3A", 800), ("3A+REP", 640)]);
        assert_eq!(candidate.total_base_hundredths, 3350);
        let manual = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                profile: q.profile.clone(),
                scoring_table: q.scoring_table,
                submission: SubmissionState::Complete,
                elements: selected,
            },
        )
        .unwrap();
        assert_eq!(manual.total.exact_hundredths, Some(3350));
        assert_eq!(
            candidate.assessment.intended_base.exact_hundredths,
            Some(3350)
        );
    }
    assert_eq!(gaps, q.insertion.boundaries.iter().copied().collect());

    q.max_proposals = 1;
    let stopped =
        complete_program_with_assumed_recognition(&registry, &q, &codes, &codes[..1]).unwrap();
    assert_eq!(stopped.status, OptimizerStatus::FeasibleUnproven);
    assert!(!stopped.enumeration_complete);
    assert_eq!(stopped.candidates[0].total_base_hundredths, 3350);

    q.max_proposals = 100;
    q.max_solutions = 1;
    let capped =
        complete_program_with_assumed_recognition(&registry, &q, &codes, &codes[..1]).unwrap();
    assert_eq!(capped.status, OptimizerStatus::OptimalProven);
    assert_eq!(
        capped.termination_reason,
        JumpTerminationReason::SolutionLimit
    );
    assert!(!capped.enumeration_complete);
    assert_eq!(capped.candidates.len(), 1);
}

#[test]
fn assumed_senior_completion_matches_eleven_slot_credited_oracle() {
    let registry = ProfileRegistry::senior();
    let originals = [
        "3A", "4T", "4S", "2T", "CCoSp4", "FCSp4", "StSq4", "ChSq1", "ChSp1",
    ]
    .map(AssumedRecognition::new);
    let mut q = senior_locked_request(SeniorCategory::Men, &originals);
    q.variants = q.original[..3]
        .iter()
        .map(|e| ProgramVariant {
            element: e.entry.planned.clone(),
            evidence: e.entry.evidence.clone(),
            recognition_assumed: true,
        })
        .collect();
    let PlannedElement::Jump(JumpElement::Solo(axel)) = &q.original[0].entry.planned else {
        unreachable!()
    };
    let PlannedElement::Jump(JumpElement::Solo(toe)) = &q.original[3].entry.planned else {
        unreachable!()
    };
    q.variants.push(ProgramVariant {
        element: PlannedElement::Jump(JumpElement::Combination(vec![
            JumpPart::Jump(axel.clone()),
            JumpPart::Jump(toe.clone()),
        ])),
        evidence: q.original[0].entry.evidence.clone(),
        recognition_assumed: true,
    });
    let additions = ["3A", "4T", "4S", "3A+2T"].map(AssumedRecognition::new);
    q.insertion.boundaries = vec![0, 2, originals.len()];
    q.min_additions = 2;
    q.max_additions = 2;
    // Independent literal oracle for the unordered addition pairs; all allowed
    // placements of each legal pair have that total. Anchors sum to 4760.
    // Pair 4T/4S repeats two quad types; 4T/3A+2T gives four Toe Loop
    // attempts; 3A/3A+2T and all equal pairs give three identical attempts.
    let credited = |a, b| match (a, b) {
        (0, 1) | (1, 0) => Some(640 + 760),
        (0, 2) | (2, 0) => Some(640 + 776),
        (2, 3) | (3, 2) => Some(776 + 930),
        _ => None,
    };
    for (exclude_combo, best_delta) in [(false, 1706), (true, 1416)] {
        q.exclusions = if exclude_combo {
            vec![PersonalExclusion::Variant(3)]
        } else {
            vec![]
        };
        let mut expected = BTreeSet::new();
        for &a_gap in &q.insertion.boundaries {
            for &b_gap in &q.insertion.boundaries {
                for a in 0..additions.len() {
                    for b in 0..additions.len() {
                        if a_gap <= b_gap && !(exclude_combo && (a == 3 || b == 3)) {
                            if let Some(value) = credited(a, b) {
                                assert!(value <= best_delta);
                                if value == best_delta {
                                    expected.insert(vec![(a_gap, a), (b_gap, b)]);
                                }
                            }
                        }
                    }
                }
            }
        }
        let result =
            complete_program_with_assumed_recognition(&registry, &q, &originals, &additions)
                .unwrap();
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(result.enumeration_complete);
        assert_eq!(result.candidates.len(), expected.len());
        let actual = result
            .candidates
            .iter()
            .map(|candidate| {
                assert_eq!(candidate.total_base_hundredths, 4760 + best_delta);
                assert_eq!(candidate.program.elements.len(), 11);
                candidate
                    .provenance
                    .iter()
                    .filter_map(|p| match p {
                        CompletionProvenance::Inserted {
                            boundary,
                            variant_index,
                            ..
                        } => Some((*boundary, *variant_index)),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
    }
}
