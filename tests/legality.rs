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
