#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::cell::Cell;

struct Control(Cell<usize>);
impl PlanningSearchControl for Control {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if phase != PlanningSearchPhase::Alternatives {
            return false;
        }
        let remaining = self.0.get();
        self.0.set(remaining.saturating_sub(1));
        remaining == 0
    }
}
fn request(category: SeniorCategory) -> PlanningSearchRequest {
    let profile = senior_profile(category);
    PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![],
    }
}
fn maximum(request: &PlanningSearchRequest, budget: usize) -> PlanningMaximum {
    let PlanningSearchOutcome::ProvenMaximum(result) = search_maximum_bv(
        &ProfileRegistry::senior(),
        request,
        &Control(Cell::new(budget)),
    )
    .unwrap() else {
        panic!("expected a proved maximum");
    };
    result
}
fn restricted(category: SeniorCategory) -> PlanningSearchRequest {
    let mut request = request(category);
    request
        .personal_limits
        .jump_rotations
        .push(JumpRotationLimit {
            kind: JumpKind::Axel,
            maximum: JumpRotationClass::Single,
        });
    request.exclusions = [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
    ]
    .into_iter()
    .map(PlanningExclusion::JumpKind)
    .collect();
    request
}

#[test]
fn both_senior_profiles_retain_the_maximum_and_five_result_contract() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let result = maximum(&request(category), usize::MAX);
        assert_eq!(result.maximum_base_hundredths, 13310);
        assert_eq!(result.alternatives.len(), 5);
        assert_eq!(
            result.alternative_status,
            PlanningAlternativeStatus::LimitReached
        );
    }
}

#[test]
fn component_presence_does_not_force_a_solo_repetition_loss() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let result = maximum(&restricted(category), usize::MAX);
        // Fixture values: 5Lz=1400, 4Lz=1150, 1A=110. Three per kind.
        // One 5Lz + two 4Lz (one in a sequence) + three 1A = 4030.
        // Two 5Lz + one 4Lz + three 1A have credited 4000 despite raw 4280.
        // Shared optimal nonjump choices contribute 1910.
        assert_eq!(result.maximum_base_hundredths, 5940);
        for candidate in &result.alternatives {
            let components: Vec<_> = candidate
                .elements
                .iter()
                .flat_map(|a| a.code.split('+'))
                .collect();
            assert_eq!(components.iter().filter(|j| **j == "5Lz").count(), 1);
            assert_eq!(components.iter().filter(|j| **j == "4Lz").count(), 2);
            assert!(candidate
                .assessment
                .legality
                .validation
                .validation
                .findings
                .is_empty());
        }
    }
}

#[test]
fn repeated_quints_keep_the_mandatory_eighty_percent_solo_credit() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let mut request = restricted(category);
        request.exclusions.extend((1..=4).map(|rotations| {
            PlanningExclusion::ExactJump(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(rotations),
            })
        }));
        let result = maximum(&request, usize::MAX);
        // 1400+1120+3*110+1910, not raw 2*1400+3*110+1910.
        assert_eq!(result.maximum_base_hundredths, 4760);
        for candidate in &result.alternatives {
            let credits: Vec<_> = candidate
                .elements
                .iter()
                .zip(&candidate.assessment.elements)
                .filter(|(a, _)| a.code == "5Lz")
                .map(|(_, a)| match &a.credited_base {
                    Ok(BaseValueOutcome::Value(v)) => v.hundredths,
                    _ => panic!("expected exact credited value"),
                })
                .collect();
            assert_eq!(credits, [1400, 1120]);
        }
    }
}

#[test]
fn cancellation_after_proof_keeps_a_valid_senior_representative() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let result = maximum(&request(category), 3);
        assert_eq!(result.maximum_base_hundredths, 13310);
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(
            result.alternative_status,
            PlanningAlternativeStatus::Cancelled
        );
        assert!(result.alternatives[0]
            .assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty());
    }
}
