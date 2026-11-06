//! Opt-in instrumentation for the bounded mixed-program search in test builds.

// The standalone benchmark fixtures also compile separately in examples and integration tests.
#![allow(clippy::duplicate_mod)]

use crate::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "../../tests/support/senior_benchmarks.rs"]
mod senior;
#[path = "../../tests/support/workflows.rs"]
mod workflows;

#[derive(Debug, Default)]
struct Diagnostics {
    preflight: Option<Duration>,
    model: Option<Duration>,
    solver: Duration,
    solver_calls: usize,
    check: Duration,
    accepted: usize,
    first_accepted: Option<Duration>,
    rejected: BTreeMap<Rejection, usize>,
    rules: BTreeMap<&'static str, usize>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Rejection {
    ModelRequirements,
    Violation,
    Unresolved,
}

thread_local! {
    static ACTIVE: RefCell<Option<Diagnostics>> = const { RefCell::new(None) };
}

fn record(update: impl FnOnce(&mut Diagnostics)) {
    ACTIVE.with_borrow_mut(|active| {
        if let Some(diagnostics) = active {
            update(diagnostics);
        }
    });
}

pub(super) fn record_preflight(elapsed: Duration) {
    record(|d| d.preflight = Some(elapsed));
}

pub(super) fn record_model(elapsed: Duration) {
    record(|d| d.model = Some(elapsed));
}

pub(super) fn record_solver(elapsed: Duration) {
    record(|d| {
        d.solver += elapsed;
        d.solver_calls += 1;
    });
}

pub(super) fn record_check(elapsed: Duration, since_start: Duration, accepted: bool) {
    record(|d| {
        d.check += elapsed;
        if accepted {
            d.accepted += 1;
            d.first_accepted.get_or_insert(since_start);
        }
    });
}

pub(super) fn record_rejection<'a>(
    category: Rejection,
    findings: impl Iterator<Item = &'a Finding>,
) {
    record(|d| {
        *d.rejected.entry(category).or_default() += 1;
        let rules: BTreeSet<_> = findings.map(|f| f.rule_id).collect();
        for rule in rules {
            *d.rules.entry(rule).or_default() += 1;
        }
    });
}

fn capture<T>(run: impl FnOnce() -> T) -> (T, Diagnostics) {
    ACTIVE.with_borrow_mut(|active| assert!(active.replace(Diagnostics::default()).is_none()));
    let result = run();
    (
        result,
        ACTIVE.with_borrow_mut(|active| active.take().unwrap()),
    )
}

fn sample(
    label: &str,
    run: impl FnOnce() -> (usize, usize, OptimizerStatus, JumpTerminationReason),
) {
    let start = Instant::now();
    let ((proposals, returned, status, termination), d) = capture(run);
    let elapsed = start.elapsed();
    assert_eq!(d.accepted + d.rejected.values().sum::<usize>(), proposals);
    assert!(returned <= d.accepted);
    println!("{label} elapsed={elapsed:?} proposals={proposals} returned={returned} {status:?}/{termination:?} {d:?}");
}

#[test]
fn zero_budget_records_preflight_without_posting_the_model() {
    let mut request = super::tests::bronze_solos();
    request.time_limit = Some(Duration::ZERO);
    let registry = ProfileRegistry::adult();
    let prior_preflight = Duration::from_millis(1);
    let start = Instant::now() - prior_preflight;
    let (result, diagnostics) =
        capture(|| super::optimize_anchored(&registry, &request, start, None, None).unwrap());

    assert_eq!(result.status, OptimizerStatus::Incomplete);
    assert_eq!(
        result.termination_reason,
        JumpTerminationReason::Interrupted
    );
    assert_eq!(result.examined_programs, 0);
    assert!(diagnostics.preflight.unwrap() >= prior_preflight);
    assert_eq!(diagnostics.model, None);
    assert_eq!(diagnostics.solver_calls, 0);
}

#[test]
#[ignore = "diagnostic run with wall-clock search budgets"]
fn source_domain_proposal_inventory() {
    let budget = Some(Duration::from_millis(250));
    let registry = ProfileRegistry::adult();
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let generation = workflows::bounds_domain(category);
        let generated = generate_program_candidates(&registry, &generation).unwrap();
        let mut request = workflows::optimization(&generation, generated.variants, 6);
        request.max_proposals = 200;
        request.max_solutions = 100;
        request.time_limit = budget;
        sample(&format!("Adult {category:?} full"), || {
            let r = optimize_program(&registry, &request).unwrap();
            (
                r.examined_programs,
                r.candidates.len(),
                r.status,
                r.termination_reason,
            )
        });
    }

    let registry = ProfileRegistry::senior();
    for source in senior::SOURCES {
        let scale = senior::Scale::Large;
        let generated =
            generate_program_candidates(&registry, &senior::generation(source, scale)).unwrap();
        let mut full = senior::full(source, generated.variants.clone(), 11);
        full.time_limit = budget;
        sample(&format!("Senior {source} full"), || {
            let r = optimize_program(&registry, &full).unwrap();
            (
                r.examined_programs,
                r.candidates.len(),
                r.status,
                r.termination_reason,
            )
        });
        let mut completion = senior::completion(source, scale, generated);
        completion.time_limit = budget;
        sample(&format!("Senior {source} completion"), || {
            let r = complete_program(&registry, &completion).unwrap();
            (
                r.examined_programs,
                r.candidates.len(),
                r.status,
                r.termination_reason,
            )
        });
    }
}
