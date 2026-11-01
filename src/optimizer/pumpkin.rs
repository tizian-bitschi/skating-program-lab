use super::*;
use crate::validation::{
    bronze_jump_source, COMBINATION_COUNT_RULE, COUNT_RULE, ELIGIBILITY_RULE, EULER_RULE,
    REPETITION_RULE, STRUCTURE_RULE,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizerObjective {
    MaximizeUnfactoredBaseValueHundredths,
}

#[derive(Debug, Clone)]
pub struct OptimizerResult {
    pub domain: ReferenceDomain,
    pub enabled_slots: [bool; 3],
    pub ruleset: RulesetId,
    pub scoring_table: ScoringTableId,
    /// The sum of unfactored supported jump base values, in hundredths.
    pub objective: OptimizerObjective,
    pub implemented_rules: Vec<&'static str>,
    pub sources: Vec<crate::SourceReference>,
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
    optimize_bronze_inner(request, false)
}

fn optimize_bronze_inner(
    request: &OptimizerRequest,
    stop_after_incumbent: bool,
) -> Result<OptimizerResult, ReferenceError> {
    if request.max_solutions == 0 {
        return Err(ReferenceError::InvalidLimit);
    }
    let reference_domain = bronze_domain(request.exclude_lutz);
    let ruleset = request.ruleset;
    let table = request.scoring_table;
    if ruleset != BRONZE_RULESET || table != BRONZE_TABLE {
        return Err(ReferenceError::UnsupportedVersion);
    }
    let unit_value = reference_base_value(
        &bronze_program([true, false, false]).elements[0],
        ruleset,
        table,
    )?;
    let unit = unit_value.hundredths;
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
     -> ControlFlow<()> {
        if stop_after_incumbent {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
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
            return Err(ReferenceError::InconsistentSolverResult);
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
                objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
                implemented_rules: implemented_rules(),
                sources: vec![bronze_jump_source(), unit_value.source.clone()],
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
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        implemented_rules: implemented_rules(),
        sources: vec![bronze_jump_source(), unit_value.source],
        status,
        enumeration_complete: false,
        candidates,
    })
}

fn checked_candidate(slots: [bool; 3], unit: u16) -> Result<ReferenceCandidate, ReferenceError> {
    let mut program = bronze_program(slots);
    let first = program
        .elements
        .first()
        .ok_or(ReferenceError::InconsistentSolverResult)?;
    let scoring_source =
        reference_base_value(first, program.ruleset, program.scoring_table)?.source;
    program.sources = vec![bronze_jump_source(), scoring_source];
    let validation = validate_program(&program);
    if !validation.findings.is_empty() {
        return Err(ReferenceError::IncompleteValidation);
    }
    let mut total = 0;
    for element in &program.elements {
        total += reference_base_value(element, program.ruleset, program.scoring_table)?.hundredths;
    }
    if total != unit * program.elements.len() as u16 {
        return Err(ReferenceError::InconsistentSolverResult);
    }
    Ok(ReferenceCandidate {
        slots,
        program,
        base_hundredths: total,
        coverage: validation.coverage,
    })
}

fn implemented_rules() -> Vec<&'static str> {
    vec![
        ELIGIBILITY_RULE,
        COUNT_RULE,
        COMBINATION_COUNT_RULE,
        STRUCTURE_RULE,
        EULER_RULE,
        REPETITION_RULE,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_after_incumbent_is_not_proven() {
        let request = OptimizerRequest {
            exclude_lutz: false,
            enabled_slots: [true; 3],
            ruleset: BRONZE_RULESET,
            scoring_table: BRONZE_TABLE,
            enumerate_equal_optima: true,
            max_solutions: 8,
            time_limit: None,
        };
        let result = optimize_bronze_inner(&request, true).unwrap();
        assert_eq!(result.status, OptimizerStatus::FeasibleUnproven);
        assert_eq!(result.candidates.len(), 1);
        assert!(!result.enumeration_complete);
        assert!(validate_program(&result.candidates[0].program)
            .findings
            .is_empty());
    }
    #[test]
    fn unsupported_ruleset_has_a_distinct_error() {
        let request = OptimizerRequest {
            exclude_lutz: false,
            enabled_slots: [true; 3],
            ruleset: RulesetId::TestOther,
            scoring_table: BRONZE_TABLE,
            enumerate_equal_optima: true,
            max_solutions: 8,
            time_limit: None,
        };
        assert!(matches!(
            optimize_bronze(&request),
            Err(ReferenceError::UnsupportedVersion)
        ));
    }
}
