#![cfg(feature = "optimizer")]
#[path = "support/senior_benchmarks.rs"]
mod domains;
use skating_program_lab::*;
use std::{collections::BTreeSet, time::Duration};

#[path = "support/protocol_recognition.rs"]
mod protocol_recognition;

#[test]
fn source_order_eleven_slot_conditional_plan_is_completed_and_rescored() {
    let registry = ProfileRegistry::senior();
    let source = "gpfra2026-W";
    let domain = generate_program_candidates(
        &registry,
        &domains::generation(source, domains::Scale::Medium),
    )
    .unwrap();
    let mut request = domains::completion(source, domains::Scale::Medium, domain);
    let anchors = request.original.clone();
    let rows = domains::source_rows(source);
    let edition = domains::records(domains::EDITIONS)
        .into_iter()
        .find(|r| r["source"] == source)
        .unwrap();
    request.original = rows
        .iter()
        .map(|row| {
            let id = format!("{source}:rank1:row{}", row["row"]);
            if !row["code"].starts_with(char::is_numeric) {
                return anchors.iter().find(|a| a.id == id).unwrap().clone();
            }
            // The source supplies recognition/order; clean plans and direct steps
            // are explicit hypothetical assumptions, not inferred movement evidence.
            let code = row["code"].trim_end_matches('!');
            let parts: Vec<_> = code
                .split('+')
                .filter(|s| *s != "SEQ")
                .map(|s| JumpPart::Jump(domains::solo(s).unwrap()))
                .collect();
            let jump = if code.ends_with("+SEQ") {
                JumpElement::Sequence(parts)
            } else if parts.len() > 1 {
                JumpElement::Combination(parts)
            } else {
                JumpElement::Solo(domains::solo(code).unwrap())
            };
            let mut recognition = protocol_recognition::recognition(code);
            recognition.source = Some(SourceReference {
                document: edition["pdf_url"].into(),
                version: edition["sha256"].into(),
                location: format!(
                    "Ami NAKAI / page 1 rank 1 row {}; conditional clean plan of {}",
                    row["row"], row["code"]
                ),
            });
            CompletionEntry {
                id,
                locked: true,
                entry: ManualProgramEntry {
                    planned: PlannedElement::Jump(jump),
                    evidence: ElementPlanEvidence::Jump {
                        direct_transitions_assumed: code.contains('+'),
                    },
                    intended_recognition: Some(recognition),
                    scenario: None,
                },
            }
        })
        .collect();
    request.variants.clear();
    request.generated_domain = None;
    request.insertion.boundaries.clear();
    request.min_additions = 0;
    request.max_additions = 0;
    let result = complete_program(&registry, &request).unwrap();
    assert_eq!(result.status, OptimizerStatus::OptimalProven);
    assert!(result.enumeration_complete);
    assert_eq!(result.candidates.len(), 1);
    let candidate = &result.candidates[0];
    domains::verify_originals(&request, candidate);
    assert_eq!(candidate.program.elements.len(), 11);
    // Independent June 2 values: jumps 800+1010+530+1150+590+490;
    // Basic roles FSSp 200, CCoSp 200, StSq 160, ChSq 350, ChSp 350.
    assert_eq!(candidate.total_base_hundredths, 5830);
    assert_eq!(candidate.added_base_hundredths, 0);
    assert!(candidate
        .validation
        .validation
        .findings
        .iter()
        .all(|f| f.rule_id == "spin-execution-recognition"));
    assert_eq!(
        candidate.assessment.request.entries,
        request
            .original
            .iter()
            .map(|o| o.entry.clone())
            .collect::<Vec<_>>()
    );
}

// Independent bounded combinatorial oracle: one Axel and each of the five roles
// exactly once. No production scorer, validator, generator or solver in this oracle.
fn permutations(items: &mut [usize], at: usize, out: &mut BTreeSet<Vec<usize>>) {
    if at == items.len() {
        out.insert(items.to_vec());
        return;
    }
    for i in at..items.len() {
        items.swap(i, at);
        permutations(items, at + 1, out);
        items.swap(i, at);
    }
}
fn words(axel: usize) -> BTreeSet<Vec<usize>> {
    let mut out = BTreeSet::new();
    permutations(&mut [axel, 2, 3, 4, 5, 6], 0, &mut out);
    out
}

#[test]
fn independent_small_oracle_checks_all_feasible_words_and_complete_optimal_ties() {
    let registry = ProfileRegistry::senior();
    for source in domains::SOURCES {
        let extra = u32::from(source == "gpfra2026-W") * 10;
        let generated = generate_program_candidates(
            &registry,
            &domains::generation(source, domains::Scale::Small),
        )
        .unwrap();
        assert_eq!(generated.variants.len(), 7);
        assert_eq!(
            generated.variants[0].element,
            PlannedElement::Jump(JumpElement::Solo(domains::solo("2A").unwrap()))
        );
        assert_eq!(
            generated.variants[1].element,
            PlannedElement::Jump(JumpElement::Solo(domains::solo("3A").unwrap()))
        );
        // Communication 2786 independent literal coefficients: 2A,3A,CCoSpB,
        // FCSpB,ChSq1,StSqB,ChSp1. Generator order is asserted through these values.
        let values = [330, 800, 200, 190 + extra, 350, 160, 350];
        let mut q = domains::small_full(source, generated.variants);
        q.max_proposals = 10_000;
        let feasible: BTreeSet<_> = words(0).union(&words(1)).cloned().collect();
        assert_eq!(feasible.len(), 1440);
        // Check every independently feasible word against shared plan validation;
        // optimal enumeration below compares every ordered tie, not just one BV.
        for word in &feasible {
            let p = ProfileProgram {
                profile: q.profile.clone(),
                scoring_table: q.scoring_table,
                elements: word
                    .iter()
                    .map(|i| q.variants[*i].element.clone())
                    .collect(),
            };
            let evidence: Vec<_> = word
                .iter()
                .map(|i| q.variants[*i].evidence.clone())
                .collect();
            let r = validate_supported_program(&registry, &p, &evidence, &q.policy).unwrap();
            assert!(r
                .validation
                .findings
                .iter()
                .all(|f| f.rule_id == "spin-execution-recognition"));
        }
        let r = optimize_program(&registry, &q).unwrap();
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(r.enumeration_complete);
        assert_eq!(
            r.candidates
                .iter()
                .map(|c| c.variant_indices.clone())
                .collect::<BTreeSet<_>>(),
            words(1)
        );
        for c in r.candidates {
            assert_eq!(c.base_hundredths, 2050 + extra);
            assert_eq!(
                c.base_hundredths,
                c.variant_indices.iter().map(|i| values[*i]).sum()
            );
        }
        q.exclusions = vec![PersonalExclusion::Variant(1)];
        let r = optimize_program(&registry, &q).unwrap();
        assert!(r.enumeration_complete);
        assert_eq!(
            r.candidates
                .iter()
                .map(|c| c.variant_indices.clone())
                .collect::<BTreeSet<_>>(),
            words(0)
        );
        assert!(r
            .candidates
            .iter()
            .all(|c| c.base_hundredths == 1580 + extra));
    }
}

#[test]
fn source_identity_order_values_and_conditional_medium_anchors_survive() {
    let registry = ProfileRegistry::senior();
    for source in domains::SOURCES {
        let rows = domains::source_rows(source);
        let p = domains::records(domains::PROGRAMS)
            .into_iter()
            .find(|r| r["source"] == source && r["rank"] == "1")
            .unwrap();
        assert!(!p["name"].is_empty());
        assert_eq!(rows.len(), 11);
        let edition = domains::records(domains::EDITIONS)
            .into_iter()
            .find(|r| r["source"] == source)
            .unwrap();
        assert_eq!(edition["sha256"].len(), 64);
        let d = generate_program_candidates(
            &registry,
            &domains::generation(source, domains::Scale::Medium),
        )
        .unwrap();
        assert_eq!(d.variants.len(), 3);
        let mut q = domains::completion(source, domains::Scale::Medium, d);
        q.max_proposals = 1000;
        let r = complete_program(&registry, &q).unwrap();
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(r.enumeration_complete);
        assert_eq!(r.candidates.len(), 30);
        let existing = match source {
            "gpfra2026-W" => 2650,
            "gpfra2026-M" => 3020,
            "gpcan2026-W" => 2540,
            "gpcan2026-M" => 3170,
            _ => unreachable!(),
        };
        for c in &r.candidates {
            domains::verify_originals(&q, c);
            // Independently: two 2A (330 each), one 2T (130); third 2A forbidden.
            assert_eq!(c.existing_base_hundredths, existing);
            assert_eq!(c.added_base_hundredths, 790);
            assert_eq!(c.total_base_hundredths, existing + 790);
            assert_eq!(c.program.elements.len(), 10);
            assert!(c
                .validation
                .validation
                .findings
                .iter()
                .all(|f| f.rule_id == "spin-execution-recognition"));
            for o in &q.original {
                let recognition = o.entry.intended_recognition.as_ref().unwrap();
                let provenance = recognition.source.as_ref().unwrap();
                assert_eq!(provenance.document, edition["pdf_url"]);
                assert_eq!(provenance.version, edition["sha256"]);
            }
        }
    }
}

#[test]
fn repeated_high_solo_completion_remains_an_explicit_unsupported_contract_case() {
    let registry = ProfileRegistry::senior();
    let source = "gpcan2026-W";
    let d = generate_program_candidates(
        &registry,
        &domains::generation(source, domains::Scale::Small),
    )
    .unwrap();
    let mut q = domains::completion(source, domains::Scale::Small, d);
    let axel = q.variants[1].clone();
    q.original
        .push(domains::anchor("conditional-3A".into(), &axel, None));
    q.variants = vec![axel];
    q.generated_domain = None;
    q.insertion.boundaries = vec![6];
    // Senior assessment credits the second clean solo at 80%; finite search
    // maximizes unadjusted catalog BV. Never report this error as a valid plan.
    assert_eq!(
        complete_program(&registry, &q).unwrap_err(),
        ProgramCompletionError::InconsistentAssessment
    );
}

#[test]
fn completion_oracle_restrictions_caps_negative_and_eleven_slot_deadline() {
    let registry = ProfileRegistry::senior();
    for source in domains::SOURCES {
        let d = generate_program_candidates(
            &registry,
            &domains::generation(source, domains::Scale::Small),
        )
        .unwrap();
        let mut q = domains::completion(source, domains::Scale::Small, d);
        let r = complete_program(&registry, &q).unwrap();
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(r.enumeration_complete);
        assert_eq!(r.candidates.len(), 6);
        let mut positions = BTreeSet::new();
        for c in &r.candidates {
            domains::verify_originals(&q, c);
            assert_eq!(
                c.total_base_hundredths,
                if source == "gpfra2026-W" { 2060 } else { 2050 }
            );
            assert_eq!(c.added_base_hundredths, 800);
            positions.insert(
                c.provenance
                    .iter()
                    .position(|p| {
                        matches!(
                            p,
                            CompletionProvenance::Inserted {
                                variant_index: 1,
                                ..
                            }
                        )
                    })
                    .unwrap(),
            );
        }
        assert_eq!(positions, (0..6).collect());
        q.insertion.boundaries = vec![0, 5];
        q.insertion.absolute_positions = vec![(q.original[0].id.clone(), 0)];
        let r = complete_program(&registry, &q).unwrap();
        assert!(r.enumeration_complete);
        assert_eq!(r.candidates.len(), 1);
        domains::verify_originals(&q, &r.candidates[0]);
        q.insertion.absolute_positions.clear();
        q.max_solutions = 1;
        let r = complete_program(&registry, &q).unwrap();
        assert_eq!(r.status, OptimizerStatus::OptimalProven);
        assert!(!r.enumeration_complete);
        q.max_solutions = 1000;
        q.max_proposals = 1;
        assert!(
            !complete_program(&registry, &q)
                .unwrap()
                .enumeration_complete
        );
        q.max_proposals = 200;
        q.variants
            .retain(|v| matches!(v.element, PlannedElement::Jump(_)));
        q.generated_domain = None;
        q.original.remove(0); // Missing combination role cannot be supplied by Axel additions.
        q.insertion.boundaries = vec![0];
        assert_eq!(
            complete_program(&registry, &q).unwrap().status,
            OptimizerStatus::InfeasibleProven
        );
        let mut g = domains::generation(source, domains::Scale::Large);
        let d = generate_program_candidates(&registry, &g).unwrap();
        assert_eq!(
            d.audit.completeness,
            CandidateDomainCompleteness::CompleteDeclaredDomain
        );
        assert!(!d.audit.omissions.is_empty());
        assert!(d
            .variants
            .iter()
            .all(|v| v.element
                != PlannedElement::Jump(JumpElement::Solo(domains::solo("4Lz").unwrap()))));
        let mut q = domains::completion(source, domains::Scale::Large, d);
        q.time_limit = Some(Duration::ZERO);
        assert_eq!(q.original.len() + q.max_additions, 11);
        let r = complete_program(&registry, &q).unwrap();
        assert_eq!(r.status, OptimizerStatus::Incomplete);
        assert!(!r.enumeration_complete);
        g.max_candidates = 1;
        assert_eq!(
            generate_program_candidates(&registry, &g)
                .unwrap()
                .audit
                .completeness,
            CandidateDomainCompleteness::IncompleteCandidateCap
        );
    }
}
