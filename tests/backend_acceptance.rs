#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::collections::BTreeSet;

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[test]
fn independent_manual_mutations_reject_source_construction_rules() {
    // A pp8–11 / R Rule612 pp70–72: independent legal inputs followed by
    // single-family mutations. No solver result supplies these manual plans.
    for (registry, profile, base, roles) in [
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Bronze),
            vec!["1Lz", "SSp1", "ChSq1"],
            vec![],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Silver),
            vec!["1A", "CLSp2", "ChSp1", "ChSq1"],
            vec!["CLSp2"],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Gold),
            vec!["1A", "CCoSp3", "FSSp3", "ChSp1", "StSq2"],
            vec!["CCoSp3", "FSSp3", "ChSp1"],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Masters),
            vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4"],
            vec!["1A", "CoSp4", "FSSp4", "ChSp1"],
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
            vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4", "ChSq1"],
            vec!["1A", "CoSp4", "FSSp4", "ChSp1"],
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Men),
            vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4", "ChSq1"],
            vec!["1A", "CoSp4", "FSSp4", "ChSp1"],
        ),
    ] {
        let assess = |codes: &[&str]| {
            assess_manual_bv(
                &registry,
                &ManualBvRequest {
                    profile: profile.id.clone(),
                    scoring_table: profile.scoring_table,
                    personal_limits: PersonalLimits::default(),
                    rotation_preference: RotationPreference::default(),
                    submission: SubmissionState::Complete,
                    elements: codes.iter().map(|c| AssumedRecognition::new(*c)).collect(),
                },
            )
            .unwrap()
        };
        assert_eq!(
            assess(&base).planning_compliance(),
            Ok(PlanningCompliance::Compliant)
        );
        let mut mutations = Vec::new();
        for remove in roles {
            mutations.push(
                base.iter()
                    .copied()
                    .filter(|c| *c != remove)
                    .collect::<Vec<_>>(),
            );
        }
        for additions in [
            vec!["1Lz+1Lo+1Lo+1Lo"],             // listed length>3 (Bronze>2)
            vec!["1Lz+1Lo", "1F+1Lo", "1S+1Lo"], // compound maximum
            vec!["1T", "1T", "1T", "1T"],        // inherited all-singles kind cap
            vec!["SSpB", "SSpB"],                // exact abbreviation repetition
            vec!["ChSp1", "ChSp1"],              // choreography/spin cap
            vec!["StSqB", "StSqB"],              // step family eligibility or count
        ] {
            let mut codes = base.clone();
            codes.extend(additions);
            mutations.push(codes);
        }
        for codes in mutations {
            let a = assess(&codes);
            assert_eq!(
                a.planning_compliance(),
                Ok(PlanningCompliance::Noncompliant),
                "{}: {codes:?}",
                profile.id.name
            );
            assert_eq!(a.planning_rule_inventory().count(), 8);
            assert!(a
                .legality
                .validation
                .validation
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::Violation
                    && !f.source.document.is_empty()
                    && !f.source.version.is_empty()
                    && !f.source.location.is_empty()));
        }
    }
}

#[test]
fn six_category_four_workflow_matrix_has_independent_credited_bounds() {
    // Adult announcement July16 pp8–11, Rule612 pp70–72 and SOV2786 June2.
    // Silver: jumps490 + CLSp2 290 + ChSp350 + ChSq350; CSSp2 costs10.
    // Gold ToeLoop exclusion loses140. Masters required1A loses60.
    // Senior required1A loses1290 relative to the nine-component advanced bound.
    for (registry, profile, required, expected) in [
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Bronze),
            vec!["1F"],
            [1100, 1100, 1100, 1100],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Silver),
            vec!["1F+1Lo", "CSSp2"],
            [1480, 1470, 1480, 1470],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Gold),
            vec!["1A"],
            [2420, 2420, 2280, 2280],
        ),
        (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Masters),
            vec!["1A"],
            [3340, 3280, 3340, 3280],
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
            vec!["1A"],
            [13310, 12020, 13310, 12020],
        ),
        (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Men),
            vec!["1A"],
            [13310, 12020, 13310, 12020],
        ),
    ] {
        for (case, bound) in expected.into_iter().enumerate() {
            let request = PlanningSearchRequest {
                profile: profile.id.clone(),
                scoring_table: profile.scoring_table,
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::default(),
                required: if case % 2 == 1 {
                    required
                        .iter()
                        .enumerate()
                        .map(|(i, c)| RequiredPlanningElement {
                            id: format!("required-{i}"),
                            element: AssumedRecognition::new(*c),
                        })
                        .collect()
                } else {
                    vec![]
                },
                exclusions: if case >= 2 {
                    vec![PlanningExclusion::JumpKind(JumpKind::ToeLoop)]
                } else {
                    vec![]
                },
            };
            let PlanningSearchOutcome::ProvenMaximum(maximum) =
                search_maximum_bv(&registry, &request, &Continue).unwrap()
            else {
                panic!("{} case {case}", profile.id.name)
            };
            assert_eq!(
                maximum.maximum_base_hundredths, bound,
                "{} case {case}",
                profile.id.name
            );
            let mut identities = BTreeSet::new();
            for candidate in maximum.alternatives {
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
                if case >= 2 {
                    assert!(candidate
                        .elements
                        .iter()
                        .all(|e| !e.code.split('+').any(|c| c.ends_with('T'))));
                }
                let manual = assess_manual_bv(
                    &registry,
                    &ManualBvRequest {
                        profile: profile.id.clone(),
                        scoring_table: profile.scoring_table,
                        personal_limits: PersonalLimits::default(),
                        rotation_preference: RotationPreference::default(),
                        submission: SubmissionState::Complete,
                        elements: candidate.elements,
                    },
                )
                .unwrap();
                assert_eq!(manual.total.exact_hundredths, Some(i64::from(bound)));
                assert_eq!(
                    manual.planning_compliance(),
                    Ok(PlanningCompliance::Compliant)
                );
                let inventory: Vec<_> = manual.planning_rule_inventory().collect();
                assert_eq!(inventory.len(), 8);
                assert!(inventory
                    .iter()
                    .all(|r| r.state == RuleCoverageState::Evaluated
                        && !r.source.document.is_empty()
                        && !r.source.version.is_empty()
                        && !r.source.location.is_empty()));
            }
        }
    }
}
