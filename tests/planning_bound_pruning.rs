#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::cell::Cell;
use std::collections::BTreeSet;

fn request(category: AdultCategory) -> PlanningSearchRequest {
    let profile = adult_profile(category);
    PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![],
    }
}

struct CollectionBudget(Cell<usize>);
impl PlanningSearchControl for CollectionBudget {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if phase != PlanningSearchPhase::Alternatives {
            return false;
        }
        let remaining = self.0.get();
        self.0.set(remaining.saturating_sub(1));
        remaining == 0
    }
}

#[test]
fn masters_returns_five_optima_before_the_callers_collection_budget_expires() {
    // A caller-controlled cooperative budget avoids wall-clock assertions in CI.
    let control = CollectionBudget(Cell::new(20_000));
    let PlanningSearchOutcome::ProvenMaximum(result) = search_maximum_bv(
        &ProfileRegistry::adult(),
        &request(AdultCategory::Masters),
        &control,
    )
    .unwrap() else {
        panic!("expected a proved maximum");
    };
    assert_eq!(result.maximum_base_hundredths, 3340);
    assert_eq!(result.alternatives.len(), 5);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
}

fn maximum(request: &PlanningSearchRequest, budget: usize) -> PlanningMaximum {
    let PlanningSearchOutcome::ProvenMaximum(result) = search_maximum_bv(
        &ProfileRegistry::adult(),
        request,
        &CollectionBudget(Cell::new(budget)),
    )
    .unwrap() else {
        panic!("expected a proved maximum");
    };
    result
}

fn retain_jumps(request: &mut PlanningSearchRequest, allowed: &[JumpKind]) {
    request.exclusions = [
        JumpKind::Waltz,
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
        JumpKind::Axel,
    ]
    .into_iter()
    .filter(|kind| !allowed.contains(kind))
    .map(PlanningExclusion::JumpKind)
    .collect();
}

#[test]
fn equal_bound_compound_orders_survive_and_exactly_five_still_reports_the_limit() {
    let mut request = request(AdultCategory::Bronze);
    request.rotation_preference = RotationPreference::AllowMixedDirections;
    retain_jumps(&mut request, &[JumpKind::Flip, JumpKind::Loop]);
    let result = maximum(&request, usize::MAX);
    // Source stock: two 1F and two 1Lo, at most four entries and one pair.
    // CCSp1 + CLSp1 + ChSq1 contribute 830; the jump stock contributes 200.
    let expected: BTreeSet<Vec<String>> = [
        vec!["1F", "1F", "1Lo", "1Lo"],
        vec!["1F+1F", "1Lo", "1Lo"],
        vec!["1F", "1F+1Lo", "1Lo"],
        vec!["1F", "1Lo", "1Lo+1F"],
        vec!["1F", "1F", "1Lo+1Lo"],
    ]
    .into_iter()
    .map(|row| row.into_iter().map(str::to_string).collect())
    .collect();
    let actual: BTreeSet<_> = result
        .alternatives
        .iter()
        .map(|p| {
            p.elements
                .iter()
                .filter(|e| e.code.starts_with('1'))
                .map(|e| e.code.clone())
                .collect()
        })
        .collect();
    assert_eq!(result.maximum_base_hundredths, 1030);
    assert_eq!(actual, expected);
    assert_eq!(result.alternatives.len(), 5);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
    assert_eq!(result, maximum(&request, usize::MAX));
}

#[test]
fn fewer_than_five_equal_optima_exhaust_and_exact_required_occurrences_stay_intact() {
    let mut request = request(AdultCategory::Bronze);
    retain_jumps(&mut request, &[JumpKind::ToeLoop]);
    let result = maximum(&request, usize::MAX);
    assert_eq!(result.maximum_base_hundredths, 910);
    assert_eq!(result.alternatives.len(), 2);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    for id in ["first", "second"] {
        request.required.push(RequiredPlanningElement {
            id: id.into(),
            element: AssumedRecognition::new("1T"),
        });
    }
    let result = maximum(&request, usize::MAX);
    assert_eq!(result.maximum_base_hundredths, 910);
    assert_eq!(result.alternatives.len(), 1);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    assert_eq!(
        result.alternatives[0]
            .elements
            .iter()
            .filter(|e| e.code == "1T")
            .count(),
        2
    );
}

#[test]
fn cancellation_in_adult_preprocessing_keeps_the_proved_representative() {
    for category in [AdultCategory::Gold, AdultCategory::Masters] {
        let result = maximum(&request(category), 3);
        assert_eq!(
            result.maximum_base_hundredths,
            if category == AdultCategory::Gold {
                2420
            } else {
                3340
            }
        );
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

#[test]
fn personal_single_jump_limits_use_the_actual_adult_repertoire() {
    for (category, expected) in [(AdultCategory::Gold, 1880), (AdultCategory::Masters, 2100)] {
        let mut request = request(category);
        request.personal_limits.jump_rotations = [
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
            JumpKind::Axel,
        ]
        .into_iter()
        .map(|kind| JumpRotationLimit {
            kind,
            maximum: JumpRotationClass::Single,
        })
        .collect();
        // Two each of 1A/1Lz/1F/1Lo contribute 540. Nonjump maxima remain 1340/1560.
        let result = maximum(&request, usize::MAX);
        assert_eq!(result.maximum_base_hundredths, expected);
        assert_eq!(result.personal_limits, request.personal_limits);
        assert_eq!(result.alternatives.len(), 5);
        for candidate in &result.alternatives {
            assert!(candidate.assessment.personal_conflicts.is_empty());
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
