#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::cell::Cell;

struct CancelReconstruction {
    incumbents: Cell<usize>,
}
impl PlanningSearchControl for CancelReconstruction {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if phase == PlanningSearchPhase::IncumbentFound {
            self.incumbents.set(self.incumbents.get() + 1);
        }
        phase == PlanningSearchPhase::Reconstruction
    }
}

#[test]
fn an_incumbent_notification_requires_completed_manual_assessment() {
    let control = CancelReconstruction {
        incumbents: Cell::new(0),
    };
    let request = PlanningSearchRequest {
        rotation_preference: RotationPreference::default(),
        profile: adult_profile(AdultCategory::Bronze).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        required: vec![],
        exclusions: vec![],
    };
    assert_eq!(
        search_maximum_bv(&ProfileRegistry::adult(), &request, &control).unwrap(),
        PlanningSearchOutcome::Cancelled
    );
    assert_eq!(control.incumbents.get(), 0);
}

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[test]
fn six_profiles_preserve_required_occurrences_and_global_component_exclusions() {
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
        let mut request = PlanningSearchRequest {
            rotation_preference: RotationPreference::default(),
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: ["first", "second"]
                .into_iter()
                .map(|id| RequiredPlanningElement {
                    id: id.into(),
                    element: AssumedRecognition::new("1F"),
                })
                .collect(),
            exclusions: vec![PlanningExclusion::JumpKind(JumpKind::ToeLoop)],
        };
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&registry, &request, &Continue).unwrap()
        else {
            panic!("expected maximum for {:?}", request.profile);
        };
        assert!((1..=5).contains(&result.alternatives.len()));
        let mut identities = std::collections::BTreeSet::new();
        for candidate in &result.alternatives {
            let mut identity: Vec<_> = candidate
                .elements
                .iter()
                .map(|e| (e.code.clone(), e.direct_transitions_assumed))
                .collect();
            identity.sort();
            assert!(identities.insert(identity));
            for (input_index, required) in request.required.iter().enumerate() {
                let index = candidate
                    .provenance
                    .iter()
                    .position(|p| {
                        *p == PlanningProvenance::Required {
                            id: required.id.clone(),
                            input_index,
                        }
                    })
                    .unwrap();
                assert_eq!(candidate.elements[index], required.element);
            }
            // Each listed ToeLoop code ends in T. Inspect all compound components.
            assert!(candidate
                .elements
                .iter()
                .all(|e| !e.code.split('+').any(|part| part.ends_with('T'))));
            assert_eq!(
                candidate.assessment.total.exact_hundredths,
                Some(i64::from(result.maximum_base_hundredths))
            );
            assert!(candidate
                .assessment
                .legality
                .validation
                .validation
                .findings
                .is_empty());
            let assessed = assess_manual_bv(
                &registry,
                &ManualBvRequest {
                    rotation_preference: RotationPreference::default(),
                    profile: request.profile.clone(),
                    scoring_table: request.scoring_table,
                    submission: SubmissionState::Complete,
                    elements: candidate.elements.clone(),
                },
            )
            .unwrap();
            assert_eq!(assessed, candidate.assessment);
        }
        request
            .exclusions
            .push(PlanningExclusion::JumpKind(JumpKind::Flip));
        let Err(PlanningSearchError::InvalidRequest { diagnostics, .. }) =
            search_maximum_bv(&registry, &request, &Continue)
        else {
            panic!("required/excluded conflict");
        };
        assert!(diagnostics
            .iter()
            .any(|d| d.required_input_indices.contains(&0)));
        request.exclusions.clear();
        for id in ["third", "fourth"] {
            request.required.push(RequiredPlanningElement {
                id: id.into(),
                element: AssumedRecognition::new("1F"),
            });
        }
        assert!(matches!(
            search_maximum_bv(&registry, &request, &Continue).unwrap(),
            PlanningSearchOutcome::Infeasible { .. }
        ));
    }
}
