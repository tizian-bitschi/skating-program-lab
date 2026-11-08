use skating_program_lab::*;
use std::collections::BTreeMap;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const ADULT: RulesetId = RulesetId::IsuAdult2026_27July16;

fn draft(profile: RuleProfile, codes: &[&str]) -> ManualBvRequest {
    ManualBvRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: TABLE,
        submission: SubmissionState::Draft,
        elements: codes
            .iter()
            .map(|code| AssumedRecognition::new(*code))
            .collect(),
    }
}

fn score(preview: &GoeScenarioPreview, index: usize) -> &ProgramElementScore {
    preview.elements[index].as_ref().unwrap().as_ref().unwrap()
}

fn scoring_error(error: ElementScoreError) -> GoeScenarioElementError {
    GoeScenarioElementError::Scoring(ProgramScoreError::Element(error))
}

fn preview_for(
    registry: &ProfileRegistry,
    request: &ManualBvRequest,
    grade: Option<i8>,
    overrides: &BTreeMap<usize, GoeScenarioOverride>,
    boundary: Option<usize>,
) -> Result<GoeScenarioPreview, ProfileSelectionError> {
    let plan = GoeScenarioPlan::prepare(registry, request)?;
    preview_goe_scenario(&plan, grade, overrides, boundary)
}

#[test]
fn gold_spin_preview_uses_published_columns_without_changing_the_plan() {
    let registry = ProfileRegistry::adult();
    let program = draft(adult_profile(AdultCategory::Gold), &["CCoSp3", "FSSp2"]);
    let original = program.clone();
    let base = assess_manual_bv(&registry, &program).unwrap();
    let plan = GoeScenarioPlan::prepare(&registry, &program).unwrap();
    let preview = |grade, overrides| preview_goe_scenario(&plan, grade, &overrides, None).unwrap();
    let absent = preview(None, BTreeMap::new());
    assert_eq!(absent.total, None);
    assert_eq!(absent.elements, [Ok(None), Ok(None)]);
    // Communication 2786 (2 June 2026), pp. 7 and 9, published GOE columns.
    for (grade, first, second, total) in [
        (-5, -180, -140, 320),
        (-2, -72, -56, 512),
        (0, 0, 0, 640),
        (2, 72, 56, 768),
        (5, 180, 140, 960),
    ] {
        let result = preview(Some(grade), BTreeMap::new());
        assert_eq!(result.scoring_table, TABLE);
        assert_eq!(result.ruleset, ADULT);
        assert_eq!(score(&result, 0).unfactored.goe_hundredths, first);
        assert_eq!(score(&result, 1).unfactored.goe_hundredths, second);
        assert_eq!(result.total.unwrap().exact_hundredths, Some(total));
    }
    let overrides = BTreeMap::from([(
        1,
        GoeScenarioOverride {
            final_goe: Some(-2),
            ..Default::default()
        },
    )]);
    let mixed = preview(Some(2), overrides.clone());
    assert_eq!(mixed.total.unwrap().exact_hundredths, Some(656));
    let partial = preview(None, overrides).total.unwrap();
    assert_eq!(partial.resolved_hundredths, 224);
    assert_eq!(partial.unresolved_indices, [0]);
    assert_eq!(partial.exact_hundredths, None);
    let invalid = preview(Some(6), BTreeMap::new());
    assert_eq!(
        invalid.elements[0],
        Err(scoring_error(ElementScoreError::InvalidGoe))
    );
    assert_eq!(invalid.total.unwrap().unresolved_indices, [0, 1]);
    assert_eq!(program, original);
    assert_eq!(plan.assessment(), &base);
    assert_eq!(assess_manual_bv(&registry, &program).unwrap(), base);
}

#[test]
fn call_and_fall_constraints_keep_independent_elements_exact() {
    let registry = ProfileRegistry::adult();
    let program = draft(
        adult_profile(AdultCategory::Gold),
        &["1Lz", "FSSp2", "ChSp1"],
    );
    let mut overrides = BTreeMap::from([
        (
            0,
            GoeScenarioOverride {
                recognition: Some(PanelRecognition {
                    ruleset: ADULT,
                    source: None,
                    planned_element: Some(0),
                    observed_element: None,
                    outcome: RecognitionOutcome::Recognized,
                    details: Some(
                        RecognitionDetails::new(
                            vec![ComponentCode::try_from("1Lz").unwrap()],
                            None,
                            Some(vec![RecordedCall {
                                target: CallTarget::Component(0),
                                call: PanelCall::Quarter,
                            }]),
                        )
                        .unwrap(),
                    ),
                }),
                fall: Some(false),
                ..Default::default()
            },
        ),
        (
            1,
            GoeScenarioOverride {
                fall: Some(true),
                ..Default::default()
            },
        ),
    ]);
    let result = preview_for(&registry, &program, Some(2), &overrides, None).unwrap();
    assert_eq!(
        result.elements[0],
        Err(scoring_error(ElementScoreError::UnsupportedMarks))
    );
    assert_eq!(
        result.elements[1],
        Err(scoring_error(ElementScoreError::InconsistentFall))
    );
    // ChSp1 has a published +2 adjustment of 1.00, on BV 3.50.
    assert_eq!(score(&result, 2).unfactored.goe_hundredths, 100);
    assert_eq!(
        result.total.unwrap(),
        AssessmentAggregate {
            resolved_hundredths: 450,
            unresolved_indices: vec![0, 1],
            exact_hundredths: None,
        }
    );
    overrides.get_mut(&1).unwrap().final_goe = Some(-5);
    let result = preview_for(&registry, &program, Some(-2), &overrides, None).unwrap();
    assert_eq!(score(&result, 0).unfactored.base.notation, "1Lzq");
    assert_eq!(score(&result, 0).score_hundredths, 48);
    assert_eq!(result.total.unwrap().exact_hundredths, Some(438));
}

#[test]
fn senior_preview_preserves_repeat_credit_and_assumes_second_half_explicitly() {
    let registry = ProfileRegistry::senior();
    let program = draft(senior_profile(SeniorCategory::Men), &["4T", "4T", "ChSp1"]);
    let result = preview_for(&registry, &program, Some(2), &BTreeMap::new(), Some(1)).unwrap();
    // 4T: BV 9.50, +2 column 1.90. Rule 612: second solo at 80%.
    // An explicitly assumed second-half factor changes 7.60 to 8.36.
    assert_eq!(score(&result, 1).unfactored.base.hundredths, 760);
    assert_eq!(score(&result, 1).unfactored.goe_hundredths, 190);
    assert_eq!(score(&result, 1).credited_base_hundredths, 836);
    assert!(score(&result, 1).second_half);
    assert_eq!(result.total.unwrap().exact_hundredths, Some(2616));
    assert_eq!(
        assess_manual_bv(&registry, &program)
            .unwrap()
            .total
            .exact_hundredths,
        Some(2060)
    );
}

#[test]
fn ungraded_jumps_still_occupy_assumed_second_half_slots() {
    let registry = ProfileRegistry::senior();
    let program = draft(
        senior_profile(SeniorCategory::Men),
        &["2A", "3T", "3S", "3Lo"],
    );
    let overrides = BTreeMap::from([(
        0,
        GoeScenarioOverride {
            final_goe: Some(0),
            ..Default::default()
        },
    )]);
    let result = preview_for(&registry, &program, None, &overrides, Some(0)).unwrap();
    assert!(!score(&result, 0).second_half);
    assert_eq!(
        result.total.unwrap(),
        AssessmentAggregate {
            resolved_hundredths: 330,
            unresolved_indices: vec![1, 2, 3],
            exact_hundredths: None,
        }
    );
}

#[test]
fn unscored_senior_spin_does_not_obscure_jump_repetition_credit() {
    let registry = ProfileRegistry::senior();
    let profile = senior_profile(SeniorCategory::Men);
    let ruleset = profile.scoring.recognition_ruleset;
    let program = draft(profile, &["4T", "CCoSp4", "4T"]);
    for (outcome, error) in [
        (RecognitionOutcome::Ignored, ElementScoreError::Ignored),
        (RecognitionOutcome::NoValue, ElementScoreError::NoValue),
    ] {
        let mut overrides = BTreeMap::from([(
            1,
            GoeScenarioOverride {
                recognition: Some(PanelRecognition {
                    ruleset,
                    source: None,
                    planned_element: Some(1),
                    observed_element: None,
                    outcome,
                    details: None,
                }),
                ..Default::default()
            },
        )]);
        let result = preview_for(&registry, &program, Some(0), &overrides, None).unwrap();
        assert_eq!(result.elements[1], Err(scoring_error(error)));
        assert_eq!(score(&result, 0).credited_base_hundredths, 950);
        assert_eq!(score(&result, 2).credited_base_hundredths, 760);
        assert_eq!(result.total.unwrap().exact_hundredths, Some(1710));

        overrides.get_mut(&1).unwrap().final_goe = Some(6);
        let invalid = preview_for(&registry, &program, Some(0), &overrides, None).unwrap();
        assert_eq!(
            invalid.elements[1],
            Err(scoring_error(ElementScoreError::InvalidGoe))
        );
        assert_eq!(
            invalid.total.unwrap(),
            AssessmentAggregate {
                resolved_hundredths: 1710,
                unresolved_indices: vec![1],
                exact_hundredths: None,
            }
        );
    }
}

#[test]
fn known_zero_outcomes_and_unavailable_selections_stay_distinct() {
    let registry = ProfileRegistry::adult();
    let program = draft(
        adult_profile(AdultCategory::Gold),
        &["1Lz", "FSSp2", "CCoSp4", "?"],
    );
    let outcome = |index, outcome| GoeScenarioOverride {
        recognition: Some(PanelRecognition {
            ruleset: ADULT,
            source: None,
            planned_element: Some(index),
            observed_element: None,
            outcome,
            details: None,
        }),
        ..Default::default()
    };
    let overrides = BTreeMap::from([
        (0, outcome(0, RecognitionOutcome::Ignored)),
        (1, outcome(1, RecognitionOutcome::NoValue)),
    ]);
    let result = preview_for(&registry, &program, Some(0), &overrides, None).unwrap();
    assert_eq!(
        result.elements[0],
        Err(scoring_error(ElementScoreError::Ignored))
    );
    assert_eq!(
        result.elements[1],
        Err(scoring_error(ElementScoreError::NoValue))
    );
    let assessment = assess_manual_bv(&registry, &program).unwrap();
    for index in [2, 3] {
        let error = assessment.elements[index]
            .credited_base
            .as_ref()
            .unwrap_err()
            .clone();
        assert_eq!(
            result.elements[index],
            Err(GoeScenarioElementError::Base(error))
        );
    }
    assert_eq!(result.total.unwrap().unresolved_indices, [2, 3]);
}
