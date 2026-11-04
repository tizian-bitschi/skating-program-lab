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
            for scale in [
                domains::Scale::Small,
                domains::Scale::Medium,
                domains::Scale::Large,
            ] {
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
    println!("{}", BenchmarkRow::HEADER);
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
        for scale in [
            domains::Scale::Small,
            domains::Scale::Medium,
            domains::Scale::Large,
        ] {
            let start = Instant::now();
            let generated =
                generate_program_candidates(&registry, &domains::generation(source, scale))
                    .unwrap();
            let generation_us = start.elapsed().as_micros();
            let slots = if scale == domains::Scale::Small {
                6
            } else if scale == domains::Scale::Medium {
                10
            } else {
                11
            };
            let mut full_variants = generated.variants.clone();
            if scale == domains::Scale::Medium {
                let anchored = domains::completion(source, scale, generated.clone());
                full_variants.extend(anchored.original.iter().map(|a| ProgramVariant {
                    element: a.entry.planned.clone(),
                    evidence: a.entry.evidence.clone(),
                    recognition_assumed: true,
                }));
            }
            let mut q = if scale == domains::Scale::Small {
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
            BenchmarkRow {
                source,
                scale,
                path: "full",
                budget_ms,
                slots,
                generated: &generated,
                solver_variants: q.variants.len(),
                addition_exclusions: 0,
                preparation_us,
                call_us: elapsed,
                proposal_cap: q.max_proposals,
                proposals: result.examined_programs,
                status: result.status,
                enumeration_complete: result.enumeration_complete,
                ties: result.candidates.len(),
                tie_cap: q.max_solutions,
                best_bv: result.candidates.first().map(|c| c.base_hundredths),
                termination: result.termination_reason,
            }
            .print();
            let start = Instant::now();
            let mut q = domains::completion(source, scale, generated);
            let preparation_us = generation_us + start.elapsed().as_micros();
            q.time_limit = Some(Duration::from_millis(budget_ms));
            if scale == domains::Scale::Small {
                q.max_proposals = 10_000;
            }
            let slots = q.original.len() + q.max_additions;
            let start = Instant::now();
            let result = complete_program(&registry, &q).unwrap();
            let elapsed = start.elapsed().as_micros();
            for c in &result.candidates {
                domains::verify_originals(&q, c);
            }
            BenchmarkRow {
                source,
                scale,
                path: "completion",
                budget_ms,
                slots,
                generated: q.generated_domain.as_ref().unwrap(),
                solver_variants: q.variants.len() + q.original.len(),
                addition_exclusions: q.exclusions.len(),
                preparation_us,
                call_us: elapsed,
                proposal_cap: q.max_proposals,
                proposals: result.examined_programs,
                status: result.status,
                enumeration_complete: result.enumeration_complete,
                ties: result.candidates.len(),
                tie_cap: q.max_solutions,
                best_bv: result.candidates.first().map(|c| c.total_base_hundredths),
                termination: result.termination_reason,
            }
            .print();
        }
    }
}

#[cfg(not(feature = "optimizer"))]
fn main() {
    eprintln!("senior_grand_prix requires the optimizer feature");
}

#[cfg(feature = "optimizer")]
struct BenchmarkRow<'a> {
    source: &'a str,
    scale: domains::Scale,
    path: &'a str,
    budget_ms: u64,
    slots: usize,
    generated: &'a skating_program_lab::CandidateGenerationResult,
    solver_variants: usize,
    addition_exclusions: usize,
    preparation_us: u128,
    call_us: u128,
    proposal_cap: usize,
    proposals: usize,
    status: skating_program_lab::OptimizerStatus,
    enumeration_complete: bool,
    ties: usize,
    tie_cap: usize,
    best_bv: Option<u32>,
    termination: skating_program_lab::JumpTerminationReason,
}

#[cfg(feature = "optimizer")]
impl BenchmarkRow<'_> {
    const HEADER: &'static str = "source,scale,path,budget_ms,slots,generated_variants,omissions,domain_complete,solver_variants,addition_exclusions,preparation_us,call_us,proposal_cap,proposals,status,enumeration_complete,ties,tie_cap,best_bv,proven_upper_bound,first_valid_us,optimum_us,memory_bytes,internal_nodes,termination";
    fn print(&self) {
        println!("{}", self.csv());
    }
    fn csv(&self) -> String {
        let bv = self.best_bv.map(|v| v.to_string()).unwrap_or("NA".into());
        let upper = if self.status == skating_program_lab::OptimizerStatus::OptimalProven {
            bv.as_str()
        } else {
            "NA"
        };
        format!(
            "{},{},{},{},{},{},{},{:?},{},{},{},{},{},{},{:?},{},{},{},{},{},NA,NA,NA,NA,{:?}",
            self.source,
            self.scale,
            self.path,
            self.budget_ms,
            self.slots,
            self.generated.variants.len(),
            self.generated.audit.omissions.len(),
            self.generated.audit.completeness,
            self.solver_variants,
            self.addition_exclusions,
            self.preparation_us,
            self.call_us,
            self.proposal_cap,
            self.proposals,
            self.status,
            self.enumeration_complete,
            self.ties,
            self.tie_cap,
            bv,
            upper,
            self.termination
        )
    }
}

#[cfg(all(test, feature = "optimizer"))]
mod tests {
    use super::*;
    use skating_program_lab::*;
    #[test]
    fn csv_columns_and_proof_bounds_match_the_shared_schema() {
        let generated = generate_program_candidates(
            &ProfileRegistry::senior(),
            &domains::generation("gpcan2026-W", domains::Scale::Small),
        )
        .unwrap();
        let mut row = BenchmarkRow {
            source: "gpcan2026-W",
            scale: domains::Scale::Small,
            path: "full",
            budget_ms: 5,
            slots: 6,
            generated: &generated,
            solver_variants: 7,
            addition_exclusions: 0,
            preparation_us: 11,
            call_us: 22,
            proposal_cap: 10000,
            proposals: 720,
            status: OptimizerStatus::OptimalProven,
            enumeration_complete: true,
            ties: 720,
            tie_cap: 1000,
            best_bv: Some(2050),
            termination: JumpTerminationReason::EnumerationExhausted,
        };
        assert_eq!(row.csv(), "gpcan2026-W,small,full,5,6,7,0,CompleteDeclaredDomain,7,0,11,22,10000,720,OptimalProven,true,720,1000,2050,2050,NA,NA,NA,NA,EnumerationExhausted");
        row.status = OptimizerStatus::FeasibleUnproven;
        let csv = row.csv();
        let cells: Vec<_> = csv.split(',').collect();
        assert_eq!(cells.len(), BenchmarkRow::HEADER.split(',').count());
        assert_eq!(&cells[18..20], &["2050", "NA"]);
        row.status = OptimizerStatus::Incomplete;
        row.best_bv = None;
        let csv = row.csv();
        assert_eq!(csv.split(',').nth(18), Some("NA"));
        assert_eq!(csv.split(',').nth(19), Some("NA"));
    }
}
