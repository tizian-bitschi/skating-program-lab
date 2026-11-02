//! Bounded, category-aware jump-only optimization.
use super::*;
use crate::{JumpPart, SourceReference};
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
        DefaultBrancher,
    },
    Solver,
};
use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

/// An explicit finite domain. Each allowed jump becomes a solo candidate; compounds
/// must be listed explicitly and may contain only allowed, non-excluded jumps.
#[derive(Debug, Clone)]
pub struct JumpOptimizerRequest {
    pub category: AdultCategory,
    pub ruleset: RulesetId,
    pub scoring_table: ScoringTableId,
    pub allowed_jumps: Vec<ListedJump>,
    pub combinations: Vec<JumpElement>,
    pub exclusions: Vec<ListedJump>,
    /// Exclude every rotation of each listed jump kind.
    pub excluded_kinds: Vec<JumpKind>,
    pub max_elements: usize,
    pub require_nonempty: bool,
    /// Maximum number of ordered subsections examined, including the empty one.
    pub max_search_nodes: usize,
    pub enumerate_equal_optima: bool,
    pub max_solutions: usize,
    /// Shared budget for candidate generation, optimization and enumeration.
    pub time_limit: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct JumpOptimizerResult {
    pub request: JumpOptimizerRequest,
    /// The actual finite element universe after exclusions, in request order.
    pub universe: Vec<JumpElement>,
    pub omitted_variants: &'static str,
    pub equivalence: &'static str,
    pub proof_scope: &'static str,
    pub objective: OptimizerObjective,
    pub implemented_rules: Vec<&'static str>,
    pub sources: Vec<SourceReference>,
    pub examined_subsections: usize,
    pub feasible_subsections: usize,
    pub status: OptimizerStatus,
    pub enumeration_complete: bool,
    pub termination_reason: JumpTerminationReason,
    pub candidates: Vec<ReferenceCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpTerminationReason {
    Infeasible,
    SearchLimit,
    Interrupted,
    EnumerationDisabled,
    SolutionLimit,
    EnumerationInterrupted,
    EnumerationExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpOptimizerError {
    InvalidRequest,
    UnsupportedVariant,
    UnknownValidation,
    Scoring(BaseValueError),
    InconsistentSolverResult,
}

struct Deadline(Option<Instant>);
impl TerminationCondition for Deadline {
    fn should_stop(&mut self) -> bool {
        self.0.is_some_and(|deadline| Instant::now() >= deadline)
    }
}

fn jump_code(jump: &ListedJump) -> Result<ComponentCode, JumpOptimizerError> {
    let suffix = match jump.kind {
        Some(JumpKind::ToeLoop) => "T",
        Some(JumpKind::Salchow) => "S",
        Some(JumpKind::Loop) => "Lo",
        Some(JumpKind::Flip) => "F",
        Some(JumpKind::Lutz) => "Lz",
        Some(JumpKind::Axel) => "A",
        None => return Err(JumpOptimizerError::UnsupportedVariant),
    };
    let rotations = jump
        .rotations
        .filter(|n| (1..=2).contains(n))
        .ok_or(JumpOptimizerError::UnsupportedVariant)?;
    ComponentCode::try_from(format!("{rotations}{suffix}").as_str())
        .map_err(|_| JumpOptimizerError::UnsupportedVariant)
}

fn parts(element: &JumpElement) -> Result<Vec<&ListedJump>, JumpOptimizerError> {
    match element {
        JumpElement::Solo(jump) => Ok(vec![jump]),
        JumpElement::Combination(parts) if parts.len() == 2 => parts
            .iter()
            .map(|part| match part {
                JumpPart::Jump(jump) => Ok(jump),
                _ => Err(JumpOptimizerError::UnsupportedVariant),
            })
            .collect(),
        _ => Err(JumpOptimizerError::UnsupportedVariant),
    }
}

fn value(
    element: &JumpElement,
    request: &JumpOptimizerRequest,
) -> Result<BaseValue, JumpOptimizerError> {
    let codes = parts(element)?
        .into_iter()
        .map(jump_code)
        .collect::<Result<Vec<_>, _>>()?;
    let recognition = PanelRecognition {
        ruleset: request.ruleset,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(codes, None, Some(vec![]))
                .map_err(|_| JumpOptimizerError::UnsupportedVariant)?,
        ),
    };
    match lookup_base_value(request.scoring_table, &recognition, &[])
        .map_err(JumpOptimizerError::Scoring)?
    {
        BaseValueOutcome::Value(value) => Ok(value),
        _ => Err(JumpOptimizerError::UnsupportedVariant),
    }
}

fn checked_candidate(
    request: &JumpOptimizerRequest,
    elements: &[JumpElement],
) -> Result<Option<ReferenceCandidate>, JumpOptimizerError> {
    let mut program = Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: request.category,
        ruleset: request.ruleset,
        scoring_table: request.scoring_table,
        sources: crate::validation::jump_rule_sources(request.category),
        elements: elements.iter().cloned().map(PlannedElement::Jump).collect(),
    };
    let validation = validate_program(&program);
    if validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation)
    {
        return Ok(None);
    }
    if !validation.findings.is_empty() {
        return Err(JumpOptimizerError::UnknownValidation);
    }
    let mut total = 0u16;
    for element in elements {
        let score = value(element, request)?;
        total = total
            .checked_add(score.hundredths)
            .ok_or(JumpOptimizerError::InvalidRequest)?;
        if !program.sources.contains(&score.source) {
            program.sources.push(score.source);
        }
    }
    Ok(Some(ReferenceCandidate {
        program,
        base_hundredths: total,
        coverage: validation.coverage,
    }))
}

/// Optimize only the supplied finite jump domain, never a complete Free Skating program.
pub fn optimize_jumps(
    request: &JumpOptimizerRequest,
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
    let mut universe = Vec::new();
    for jump in &request.allowed_jumps {
        jump_code(jump)?;
        if !request.exclusions.contains(jump)
            && !request.excluded_kinds.contains(&jump.kind.unwrap())
        {
            let element = JumpElement::Solo(jump.clone());
            if !universe.contains(&element) {
                universe.push(element);
            }
        }
    }
    for element in &request.combinations {
        if !matches!(element, JumpElement::Combination(_)) {
            return Err(JumpOptimizerError::UnsupportedVariant);
        }
        let components = parts(element)?;
        if components
            .iter()
            .any(|j| !request.allowed_jumps.contains(j))
        {
            return Err(JumpOptimizerError::InvalidRequest);
        }
        if components.iter().any(|j| {
            request.exclusions.contains(j)
                || j.kind.is_some_and(|k| request.excluded_kinds.contains(&k))
        }) {
            continue;
        }
        value(element, request)?;
        if !universe.contains(element) {
            universe.push(element.clone());
        }
    }
    let mut sources = crate::validation::jump_rule_sources(request.category);
    for element in &universe {
        let score = value(element, request)?;
        if !sources.contains(&score.source) {
            sources.push(score.source);
        }
    }
    let mut result = JumpOptimizerResult {
        request: request.clone(), universe: universe.clone(),
        omitted_variants: "All unlisted jumps and compounds, sequences, Euler, transitions, marks and non-jump elements are omitted.",
        equivalence: "Identical ordered jump-element sequences.",
        proof_scope: "Maximum unfactored table BV within this finite modeled jump subsection; not a complete or official program optimum.",
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        implemented_rules: crate::validation::jump_rule_ids(request.category), sources,
        examined_subsections: 0, feasible_subsections: 0, status: OptimizerStatus::Incomplete, enumeration_complete: false,
        termination_reason: JumpTerminationReason::Interrupted, candidates: Vec::new(),
    };
    let mut stack = vec![Vec::<JumpElement>::new()];
    let mut feasible = Vec::new();
    let mut interrupted = None;
    while let Some(elements) = stack.pop() {
        if result.examined_subsections == request.max_search_nodes {
            interrupted = Some(JumpTerminationReason::SearchLimit);
            break;
        }
        if end.is_some_and(|deadline| Instant::now() >= deadline) {
            interrupted = Some(JumpTerminationReason::Interrupted);
            break;
        }
        result.examined_subsections += 1;
        if !elements.is_empty() || !request.require_nonempty {
            if let Some(candidate) = checked_candidate(request, &elements)? {
                result.feasible_subsections += 1;
                feasible.push(candidate);
            }
        }
        if elements.len() < request.max_elements {
            for item in universe.iter().rev() {
                let mut next = elements.clone();
                next.push(item.clone());
                stack.push(next);
            }
        }
    }
    if let Some(reason) = interrupted {
        if let Some(best) = feasible.into_iter().max_by_key(|c| c.base_hundredths) {
            let elements = best
                .program
                .elements
                .iter()
                .map(|e| match e {
                    PlannedElement::Jump(j) => Ok(j.clone()),
                    _ => Err(JumpOptimizerError::InconsistentSolverResult),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let checked = checked_candidate(request, &elements)?
                .ok_or(JumpOptimizerError::InconsistentSolverResult)?;
            if checked.base_hundredths != best.base_hundredths {
                return Err(JumpOptimizerError::InconsistentSolverResult);
            }
            result.status = OptimizerStatus::FeasibleUnproven;
            result.candidates.push(checked);
        }
        result.termination_reason = reason;
        return Ok(result);
    }
    if feasible.is_empty() {
        result.status = OptimizerStatus::InfeasibleProven;
        result.termination_reason = JumpTerminationReason::Infeasible;
        return Ok(result);
    }
    let mut solver = Solver::default();
    let index = solver.new_bounded_integer(0, (feasible.len() - 1) as i32);
    let max_score = feasible
        .iter()
        .map(|c| i32::from(c.base_hundredths))
        .max()
        .unwrap();
    let objective = solver.new_bounded_integer(0, max_score);
    for (i, candidate) in feasible.iter().enumerate() {
        let tag = solver.new_constraint_tag();
        solver.add_clause(
            vec![
                predicate!(index != i as i32),
                predicate!(objective == i32::from(candidate.base_hundredths)),
            ],
            tag,
        );
    }
    let mut deadline = Deadline(end);
    let mut brancher = solver.default_brancher();
    let mut resolver = ResolutionResolver::default();
    let callback = |_: &Solver,
                    _: SolutionReference,
                    _: &DefaultBrancher,
                    _: &ResolutionResolver|
     -> ControlFlow<()> { ControlFlow::Continue(()) };
    let outcome = solver.optimise(
        &mut brancher,
        &mut deadline,
        &mut resolver,
        LinearSatUnsat::new(OptimisationDirection::Maximise, objective, callback),
    );
    let solution = match outcome {
        OptimisationResult::Optimal(s) => {
            result.status = OptimizerStatus::OptimalProven;
            Some(s)
        }
        OptimisationResult::Satisfiable(s) | OptimisationResult::Stopped(s, _) => {
            result.status = OptimizerStatus::FeasibleUnproven;
            Some(s)
        }
        OptimisationResult::Unknown => {
            result.status = OptimizerStatus::FeasibleUnproven;
            result.candidates.push(
                feasible
                    .iter()
                    .max_by_key(|c| c.base_hundredths)
                    .unwrap()
                    .clone(),
            );
            None
        }
        OptimisationResult::Unsatisfiable => {
            return Err(JumpOptimizerError::InconsistentSolverResult)
        }
    };
    if let Some(solution) = solution {
        let i = solution.get_integer_value(index) as usize;
        let chosen = feasible
            .get(i)
            .ok_or(JumpOptimizerError::InconsistentSolverResult)?;
        if i32::from(chosen.base_hundredths) != solution.get_integer_value(objective) {
            return Err(JumpOptimizerError::InconsistentSolverResult);
        }
        result.candidates.push(chosen.clone());
        if result.status == OptimizerStatus::OptimalProven {
            result.termination_reason = JumpTerminationReason::EnumerationDisabled;
            if request.enumerate_equal_optima {
                result.termination_reason = JumpTerminationReason::SolutionLimit;
                if request.max_solutions > 1 {
                    let tag = solver.new_constraint_tag();
                    solver
                        .add_constraint(pumpkin_solver::equals(
                            vec![objective],
                            solution.get_integer_value(objective),
                            tag,
                        ))
                        .post();
                    let mut brancher = solver.default_brancher();
                    let mut resolver = ResolutionResolver::default();
                    let mut iter =
                        solver.get_solution_iterator(&mut brancher, &mut deadline, &mut resolver);
                    loop {
                        match iter.next_solution() {
                            IteratedSolution::Solution(s, ..) => {
                                let candidate =
                                    feasible
                                        .get(s.get_integer_value(index) as usize)
                                        .ok_or(JumpOptimizerError::InconsistentSolverResult)?;
                                if !result
                                    .candidates
                                    .iter()
                                    .any(|c| c.program.elements == candidate.program.elements)
                                {
                                    result.candidates.push(candidate.clone());
                                }
                                if result.candidates.len() >= request.max_solutions {
                                    break;
                                }
                            }
                            IteratedSolution::Finished => {
                                result.enumeration_complete = true;
                                result.termination_reason =
                                    JumpTerminationReason::EnumerationExhausted;
                                break;
                            }
                            IteratedSolution::Unknown | IteratedSolution::Unsatisfiable => {
                                result.termination_reason =
                                    JumpTerminationReason::EnumerationInterrupted;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // Revalidate and recompute each returned candidate independently of the solver assignment.
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
        if elements.iter().any(|e| !universe.contains(e)) {
            return Err(JumpOptimizerError::InconsistentSolverResult);
        }
        let checked = checked_candidate(request, &elements)?
            .ok_or(JumpOptimizerError::InconsistentSolverResult)?;
        if checked.base_hundredths != candidate.base_hundredths {
            return Err(JumpOptimizerError::InconsistentSolverResult);
        }
        *candidate = checked;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_small_domain_matches_independent_feasibility_fixtures() {
        let a = ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        };
        let lo = ListedJump {
            kind: Some(JumpKind::Loop),
            rotations: Some(2),
        };
        let universe = [JumpElement::Solo(a), JumpElement::Solo(lo)];
        // Order is empty, A, 2Lo, A+A, A+2Lo, 2Lo+A, 2Lo+2Lo.
        let fixtures = [
            (
                AdultCategory::Bronze,
                [true, false, false, false, false, false, false],
            ),
            (
                AdultCategory::Silver,
                [true, true, false, true, false, false, false],
            ),
            (
                AdultCategory::Gold,
                [true, true, true, true, true, true, true],
            ),
            (
                AdultCategory::Masters,
                [false, true, false, true, true, true, false],
            ),
        ];
        let elements = [
            vec![],
            vec![universe[0].clone()],
            vec![universe[1].clone()],
            vec![universe[0].clone(), universe[0].clone()],
            vec![universe[0].clone(), universe[1].clone()],
            vec![universe[1].clone(), universe[0].clone()],
            vec![universe[1].clone(), universe[1].clone()],
        ];
        for (category, expected) in fixtures {
            let request = JumpOptimizerRequest {
                category,
                ruleset: BRONZE_RULESET,
                scoring_table: BRONZE_TABLE,
                allowed_jumps: vec![],
                combinations: vec![],
                exclusions: vec![],
                excluded_kinds: vec![],
                max_elements: 2,
                require_nonempty: false,
                max_search_nodes: 100,
                enumerate_equal_optima: true,
                max_solutions: 20,
                time_limit: None,
            };
            let actual: Vec<_> = elements
                .iter()
                .map(|sequence| checked_candidate(&request, sequence).unwrap().is_some())
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
