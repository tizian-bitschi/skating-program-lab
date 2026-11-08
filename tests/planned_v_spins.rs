use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

#[test]
fn all_planned_combination_v_levels_have_the_published_value_and_named_type() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Women).id;
    // ISU 2786 pp. 8–9, reduced combination-spin BASE rows.
    for (stem, values, entry, change_foot) in [
        (
            "CoSp",
            [135, 150, 180, 225, 270],
            SpinEntry::NonFlying,
            false,
        ),
        ("FCoSp", [135, 150, 180, 225, 270], SpinEntry::Flying, false),
        (
            "CCoSp",
            [150, 180, 225, 270, 315],
            SpinEntry::NonFlying,
            true,
        ),
        ("FCCoSp", [150, 180, 225, 270, 315], SpinEntry::Flying, true),
    ] {
        for (level, value) in ["B", "1", "2", "3", "4"].into_iter().zip(values) {
            let code = format!("{stem}{level}V");
            let selected = check_selectable_element(&registry, &profile, TABLE, &code).unwrap();
            assert_eq!(
                selected.kind,
                SelectableElementKind::RegularSpin(SpinAbbreviation {
                    kind: RegularSpinKind::Combination,
                    entry,
                    change_foot,
                })
            );
            let result = assess_manual_bv(
                &registry,
                &ManualBvRequest {
                    personal_limits: PersonalLimits::default(),
                    rotation_preference: RotationPreference::default(),
                    profile: profile.clone(),
                    scoring_table: TABLE,
                    submission: SubmissionState::Draft,
                    elements: vec![AssumedRecognition::new(&code)],
                },
            )
            .unwrap();
            assert_eq!(result.total.exact_hundredths, Some(value));
            let base = result.elements[0].table_base.as_ref().unwrap();
            assert_eq!(base.notation, code);
            assert_eq!(base.source, selected.row.source.reference(&code));
        }
    }
}

#[test]
fn planned_v_spins_keep_published_values_and_spin_roles() {
    let registry = ProfileRegistry::adult();
    let profile = adult_profile(AdultCategory::Gold).id;
    for (code, expected) in [("CCoSp3", 360), ("CCoSp3V", 270)] {
        let result = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::default(),
                profile: profile.clone(),
                scoring_table: TABLE,
                submission: SubmissionState::Complete,
                elements: [code, "FSSp3", "ChSp1"]
                    .map(AssumedRecognition::new)
                    .to_vec(),
            },
        )
        .unwrap();
        // ISU 2786 pp. 7, 9 and 14: FSSp3 3.10, ChSp1 3.50.
        let base = result.elements[0].table_base.as_ref().unwrap();
        assert_eq!((base.notation.as_str(), base.hundredths), (code, expected));
        assert_eq!(
            result.total.exact_hundredths,
            Some(i64::from(expected) + 660)
        );
        assert!(!result
            .legality
            .validation
            .validation
            .findings
            .iter()
            .any(|f| ["spin-required-role", "spin-distinct-abbreviation"].contains(&f.rule_id)));
    }
}

#[test]
fn full_and_v_cannot_disguise_a_repeated_spin_abbreviation() {
    let registry = ProfileRegistry::adult();
    for stem in ["CoSp", "CCoSp", "FCoSp", "FCCoSp"] {
        let result = assess_manual_bv(
            &registry,
            &ManualBvRequest {
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::default(),
                profile: adult_profile(AdultCategory::Silver).id,
                scoring_table: TABLE,
                submission: SubmissionState::Complete,
                elements: [format!("{stem}1"), format!("{stem}2V")]
                    .map(AssumedRecognition::new)
                    .to_vec(),
            },
        )
        .unwrap();
        assert!(
            result
                .legality
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.rule_id == "spin-distinct-abbreviation"
                    && f.kind == FindingKind::Violation),
            "{stem}"
        );
    }
}

#[test]
fn v_choices_report_the_same_type_and_level_restrictions() {
    let registry = ProfileRegistry::adult();
    for code in ["CoSp1V", "CCoSp1V", "FCoSp1V", "FCCoSp1V"] {
        assert!(matches!(check_selectable_element(&registry,
            &adult_profile(AdultCategory::Bronze).id, TABLE, code),
            Err(ElementSelectionError::TypeNotSelectable(f)) if f.rule_id == "spin-eligibility"));
    }
    assert!(matches!(check_selectable_element(&registry,
        &adult_profile(AdultCategory::Silver).id, TABLE, "CCoSp3V"),
        Err(ElementSelectionError::LevelNotSelectable(f)) if f.rule_id == "spin-level-cap"));
}
