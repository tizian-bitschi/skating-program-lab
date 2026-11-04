#[cfg(feature = "optimizer")]
#[path = "../tests/support/senior_benchmarks.rs"]
mod domains;

#[cfg(feature = "optimizer")]
fn main() {
    use skating_program_lab::*;
    use std::time::{Duration, Instant};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--describe") {
        for source in domains::SOURCES {
            println!(
                "SOURCE {source}\n{:#?}\n{:#?}",
                domains::records(domains::PROGRAMS)
                    .into_iter()
                    .find(|r| r["source"] == source && r["rank"] == "1")
                    .unwrap(),
                domains::source_rows(source)
            );
            for scale in ["small", "medium", "large"] {
                let request = domains::generation(source, scale);
                let generated =
                    generate_program_candidates(&ProfileRegistry::senior(), &request).unwrap();
                println!(
                    "DOMAIN {source} {scale}\n{request:#?}\n{generated:#?}\nCOMPLETION\n{:#?}",
                    domains::completion(source, scale, generated.clone())
                );
            }
        }
        return;
    }
    let budget_ms: u64 = args
        .first()
        .map_or(250, |s| s.parse().expect("budget milliseconds"));
    let registry = ProfileRegistry::senior();
    println!("source,scale,path,budget_ms,slots,generated_variants,omissions,domain_complete,solver_variants,addition_exclusions,preparation_us,call_us,proposal_cap,proposals,status,enumeration_complete,ties,tie_cap,best_bv,proven_upper_bound,first_valid_us,optimum_us,memory_bytes,internal_nodes,termination");
    for source in domains::SOURCES {
        // Reads every unchanged ordered source row and identity before deriving a domain.
        let rows = domains::source_rows(source);
        assert_eq!(rows.len(), 11);
        assert_eq!(
            domains::records(domains::PROGRAMS)
                .iter()
                .filter(|r| r["source"] == source && r["rank"] == "1")
                .count(),
            1
        );
        for scale in ["small", "medium", "large"] {
            let start = Instant::now();
            let generated =
                generate_program_candidates(&registry, &domains::generation(source, scale))
                    .unwrap();
            let generation_us = start.elapsed().as_micros();
            let audit = format!(
                "{},{},{:?}",
                generated.variants.len(),
                generated.audit.omissions.len(),
                generated.audit.completeness
            );
            let slots = if scale == "small" {
                6
            } else if scale == "medium" {
                10
            } else {
                11
            };
            let mut full_variants = generated.variants.clone();
            if scale == "medium" {
                let anchored = domains::completion(source, scale, generated.clone());
                full_variants.extend(anchored.original.iter().map(|a| ProgramVariant {
                    element: a.entry.planned.clone(),
                    evidence: a.entry.evidence.clone(),
                    recognition_assumed: true,
                }));
            }
            let mut q = if scale == "small" {
                domains::small_full(source, full_variants)
            } else {
                domains::full(source, full_variants, slots)
            };
            let preparation_us = start.elapsed().as_micros();
            q.time_limit = Some(Duration::from_millis(budget_ms));
            // Medium full is the same bounded abstraction without immutable anchors.
            let start = Instant::now();
            let result = optimize_program(&registry, &q).unwrap();
            let elapsed = start.elapsed().as_micros();
            let bv = result
                .candidates
                .first()
                .map(|c| c.base_hundredths.to_string())
                .unwrap_or("NA".into());
            let upper = if result.status == OptimizerStatus::OptimalProven {
                bv.clone()
            } else {
                "NA".into()
            };
            println!("{source},{scale},full,{budget_ms},{slots},{audit},{},0,{preparation_us},{elapsed},{},{},{:?},{},{},{},{bv},{upper},NA,NA,NA,NA,{:?}", q.variants.len(), q.max_proposals, result.examined_programs, result.status, result.enumeration_complete, result.candidates.len(), q.max_solutions, result.termination_reason);
            let start = Instant::now();
            let mut q = domains::completion(source, scale, generated);
            let preparation_us = generation_us + start.elapsed().as_micros();
            q.time_limit = Some(Duration::from_millis(budget_ms));
            if scale == "small" {
                q.max_proposals = 10_000;
            }
            let slots = q.original.len() + q.max_additions;
            let start = Instant::now();
            let result = complete_program(&registry, &q).unwrap();
            let elapsed = start.elapsed().as_micros();
            for c in &result.candidates {
                domains::verify_originals(&q, c);
            }
            let bv = result
                .candidates
                .first()
                .map(|c| c.total_base_hundredths.to_string())
                .unwrap_or("NA".into());
            let upper = if result.status == OptimizerStatus::OptimalProven {
                bv.clone()
            } else {
                "NA".into()
            };
            println!("{source},{scale},completion,{budget_ms},{slots},{audit},{},{},{preparation_us},{elapsed},{},{},{:?},{},{},{},{bv},{upper},NA,NA,NA,NA,{:?}", q.variants.len() + q.original.len(), q.exclusions.len(), q.max_proposals, result.examined_programs, result.status, result.enumeration_complete, result.candidates.len(), q.max_solutions, result.termination_reason);
        }
    }
}

#[cfg(not(feature = "optimizer"))]
fn main() {
    eprintln!("senior_grand_prix requires the optimizer feature");
}
