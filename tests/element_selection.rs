use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

#[test]
fn six_profile_source_snapshots() {
    let adults = ProfileRegistry::adult();
    let seniors = ProfileRegistry::senior();
    let cases = [
        (
            &adults,
            adult_profile(AdultCategory::Bronze),
            1,
            0,
            true,
            false,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Silver),
            2,
            0,
            true,
            false,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Gold),
            3,
            3,
            false,
            true,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Masters),
            4,
            5,
            false,
            true,
        ),
        (
            &seniors,
            senior_profile(SeniorCategory::Women),
            4,
            5,
            true,
            true,
        ),
        (
            &seniors,
            senior_profile(SeniorCategory::Men),
            4,
            5,
            true,
            true,
        ),
    ];
    // ISU 2786 (2 June 2026), p. 9, value rows 1–5 and 21–25.
    let spins = [
        ("CCoSpB", Level::Basic, 200),
        ("CCoSp1", Level::One, 240),
        ("CCoSp2", Level::Two, 300),
        ("CCoSp3", Level::Three, 360),
        ("CCoSp4", Level::Four, 420),
    ];
    let steps = [
        ("StSqB", Level::Basic, 160),
        ("StSq1", Level::One, 190),
        ("StSq2", Level::Two, 270),
        ("StSq3", Level::Three, 350),
        ("StSq4", Level::Four, 410),
    ];
    for (registry, profile, spin_cap, step_count, chsq, chsp) in cases {
        let choices = selectable_elements(registry, &profile.id, TABLE).unwrap();
        let get = |code| choices.iter().find(|choice| choice.row.notation == code);
        for (i, (code, level, base)) in spins.into_iter().enumerate() {
            let option = get(code);
            assert_eq!(
                option.is_some(),
                spin_cap != 1 && i <= spin_cap,
                "{:?} {code}",
                profile.id
            );
            if let Some(option) = option {
                assert_eq!(
                    (option.row.level, option.row.base_hundredths),
                    (Some(level), base)
                );
                assert_eq!(
                    (option.row.source.page, option.row.source.row),
                    (9, i as u8 + 1)
                );
            }
        }
        for (i, (code, level, base)) in steps.into_iter().enumerate() {
            let option = get(code);
            assert_eq!(option.is_some(), i < step_count);
            if let Some(option) = option {
                assert_eq!(
                    (option.row.level, option.row.base_hundredths),
                    (Some(level), base)
                );
                assert_eq!(
                    (option.row.source.page, option.row.source.row),
                    (9, i as u8 + 21)
                );
            }
        }
        if spin_cap == 1 {
            assert_eq!(get("CSpB").unwrap().row.level, Some(Level::Basic));
            assert_eq!(get("CSp1").unwrap().row.level, Some(Level::One));
            assert!(get("CSp2").is_none());
            assert!(get("FSSpB").is_none());
        }
        for (code, allowed, row) in [("ChSq1", chsq, 11), ("ChSp1", chsp, 12)] {
            assert_eq!(get(code).is_some(), allowed);
            if let Some(option) = get(code) {
                assert_eq!(option.row.level, None);
                assert_eq!((option.row.source.page, option.row.source.row), (14, row));
            }
        }
        for code in ["1LiB", "Eu", "1Tq", "CCoSp5", "ChSp2", "ChSq3"] {
            assert!(get(code).is_none(), "{code}");
        }
        assert!(get("1T").is_some());
        assert!(choices.iter().all(|option| {
            option.row.discipline == CatalogDiscipline::SingleAndPair
                && std::ptr::eq(option.row, catalog_row(TABLE, option.row.notation).unwrap())
        }));
    }
}

#[test]
fn direct_selection_and_manual_assessment_keep_the_exact_code() {
    let registry = ProfileRegistry::adult();
    let bronze = adult_profile(AdultCategory::Bronze).id;
    let silver = adult_profile(AdultCategory::Silver).id;
    let gold = adult_profile(AdultCategory::Gold).id;
    for code in ["CCoSp1", "CCoSp4"] {
        assert!(matches!(
            check_selectable_element(&registry, &bronze, TABLE, code),
            Err(ElementSelectionError::TypeNotSelectable(f)) if f.rule_id == "spin-eligibility"
        ));
    }
    for code in ["1LiB", "CCSp3V"] {
        assert!(matches!(
            check_selectable_element(&registry, &gold, TABLE, code),
            Err(ElementSelectionError::UnsupportedCode { source }) if source.location.contains(code)
        ));
    }
    assert!(matches!(
        check_selectable_element(&registry, &gold, TABLE, "ChSp2"),
        Err(ElementSelectionError::UnknownCode { .. })
    ));
    assert!(matches!(
        check_selectable_element(&registry, &silver, TABLE, "CCoSp3"),
        Err(ElementSelectionError::LevelNotSelectable(f)) if f.rule_id == "spin-level-cap"
    ));
    let mut unknown = gold.clone();
    unknown.version = "unknown".into();
    assert!(selectable_elements(&registry, &unknown, TABLE).is_err());
    let assess = |category, id| {
        assess_manual_program_with_assumed_recognition(
            &registry,
            &ProfileProgram {
                profile: id,
                scoring_table: TABLE,
                elements: vec![PlannedElement::RegularSpin(RegularSpin {
                    entry: None,
                    steps: None,
                })],
            },
            &[AssumedRecognition::new("CCoSp3")],
            &adult_program_policy(category),
            SubmissionState::Draft,
        )
    };
    let err = assess(AdultCategory::Silver, silver).unwrap_err();
    assert!(err.reason.contains("CCoSp3") && err.reason.contains("exceeds the profile cap"));
    let choice = check_selectable_element(&registry, &gold, TABLE, "CCoSp3").unwrap();
    let result = assess(AdultCategory::Gold, gold).unwrap();
    let value = &result.unfactored_base[0];
    assert_eq!(value.notation, choice.row.notation);
    assert_eq!(value.hundredths, choice.row.base_hundredths);
    assert_eq!(
        value.source,
        choice.row.source.reference(choice.row.notation)
    );
}

#[test]
fn selection_does_not_fall_back_to_a_builtin_profile() {
    let mut registry = ProfileRegistry::adult();
    let mut short = adult_profile(AdultCategory::Silver);
    short.id.name = "adult-silver-short".into();
    short.segment = ProgramSegment::ShortProgram;
    let mut unimplemented = adult_profile(AdultCategory::Silver);
    unimplemented.id.name = "unimplemented-official-category".into();
    unimplemented.category = "Unimplemented".into();
    for profile in [short, unimplemented] {
        let id = profile.id.clone();
        registry.register(profile).unwrap();
        assert!(matches!(
            check_selectable_element(&registry, &id, TABLE, "CCoSp1"),
            Err(ElementSelectionError::Profile(ProfileSelectionError {
                kind: ProfileErrorKind::Unsupported,
                profile,
                ..
            })) if profile == id
        ));
    }
}

#[test]
fn full_selection_matrix_matches_independent_category_and_catalog_fixtures() {
    use std::collections::BTreeSet;
    let adult = ProfileRegistry::adult();
    let senior = ProfileRegistry::senior();
    // Adult announcement pp. 8–11 and ISU Sports Rules, Rule 612 pp. 70–71.
    let bronze_spins = "CSp SSp USp LSp CCSp CSSp CUSp CLSp";
    let silver_spins = "CSp SSp USp LSp CCSp CSSp CUSp CLSp FCSp FSSp FUSp FLSp FCCSp FCSSp FCUSp FCLSp CoSp CCoSp FCoSp FCCoSp";
    let gold_spins = "CCoSp FCSp FSSp FUSp FLSp FCCSp FCSSp FCUSp FCLSp FCoSp FCCoSp";
    let masters_spins = "CoSp CCoSp FCSp FSSp FUSp FLSp FCoSp";
    let senior_spins = "CoSp CCoSp FCoSp FCCoSp FCSp FSSp FUSp FLSp";
    let singles = "1T 1S 1Lo 1F 1Lz";
    let doubles = "1A 2T 2S 2Lo 2F 2Lz 2A";
    let senior_jumps =
        "1A 2T 2S 2Lo 2F 2Lz 2A 3T 3S 3Lo 3F 3Lz 3A 4T 4S 4Lo 4F 4Lz 4A 5T 5S 5Lo 5F 5Lz";
    let cases = [
        (
            &adult,
            adult_profile(AdultCategory::Bronze),
            bronze_spins,
            "B1",
            "",
            "ChSq1",
        ),
        (
            &adult,
            adult_profile(AdultCategory::Silver),
            silver_spins,
            "B12",
            "1A",
            "ChSq1",
        ),
        (
            &adult,
            adult_profile(AdultCategory::Gold),
            gold_spins,
            "B123",
            "1A 2T 2S 2Lo",
            "StSqB StSq1 StSq2 ChSp1",
        ),
        (
            &adult,
            adult_profile(AdultCategory::Masters),
            masters_spins,
            "B1234",
            doubles,
            "StSqB StSq1 StSq2 StSq3 StSq4 ChSp1",
        ),
        (
            &senior,
            senior_profile(SeniorCategory::Women),
            senior_spins,
            "B1234",
            senior_jumps,
            "StSqB StSq1 StSq2 StSq3 StSq4 ChSp1 ChSq1",
        ),
        (
            &senior,
            senior_profile(SeniorCategory::Men),
            senior_spins,
            "B1234",
            senior_jumps,
            "StSqB StSq1 StSq2 StSq3 StSq4 ChSp1 ChSq1",
        ),
    ];
    let inventory = include_str!("fixtures/isu-2786-2026-06-02-source-codes.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.rsplit('|').next())
        .flat_map(str::split_whitespace);
    for (registry, profile, spins, levels, jumps, movement) in cases {
        let mut expected: BTreeSet<_> = singles
            .split_whitespace()
            .chain(jumps.split_whitespace())
            .chain(movement.split_whitespace())
            .map(str::to_owned)
            .collect();
        for stem in spins.split_whitespace() {
            expected.extend(levels.chars().map(|level| format!("{stem}{level}")));
            if ["CoSp", "CCoSp", "FCoSp", "FCCoSp"].contains(&stem) {
                expected.extend(levels.chars().map(|level| format!("{stem}{level}V")));
            }
        }
        let listed = selectable_elements(registry, &profile.id, TABLE).unwrap();
        let actual: BTreeSet<_> = listed
            .iter()
            .map(|item| item.row.notation.to_owned())
            .collect();
        assert_eq!(actual, expected, "{:?}", profile.id);
        assert_eq!(listed.len(), actual.len());
        for code in inventory.clone() {
            let direct = check_selectable_element(registry, &profile.id, TABLE, code);
            assert_eq!(
                direct.is_ok(),
                actual.contains(code),
                "{:?} {code}",
                profile.id
            );
        }
    }
}

#[test]
fn manual_base_at_each_level_boundary_uses_the_exact_selection_source() {
    let profile = senior_profile(SeniorCategory::Women);
    let registry = ProfileRegistry::senior();
    let plan = ProfileProgram {
        profile: profile.id,
        scoring_table: TABLE,
        elements: vec![PlannedElement::RegularSpin(RegularSpin {
            entry: None,
            steps: None,
        })],
    };
    // ISU 2786 (2 June 2026), p. 9, BASE column and value rows 1–5.
    for (i, (code, base)) in [
        ("CCoSpB", 200),
        ("CCoSp1", 240),
        ("CCoSp2", 300),
        ("CCoSp3", 360),
        ("CCoSp4", 420),
    ]
    .into_iter()
    .enumerate()
    {
        let selected = check_selectable_element(&registry, &plan.profile, TABLE, code).unwrap();
        let assessed = assess_manual_program_with_assumed_recognition(
            &registry,
            &plan,
            &[AssumedRecognition::new(code)],
            &senior_program_policy(SeniorCategory::Women),
            SubmissionState::Draft,
        )
        .unwrap();
        let value = &assessed.unfactored_base[0];
        assert_eq!(value.hundredths, base);
        assert_eq!(value.notation, code);
        assert_eq!(value.source, selected.row.source.reference(code));
        assert_eq!(
            (selected.row.source.page, selected.row.source.row),
            (9, i as u8 + 1)
        );
    }
}

#[test]
fn explicit_policy_supports_registered_profiles_without_overriding_builtins() {
    let mut registry = ProfileRegistry::adult();
    let mut profile = adult_profile(AdultCategory::Silver);
    let builtin = profile.id.clone();
    profile.id.name = "test-custom-singles-free".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile.category = "Test custom singles".into();
    let id = profile.id.clone();
    registry.register(profile.clone()).unwrap();
    let mut policy = adult_program_policy(AdultCategory::Silver);
    policy.spins.profile = id.clone();
    policy.spins.max_level = Level::Three;

    assert!(selectable_elements(&registry, &id, TABLE).is_err());
    let selected =
        check_selectable_element_with_policy(&registry, &id, TABLE, "CCoSp3", &policy).unwrap();
    assert!(
        selectable_elements_with_policy(&registry, &id, TABLE, &policy)
            .unwrap()
            .contains(&selected)
    );
    assert!(matches!(
        check_selectable_element_with_policy(&registry, &id, TABLE, "CCoSp4", &policy),
        Err(ElementSelectionError::LevelNotSelectable(_))
    ));
    let plan = ProfileProgram {
        profile: id.clone(),
        scoring_table: TABLE,
        elements: vec![PlannedElement::RegularSpin(RegularSpin {
            entry: None,
            steps: None,
        })],
    };
    let assessment = assess_manual_program_with_assumed_recognition(
        &registry,
        &plan,
        &[AssumedRecognition::new("CCoSp3")],
        &policy,
        SubmissionState::Draft,
    )
    .unwrap();
    assert_eq!(
        assessment.unfactored_base[0].source,
        selected.row.source.reference("CCoSp3")
    );

    policy.spins.profile = builtin.clone();
    assert!(selectable_elements_with_policy(&registry, &builtin, TABLE, &policy).is_err());
    assert!(selectable_elements_with_policy(&registry, &id, TABLE, &policy).is_err());

    profile.id.name = "test-custom-singles-short".into();
    profile.segment = ProgramSegment::ShortProgram;
    registry.register(profile.clone()).unwrap();
    policy.spins.profile = profile.id.clone();
    assert!(selectable_elements_with_policy(&registry, &profile.id, TABLE, &policy).is_err());
    profile.id.name = "test-unsupported-official-free".into();
    profile.segment = ProgramSegment::FreeSkating;
    profile.provenance = ProfileProvenance::Official;
    profile.bindings[0].mechanism = RuleMechanism::Unsupported("unimplemented".into());
    registry.register(profile.clone()).unwrap();
    policy.spins.profile = profile.id.clone();
    assert!(matches!(
        selectable_elements_with_policy(&registry, &profile.id, TABLE, &policy),
        Err(ProfileSelectionError {
            kind: ProfileErrorKind::Unsupported,
            ..
        })
    ));
    let mut unknown = id;
    unknown.version = "unknown".into();
    assert!(
        check_selectable_element_with_policy(&registry, &unknown, TABLE, "CCoSp3", &policy)
            .is_err()
    );
}
