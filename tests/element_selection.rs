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
        for code in ["1LiB", "Eu", "1Tq", "CCoSp3V", "CCoSp5", "ChSp2", "ChSq3"] {
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
    for code in ["1LiB", "CCoSp3V"] {
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
