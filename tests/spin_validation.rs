use skating_program_lab::*;

fn position(p: Option<SpinPosition>, n: Option<u16>) -> SpinStep {
    SpinStep::Position {
        position: p,
        revolutions: n,
        feature_attempts: Some(vec![]),
    }
}
fn spin(entry: SpinEntry, steps: Vec<SpinStep>) -> PlannedElement {
    PlannedElement::RegularSpin(RegularSpin {
        entry: Some(entry),
        steps: Some(steps),
    })
}
fn data(kind: RegularSpinKind, entry: SpinEntry, change_foot: bool) -> ProgramValidationEvidence {
    ProgramValidationEvidence {
        spins: vec![IndexedSpinEvidence {
            element_index: 0,
            spin: RegularSpinEvidence {
                abbreviation: Some(SpinAbbreviation {
                    kind,
                    entry,
                    change_foot,
                }),
                planned_level: Some(Level::Basic),
                feature_attempts: Some(vec![]),
                panel: None,
            },
        }],
    }
}
fn program(category: AdultCategory, elements: Vec<PlannedElement>) -> Program {
    Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category,
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        sources: vec![],
        elements,
    }
}
fn check(
    category: AdultCategory,
    steps: Vec<SpinStep>,
    evidence: &ProgramValidationEvidence,
) -> EvidenceValidationReport {
    validate_program_with_evidence(
        &program(category, vec![spin(SpinEntry::NonFlying, steps)]),
        evidence,
    )
    .unwrap()
}
fn findings<'a>(r: &'a EvidenceValidationReport, rule: &str) -> Vec<&'a Finding> {
    r.validation
        .findings
        .iter()
        .filter(|f| f.rule_id == rule)
        .collect()
}

#[test]
fn independently_transcribed_adult_revolution_and_level_boundaries() {
    // Announcement July 16, printed pp. 11, 10, 9, 8 respectively.
    // Constants are source fixtures, not read back from production policies.
    for (category, plain_min, changed_min, cap, page) in [
        (AdultCategory::Bronze, 3, 6, Level::One, "p. 11"),
        (AdultCategory::Silver, 4, 6, Level::Two, "p. 10"),
        (AdultCategory::Gold, 4, 8, Level::Three, "p. 9"),
        (AdultCategory::Masters, 5, 8, Level::Four, "p. 8"),
    ] {
        for changed in [false, true] {
            let minimum = if changed { changed_min } else { plain_min };
            for (total, expected) in [
                (minimum - 1, Some(FindingKind::Violation)),
                (minimum, None),
                (minimum + 1, None),
            ] {
                let steps = if changed {
                    vec![
                        position(Some(SpinPosition::Sit), Some(3)),
                        SpinStep::ChangeFoot,
                        position(Some(SpinPosition::Sit), Some(total - 3)),
                    ]
                } else {
                    vec![position(Some(SpinPosition::Sit), Some(total))]
                };
                let mut evidence = data(RegularSpinKind::Sit, SpinEntry::NonFlying, changed);
                evidence.spins[0].spin.planned_level = Some(cap);
                let r = check(category, steps, &evidence);
                let f = findings(&r, "spin-minimum-revolutions");
                assert_eq!(
                    f.first().map(|f| f.kind),
                    expected,
                    "{category:?} changed={changed} total={total}"
                );
                if let Some(f) = f.first() {
                    assert!(f.source.location.starts_with(page));
                    assert_eq!(f.source.version, "2026-07-16");
                }
                assert!(findings(&r, "spin-level-cap").is_empty());
                // Plan success never establishes execution or complete compliance.
                assert_eq!(r.validation.coverage, Coverage::Partial);
                assert_eq!(
                    findings(&r, "spin-execution-recognition")[0].kind,
                    FindingKind::Unknown
                );
            }
            let unknown = check(
                category,
                vec![position(Some(SpinPosition::Sit), None)],
                &data(RegularSpinKind::Sit, SpinEntry::NonFlying, changed),
            );
            assert_eq!(
                findings(&unknown, "spin-minimum-revolutions")[0].kind,
                FindingKind::Unknown
            );
        }
        let higher = match cap {
            Level::One => Some(Level::Two),
            Level::Two => Some(Level::Three),
            Level::Three => Some(Level::Four),
            _ => None,
        };
        if let Some(higher) = higher {
            let mut evidence = data(RegularSpinKind::Sit, SpinEntry::NonFlying, false);
            evidence.spins[0].spin.planned_level = Some(higher);
            let r = check(
                category,
                vec![position(Some(SpinPosition::Sit), Some(8))],
                &evidence,
            );
            let f = findings(&r, "spin-level-cap")[0];
            assert_eq!(f.kind, FindingKind::Unknown);
            assert!(f.explanation.contains("ignored"));
        }
    }
}

#[test]
fn all_twenty_spin_abbreviations_use_profile_roles() {
    use RegularSpinKind::*;
    // Adult pp. 8–11; Gold flying change optional, Masters flying no change.
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        for kind in [Camel, Sit, Upright, Layback, Combination] {
            for entry in [SpinEntry::NonFlying, SpinEntry::Flying] {
                for changed in [false, true] {
                    let mut steps = vec![position(Some(SpinPosition::Camel), Some(4))];
                    if changed {
                        steps.extend([
                            SpinStep::ChangeFoot,
                            position(Some(SpinPosition::Camel), Some(4)),
                        ]);
                    }
                    let r = validate_program_with_evidence(
                        &program(category, vec![spin(entry, steps)]),
                        &data(kind, entry, changed),
                    )
                    .unwrap();
                    let allowed = match category {
                        AdultCategory::Bronze => {
                            entry == SpinEntry::NonFlying && kind != Combination
                        }
                        AdultCategory::Silver => true,
                        AdultCategory::Gold => {
                            entry == SpinEntry::Flying || (kind == Combination && changed)
                        }
                        AdultCategory::Masters => {
                            (entry == SpinEntry::Flying && !changed)
                                || (entry == SpinEntry::NonFlying && kind == Combination)
                        }
                    };
                    assert_eq!(
                        findings(&r, "spin-eligibility").is_empty(),
                        allowed,
                        "{category:?} {kind:?} {entry:?} {changed}"
                    );
                }
            }
        }
    }
}

#[test]
fn basic_positions_per_foot_and_no_value_are_conditional_and_distinct_from_prohibition() {
    // Adult p. 11 and 2788 p. 4: 2 + 4 total meets six, but first foot lacks 3 -> V.
    let d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, true);
    let r = check(
        AdultCategory::Bronze,
        vec![
            position(Some(SpinPosition::Sit), Some(2)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), Some(4)),
        ],
        &d,
    );
    assert!(findings(&r, "spin-minimum-revolutions").is_empty());
    let v = findings(&r, "spin-per-foot-revolutions")[0];
    assert_eq!(v.kind, FindingKind::Unknown);
    assert!(v.explanation.contains("conditional V"));
    assert_eq!(
        v.source.location,
        "p. 4, Free Skating change-foot revolutions and basic positions"
    );
    assert!(findings(&r, "spin-eligibility").is_empty());
    let r = check(
        AdultCategory::Bronze,
        vec![
            position(Some(SpinPosition::Sit), Some(1)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), Some(1)),
        ],
        &d,
    );
    assert!(findings(&r, "spin-basic-position")[0]
        .explanation
        .contains("conditional no-value"));
    let valid = check(
        AdultCategory::Bronze,
        vec![
            position(Some(SpinPosition::Sit), Some(3)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), Some(3)),
        ],
        &d,
    );
    assert!(findings(&valid, "spin-per-foot-revolutions").is_empty());
    let unknown = check(
        AdultCategory::Bronze,
        vec![
            position(None, Some(3)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), None),
        ],
        &d,
    );
    assert!(findings(&unknown, "spin-position-evidence")
        .iter()
        .all(|f| f.kind == FindingKind::Unknown));
}

#[test]
fn combination_two_positions_v_three_full_value_and_missing_unknown() {
    // Adult p. 10; a non-basic position on a foot does not alone cause V for a combination.
    let d = data(RegularSpinKind::Combination, SpinEntry::NonFlying, false);
    let two = vec![
        position(Some(SpinPosition::Sit), Some(2)),
        position(Some(SpinPosition::Camel), Some(2)),
    ];
    let r = check(AdultCategory::Silver, two.clone(), &d);
    assert!(findings(&r, "spin-combination-positions")[0]
        .explanation
        .contains("conditional V"));
    let mut three = two;
    three.push(position(Some(SpinPosition::Upright), Some(2)));
    let r = check(AdultCategory::Silver, three.clone(), &d);
    assert!(findings(&r, "spin-combination-positions").is_empty());
    three.push(SpinStep::ChangeFoot);
    three.push(position(None, Some(3)));
    let r = check(
        AdultCategory::Silver,
        three,
        &data(RegularSpinKind::Combination, SpinEntry::NonFlying, true),
    );
    assert!(findings(&r, "spin-per-foot-revolutions").is_empty());
    let r = check(
        AdultCategory::Silver,
        vec![position(Some(SpinPosition::Sit), Some(4))],
        &d,
    );
    assert!(findings(&r, "spin-combination-positions")[0]
        .explanation
        .contains("fallback"));
    let r = check(AdultCategory::Silver, vec![position(None, None)], &d);
    assert!(findings(&r, "spin-combination-positions")[0]
        .explanation
        .contains("Missing"));
}

#[test]
fn explicit_empty_unknown_and_legacy_text_features_are_not_conflated() {
    let mut d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, false);
    let steps = vec![SpinStep::Position {
        position: Some(SpinPosition::Sit),
        revolutions: Some(3),
        feature_attempts: Some(vec!["confirmed feature 10".into()]),
    }];
    assert!(findings(
        &check(AdultCategory::Bronze, steps.clone(), &d),
        "spin-feature-evidence"
    )
    .is_empty());
    d.spins[0].spin.feature_attempts = None;
    let r = check(AdultCategory::Bronze, steps.clone(), &d);
    assert_eq!(
        findings(&r, "spin-feature-evidence")[0].kind,
        FindingKind::Unknown
    );
    d.spins[0].spin.feature_attempts = Some(vec![SpinFeatureAttempt {
        feature: SpinFeature::EightRevolutions,
        position_step: 0,
        outcome: FeatureAttemptOutcome::Successful,
    }]);
    let r = check(AdultCategory::Bronze, steps, &d);
    assert_eq!(
        findings(&r, "spin-feature-credit")[0].kind,
        FindingKind::Unsupported
    );
    assert_eq!(d.spins[0].spin.planned_level, Some(Level::Basic));
}

#[test]
fn invalid_evidence_targets_duplicates_order_entry_and_foot_change_are_rejected() {
    let mut d = data(RegularSpinKind::Sit, SpinEntry::Flying, true);
    d.spins[0].spin.feature_attempts = Some(vec![
        SpinFeatureAttempt {
            feature: SpinFeature::DifficultExit,
            position_step: 2,
            outcome: FeatureAttemptOutcome::Unknown,
        },
        SpinFeatureAttempt {
            feature: SpinFeature::DifficultEntrance,
            position_step: 0,
            outcome: FeatureAttemptOutcome::Unsuccessful,
        },
    ]);
    let r = check(
        AdultCategory::Silver,
        vec![position(Some(SpinPosition::Sit), Some(4))],
        &d,
    );
    for rule in [
        "spin-feature-target",
        "spin-evidence-entry",
        "spin-evidence-foot-change",
    ] {
        assert_eq!(findings(&r, rule)[0].kind, FindingKind::Violation);
    }
    d.spins.push(d.spins[0].clone());
    d.spins.push(IndexedSpinEvidence {
        element_index: 200,
        spin: RegularSpinEvidence::default(),
    });
    let r = check(AdultCategory::Silver, vec![], &d);
    assert_eq!(findings(&r, "spin-evidence-target").len(), 2);
    assert_eq!(
        findings(&r, "spin-abbreviation")[0].kind,
        FindingKind::Unknown
    );
    let r = validate_program_with_evidence(
        &program(AdultCategory::Bronze, vec![PlannedElement::StepSequence]),
        &data(RegularSpinKind::Sit, SpinEntry::NonFlying, false),
    )
    .unwrap();
    assert_eq!(
        findings(&r, "spin-evidence-target")[0].kind,
        FindingKind::Violation
    );
}

#[test]
fn counts_abbreviation_duplicates_and_required_roles_use_ordered_program() {
    let mut d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, false);
    let s = spin(
        SpinEntry::NonFlying,
        vec![position(Some(SpinPosition::Sit), Some(4))],
    );
    let mut second = d.spins[0].clone();
    second.element_index = 1;
    second.spin.planned_level = Some(Level::One);
    d.spins.push(second);
    let r = validate_program_with_evidence(
        &program(
            AdultCategory::Silver,
            vec![s.clone(), s.clone(), PlannedElement::ChoreographicSpin],
        ),
        &d,
    )
    .unwrap();
    assert_eq!(findings(&r, "spin-count")[0].kind, FindingKind::Violation);
    assert_eq!(
        findings(&r, "spin-distinct-abbreviation")[0].element_index,
        Some(1)
    );
    let r = validate_program_with_evidence(
        &program(
            AdultCategory::Silver,
            vec![s.clone(), s, PlannedElement::Unspecified],
        ),
        &d,
    )
    .unwrap();
    assert_eq!(findings(&r, "spin-count")[0].kind, FindingKind::Unknown);
    for category in [
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let r = validate_program_with_evidence(
            &program(category, vec![]),
            &ProgramValidationEvidence::default(),
        )
        .unwrap();
        assert!(findings(&r, "spin-required-role")
            .iter()
            .all(|f| f.kind == FindingKind::Violation));
        assert!(!findings(&r, "spin-required-role").is_empty());
        let r = validate_program_with_evidence(
            &program(category, vec![PlannedElement::Unspecified]),
            &ProgramValidationEvidence::default(),
        )
        .unwrap();
        assert!(findings(&r, "spin-required-role")
            .iter()
            .all(|f| f.kind == FindingKind::Unknown));
    }
}

#[test]
fn synthetic_profile_uses_same_policy_mechanisms_without_adult_fallback() {
    let mut profile = adult_profile(AdultCategory::Silver);
    profile.id.name = "synthetic-spin".into();
    profile.provenance = ProfileProvenance::Synthetic;
    let mut policy = adult_regular_spin_policy(AdultCategory::Silver);
    policy.profile = profile.id.clone();
    policy.source = SourceReference {
        document: "Synthetic spin fixture".into(),
        version: "v1".into(),
        location: "one sit spin; 7 revolutions; level 1".into(),
    };
    policy.max_spins = 1;
    policy.min_revolutions = 7;
    policy.max_level = Level::One;
    policy.allowed = vec![SpinRequirement {
        kind: Some(RegularSpinKind::Sit),
        entry: None,
        change_foot: None,
    }];
    policy.required = vec![policy.allowed.clone()];
    let mut registry = ProfileRegistry::adult();
    registry.register(profile.clone()).unwrap();
    let p = ProfileProgram {
        profile: profile.id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        elements: vec![spin(
            SpinEntry::NonFlying,
            vec![position(Some(SpinPosition::Sit), Some(6))],
        )],
    };
    let d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, false);
    let r = validate_profile_program_with_evidence(&registry, &p, &d, None).unwrap();
    assert_eq!(
        findings(&r, "spin-policy")[0].kind,
        FindingKind::Unsupported
    );
    let r = validate_profile_program_with_evidence(&registry, &p, &d, Some(&policy)).unwrap();
    assert_eq!(
        findings(&r, "spin-minimum-revolutions")[0].source,
        policy.source
    );
    policy.profile.version = "wrong".into();
    assert_eq!(
        validate_profile_program_with_evidence(&registry, &p, &d, Some(&policy))
            .unwrap_err()
            .kind,
        ProfileErrorKind::Invalid
    );
}

#[test]
fn empty_missing_and_multiple_change_plans_preserve_uncertainty() {
    let d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, true);
    let r = check(AdultCategory::Bronze, vec![], &d);
    assert_eq!(
        findings(&r, "spin-minimum-revolutions")[0].kind,
        FindingKind::Violation
    );
    let r = validate_program_with_evidence(
        &program(
            AdultCategory::Bronze,
            vec![PlannedElement::RegularSpin(RegularSpin {
                entry: None,
                steps: None,
            })],
        ),
        &ProgramValidationEvidence::default(),
    )
    .unwrap();
    for rule in [
        "spin-entry",
        "spin-rotations",
        "spin-abbreviation",
        "spin-level-cap",
    ] {
        assert_eq!(findings(&r, rule)[0].kind, FindingKind::Unknown);
    }
    let r = check(
        AdultCategory::Bronze,
        vec![
            position(Some(SpinPosition::Sit), Some(3)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), Some(3)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Sit), Some(3)),
        ],
        &d,
    );
    assert_eq!(
        findings(&r, "spin-multiple-foot-changes")[0].kind,
        FindingKind::Unsupported
    );
}

#[test]
fn legacy_jump_findings_are_preserved_and_coverage_cites_unsupported_slices() {
    let p = program(
        AdultCategory::Bronze,
        vec![PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        }))],
    );
    let legacy = validate_program(&p);
    let r = validate_program_with_evidence(&p, &ProgramValidationEvidence::default()).unwrap();
    assert_eq!(r.validation.findings, legacy.findings);
    assert_eq!(r.coverage.len(), 4);
    assert!(r
        .coverage
        .iter()
        .filter(|c| matches!(
            c.family,
            ValidationFamily::SpinFeatures | ValidationFamily::StepsAndChoreography
        ))
        .all(|c| c.coverage == FamilyCoverage::Unsupported));
    assert!(r
        .sources
        .iter()
        .any(|s| s.document == "ISU Communication 2788" && s.version == "2026-05-19"));
    let mut p = p;
    p.season.start_year = 2025;
    assert_eq!(
        validate_program_with_evidence(&p, &ProgramValidationEvidence::default())
            .unwrap_err()
            .kind,
        ProfileErrorKind::Unsupported
    );
}
#[test]
fn one_position_foot_checks_require_the_intended_position_and_panel_never_fills_gaps() {
    let mut d = data(RegularSpinKind::Sit, SpinEntry::NonFlying, true);
    let r = check(
        AdultCategory::Bronze,
        vec![
            position(Some(SpinPosition::Sit), Some(3)),
            SpinStep::ChangeFoot,
            position(Some(SpinPosition::Camel), Some(3)),
        ],
        &d,
    );
    assert_eq!(findings(&r, "spin-per-foot-revolutions").len(), 1);
    d.spins[0].spin.panel = Some(PanelRecognition {
        ruleset: RulesetId::IsuAdult2026_27July16,
        source: None,
        planned_element: Some(0),
        observed_element: None,
        outcome: RecognitionOutcome::NoValue,
        details: None,
    });
    let r = check(AdultCategory::Bronze, vec![position(None, None)], &d);
    assert_eq!(
        findings(&r, "spin-position-evidence")[0].kind,
        FindingKind::Unknown
    );
    assert_eq!(
        findings(&r, "spin-panel-reconciliation")[0].kind,
        FindingKind::Unsupported
    );
    assert!(findings(&r, "spin-panel-target").is_empty());
    d.spins[0].spin.panel.as_mut().unwrap().planned_element = Some(5);
    let r = check(AdultCategory::Bronze, vec![], &d);
    assert_eq!(
        findings(&r, "spin-panel-target")[0].kind,
        FindingKind::Violation
    );
}
