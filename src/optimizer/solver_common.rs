use super::{OptimizerStatus, ReferenceCandidate};
use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        results::{solution_iterator::IteratedSolution, OptimisationResult, SolutionReference},
        termination::TerminationCondition,
    },
    Solver,
};
use std::time::Instant;

/// Ordered candidate slots with a suffix of empty slots and catalog value sum.
pub(super) fn ordered_slots(
    values: &[i32],
    max_elements: usize,
    min_elements: usize,
) -> (
    Solver,
    Vec<pumpkin_solver::core::variables::DomainId>,
    pumpkin_solver::core::variables::DomainId,
) {
    use pumpkin_solver::core::{predicate, variables::TransformableVariable};
    let mut solver = Solver::default();
    let max_value = *values.iter().max().unwrap();
    let slots: Vec<_> = (0..max_elements)
        .map(|_| solver.new_bounded_integer(0, values.len() as i32 - 1))
        .collect();
    let scores: Vec<_> = slots
        .iter()
        .map(|_| solver.new_bounded_integer(0, max_value))
        .collect();
    let objective = solver.new_bounded_integer(0, max_value * max_elements as i32);
    let mut terms: Vec<_> = scores.iter().map(|score| score.scaled(1)).collect();
    terms.push(objective.scaled(-1));
    let tag = solver.new_constraint_tag();
    solver
        .add_constraint(pumpkin_solver::equals(terms, 0, tag))
        .post();
    for (&slot, &score) in slots.iter().zip(&scores) {
        for (index, &value) in values.iter().enumerate() {
            let tag = solver.new_constraint_tag();
            solver.add_clause(
                vec![predicate!(slot != index as i32), predicate!(score == value)],
                tag,
            );
        }
    }
    for pair in slots.windows(2) {
        let tag = solver.new_constraint_tag();
        solver.add_clause(
            vec![predicate!(pair[0] != 0), predicate!(pair[1] == 0)],
            tag,
        );
    }
    for &slot in slots.iter().take(min_elements) {
        let tag = solver.new_constraint_tag();
        solver.add_clause(vec![predicate!(slot != 0)], tag);
    }
    (solver, slots, objective)
}

pub(super) struct Deadline(pub(super) Option<Instant>);
impl TerminationCondition for Deadline {
    fn should_stop(&mut self) -> bool {
        self.0.is_some_and(|deadline| Instant::now() >= deadline)
    }
}

pub(super) fn status<T>(outcome: &OptimisationResult<T>) -> OptimizerStatus {
    match outcome {
        OptimisationResult::Optimal(_) => OptimizerStatus::OptimalProven,
        OptimisationResult::Satisfiable(_) | OptimisationResult::Stopped(_, _) => {
            OptimizerStatus::FeasibleUnproven
        }
        OptimisationResult::Unsatisfiable => OptimizerStatus::InfeasibleProven,
        OptimisationResult::Unknown => OptimizerStatus::Incomplete,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnumerationEnd {
    Exhausted,
    SolutionLimit,
    SearchLimit,
    Interrupted,
}

pub(super) fn enumerate<E>(
    solver: &mut Solver,
    deadline: &mut Deadline,
    max_solutions: usize,
    max_examined: usize,
    examined: &mut usize,
    candidates: &mut Vec<ReferenceCandidate>,
    mut check: impl FnMut(SolutionReference<'_>) -> Result<Option<ReferenceCandidate>, E>,
) -> Result<EnumerationEnd, E> {
    if candidates.len() >= max_solutions {
        return Ok(EnumerationEnd::SolutionLimit);
    }
    let mut brancher = solver.default_brancher();
    let mut resolver = ResolutionResolver::default();
    let mut iter = solver.get_solution_iterator(&mut brancher, deadline, &mut resolver);
    loop {
        if *examined >= max_examined {
            return Ok(EnumerationEnd::SearchLimit);
        }
        match iter.next_solution() {
            IteratedSolution::Solution(solution, ..) => {
                *examined += 1;
                if let Some(candidate) = check(solution.as_reference())? {
                    if !candidates
                        .iter()
                        .any(|c| c.program.elements == candidate.program.elements)
                    {
                        candidates.push(candidate);
                    }
                    if candidates.len() >= max_solutions {
                        return Ok(EnumerationEnd::SolutionLimit);
                    }
                }
            }
            IteratedSolution::Finished => return Ok(EnumerationEnd::Exhausted),
            IteratedSolution::Unsatisfiable | IteratedSolution::Unknown => {
                return Ok(EnumerationEnd::Interrupted)
            }
        }
    }
}
