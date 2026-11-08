use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;

fn request(profile: ProfileId, codes: &[&str]) -> ManualBvRequest {
    ManualBvRequest {
        profile,
        scoring_table: TABLE,
        submission: SubmissionState::Complete,
        elements: codes
            .iter()
            .map(|code| AssumedRecognition::new(*code))
            .collect(),
    }
}

#[test]
fn selected_codes_form_category_programs_without_performance_evidence() {
    let adults = ProfileRegistry::adult();
    let seniors = ProfileRegistry::senior();
    // Adult announcement pp. 8–11; Senior Rule 612 pp. 70–72.
    // Independent BASE literals: Communication 2786, 2 June 2026, pp. 2–9, 14.
    type CategoryExample<'a> = (
        &'a ProfileRegistry,
        RuleProfile,
        &'a [&'a str],
        &'a [u16],
        i64,
    );
    let cases: &[CategoryExample<'_>] = &[
        (
            &adults,
            adult_profile(AdultCategory::Bronze),
            &["1Lz", "SSpB", "ChSq1"],
            &[60, 130, 350],
            540,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Silver),
            &["1A", "CCoSp2", "SSpB", "ChSq1"],
            &[110, 300, 130, 350],
            890,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Gold),
            &["2Lo", "CCoSp3", "FSSp2", "StSq2", "ChSp1"],
            &[170, 360, 280, 270, 350],
            1430,
        ),
        (
            &adults,
            adult_profile(AdultCategory::Masters),
            &["1A", "CCoSp4", "FSSp2", "StSq4", "ChSp1"],
            &[110, 420, 280, 410, 350],
            1570,
        ),
        (
            &seniors,
            senior_profile(SeniorCategory::Women),
            &["3A", "3A", "CCoSp4", "FSSp2", "StSq4", "ChSq1", "ChSp1"],
            &[800, 640, 420, 280, 410, 350, 350],
            3250,
        ),
        (
            &seniors,
            senior_profile(SeniorCategory::Men),
            &["3A", "3A", "CCoSp4", "FSSp2", "StSq4", "ChSq1", "ChSp1"],
            &[800, 640, 420, 280, 410, 350, 350],
            3250,
        ),
    ];
    for (registry, profile, codes, values, total) in cases {
        assert_eq!(profile.scoring_table, TABLE);
        assert_eq!(registry.resolve(&profile.id, TABLE).unwrap().id, profile.id);
        let choices = selectable_elements(registry, &profile.id, TABLE).unwrap();
        for code in *codes {
            assert!(
                choices.iter().any(|choice| choice.row.notation == *code),
                "{:?}: {code}",
                profile.id
            );
        }
        let assessed = assess_manual_bv(registry, &request(profile.id.clone(), codes)).unwrap();
        assert_eq!(
            assessed.total.exact_hundredths,
            Some(*total),
            "{:?}",
            profile.id
        );
        for (index, expected) in values.iter().enumerate() {
            assert_eq!(assessed.elements[index].element_index, index);
            let Ok(BaseValueOutcome::Value(value)) = &assessed.elements[index].credited_base else {
                panic!("{:?}", assessed.elements[index]);
            };
            assert_eq!((value.table, value.hundredths), (TABLE, *expected));
        }
        assert!(assessed.legality.repairable_absences.is_empty());
        assert!(
            assessed.legality.validation.validation.findings.is_empty(),
            "{:?}: {:?}",
            profile.id,
            assessed.legality.validation.validation.findings
        );
        // A construction report still inventories rules outside its scope.
        assert!(!assessed.legality.rules.is_empty());
    }
}

#[test]
fn unknown_profile_and_table_versions_never_enter_the_planning_workflow() {
    for (registry, profile) in [
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Bronze),
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Silver),
        ),
        (ProfileRegistry::adult(), adult_profile(AdultCategory::Gold)),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Masters),
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Men),
        ),
    ] {
        let mut unknown = profile.id;
        unknown.version = "unresearched-2027".into();
        let selection = selectable_elements(&registry, &unknown, TABLE).unwrap_err();
        let assessment =
            assess_manual_bv(&registry, &request(unknown.clone(), &["1A"])).unwrap_err();
        assert_eq!(selection, assessment);
        assert_eq!(assessment.profile, unknown);
        assert_eq!(assessment.kind, ProfileErrorKind::Unknown);
    }
    assert_eq!(
        "unresearched-sov-2027".parse::<ScoringTableId>(),
        Err(UnknownVersion("unresearched-sov-2027".into()))
    );
}

#[test]
fn draft_omissions_prohibitions_and_unsupported_choices_remain_separate() {
    let registry = ProfileRegistry::adult();
    let mut input = request(adult_profile(AdultCategory::Silver).id, &["1Lz"]);
    input.submission = SubmissionState::Draft;
    let draft = assess_manual_bv(&registry, &input).unwrap();
    assert_eq!(draft.total.exact_hundredths, Some(60));
    assert!(!draft.legality.repairable_absences.is_empty());
    input.submission = SubmissionState::Complete;
    let complete = assess_manual_bv(&registry, &input).unwrap();
    assert!(complete.legality.repairable_absences.is_empty());
    for index in draft.legality.repairable_absences {
        assert!(complete
            .legality
            .validation
            .validation
            .findings
            .contains(&draft.legality.validation.validation.findings[index]));
    }

    // Silver caps regular spins at level 2; pair twist is a published,
    // unsupported singles choice. Neither is an unfinished required role.
    input.elements = ["CCoSp3", "3Tw1", "ChSq1"]
        .iter()
        .map(|code| AssumedRecognition::new(*code))
        .collect();
    let rejected = assess_manual_bv(&registry, &input).unwrap();
    let Err(ManualBvError::Selection {
        error: ElementSelectionError::LevelNotSelectable(finding),
        ..
    }) = &rejected.elements[0].credited_base
    else {
        panic!("{:?}", rejected.elements[0]);
    };
    assert_eq!(finding.element_index, Some(0));
    assert_eq!(finding.kind, FindingKind::Violation);
    assert!(matches!(
        rejected.elements[1].credited_base,
        Err(ManualBvError::Selection {
            error: ElementSelectionError::UnsupportedCode { .. },
            ..
        })
    ));
    assert_eq!(rejected.total.unresolved_indices, [0, 1]);
    assert_eq!(rejected.total.resolved_hundredths, 350);
    assert_eq!(rejected.total.exact_hundredths, None);
}
