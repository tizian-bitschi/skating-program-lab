use super::*;
use crate::optimizer::solver_common::{self, Deadline, EnumerationEnd};
use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        optimisation::{linear_sat_unsat::LinearSatUnsat, OptimisationDirection},
        predicate,
        results::{OptimisationResult, ProblemSolution, SolutionReference},
        variables::DomainId,
        DefaultBrancher,
    },
    Solver,
};
use std::{ops::ControlFlow, time::Instant};

fn validate_compound_source(
    request: &JumpOptimizerRequest,
    element: &JumpElement,
    profile: &crate::RuleProfile,
) -> Result<(), JumpOptimizerError> {
    let mut elements = vec![PlannedElement::Jump(element.clone())];
    if profile.jumps.require_axel {
        elements.push(PlannedElement::Jump(JumpElement::Solo(ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        })));
    }
    let program = Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: request.category,
        ruleset: request.ruleset,
        scoring_table: request.scoring_table,
        sources: vec![],
        elements,
    };
    let findings: Vec<_> = crate::validation::validate_elements(&program.elements, profile)
        .findings
        .into_iter()
        .filter(|f| f.kind != FindingKind::Violation)
        .collect();
    if findings.is_empty() {
        Ok(())
    } else {
        Err(JumpOptimizerError::UnknownValidation {
            elements: vec![element.clone()],
            findings,
        })
    }
}

fn universe(
    request: &JumpOptimizerRequest,
    profile: &crate::RuleProfile,
) -> Result<Vec<JumpElement>, JumpOptimizerError> {
    let mut universe = Vec::new();
    for jump in &request.allowed_jumps {
        jump_code(jump)?;
        let element = JumpElement::Solo(jump.clone());
        if request_allows(request, std::slice::from_ref(&element))? && !universe.contains(&element)
        {
            universe.push(element);
        }
    }
    for element in &request.combinations {
        if !matches!(element, JumpElement::Combination(_)) {
            return Err(JumpOptimizerError::UnsupportedVariant);
        }
        let components = parts(element)?;
        if components
            .iter()
            .any(|jump| !request.allowed_jumps.contains(jump))
        {
            return Err(JumpOptimizerError::InvalidRequest);
        }
        if components.iter().any(|jump| {
            request.exclusions.contains(jump)
                || jump
                    .kind
                    .is_some_and(|k| request.excluded_kinds.contains(&k))
        }) {
            continue;
        }
        validate_compound_source(request, element, profile)?;
        value(element, request)?;
        if !universe.contains(element) {
            universe.push(element.clone());
        }
    }
    Ok(universe)
}

fn decode(
    solution: SolutionReference<'_>,
    slots: &[DomainId],
    universe: &[JumpElement],
) -> Result<Vec<JumpElement>, JumpOptimizerError> {
    let mut elements = Vec::new();
    let mut ended = false;
    for slot in slots {
        let value = solution.get_integer_value(*slot);
        if value == 0 {
            ended = true;
            continue;
        }
        if ended || value < 0 {
            return Err(JumpOptimizerError::InconsistentSolverResult);
        }
        elements.push(
            universe
                .get(value as usize - 1)
                .ok_or(JumpOptimizerError::InconsistentSolverResult)?
                .clone(),
        );
    }
    Ok(elements)
}

fn result(
    request: &JumpOptimizerRequest,
    universe: Vec<JumpElement>,
    sources: Vec<SourceReference>,
    profile: &crate::RuleProfile,
) -> JumpOptimizerResult {
    JumpOptimizerResult {
        request: request.clone(), universe,
        omitted_variants: "All unlisted jumps and compounds, sequences, Euler, transitions, marks and non-jump elements are omitted.",
        equivalence: "Identical ordered jump-element sequences.",
        proof_scope: "Maximum unfactored table BV within this finite modeled jump subsection; not a complete or official program optimum.",
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        implemented_rules: profile.jump_bindings().iter().map(|b| b.id).collect(), sources,
        examined_subsections: 0, status: OptimizerStatus::Incomplete, enumeration_complete: false,
        termination_reason: JumpTerminationReason::Interrupted, candidates: Vec::new(),
    }
}

pub(super) fn optimize(
    request: &JumpOptimizerRequest,
) -> Result<JumpOptimizerResult, JumpOptimizerError> {
    optimize_inner(request, false)
}

fn optimize_inner(
    request: &JumpOptimizerRequest,
    stop_after_incumbent: bool,
) -> Result<JumpOptimizerResult, JumpOptimizerError> {
    optimize_profile_inner(
        request,
        stop_after_incumbent,
        &crate::adult_profile(request.category),
    )
}

pub(super) fn optimize_profile(
    request: &JumpOptimizerRequest,
    profile: &crate::RuleProfile,
) -> Result<JumpOptimizerResult, JumpOptimizerError> {
    optimize_profile_inner(request, false, profile)
}

fn optimize_profile_inner(
    request: &JumpOptimizerRequest,
    stop_after_incumbent: bool,
    profile: &crate::RuleProfile,
) -> Result<JumpOptimizerResult, JumpOptimizerError> {
    if request.ruleset != BRONZE_RULESET
        || request.scoring_table != BRONZE_TABLE
        || request.max_elements > 5
        || request.max_solutions == 0
        || request.max_search_nodes == 0
        || request.max_search_nodes > 100_000
    {
        return Err(JumpOptimizerError::InvalidRequest);
    }
    let end = request
        .time_limit
        .map(|duration| {
            Instant::now()
                .checked_add(duration)
                .ok_or(JumpOptimizerError::InvalidRequest)
        })
        .transpose()?;
    let universe = universe(request, profile)?;
    let mut sources: Vec<_> = profile.jump_sources();
    let mut values = vec![0i32];
    for element in &universe {
        let score = value(element, request)?;
        values.push(i32::from(score.hundredths));
        if !sources.contains(&score.source) {
            sources.push(score.source);
        }
    }
    let mut result = result(request, universe, sources, profile);
    if end.is_some_and(|deadline| Instant::now() >= deadline) {
        return Ok(result);
    }
    if result.universe.is_empty() || request.max_elements == 0 {
        result.examined_subsections = 1;
        match checked_profile_candidate(request, &[], profile)? {
            Some(candidate) => {
                result.status = OptimizerStatus::OptimalProven;
                result.candidates.push(candidate);
                if !request.enumerate_equal_optima {
                    result.termination_reason = JumpTerminationReason::EnumerationDisabled;
                } else if request.max_solutions == 1 {
                    result.termination_reason = JumpTerminationReason::SolutionLimit;
                } else {
                    result.enumeration_complete = true;
                    result.termination_reason = JumpTerminationReason::EnumerationExhausted;
                }
            }
            None => {
                result.status = OptimizerStatus::InfeasibleProven;
                result.termination_reason = JumpTerminationReason::Infeasible;
            }
        }
        return Ok(result);
    }
    let (mut solver, slots, objective) = solver_common::ordered_slots(
        &values,
        request.max_elements,
        usize::from(request.require_nonempty),
    );
    let mut deadline = Deadline(end);
    loop {
        if result.examined_subsections >= request.max_search_nodes {
            result.termination_reason = JumpTerminationReason::SearchLimit;
            return Ok(result);
        }
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
            &mut deadline,
            &mut resolver,
            LinearSatUnsat::new(OptimisationDirection::Maximise, objective, callback),
        );
        let status = solver_common::status(&outcome);
        let solution = match outcome {
            OptimisationResult::Optimal(s)
            | OptimisationResult::Satisfiable(s)
            | OptimisationResult::Stopped(s, _) => s,
            OptimisationResult::Unsatisfiable => {
                result.status = OptimizerStatus::InfeasibleProven;
                result.termination_reason = JumpTerminationReason::Infeasible;
                return Ok(result);
            }
            OptimisationResult::Unknown => return Ok(result),
        };
        result.examined_subsections += 1;
        let elements = decode(solution.as_reference(), &slots, &result.universe)?;
        if let Some(candidate) = checked_profile_candidate(request, &elements, profile)? {
            if i32::from(candidate.base_hundredths) != solution.get_integer_value(objective) {
                return Err(JumpOptimizerError::InconsistentSolverResult);
            }
            result.status = status;
            result.candidates.push(candidate);
            if status == OptimizerStatus::OptimalProven {
                result.termination_reason = JumpTerminationReason::EnumerationDisabled;
                if request.enumerate_equal_optima {
                    let tag = solver.new_constraint_tag();
                    solver
                        .add_constraint(pumpkin_solver::equals(
                            vec![objective],
                            solution.get_integer_value(objective),
                            tag,
                        ))
                        .post();
                    let outcome = solver_common::enumerate(
                        &mut solver,
                        &mut deadline,
                        request.max_solutions,
                        request.max_search_nodes,
                        &mut result.examined_subsections,
                        &mut result.candidates,
                        |s| {
                            let elements = decode(s, &slots, &result.universe)?;
                            let candidate = checked_profile_candidate(request, &elements, profile)?;
                            if let Some(ref candidate) = candidate {
                                if i32::from(candidate.base_hundredths)
                                    != s.get_integer_value(objective)
                                {
                                    return Err(JumpOptimizerError::InconsistentSolverResult);
                                }
                            }
                            Ok(candidate)
                        },
                    )?;
                    match outcome {
                        EnumerationEnd::Exhausted => {
                            result.enumeration_complete = true;
                            result.termination_reason = JumpTerminationReason::EnumerationExhausted;
                        }
                        EnumerationEnd::SolutionLimit => {
                            result.termination_reason = JumpTerminationReason::SolutionLimit
                        }
                        EnumerationEnd::SearchLimit => {
                            result.termination_reason = JumpTerminationReason::SearchLimit
                        }
                        EnumerationEnd::Interrupted => {
                            result.termination_reason =
                                JumpTerminationReason::EnumerationInterrupted
                        }
                    }
                }
            }
            for candidate in &mut result.candidates {
                let elements = candidate
                    .program
                    .elements
                    .iter()
                    .map(|e| match e {
                        PlannedElement::Jump(j) => Ok(j.clone()),
                        _ => Err(JumpOptimizerError::InconsistentSolverResult),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let checked = checked_profile_candidate(request, &elements, profile)?
                    .ok_or(JumpOptimizerError::InconsistentSolverResult)?;
                if checked.base_hundredths != candidate.base_hundredths {
                    return Err(JumpOptimizerError::InconsistentSolverResult);
                }
                *candidate = checked;
            }
            return Ok(result);
        }
        if status != OptimizerStatus::OptimalProven {
            return Ok(result);
        }
        let clause = slots
            .iter()
            .map(|&slot| predicate!(slot != solution.get_integer_value(slot)))
            .collect::<Vec<_>>();
        let tag = solver.new_constraint_tag();
        solver.add_clause(clause, tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_with_valid_incumbent_is_not_proven() {
        let request = JumpOptimizerRequest {
            category: AdultCategory::Bronze,
            ruleset: BRONZE_RULESET,
            scoring_table: BRONZE_TABLE,
            allowed_jumps: vec![ListedJump {
                kind: Some(JumpKind::ToeLoop),
                rotations: Some(1),
            }],
            combinations: vec![],
            exclusions: vec![],
            excluded_kinds: vec![],
            max_elements: 1,
            require_nonempty: true,
            max_search_nodes: 100,
            enumerate_equal_optima: true,
            max_solutions: 5,
            time_limit: None,
        };
        let result = optimize_inner(&request, true).unwrap();
        assert_eq!(result.status, OptimizerStatus::FeasibleUnproven);
        assert_eq!(
            result.termination_reason,
            JumpTerminationReason::Interrupted
        );
        assert_eq!(result.candidates.len(), 1);
        assert!(!result.enumeration_complete);
    }
}
