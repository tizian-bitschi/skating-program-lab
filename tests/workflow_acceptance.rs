#![cfg(feature = "optimizer")]
#[path = "support/workflows.rs"]
mod workflows;
use skating_program_lab::*;
use std::collections::BTreeSet;

// Independent June 2 Communication 2786 rows: 1T=.40, 1S=.40,
// SSpB=1.30 (p.6 value row 16), ChSq1=3.50 (p.14 value row 11).
const VALUES: [u32; 4] = [40, 40, 130, 350];
fn symbol(v: &PlannedElement) -> usize {
    match v {
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::ToeLoop),
            ..
        })) => 0,
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Salchow),
            ..
        })) => 1,
        PlannedElement::RegularSpin(_) => 2,
        PlannedElement::ChoreographicSequence { .. } => 3,
        _ => panic!("outside independent fixture alphabet"),
    }
}
fn oracle(n: usize, valid: impl Fn(&[usize]) -> bool) -> (u32, BTreeSet<Vec<usize>>) {
    let mut best = 0;
    let mut ties = BTreeSet::new();
    for mut code in 0..4usize.pow(n as u32) {
        let w: Vec<_> = (0..n)
            .map(|_| {
                let i = code % 4;
                code /= 4;
                i
            })
            .collect();
        if !valid(&w) {
            continue;
        }
        let bv = w.iter().map(|i| VALUES[*i]).sum();
        if bv > best {
            best = bv;
            ties.clear();
        }
        if bv == best {
            ties.insert(w);
        }
    }
    (best, ties)
}
fn entry(v: &ProgramVariant) -> ManualProgramEntry {
    let (code, level) = match symbol(&v.element) {
        0 => ("1T", None),
        1 => ("1S", None),
        2 => ("SSp", Some(Level::Basic)),
        3 => ("ChSq1", None),
        _ => unreachable!(),
    };
    ManualProgramEntry {
        planned: v.element.clone(),
        evidence: v.evidence.clone(),
        scenario: None,
        intended_recognition: Some(PanelRecognition {
            ruleset: RulesetId::IsuAdult2026_27July16,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: Some(
                RecognitionDetails::new(
                    vec![ComponentCode::try_from(code).unwrap()],
                    level,
                    Some(vec![]),
                )
                .unwrap(),
            ),
        }),
    }
}
fn recheck(
    q: &ProgramOptimizationRequest,
    elements: &[PlannedElement],
    evidence: &[ElementPlanEvidence],
    expected: u32,
) {
    let entries = elements
        .iter()
        .zip(evidence)
        .map(|(e, ev)| {
            entry(&ProgramVariant {
                element: e.clone(),
                evidence: ev.clone(),
                recognition_assumed: true,
            })
        })
        .collect();
    let r = assess_manual_program(
        &ProfileRegistry::adult(),
        &ManualProgramRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile: q.profile.clone(),
            scoring_table: q.scoring_table,
            policy: q.policy.clone(),
            submission: SubmissionState::Complete,
            entries,
            request_hypothetical_total: true,
            second_half_boundary: None,
        },
    )
    .unwrap();
    assert_eq!(r.intended_base.exact_hundredths, Some(i64::from(expected)));
    assert!(r.goe_adjustment.is_none());
    let legality = assess_program_legality(
        &ProfileRegistry::adult(),
        &ProfileProgram {
            profile: q.profile.clone(),
            scoring_table: q.scoring_table,
            elements: elements.to_vec(),
        },
        evidence,
        &q.policy,
        SubmissionState::Complete,
    )
    .unwrap();
    assert_eq!(legality.validation, r.validation);
    assert_eq!(legality.validation.validation.coverage, Coverage::Partial);
    assert!(legality
        .validation
        .validation
        .findings
        .iter()
        .all(|f| f.rule_id == "spin-execution-recognition"));
    assert!(legality
        .rules
        .iter()
        .any(|r| r.state == RuleCoverageState::Unsupported));
    for (i, v) in r.elements.iter().enumerate() {
        let BaseValueOutcome::Value(bv) = v.intended_base.as_ref().unwrap() else {
            panic!()
        };
        assert_eq!(u32::from(bv.hundredths), VALUES[symbol(&elements[i])]);
        assert_eq!(bv.source.document, "ISU Communication 2786");
        assert_eq!(bv.source.version, "2026-06-02");
    }
}
#[test]
fn generated_mixed_optimum_matches_every_independently_enumerated_order_and_rechecks() {
    let g = workflows::domain(AdultCategory::Bronze, false);
    let domain = generate_program_candidates(&ProfileRegistry::adult(), &g).unwrap();
    assert_eq!(
        domain.audit.completeness,
        CandidateDomainCompleteness::CompleteDeclaredDomain
    );
    assert_eq!(
        domain
            .variants
            .iter()
            .map(|v| symbol(&v.element))
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    let q = workflows::optimization(&g, domain.variants, 3);
    let expected = oracle(3, |w| {
        w.iter().filter(|i| **i == 2).count() == 1 && w.iter().filter(|i| **i == 3).count() == 1
    });
    let r = optimize_program(&ProfileRegistry::adult(), &q).unwrap();
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    let actual: BTreeSet<_> = r
        .candidates
        .iter()
        .map(|c| c.program.elements.iter().map(symbol).collect::<Vec<_>>())
        .collect();
    assert_eq!(actual, expected.1);
    assert_eq!(r.candidates.len(), actual.len());
    assert_eq!(expected.0, 520);
    for c in &r.candidates {
        assert_eq!(c.base_hundredths, expected.0);
        recheck(&q, &c.program.elements, &c.evidence, expected.0);
    }
    // An excluded supported alternative cannot change the visible optimum.
    let mut extended = g.clone();
    extended.jump_kinds.push(JumpKind::Lutz);
    extended
        .exclusions
        .push(PersonalExclusion::JumpKind(JumpKind::Lutz));
    let added = generate_program_candidates(&ProfileRegistry::adult(), &extended).unwrap();
    assert_eq!(added.variants, q.variants);
    let mut narrowed = q.clone();
    narrowed
        .exclusions
        .push(PersonalExclusion::JumpKind(JumpKind::Salchow));
    let r = optimize_program(&ProfileRegistry::adult(), &narrowed).unwrap();
    assert!(r.candidates.iter().all(|c| c.base_hundredths <= expected.0));
    assert_eq!(r.candidates.len(), 6);
    let mut capped = q.clone();
    capped.max_solutions = 1;
    let r = optimize_program(&ProfileRegistry::adult(), &capped).unwrap();
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(!r.enumeration_complete);
    assert_eq!(r.candidates.len(), 1);
}
#[test]
fn generated_completion_preserves_two_anchors_and_every_oracle_tie() {
    let g = workflows::domain(AdultCategory::Bronze, false);
    let d = generate_program_candidates(&ProfileRegistry::adult(), &g).unwrap();
    let q = workflows::optimization(&g, d.variants.clone(), 4);
    let request = ProgramCompletionRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        season: q.season,
        profile: q.profile.clone(),
        scoring_table: q.scoring_table,
        policy: q.policy.clone(),
        original: vec![
            CompletionEntry {
                id: "toe".into(),
                entry: entry(&d.variants[0]),
                locked: true,
            },
            CompletionEntry {
                id: "sit".into(),
                entry: entry(&d.variants[2]),
                locked: true,
            },
        ],
        variants: d.variants.clone(),
        generated_domain: Some(d),
        exclusions: vec![],
        global_exclusions: vec![],
        insertion: CompletionInsertionPolicy {
            boundaries: vec![0, 1, 2],
            absolute_positions: vec![],
        },
        min_additions: 2,
        max_additions: 2,
        objective: q.objective,
        max_proposals: 100_000,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    };
    // Original T precedes original SSp. Add one T/S and exactly one ChSq.
    // Repeated T placements remain distinct by anchor provenance.
    let expected = oracle(4, |w| {
        w.iter().filter(|i| **i == 2).count() == 1
            && w.iter().filter(|i| **i == 3).count() == 1
            && w.iter()
                .position(|i| *i == 0)
                .is_some_and(|t| t < w.iter().position(|i| *i == 2).unwrap())
    });
    let r = complete_program(&ProfileRegistry::adult(), &request).unwrap();
    assert_eq!(r.status, OptimizerStatus::OptimalProven);
    assert!(r.enumeration_complete);
    assert_eq!(
        r.domain_completeness,
        Some(CandidateDomainCompleteness::CompleteDeclaredDomain)
    );
    let actual: BTreeSet<_> = r
        .candidates
        .iter()
        .map(|c| {
            c.assessment
                .request
                .entries
                .iter()
                .map(|e| symbol(&e.planned))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(actual, expected.1);
    let mut expected_identities = BTreeSet::new();
    for w in &expected.1 {
        let sit = w.iter().position(|i| *i == 2).unwrap();
        for toe in 0..sit {
            if w[toe] != 0 {
                continue;
            }
            expected_identities.insert(
                w.iter()
                    .enumerate()
                    .map(|(i, s)| {
                        (
                            *s,
                            if i == toe {
                                Some(0)
                            } else if i == sit {
                                Some(1)
                            } else {
                                None
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    let mut identities = BTreeSet::new();
    for c in &r.candidates {
        assert_eq!(c.total_base_hundredths, expected.0);
        let identity: Vec<_> = c
            .provenance
            .iter()
            .zip(&c.assessment.request.entries)
            .map(|(p, e)| {
                (
                    symbol(&e.planned),
                    match p {
                        CompletionProvenance::Original { original_index, .. } => {
                            Some(*original_index)
                        }
                        _ => None,
                    },
                )
            })
            .collect();
        assert!(identities.insert(identity));
        let anchors: Vec<_> = c
            .provenance
            .iter()
            .zip(&c.assessment.request.entries)
            .filter_map(|(p, e)| {
                if let CompletionProvenance::Original { original_index, .. } = p {
                    assert_eq!(e, &request.original[*original_index].entry);
                    Some(*original_index)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(anchors, vec![0, 1]);
        let elements: Vec<_> = c
            .assessment
            .request
            .entries
            .iter()
            .map(|e| e.planned.clone())
            .collect();
        let evidence: Vec<_> = c
            .assessment
            .request
            .entries
            .iter()
            .map(|e| e.evidence.clone())
            .collect();
        recheck(&q, &elements, &evidence, expected.0);
    }
    assert_eq!(identities, expected_identities);
}

#[test]
fn finite_bounds_and_identity_errors_never_produce_a_plausible_optimum() {
    let g = workflows::domain(AdultCategory::Bronze, false);
    let d = generate_program_candidates(&ProfileRegistry::adult(), &g).unwrap();
    let q = workflows::optimization(&g, d.variants, 3);
    for (slots, proposals, definitions) in [(11, 100, 4), (3, 100_001, 4), (3, 100, 1001)] {
        let mut invalid = q.clone();
        invalid.max_elements = slots;
        invalid.max_proposals = proposals;
        invalid.variants.resize(definitions, q.variants[0].clone());
        assert!(matches!(
            optimize_program(&ProfileRegistry::adult(), &invalid),
            Err(ProgramOptimizationError::InvalidRequest(_))
        ));
    }
    let mut invalid = q.clone();
    invalid.profile.version = "unknown".into();
    assert!(matches!(
        optimize_program(&ProfileRegistry::adult(), &invalid),
        Err(ProgramOptimizationError::Profile(_))
    ));
    let mut unsupported = g.clone();
    unsupported.discipline = ProfileDiscipline::Pair;
    assert!(generate_program_candidates(&ProfileRegistry::adult(), &unsupported).is_err());
    unsupported = g.clone();
    unsupported.recognition_assumed = false;
    let omitted = generate_program_candidates(&ProfileRegistry::adult(), &unsupported).unwrap();
    assert!(omitted.variants.is_empty());
    assert!(!omitted.audit.omissions.is_empty());
    let mut timed = q.clone();
    timed.time_limit = Some(std::time::Duration::ZERO);
    let r = optimize_program(&ProfileRegistry::adult(), &timed).unwrap();
    assert_eq!(r.status, OptimizerStatus::Incomplete);
    assert!(!r.enumeration_complete);
    assert!(r.candidates.is_empty());
    let mut impossible = q.clone();
    impossible.must_have = vec![vec![2]];
    impossible.exclusions = vec![PersonalExclusion::SpinKind(RegularSpinKind::Sit)];
    let r = optimize_program(&ProfileRegistry::adult(), &impossible).unwrap();
    assert_eq!(r.status, OptimizerStatus::InfeasibleProven);
    assert!(r.candidates.is_empty());
}
