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
