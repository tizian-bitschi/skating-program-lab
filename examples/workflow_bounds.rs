#[cfg(feature = "optimizer")]
#[path = "../tests/support/workflows.rs"]
mod workflows;

#[cfg(feature = "optimizer")]
fn main() {
    use skating_program_lab::*;
    use std::time::{Duration, Instant};
    println!("category,variants,slots,solution_cap,generation_us,search_us,proposals,status,enumeration_complete,returned,best_bv,termination");
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let mut g = workflows::domain(category, true);
        g.base_spins.extend([
            SpinAbbreviation {
                kind: RegularSpinKind::Combination,
                entry: SpinEntry::NonFlying,
                change_foot: true,
            },
            SpinAbbreviation {
                kind: RegularSpinKind::Camel,
                entry: SpinEntry::Flying,
                change_foot: false,
            },
        ]);
        let start = Instant::now();
        let generated = generate_program_candidates(&ProfileRegistry::adult(), &g).unwrap();
        let generation_us = start.elapsed().as_micros();
        for cap in [1, 10, 100] {
            let mut q = workflows::optimization(&g, generated.variants.clone(), 6);
            q.max_solutions = cap;
            q.max_proposals = 200;
            q.time_limit = Some(Duration::from_millis(250));
            let start = Instant::now();
            let r = optimize_program(&ProfileRegistry::adult(), &q).unwrap();
            println!(
                "{category:?},{},6,{cap},{generation_us},{},{},{:?},{},{},{},{:?}",
                q.variants.len(),
                start.elapsed().as_micros(),
                r.examined_programs,
                r.status,
                r.enumeration_complete,
                r.candidates.len(),
                r.candidates.first().map(|c| c.base_hundredths).unwrap_or(0),
                r.termination_reason
            );
        }
    }
}
#[cfg(not(feature = "optimizer"))]
fn main() {
    eprintln!("workflow_bounds requires the optimizer feature");
}
