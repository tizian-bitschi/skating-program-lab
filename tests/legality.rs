use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
fn jump(kind: JumpKind, rotations: u8) -> (PlannedElement, ElementPlanEvidence) {
    (
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(kind),
            rotations: Some(rotations),
        })),
        ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
    )
}
fn spin(
    position: SpinPosition,
    level: Level,
    feature: Option<SpinFeatureAttempt>,
    revs: u16,
) -> (PlannedElement, ElementPlanEvidence) {
    let kind = match position {
        SpinPosition::Camel => RegularSpinKind::Camel,
        SpinPosition::Sit => RegularSpinKind::Sit,
        SpinPosition::Upright => RegularSpinKind::Upright,
    };
    (
        PlannedElement::RegularSpin(RegularSpin {
            entry: Some(SpinEntry::NonFlying),
            steps: Some(vec![SpinStep::Position {
                position: Some(position),
                revolutions: Some(revs),
                feature_attempts: Some(vec![]),
            }]),
        }),
        ElementPlanEvidence::Spin {
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry: SpinEntry::NonFlying,
                    change_foot: false,
                }),
                planned_level: Some(level),
                feature_attempts: Some(feature.into_iter().collect()),
                ..Default::default()
            },
            local_feature_qualification_assumed: true,
        },
    )
}
fn attempt(outcome: FeatureAttemptOutcome) -> SpinFeatureAttempt {
    SpinFeatureAttempt {
        feature: SpinFeature::EightRevolutions,
        position_step: 0,
        outcome,
    }
}
fn report(
    category: AdultCategory,
    entries: Vec<(PlannedElement, ElementPlanEvidence)>,
    submission: SubmissionState,
) -> ProgramLegalityReport {
    let (elements, evidence): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
    assess_program_legality(
        &ProfileRegistry::adult(),
        &ProfileProgram {
            profile: adult_profile(category).id,
            scoring_table: TABLE,
            elements,
        },
        &evidence,
        &adult_program_policy(category),
        submission,
    )
    .unwrap()
}
fn finding(r: &ProgramLegalityReport, id: &str, index: Option<usize>, kind: FindingKind) -> bool {
    r.validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == id && f.element_index == index && f.kind == kind)
}

#[test]
fn gold_prohibition_survives_missing_content_and_draft_omissions_are_indexed() {
    // July 16 Adult p. 9: 2F prohibited, combination/flying/ChSp required.
    let entries = vec![jump(JumpKind::Flip, 2)];
    let draft = report(AdultCategory::Gold, entries.clone(), SubmissionState::Draft);
    let final_report = report(AdultCategory::Gold, entries, SubmissionState::Complete);
    assert!(finding(
        &draft,
        "isu-adult-2026-27-gold-jump-eligibility",
        Some(0),
        FindingKind::Violation
    ));
    assert!(!draft.repairable_absences.is_empty());
    assert!(draft
        .repairable_absences
        .iter()
        .all(|i| draft.validation.validation.findings[*i]
            .element_index
            .is_none()));
    assert!(final_report.repairable_absences.is_empty());
    assert!(finding(
        &final_report,
        "program-required-choreographic-spin",
        None,
        FindingKind::Violation
    ));
    assert!(draft
        .rules
        .iter()
        .any(|r| r.rule_id == "program-required-choreographic-spin"
            && r.state == RuleCoverageState::Conditional));
    assert!(final_report
        .rules
        .iter()
        .any(|r| r.rule_id == "program-required-choreographic-spin"
            && r.state == RuleCoverageState::Evaluated));
}

#[test]
fn full_bronze_plan_has_no_covered_violation_and_retains_open_rules() {
    let r = report(
        AdultCategory::Bronze,
        vec![
            jump(JumpKind::Lutz, 1),
            spin(SpinPosition::Camel, Level::Basic, None, 4),
            spin(SpinPosition::Sit, Level::Basic, None, 4),
            (
                PlannedElement::ChoreographicSequence { movements: None },
                ElementPlanEvidence::ChoreographicSequence {
                    movements: Some(vec![SkatingMovement::Spiral, SkatingMovement::SpreadEagle]),
                    clearly_visible: Some(true),
                },
            ),
        ],
        SubmissionState::Complete,
    );
    assert!(!r
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation));
    assert!(r
        .rules
        .iter()
        .any(|r| r.state == RuleCoverageState::Evaluated));
    assert!(r
        .rules
        .iter()
        .any(|r| r.rule_id == "program-duration" && r.state == RuleCoverageState::MissingEvidence));
    assert!(r
        .rules
        .iter()
        .any(|r| r.rule_id == "program-variation-categories"
            && r.state == RuleCoverageState::Unsupported));
    assert!(r
        .rules
        .iter()
        .any(|r| r.rule_id == "program-step-recognition"
            && r.state == RuleCoverageState::NotApplicable));
    assert!(r.rules.iter().all(|r| !r.source.document.is_empty()
        && !r.source.version.is_empty()
        && !r.source.location.is_empty()
        && !r.reason.is_empty()));
}

#[test]
fn unknown_first_success_blocks_later_credit_and_insufficient_hold_does_not_consume_it() {
    // Communication 2788 p. 2 feature 10: first successful spin, not first assertion.
    let first = spin(
        SpinPosition::Camel,
        Level::Basic,
        Some(attempt(FeatureAttemptOutcome::Unknown)),
        8,
    );
    let later = spin(
        SpinPosition::Camel,
        Level::One,
        Some(attempt(FeatureAttemptOutcome::Successful)),
        8,
    );
    let r = report(
        AdultCategory::Silver,
        vec![first, later.clone()],
        SubmissionState::Complete,
    );
    assert!(finding(
        &r,
        "program-feature-once-per-program",
        Some(1),
        FindingKind::Unknown
    ));
    assert!(finding(
        &r,
        "program-spin-level-credit",
        Some(1),
        FindingKind::Unknown
    ));
    let invalid_hold = spin(
        SpinPosition::Camel,
        Level::Basic,
        Some(attempt(FeatureAttemptOutcome::Successful)),
        7,
    );
    let r = report(
        AdultCategory::Silver,
        vec![invalid_hold, later],
        SubmissionState::Complete,
    );
    assert!(!finding(
        &r,
        "program-spin-level-credit",
        Some(1),
        FindingKind::Unknown
    ));
}

#[test]
fn plain_upright_does_not_supply_a_difficult_variation_prerequisite() {
    let feature = SpinFeatureAttempt {
        feature: SpinFeature::IncreaseOfSpeed,
        position_step: 0,
        outcome: FeatureAttemptOutcome::Successful,
    };
    let r = report(
        AdultCategory::Bronze,
        vec![spin(SpinPosition::Upright, Level::One, Some(feature), 8)],
        SubmissionState::Complete,
    );
    assert!(finding(
        &r,
        "program-feature-position",
        Some(0),
        FindingKind::Unsupported
    ));
    assert!(finding(
        &r,
        "program-spin-level-credit",
        Some(0),
        FindingKind::Unknown
    ));
}

#[test]
fn sequence_without_direct_step_assumption_and_repeated_components_stay_visible() {
    let t = || {
        JumpPart::Jump(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(1),
        })
    };
    let a = JumpPart::Jump(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(1),
    });
    let r = report(
        AdultCategory::Silver,
        vec![
            jump(JumpKind::ToeLoop, 1),
            jump(JumpKind::ToeLoop, 1),
            (
                PlannedElement::Jump(JumpElement::Sequence(vec![t(), a])),
                ElementPlanEvidence::Jump {
                    direct_transitions_assumed: false,
                },
            ),
        ],
        SubmissionState::Draft,
    );
    assert!(r
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "isu-adult-2026-27-jump-repetition"
            && f.kind == FindingKind::Violation));
    assert!(finding(
        &r,
        "isu-adult-2026-27-combination-structure",
        Some(2),
        FindingKind::Unknown
    ));
    assert!(r.repairable_absences.iter().all(
        |i| r.validation.validation.findings[*i].rule_id != "isu-adult-2026-27-jump-repetition"
    ));
}

#[test]
fn custom_profile_uses_explicit_override_and_never_inherits_adult_audit() {
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id = ProfileId {
        name: "synthetic-national".into(),
        version: "test-1".into(),
    };
    profile.provenance = ProfileProvenance::Synthetic;
    profile.event_scope = "test-event".into();
    let mut replacement = profile
        .bindings
        .iter()
        .find(|b| b.rule == ProfileRule::Count)
        .unwrap()
        .clone();
    let replaces = replacement.id;
    replacement.id = "synthetic-jump-count";
    replacement.source = SourceReference {
        document: "Synthetic national fixture".into(),
        version: "test-1".into(),
        location: "one jump only".into(),
    };
    profile.sources.push(replacement.source.clone());
    profile.jumps.max_jumps = 1;
    profile.overrides.push(RuleOverride {
        replaces,
        replacement,
        event_scope: profile.event_scope.clone(),
        justification: "test explicit count replacement".into(),
    });
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.positions = None;
    policy.spins.per_foot = None;
    let mut registry = ProfileRegistry::adult();
    registry.register(profile.clone()).unwrap();
    let (elements, evidence): (Vec<_>, Vec<_>) =
        vec![jump(JumpKind::ToeLoop, 1), jump(JumpKind::Loop, 1)]
            .into_iter()
            .unzip();
    let r = assess_program_legality(
        &registry,
        &ProfileProgram {
            profile: profile.id,
            scoring_table: TABLE,
            elements,
        },
        &evidence,
        &policy,
        SubmissionState::Complete,
    )
    .unwrap();
    assert!(finding(
        &r,
        "synthetic-jump-count",
        None,
        FindingKind::Violation
    ));
    assert!(r.rules.iter().any(|r| r.rule_id == "synthetic-jump-count"
        && r.event_override.is_some()
        && r.source.document == "Synthetic national fixture"));
    assert!(!r.rules.iter().any(|r| matches!(
        r.rule_id,
        "program-duration" | "program-waltz-recognition" | "program-variation-categories"
    )));
}

#[test]
fn all_four_profiles_keep_distinct_obligations() {
    for (category, required) in [
        (AdultCategory::Bronze, false),
        (AdultCategory::Silver, false),
        (AdultCategory::Gold, true),
        (AdultCategory::Masters, true),
    ] {
        let r = report(category, vec![], SubmissionState::Complete);
        assert_eq!(
            finding(
                &r,
                "program-required-choreographic-spin",
                None,
                FindingKind::Violation
            ),
            required
        );
        assert_eq!(
            finding(
                &r,
                "isu-adult-2026-27-masters-axel-required",
                None,
                FindingKind::Violation
            ),
            category == AdultCategory::Masters
        );
    }
}

#[test]
fn saturated_draft_does_not_label_missing_axel_as_insertable() {
    let r = report(
        AdultCategory::Masters,
        vec![
            jump(JumpKind::ToeLoop, 1),
            jump(JumpKind::Loop, 1),
            jump(JumpKind::Salchow, 1),
            jump(JumpKind::Flip, 1),
            jump(JumpKind::Lutz, 1),
        ],
        SubmissionState::Draft,
    );
    assert!(finding(
        &r,
        "isu-adult-2026-27-masters-axel-required",
        None,
        FindingKind::Violation
    ));
    assert!(!r
        .repairable_absences
        .iter()
        .any(|i| r.validation.validation.findings[*i].rule_id
            == "isu-adult-2026-27-masters-axel-required"));
}

#[test]
fn unqualified_spin_is_missing_evidence_in_both_stages() {
    let mut entry = spin(
        SpinPosition::Camel,
        Level::One,
        Some(SpinFeatureAttempt {
            feature: SpinFeature::IncreaseOfSpeed,
            position_step: 0,
            outcome: FeatureAttemptOutcome::Successful,
        }),
        8,
    );
    if let ElementPlanEvidence::Spin {
        local_feature_qualification_assumed,
        ..
    } = &mut entry.1
    {
        *local_feature_qualification_assumed = false;
    }
    for submission in [SubmissionState::Draft, SubmissionState::Complete] {
        let r = report(AdultCategory::Bronze, vec![entry.clone()], submission);
        assert!(r
            .rules
            .iter()
            .any(|r| r.rule_id == "program-feature-qualification"
                && r.element_index == Some(0)
                && r.state == RuleCoverageState::MissingEvidence));
        assert!(!r
            .repairable_absences
            .iter()
            .any(|i| r.validation.validation.findings[*i].element_index == Some(0)));
    }
}

#[test]
fn missing_local_qualification_preserves_earlier_possible_eight_revolution_success() {
    let mut first = spin(
        SpinPosition::Camel,
        Level::Basic,
        Some(attempt(FeatureAttemptOutcome::Successful)),
        8,
    );
    if let ElementPlanEvidence::Spin {
        local_feature_qualification_assumed,
        ..
    } = &mut first.1
    {
        *local_feature_qualification_assumed = false;
    }
    let later = spin(
        SpinPosition::Camel,
        Level::One,
        Some(attempt(FeatureAttemptOutcome::Successful)),
        8,
    );
    let r = report(
        AdultCategory::Silver,
        vec![first, later],
        SubmissionState::Complete,
    );
    assert!(finding(
        &r,
        "program-feature-once-per-program",
        Some(1),
        FindingKind::Unknown
    ));
    assert!(finding(
        &r,
        "program-spin-level-credit",
        Some(1),
        FindingKind::Unknown
    ));
}

#[test]
fn first_success_is_spin_scoped_and_can_use_the_second_foot() {
    // 2788 p. 2: feature 10 may use either qualifying foot in its first spin.
    let mut entry = spin(SpinPosition::Camel, Level::Three, None, 8);
    if let PlannedElement::RegularSpin(plan) = &mut entry.0 {
        let steps = plan.steps.as_mut().unwrap();
        steps.push(SpinStep::ChangeFoot);
        steps.push(steps[0].clone());
        for position in [SpinPosition::Sit, SpinPosition::Upright] {
            steps.push(SpinStep::Position {
                position: Some(position),
                revolutions: Some(2),
                feature_attempts: Some(vec![]),
            });
        }
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entry.1 {
        spin.abbreviation.as_mut().unwrap().change_foot = true;
        spin.abbreviation.as_mut().unwrap().kind = RegularSpinKind::Combination;
        spin.feature_attempts = Some(vec![
            SpinFeatureAttempt {
                feature: SpinFeature::ChangeOfEdge,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            },
            SpinFeatureAttempt {
                feature: SpinFeature::IncreaseOfSpeed,
                position_step: 0,
                outcome: FeatureAttemptOutcome::Successful,
            },
            attempt(FeatureAttemptOutcome::Successful),
            SpinFeatureAttempt {
                position_step: 2,
                ..attempt(FeatureAttemptOutcome::Successful)
            },
        ]);
    }
    let mut later = spin(
        SpinPosition::Camel,
        Level::One,
        Some(attempt(FeatureAttemptOutcome::Successful)),
        8,
    );
    if let PlannedElement::RegularSpin(plan) = &mut later.0 {
        plan.entry = Some(SpinEntry::Flying);
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut later.1 {
        spin.abbreviation.as_mut().unwrap().entry = SpinEntry::Flying;
    }
    let r = report(
        AdultCategory::Gold,
        vec![entry.clone(), later],
        SubmissionState::Complete,
    );
    assert!(!r
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-eligibility" && f.kind == FindingKind::Violation));
    assert!(!finding(
        &r,
        "program-spin-level-credit",
        Some(0),
        FindingKind::Unknown
    ));
    assert!(finding(
        &r,
        "program-spin-level-credit",
        Some(1),
        FindingKind::Unknown
    ));
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entry.1 {
        spin.feature_attempts.as_mut().unwrap().swap(0, 2);
    }
    let r = report(
        AdultCategory::Gold,
        vec![entry.clone()],
        SubmissionState::Complete,
    );
    assert!(!r
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-eligibility" && f.kind == FindingKind::Violation));
    assert!(!finding(
        &r,
        "program-spin-level-credit",
        Some(0),
        FindingKind::Unknown
    ));
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entry.1 {
        spin.feature_attempts.as_mut().unwrap().swap(0, 2);
    }
    // An unknown attempt on one foot does not change the first successful spin.
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entry.1 {
        spin.feature_attempts.as_mut().unwrap()[2].outcome = FeatureAttemptOutcome::Unknown;
    }
    let r = report(AdultCategory::Gold, vec![entry], SubmissionState::Complete);
    assert!(!r
        .validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id == "spin-eligibility" && f.kind == FindingKind::Violation));
    assert!(!finding(
        &r,
        "program-spin-level-credit",
        Some(0),
        FindingKind::Unknown
    ));
}

#[test]
fn coverage_retains_violation_and_unknown_for_the_same_rule_instance() {
    let entry = (
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: None,
        })),
        ElementPlanEvidence::Jump {
            direct_transitions_assumed: false,
        },
    );
    let r = report(AdultCategory::Bronze, vec![entry], SubmissionState::Draft);
    let eligibility: Vec<_> = r
        .rules
        .iter()
        .filter(|r| {
            r.rule_id == "isu-adult-2026-27-bronze-jump-eligibility" && r.element_index == Some(0)
        })
        .collect();
    assert!(eligibility
        .iter()
        .any(|r| r.state == RuleCoverageState::MissingEvidence));
    assert!(eligibility
        .iter()
        .any(|r| r.state == RuleCoverageState::Evaluated
            && r.reason == "Axel jumps are prohibited in Bronze."));
}

#[test]
fn successful_position_prerequisites_are_traced_and_combination_type_is_not_applicable() {
    let entry = spin(
        SpinPosition::Camel,
        Level::One,
        Some(SpinFeatureAttempt {
            feature: SpinFeature::IncreaseOfSpeed,
            position_step: 0,
            outcome: FeatureAttemptOutcome::Successful,
        }),
        8,
    );
    let r = report(
        AdultCategory::Bronze,
        vec![entry],
        SubmissionState::Complete,
    );
    assert!(r.rules.iter().any(
        |r| r.rule_id == "program-feature-position" && r.state == RuleCoverageState::Evaluated
    ));
    let mut combo = spin(SpinPosition::Camel, Level::Basic, None, 8);
    if let PlannedElement::RegularSpin(plan) = &mut combo.0 {
        for position in [SpinPosition::Sit, SpinPosition::Upright] {
            plan.steps.as_mut().unwrap().push(SpinStep::Position {
                position: Some(position),
                revolutions: Some(2),
                feature_attempts: Some(vec![]),
            });
        }
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut combo.1 {
        spin.abbreviation.as_mut().unwrap().kind = RegularSpinKind::Combination;
    }
    let r = report(
        AdultCategory::Silver,
        vec![combo],
        SubmissionState::Complete,
    );
    assert!(r
        .rules
        .iter()
        .any(|r| r.rule_id == "spin-position-type" && r.state == RuleCoverageState::NotApplicable));
    assert!(r
        .rules
        .iter()
        .any(|r| r.rule_id == "spin-combination-positions"
            && r.state == RuleCoverageState::Evaluated));
}

fn custom_report(profile: RuleProfile, policy: ProgramPolicy) -> ProgramLegalityReport {
    custom_report_entries(profile, policy, vec![])
}

fn custom_report_entries(
    profile: RuleProfile,
    policy: ProgramPolicy,
    entries: Vec<(PlannedElement, ElementPlanEvidence)>,
) -> ProgramLegalityReport {
    let mut registry = ProfileRegistry::adult();
    registry.register(profile.clone()).unwrap();
    let (elements, evidence): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
    assess_program_legality(
        &registry,
        &ProfileProgram {
            profile: profile.id,
            scoring_table: TABLE,
            elements,
        },
        &evidence,
        &policy,
        SubmissionState::Draft,
    )
    .unwrap()
}
fn custom_identity() -> (RuleProfile, ProgramPolicy) {
    let mut profile = adult_profile(AdultCategory::Silver);
    profile.id = ProfileId {
        name: "synthetic-review".into(),
        version: "test-1".into(),
    };
    profile.provenance = ProfileProvenance::Synthetic;
    let mut policy = adult_program_policy(AdultCategory::Silver);
    policy.spins.profile = profile.id.clone();
    (profile, policy)
}

#[test]
fn competing_required_jump_families_are_not_jointly_insertable_in_one_slot() {
    let (mut profile, policy) = custom_identity();
    profile.jumps.max_jumps = 1;
    profile.families.required = vec![
        ProfileElementFamily::SoloJump,
        ProfileElementFamily::Combination,
    ];
    // The exact empty/max-one reproduction is rejected by the registry already.
    assert!(ProfileRegistry::adult().register(profile.clone()).is_err());
    profile.jumps.max_jumps = 2;
    let eligibility = profile
        .bindings
        .iter()
        .find(|b| b.rule == ProfileRule::Eligibility)
        .unwrap()
        .id;
    let sequence = (
        PlannedElement::Jump(JumpElement::Sequence(vec![
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::ToeLoop),
                rotations: Some(1),
            }),
            JumpPart::Jump(ListedJump {
                kind: Some(JumpKind::Axel),
                rotations: Some(1),
            }),
        ])),
        ElementPlanEvidence::Jump {
            direct_transitions_assumed: true,
        },
    );
    let r = custom_report_entries(profile.clone(), policy.clone(), vec![sequence]);
    assert!(finding(&r, eligibility, None, FindingKind::Violation));
    assert!(!r
        .repairable_absences
        .iter()
        .any(|i| r.validation.validation.findings[*i].rule_id == eligibility));
    let r = custom_report(profile, policy);
    assert!(r
        .repairable_absences
        .iter()
        .any(|i| r.validation.validation.findings[*i].rule_id == eligibility));
}

#[test]
fn unevaluated_transition_retains_selected_event_replacement() {
    let (mut profile, policy) = custom_identity();
    let mut binding = profile
        .bindings
        .iter()
        .find(|b| b.rule == ProfileRule::Transition)
        .unwrap()
        .clone();
    let replaces = binding.id;
    binding.id = "synthetic-transition-override";
    binding.source = SourceReference {
        document: "Synthetic transition fixture".into(),
        version: "test-1".into(),
        location: "no compound in this plan".into(),
    };
    profile.sources.push(binding.source.clone());
    let selected = RuleOverride {
        replaces,
        replacement: binding,
        event_scope: profile.event_scope.clone(),
        justification: "test inventory provenance".into(),
    };
    profile.overrides.push(selected.clone());
    let r = custom_report(profile, policy);
    assert!(r.rules.iter().any(|r| r.rule_id == selected.replacement.id
        && r.state == RuleCoverageState::NotApplicable
        && r.event_override.as_ref() == Some(&selected)));
}

#[test]
fn compatible_spin_obligations_can_share_one_insertion() {
    let (profile, mut policy) = custom_identity();
    policy.spins.max_spins = 1;
    policy.spins.required = vec![
        vec![SpinRequirement {
            kind: None,
            entry: Some(SpinEntry::NonFlying),
            change_foot: Some(false),
        }],
        vec![SpinRequirement {
            kind: Some(RegularSpinKind::Sit),
            entry: Some(SpinEntry::NonFlying),
            change_foot: Some(false),
        }],
    ];
    let r = custom_report(profile, policy);
    let missing: Vec<_> = r
        .validation
        .validation
        .findings
        .iter()
        .enumerate()
        .filter(|(_, f)| f.rule_id == "spin-required-role")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(missing.len(), 2);
    assert!(missing.iter().all(|i| r.repairable_absences.contains(i)));
}

#[test]
fn unknown_feature_lists_and_uncheckable_targets_differ_from_explicitly_empty_lists() {
    // None is missing data. An explicitly empty list establishes no prior attempt.
    for feature in [SpinFeature::EightRevolutions, SpinFeature::IncreaseOfSpeed] {
        let attempted = SpinFeatureAttempt {
            feature,
            position_step: 0,
            outcome: FeatureAttemptOutcome::Successful,
        };
        for (attempts, missing_steps, uncertain) in [
            (None, false, true),
            (Some(vec![]), false, false),
            (Some(vec![attempted.clone()]), true, true),
        ] {
            let mut first = spin(SpinPosition::Camel, Level::Basic, None, 8);
            if let ElementPlanEvidence::Spin { spin, .. } = &mut first.1 {
                spin.feature_attempts = attempts;
            }
            if missing_steps {
                if let PlannedElement::RegularSpin(plan) = &mut first.0 {
                    plan.steps = None;
                }
            }
            let mut later = spin(SpinPosition::Camel, Level::One, Some(attempted.clone()), 8);
            if let PlannedElement::RegularSpin(plan) = &mut later.0 {
                plan.entry = Some(SpinEntry::Flying);
            }
            if let ElementPlanEvidence::Spin { spin, .. } = &mut later.1 {
                spin.abbreviation.as_mut().unwrap().entry = SpinEntry::Flying;
            }
            let r = report(
                AdultCategory::Silver,
                vec![first, later],
                SubmissionState::Complete,
            );
            assert!(!r
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.rule_id == "spin-eligibility" && f.kind == FindingKind::Violation));
            assert_eq!(
                finding(
                    &r,
                    "program-feature-once-per-program",
                    Some(1),
                    FindingKind::Unknown
                ),
                uncertain
            );
            assert_eq!(
                finding(
                    &r,
                    "program-spin-level-credit",
                    Some(1),
                    FindingKind::Unknown
                ),
                uncertain
            );
            for id in [
                "program-feature-qualification",
                "program-feature-position",
                "program-feature-once-per-program",
            ] {
                let expected = if uncertain
                    && !(id == "program-feature-position"
                        && feature == SpinFeature::EightRevolutions
                        && missing_steps)
                {
                    RuleCoverageState::MissingEvidence
                } else {
                    RuleCoverageState::NotApplicable
                };
                assert!(
                    r.rules.iter().any(|r| r.rule_id == id
                        && r.element_index == Some(0)
                        && r.state == expected),
                    "{id} should be {expected:?}"
                );
            }
        }
    }
}

#[test]
fn missing_basic_position_is_unknown_and_known_unmodeled_position_is_unsupported() {
    for feature in [SpinFeature::EightRevolutions, SpinFeature::IncreaseOfSpeed] {
        for position in [None, Some(SpinPosition::Upright)] {
            let mut entry = spin(
                SpinPosition::Upright,
                Level::One,
                Some(SpinFeatureAttempt {
                    feature,
                    position_step: 0,
                    outcome: FeatureAttemptOutcome::Successful,
                }),
                8,
            );
            if let PlannedElement::RegularSpin(plan) = &mut entry.0 {
                if let SpinStep::Position {
                    position: target, ..
                } = &mut plan.steps.as_mut().unwrap()[0]
                {
                    *target = position;
                }
            }
            let r = report(
                AdultCategory::Silver,
                vec![entry],
                SubmissionState::Complete,
            );
            let id = if feature == SpinFeature::EightRevolutions {
                "program-eight-position"
            } else {
                "program-feature-position"
            };
            let kind = if position.is_none() {
                FindingKind::Unknown
            } else {
                FindingKind::Unsupported
            };
            let state = if position.is_none() {
                RuleCoverageState::MissingEvidence
            } else {
                RuleCoverageState::Unsupported
            };
            assert!(finding(&r, id, Some(0), kind));
            assert!(!finding(
                &r,
                id,
                Some(0),
                if kind == FindingKind::Unknown {
                    FindingKind::Unsupported
                } else {
                    FindingKind::Unknown
                }
            ));
            assert!(r
                .rules
                .iter()
                .any(|r| r.rule_id == id && r.element_index == Some(0) && r.state == state));
            if feature == SpinFeature::EightRevolutions {
                assert!(r
                    .rules
                    .iter()
                    .any(|r| r.rule_id == "program-feature-once-per-program"
                        && r.element_index == Some(0)
                        && r.state == state));
                assert!(!r
                    .rules
                    .iter()
                    .any(|r| r.rule_id == "program-feature-once-per-program"
                        && r.element_index == Some(0)
                        && r.state == RuleCoverageState::NotApplicable));
            }
        }
    }
}
