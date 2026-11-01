//! Exhaustive reference for the fixed three-slot Bronze jump domain.
use crate::{
    lookup_base_value, validate_program, AdultCategory, BaseValueError, BaseValueOutcome,
    ComponentCode, Coverage, FindingKind, JumpElement, JumpKind, ListedJump, PanelRecognition,
    PlannedElement, Program, RecognitionDetails, RecognitionOutcome, RulesetId, ScoringTableId,
    Season,
};

/// Fixed assumptions for this exhaustive model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDomain {
    pub slot_start_seconds: [u16; 3],
    pub duration_seconds: u16,
    pub uninterrupted_timing_assumed: bool,
    pub no_event_exception_assumed: bool,
    pub clean_recognition_assumed: bool,
    pub exclude_lutz: bool,
}

/// Slot occupancy corresponds to the result's domain times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub slots: [bool; 3],
    pub program: Program,
    pub base_hundredths: u16,
    pub coverage: Coverage,
}

/// Complete enumeration within the documented domain, not official program feasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceOutcome {
    Optimal {
        base_hundredths: u16,
        candidates: Vec<ReferenceCandidate>,
    },
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceResult {
    pub domain: ReferenceDomain,
    pub outcome: ReferenceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    IncompleteValidation,
    UnsupportedElement,
    BaseValue(BaseValueError),
    InvalidLimit,
}

/// Enumerate every slot assignment. Exclusion removes 1Lz from every slot.
/// Lookup errors abort the search rather than producing an incomplete optimum.
pub fn enumerate_bronze_reference(exclude_lutz: bool) -> Result<ReferenceResult, ReferenceError> {
    let domain = ReferenceDomain {
        slot_start_seconds: [10, 20, 30],
        duration_seconds: 100,
        uninterrupted_timing_assumed: true,
        no_event_exception_assumed: true,
        clean_recognition_assumed: true,
        exclude_lutz,
    };
    let ruleset = RulesetId::IsuAdult2026_27July16;
    let table = ScoringTableId::Isu2786June2_2026;
    let mut best = None;
    let mut candidates = Vec::new();
    for mask in 0..8 {
        let slots = std::array::from_fn(|i| mask & (1 << i) != 0);
        if !slots.iter().any(|&occupied| occupied) || (exclude_lutz && slots.contains(&true)) {
            continue;
        }
        let program = bronze_program(slots);
        let validation = validate_program(&program);
        if validation
            .findings
            .iter()
            .any(|f| f.kind != FindingKind::Violation)
        {
            return Err(ReferenceError::IncompleteValidation);
        }
        if !validation.findings.is_empty() {
            continue;
        }
        let mut total = 0;
        for element in &program.elements {
            total += reference_base_value(element, ruleset, table)?;
        }
        let candidate = ReferenceCandidate {
            slots,
            program,
            base_hundredths: total,
            coverage: validation.coverage,
        };
        match best {
            None => {
                best = Some(total);
                candidates.push(candidate);
            }
            Some(previous) if total > previous => {
                best = Some(total);
                candidates.clear();
                candidates.push(candidate);
            }
            Some(previous) if total == previous => candidates.push(candidate),
            _ => {}
        }
    }
    Ok(ReferenceResult {
        domain,
        outcome: match best {
            Some(base_hundredths) => ReferenceOutcome::Optimal {
                base_hundredths,
                candidates,
            },
            None => ReferenceOutcome::Infeasible,
        },
    })
}

fn bronze_program(slots: [bool; 3]) -> Program {
    let elements = slots
        .iter()
        .filter(|&&occupied| occupied)
        .map(|_| {
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(1),
            }))
        })
        .collect();
    Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: AdultCategory::Bronze,
        ruleset: RulesetId::IsuAdult2026_27July16,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        sources: vec![],
        elements,
    }
}

fn reference_base_value(
    element: &PlannedElement,
    ruleset: RulesetId,
    table: ScoringTableId,
) -> Result<u16, ReferenceError> {
    let code = match element {
        PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Lutz),
            rotations: Some(1),
        })) => "1Lz",
        _ => return Err(ReferenceError::UnsupportedElement),
    };
    let recognition = PanelRecognition {
        ruleset,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(
                vec![ComponentCode::try_from(code).expect("known code")],
                None,
                Some(vec![]),
            )
            .expect("known recognition"),
        ),
    };
    match lookup_base_value(table, &recognition, &[]).map_err(ReferenceError::BaseValue)? {
        BaseValueOutcome::Value(value) => Ok(value.hundredths),
        _ => Err(ReferenceError::BaseValue(BaseValueError::Unsupported)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scoring_rejects_elements_outside_the_domain() {
        for element in [
            PlannedElement::Unspecified,
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Flip),
                rotations: Some(1),
            })),
        ] {
            assert_eq!(
                reference_base_value(
                    &element,
                    RulesetId::IsuAdult2026_27July16,
                    ScoringTableId::Isu2786June2_2026
                ),
                Err(ReferenceError::UnsupportedElement)
            );
        }
    }
}

use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        optimisation::{linear_sat_unsat::LinearSatUnsat, OptimisationDirection},
        predicate,
        results::{
            solution_iterator::IteratedSolution, OptimisationResult, ProblemSolution,
            SolutionReference,
        },
        termination::TerminationCondition,
        variables::TransformableVariable,
        DefaultBrancher,
    },
    Solver,
};
use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

/// The fixed reference domain and a shared time budget for optimization and enumeration.
#[derive(Debug, Clone)]
pub struct OptimizerRequest {
    pub exclude_lutz: bool,
    /// Whether clean 1Lz is a candidate in each of the three ordered slots.
    pub enabled_slots: [bool; 3],
    pub ruleset: RulesetId,
    pub scoring_table: ScoringTableId,
    pub enumerate_equal_optima: bool,
    pub max_solutions: usize,
    pub time_limit: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizerStatus {
    OptimalProven,
    FeasibleUnproven,
    InfeasibleProven,
    Incomplete,
}

#[derive(Debug, Clone)]
pub struct OptimizerResult {
    pub domain: ReferenceDomain,
    pub enabled_slots: [bool; 3],
    pub ruleset: RulesetId,
    pub scoring_table: ScoringTableId,
    pub status: OptimizerStatus,
    pub enumeration_complete: bool,
    pub candidates: Vec<ReferenceCandidate>,
}

struct Deadline(Option<Instant>);
impl TerminationCondition for Deadline {
    fn should_stop(&mut self) -> bool {
        self.0.is_some_and(|deadline| Instant::now() >= deadline)
    }
}

/// Search only the documented reference domain. Unknown validation or BV aborts the request.
pub fn optimize_bronze(request: &OptimizerRequest) -> Result<OptimizerResult, ReferenceError> {
    if request.max_solutions == 0 {
        return Err(ReferenceError::InvalidLimit);
    }
    let reference_domain = ReferenceDomain {
        slot_start_seconds: [10, 20, 30],
        duration_seconds: 100,
        uninterrupted_timing_assumed: true,
        no_event_exception_assumed: true,
        clean_recognition_assumed: true,
        exclude_lutz: request.exclude_lutz,
    };
    let ruleset = request.ruleset;
    let table = request.scoring_table;
    if ruleset != RulesetId::IsuAdult2026_27July16 || table != ScoringTableId::Isu2786June2_2026 {
        return Err(ReferenceError::UnsupportedElement);
    }
    let unit = reference_base_value(
        &bronze_program([true, false, false]).elements[0],
        ruleset,
        table,
    )?;
    let mut solver = Solver::default();
    let x = std::array::from_fn::<_, 3, _>(|_| solver.new_bounded_integer(0, 1));
    let objective = solver.new_bounded_integer(0, 3);
    let tag = solver.new_constraint_tag();
    let mut terms = x.iter().map(|v| v.scaled(1)).collect::<Vec<_>>();
    terms.push(objective.scaled(-1));
    solver
        .add_constraint(pumpkin_solver::equals(terms, 0, tag))
        .post();
    // Build the finite feasibility relation from the shared validator, not duplicated rule limits.
    for mask in 0..8 {
        let slots = std::array::from_fn(|i| mask & (1 << i) != 0);
        let validation = validate_program(&bronze_program(slots));
        if validation
            .findings
            .iter()
            .any(|f| f.kind != FindingKind::Violation)
        {
            return Err(ReferenceError::IncompleteValidation);
        }
        if mask == 0
            || (request.exclude_lutz && mask != 0)
            || slots
                .iter()
                .zip(request.enabled_slots)
                .any(|(&occupied, enabled)| occupied && !enabled)
            || !validation.findings.is_empty()
        {
            let clause = x
                .iter()
                .zip(slots)
                .map(|(&v, occupied)| {
                    if occupied {
                        predicate!(v == 0)
                    } else {
                        predicate!(v == 1)
                    }
                })
                .collect::<Vec<_>>();
            let tag = solver.new_constraint_tag();
            solver.add_clause(clause, tag);
        }
    }
    let deadline = match request.time_limit {
        Some(d) => Some(
            Instant::now()
                .checked_add(d)
                .ok_or(ReferenceError::InvalidLimit)?,
        ),
        None => None,
    };
    let mut termination = Deadline(deadline);
    let mut brancher = solver.default_brancher();
    let mut resolver = ResolutionResolver::default();
    let callback = |_: &Solver,
                    _: SolutionReference,
                    _: &DefaultBrancher,
                    _: &ResolutionResolver|
     -> ControlFlow<()> { ControlFlow::Continue(()) };
    let outcome = solver.optimise(
        &mut brancher,
        &mut termination,
        &mut resolver,
        LinearSatUnsat::new(OptimisationDirection::Maximise, objective, callback),
    );
    let mut candidates = Vec::new();
    let (status, solution) = match outcome {
        OptimisationResult::Optimal(s) => (OptimizerStatus::OptimalProven, Some(s)),
        OptimisationResult::Satisfiable(s) | OptimisationResult::Stopped(s, _) => {
            (OptimizerStatus::FeasibleUnproven, Some(s))
        }
        OptimisationResult::Unsatisfiable => (OptimizerStatus::InfeasibleProven, None),
        OptimisationResult::Unknown => (OptimizerStatus::Incomplete, None),
    };
    if let Some(solution) = solution {
        let slots = x.map(|v| solution.get_integer_value(v) == 1);
        let candidate = checked_candidate(slots, unit)?;
        if i32::from(candidate.base_hundredths)
            != solution.get_integer_value(objective) * i32::from(unit)
        {
            return Err(ReferenceError::UnsupportedElement);
        }
        candidates.push(candidate);
        if status == OptimizerStatus::OptimalProven
            && request.enumerate_equal_optima
            && request.max_solutions > 1
        {
            let optimum = solution.get_integer_value(objective);
            let tag = solver.new_constraint_tag();
            solver
                .add_constraint(pumpkin_solver::equals(vec![objective], optimum, tag))
                .post();
            let mut brancher = solver.default_brancher();
            let mut resolver = ResolutionResolver::default();
            let mut iter =
                solver.get_solution_iterator(&mut brancher, &mut termination, &mut resolver);
            let mut complete = false;
            loop {
                match iter.next_solution() {
                    IteratedSolution::Solution(s, ..) => {
                        let slots = x.map(|v| s.get_integer_value(v) == 1);
                        if !candidates.iter().any(|c| c.slots == slots) {
                            candidates.push(checked_candidate(slots, unit)?);
                        }
                        if candidates.len() >= request.max_solutions {
                            break;
                        }
                    }
                    IteratedSolution::Finished => {
                        complete = true;
                        break;
                    }
                    IteratedSolution::Unsatisfiable | IteratedSolution::Unknown => break,
                }
            }
            candidates.sort_by_key(|c| c.slots);
            return Ok(OptimizerResult {
                domain: reference_domain,
                enabled_slots: request.enabled_slots,
                ruleset,
                scoring_table: table,
                status,
                enumeration_complete: complete,
                candidates,
            });
        }
    }
    Ok(OptimizerResult {
        domain: reference_domain,
        enabled_slots: request.enabled_slots,
        ruleset,
        scoring_table: table,
        status,
        enumeration_complete: false,
        candidates,
    })
}

fn checked_candidate(slots: [bool; 3], unit: u16) -> Result<ReferenceCandidate, ReferenceError> {
    let program = bronze_program(slots);
    let validation = validate_program(&program);
    if !validation.findings.is_empty() {
        return Err(ReferenceError::IncompleteValidation);
    }
    let mut total = 0;
    for element in &program.elements {
        total += reference_base_value(element, program.ruleset, program.scoring_table)?;
    }
    if total != unit * program.elements.len() as u16 {
        return Err(ReferenceError::UnsupportedElement);
    }
    Ok(ReferenceCandidate {
        slots,
        program,
        base_hundredths: total,
        coverage: validation.coverage,
    })
}
