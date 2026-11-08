#![cfg(feature = "optimizer")]
use skating_program_lab::*;
struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}
#[test]
fn senior_two_identical_advanced_solos_receive_exact_contextual_credit() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let profile = senior_profile(category);
        let request = PlanningSearchRequest {
            rotation_preference: RotationPreference::AllowMixedDirections,
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: ["3A", "3A", "1A"]
                .into_iter()
                .enumerate()
                .map(|(i, code)| RequiredPlanningElement {
                    id: i.to_string(),
                    element: AssumedRecognition::new(code),
                })
                .collect(),
            exclusions: [
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
                JumpKind::Lutz,
            ]
            .into_iter()
            .map(PlanningExclusion::JumpKind)
            .collect(),
        };
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
        else {
            panic!("expected maximum")
        };
        // Communication 2786: 3A=800, repeated solo=640, 1A=110;
        // CCoSp4+FCSp4+StSq4+ChSq1+ChSp1=1910. Three Axel-kind occurrences exhaust that stock.
        assert_eq!(result.maximum_base_hundredths, 3460);
        assert_eq!(
            result.alternatives[0].assessment.total.exact_hundredths,
            Some(3460)
        );
        let credits: Vec<_> = result.alternatives[0]
            .elements
            .iter()
            .zip(&result.alternatives[0].assessment.elements)
            .filter(|(e, _)| e.code == "3A")
            .map(|(_, e)| match &e.credited_base {
                Ok(BaseValueOutcome::Value(v)) => v.hundredths,
                _ => panic!("expected credit"),
            })
            .collect();
        assert_eq!(credits, [800, 640]);
    }
}

#[test]
fn senior_single_jump_types_can_occur_three_times_and_limits_are_not_fill_targets() {
    let profile = senior_profile(SeniorCategory::Women);
    let mut request = PlanningSearchRequest {
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![],
    };
    request.required = ["1T", "1T", "1T", "1A", "1A", "1A"]
        .into_iter()
        .enumerate()
        .map(|(i, code)| RequiredPlanningElement {
            id: i.to_string(),
            element: AssumedRecognition::new(code),
        })
        .collect();
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
    else {
        panic!("expected maximum")
    };
    // Six required solos fill the jump-element allowance: 3*.40+3*1.10=4.50.
    assert_eq!(result.maximum_base_hundredths, 2360);
    assert_eq!(result.alternatives[0].elements.len(), 11);
}

#[test]
fn senior_advanced_compound_occurrence_prevents_two_solo_reduction() {
    let profile = senior_profile(SeniorCategory::Men);
    let required = ["3A", "1A+3A+SEQ", "1T", "1T", "1T", "1S"];
    let request = PlanningSearchRequest {
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: required
            .into_iter()
            .enumerate()
            .map(|(i, code)| {
                let mut element = AssumedRecognition::new(code);
                element.direct_transitions_assumed = code.ends_with("+SEQ");
                RequiredPlanningElement {
                    id: i.to_string(),
                    element,
                }
            })
            .collect(),
        exclusions: vec![],
    };
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
    else {
        panic!("expected maximum")
    };
    // 800+(110+800)+3*40+40, with no REP discount, plus 1910 non-jumps.
    assert_eq!(result.maximum_base_hundredths, 3780);
}
#[test]
fn senior_empty_complete_domain_matches_literal_component_bound() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let profile = senior_profile(category);
        let request = PlanningSearchRequest {
            rotation_preference: RotationPreference::AllowMixedDirections,
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: vec![],
            exclusions: vec![],
        };
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
        else {
            panic!("expected maximum")
        };
        // Fixture pp.4–5: 4A twice(2500),4Lz(1150),4F(1100),4Lo(1050),
        // four distinct quints(5600). Two compounds permit nine components;
        // quints remain solos. Non-jumps add 1910. No solo repetition loss applies.
        assert_eq!(result.maximum_base_hundredths, 13310);
        assert_eq!(result.alternatives.len(), 5);
        assert_eq!(
            result.alternative_status,
            PlanningAlternativeStatus::LimitReached
        );
        let identities: std::collections::HashSet<Vec<_>> = result
            .alternatives
            .iter()
            .map(|a| {
                a.elements
                    .iter()
                    .map(|e| (&e.code, e.direct_transitions_assumed))
                    .collect()
            })
            .collect();
        assert_eq!(identities.len(), 5);
        assert!(result.alternatives[0]
            .assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty());
    }
}
#[test]
fn senior_maximizes_credited_value_instead_of_the_raw_catalog_sum() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let profile = senior_profile(category);
        let request = PlanningSearchRequest {
            rotation_preference: RotationPreference::AllowMixedDirections,
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: ["5Lz", "1A+1S", "1Lo+1T", "1F", "1Lz"]
                .into_iter()
                .enumerate()
                .map(|(i, code)| RequiredPlanningElement {
                    id: i.to_string(),
                    element: AssumedRecognition::new(code),
                })
                .collect(),
            exclusions: [
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
            ]
            .into_iter()
            .map(|kind| {
                PlanningExclusion::ExactJump(ListedJump {
                    kind: Some(kind),
                    rotations: Some(5),
                })
            })
            .collect(),
        };
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
        else {
            panic!("expected maximum")
        };
        // Required jumps=1750 and non-jumps=1910. Two compounds are already used.
        // One remaining solo: raw 5Lz=1400 beats 4A=1250, but its credited
        // repeat is1120. Thus raw maximum5060 differs from credited maximum4910.
        assert_eq!(result.maximum_base_hundredths, 4910);
        // Two equal-value combination abbreviations × two flying abbreviations.
        assert_eq!(result.alternatives.len(), 4);
        assert_eq!(
            result.alternative_status,
            PlanningAlternativeStatus::Exhausted
        );
        assert!(result.alternatives[0]
            .elements
            .iter()
            .any(|e| e.code == "4A"));
        assert_eq!(
            result.alternatives[0]
                .elements
                .iter()
                .filter(|e| e.code == "5Lz")
                .count(),
            1
        );
    }
}
#[test]
fn senior_preserves_required_v_spin_euler_and_exact_multiplicities() {
    let profile = senior_profile(SeniorCategory::Women);
    let request = PlanningSearchRequest {
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: [
            "1A+Eu+1Lo",
            "1T",
            "1T",
            "1T",
            "1S",
            "1S",
            "CCoSp4V",
            "FCSp4",
            "StSq4",
            "ChSq1",
            "ChSp1",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, code)| RequiredPlanningElement {
            id: i.to_string(),
            element: AssumedRecognition::new(code),
        })
        .collect(),
        exclusions: vec![],
    };
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap()
    else {
        panic!("expected maximum")
    };
    // Jumps360; intended V combination spin315, flying380, steps410, ChSq/ChSp350 each.
    assert_eq!(result.maximum_base_hundredths, 2165);
    let first = &result.alternatives[0];
    assert_eq!(first.elements.len(), request.required.len());
    assert!(first
        .provenance
        .iter()
        .all(|p| matches!(p, PlanningProvenance::Required { .. })));
    assert!(first.elements.iter().any(|e| e.code == "CCoSp4V"));
}

#[test]
fn senior_required_regular_spins_cannot_displace_mandatory_choreographic_spin() {
    let profile = senior_profile(SeniorCategory::Women);
    let request = PlanningSearchRequest {
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: ["CCoSp4", "FCSp4", "FSSp4"]
            .into_iter()
            .enumerate()
            .map(|(i, code)| RequiredPlanningElement {
                id: i.to_string(),
                element: AssumedRecognition::new(code),
            })
            .collect(),
        exclusions: vec![],
    };
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::senior(), &request, &Continue).unwrap(),
        PlanningSearchOutcome::Infeasible { .. }
    ));
}
