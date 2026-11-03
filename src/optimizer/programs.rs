//! Finite ordered mixed-element optimization using the existing Pumpkin backend.
use super::{
    solver_common::{self, Deadline},
    OptimizerObjective, OptimizerStatus,
};
use crate::*;
use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        optimisation::{linear_sat_unsat::LinearSatUnsat, OptimisationDirection},
        predicate,
        results::{OptimisationResult, ProblemSolution, SolutionReference},
        DefaultBrancher,
    },
    Solver,
};
use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramVariant {
    pub element: PlannedElement,
    pub evidence: ElementPlanEvidence,
    /// Explicit hypothetical recognition, never inferred from missing execution.
    pub recognition_assumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalExclusion {
    JumpKind(JumpKind),
    Jump(ListedJump),
    SpinKind(RegularSpinKind),
    FlyingEntry,
    SpinFeature(SpinFeature),
    Variant(usize),
    Unsupported(String),
}

#[derive(Debug, Clone)]
pub struct ProgramOptimizationRequest {
    pub season: Season,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub policy: ProgramPolicy,
    pub variants: Vec<ProgramVariant>,
    pub exclusions: Vec<PersonalExclusion>,
    /// Each group requires at least one of the original candidate indices.
    pub must_have: Vec<Vec<usize>>,
    pub min_elements: usize,
    pub max_elements: usize,
    pub objective: OptimizerObjective,
    pub equivalence: ProgramEquivalence,
    /// Validator-checked solver proposals; does not count or bound internal solver nodes.
    pub max_proposals: usize,
    pub max_solutions: usize,
    pub enumerate_equal_optima: bool,
    pub time_limit: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramEquivalence {
    OrderedVariants,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCandidate {
    pub program: ProfileProgram,
    pub variant_indices: Vec<usize>,
    pub evidence: Vec<ElementPlanEvidence>,
    pub per_element: Vec<BaseValue>,
    pub base_hundredths: u32,
    pub validation: EvidenceValidationReport,
}

#[derive(Debug, Clone)]
pub struct ProgramOptimizationResult {
    pub request: ProgramOptimizationRequest,
    pub universe: Vec<usize>,
    pub excluded_variants: Vec<usize>,
    pub unmodeled_variants: &'static str,
    pub equivalence: &'static str,
    pub proof_scope: &'static str,
    pub assumptions: &'static str,
    pub sources: Vec<SourceReference>,
    pub supported_rules: Vec<&'static str>,
    /// Union of source-bound checks actually evaluated for returned candidates.
    pub applied_rules: Vec<AppliedRule>,
    pub status: OptimizerStatus,
    pub enumeration_complete: bool,
    pub termination_reason: JumpTerminationReason,
    pub examined_programs: usize,
    pub candidates: Vec<ProgramCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramOptimizationError {
    InvalidRequest(&'static str),
    Profile(ProfileSelectionError),
    UnsupportedVariant(usize),
    UnsupportedRestriction,
    Scoring(BaseValueError),
    UnresolvedValidation(Vec<Finding>),
    InconsistentSolverResult,
}

/// Construct only the declared clean recognition subset, using the shared scorer.
fn score(
    variant: &ProgramVariant,
    request: &ProgramOptimizationRequest,
    ruleset: RulesetId,
) -> Result<BaseValue, ProgramOptimizationError> {
    use ProgramOptimizationError as E;
    let unsupported = || E::InvalidRequest("unsupported plan/evidence/recognition variant");
    if !variant.recognition_assumed {
        return Err(unsupported());
    }
    let mut calls = vec![];
    let mut target_level = None;
    let codes = match (&variant.element, &variant.evidence) {
        (
            PlannedElement::Jump(jump),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed,
            },
        ) => {
            match jump {
                JumpElement::Solo(j) => {
                    vec![super::jumps::jump_code(j).map_err(|_| unsupported())?]
                }
                JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                    let sequence = matches!(jump, JumpElement::Sequence(_));
                    if sequence && !direct_transitions_assumed {
                        return Err(unsupported());
                    }
                    let mut codes = vec![];
                    for part in parts {
                        codes.push(match part {
                            JumpPart::Jump(j) => {
                                super::jumps::jump_code(j).map_err(|_| unsupported())?
                            }
                            // Euler consequences remain unresolved for this clean subset.
                            _ => return Err(unsupported()),
                        });
                    }
                    if !(2..=3).contains(&codes.len()) {
                        return Err(unsupported());
                    }
                    if sequence {
                        calls.push(RecordedCall {
                            target: CallTarget::BetweenComponents(codes.len()),
                            call: PanelCall::Sequence,
                        });
                    }
                    codes
                }
            }
        }
        (PlannedElement::RegularSpin(_), ElementPlanEvidence::Spin { spin, .. }) => {
            let a = spin.abbreviation.ok_or_else(unsupported)?;
            if spin.panel.is_some() {
                return Err(unsupported());
            }
            let name = match a.kind {
                RegularSpinKind::Camel => "CSp",
                RegularSpinKind::Sit => "SSp",
                RegularSpinKind::Upright => "USp",
                RegularSpinKind::Layback => "LSp",
                RegularSpinKind::Combination => "CoSp",
            };
            let code = format!(
                "{}{}{name}",
                if a.entry == SpinEntry::Flying {
                    "F"
                } else {
                    ""
                },
                if a.change_foot { "C" } else { "" }
            );
            target_level = spin.planned_level;
            if target_level.is_none() {
                return Err(unsupported());
            }
            vec![ComponentCode::try_from(code.as_str()).map_err(|_| unsupported())?]
        }
        (PlannedElement::StepSequence, ElementPlanEvidence::Step { level, .. }) => {
            target_level = Some(*level);
            vec![ComponentCode::try_from("StSq").unwrap()]
        }
        (
            PlannedElement::ChoreographicSequence { .. },
            ElementPlanEvidence::ChoreographicSequence { .. },
        ) => vec![ComponentCode::try_from("ChSq1").unwrap()],
        (PlannedElement::ChoreographicSpin, ElementPlanEvidence::ChoreographicSpin { .. }) => {
            vec![ComponentCode::try_from("ChSp1").unwrap()]
        }
        _ => return Err(unsupported()),
    };
    let panel = PanelRecognition {
        ruleset,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(
            RecognitionDetails::new(codes, target_level, Some(calls)).map_err(|_| unsupported())?,
        ),
    };
    match lookup_base_value(request.scoring_table, &panel, &[]).map_err(E::Scoring)? {
        BaseValueOutcome::Value(v) => Ok(v),
        _ => Err(unsupported()),
    }
}

fn excluded(variant: &ProgramVariant, exclusion: &PersonalExclusion, index: usize) -> bool {
    let jumps: Vec<_> = match &variant.element {
        PlannedElement::Jump(JumpElement::Solo(j)) => vec![j],
        PlannedElement::Jump(JumpElement::Combination(parts) | JumpElement::Sequence(parts)) => {
            parts
                .iter()
                .filter_map(|p| {
                    if let JumpPart::Jump(j) = p {
                        Some(j)
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => vec![],
    };
    match exclusion {
        PersonalExclusion::Variant(i) => *i == index,
        PersonalExclusion::Jump(j) => jumps.contains(&j),
        PersonalExclusion::JumpKind(k) => jumps.iter().any(|j| j.kind == Some(*k)),
        PersonalExclusion::FlyingEntry => {
            matches!(&variant.element, PlannedElement::RegularSpin(s) if s.entry == Some(SpinEntry::Flying))
        }
        PersonalExclusion::SpinKind(k) => {
            matches!(&variant.evidence, ElementPlanEvidence::Spin { spin, .. } if spin.abbreviation.is_some_and(|a| a.kind == *k))
        }
        PersonalExclusion::SpinFeature(f) => {
            matches!(&variant.evidence, ElementPlanEvidence::Spin { spin, .. } if spin.feature_attempts.iter().flatten().any(|a| a.feature == *f))
        }
        PersonalExclusion::Unsupported(_) => false,
    }
}

fn checked(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    indices: &[usize],
    ruleset: RulesetId,
) -> Result<Option<ProgramCandidate>, ProgramOptimizationError> {
    if indices.len() < request.min_elements
        || request
            .must_have
            .iter()
            .any(|group| !indices.iter().any(|i| group.contains(i)))
    {
        return Ok(None);
    }
    let program = ProfileProgram {
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        elements: indices
            .iter()
            .map(|i| request.variants[*i].element.clone())
            .collect(),
    };
    let evidence: Vec<_> = indices
        .iter()
        .map(|i| request.variants[*i].evidence.clone())
        .collect();
    let mut validation = validate_supported_program(registry, &program, &evidence, &request.policy)
        .map_err(ProgramOptimizationError::Profile)?;
    if validation
        .validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation)
    {
        return Ok(None);
    }
    // Execution remains a declared conditional assumption, not an established fact.
    if validation
        .validation
        .findings
        .iter()
        .any(|f| f.rule_id != "spin-execution-recognition")
    {
        return Ok(None);
    }
    let per_element = indices
        .iter()
        .map(|i| score(&request.variants[*i], request, ruleset))
        .collect::<Result<Vec<_>, _>>()?;
    for (index, value) in per_element.iter().enumerate() {
        crate::validation::record_rule(
            &mut validation.applied_rules,
            "catalog-unfactored-base-value",
            Some(index),
            None,
            &value.source,
        );
        if !validation.sources.contains(&value.source) {
            validation.sources.push(value.source.clone());
        }
    }
    let base_hundredths = per_element.iter().map(|v| u32::from(v.hundredths)).sum();
    Ok(Some(ProgramCandidate {
        program,
        variant_indices: indices.to_vec(),
        evidence,
        per_element,
        base_hundredths,
        validation,
    }))
}

/// Optimize a finite ordered program under explicit recognition assumptions.
pub fn optimize_program(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
) -> Result<ProgramOptimizationResult, ProgramOptimizationError> {
    optimize(registry, request, false)
}

fn optimize(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    stop_after_incumbent: bool,
) -> Result<ProgramOptimizationResult, ProgramOptimizationError> {
    use ProgramOptimizationError as E;
    let start = Instant::now();
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?;
    if request.season != profile.season
        || request.max_elements == 0
        || request.max_elements > 10
        || request.min_elements > request.max_elements
        || request.max_solutions == 0
        || request.max_proposals == 0
        || request.max_proposals > 100_000
        || request.variants.len() > 1000
        || request
            .must_have
            .iter()
            .flatten()
            .any(|i| *i >= request.variants.len())
    {
        return Err(E::InvalidRequest(
            "invalid identity, finite bounds or candidate indices",
        ));
    }
    for x in &request.exclusions {
        if matches!(x, PersonalExclusion::Unsupported(_)) {
            return Err(E::UnsupportedRestriction);
        }
        if matches!(x, PersonalExclusion::Variant(i) if *i >= request.variants.len()) {
            return Err(E::InvalidRequest("exclusion index outside domain"));
        }
    }
    let end = request
        .time_limit
        .map(|d| {
            start
                .checked_add(d)
                .ok_or(E::InvalidRequest("unrepresentable time limit"))
        })
        .transpose()?;
    let ruleset = profile.scoring.recognition_ruleset;
    let mut universe = vec![];
    let mut excluded_variants = vec![];
    let mut values = vec![0i32];
    let mut sources = profile.sources.clone();
    // Verify all requested variants before exclusions; unsupported inputs are never hidden.
    for (i, variant) in request.variants.iter().enumerate() {
        if request.variants[..i].contains(variant) {
            return Err(E::InvalidRequest("duplicate candidate variants"));
        }
        let value = score(variant, request, ruleset).map_err(|_| E::UnsupportedVariant(i))?;
        let single = ProfileProgram {
            profile: request.profile.clone(),
            scoring_table: request.scoring_table,
            elements: vec![variant.element.clone()],
        };
        let report = validate_supported_program(
            registry,
            &single,
            std::slice::from_ref(&variant.evidence),
            &request.policy,
        )
        .map_err(E::Profile)?;
        let unresolved: Vec<_> = report
            .validation
            .findings
            .iter()
            .filter(|f| {
                f.kind != FindingKind::Violation && f.rule_id != "spin-execution-recognition"
            })
            .cloned()
            .collect();
        if !unresolved.is_empty() {
            return Err(E::UnresolvedValidation(unresolved));
        }
        for src in report.sources.iter().chain(std::iter::once(&value.source)) {
            if !sources.contains(src) {
                sources.push(src.clone());
            }
        }
        if request.exclusions.iter().any(|x| excluded(variant, x, i)) {
            excluded_variants.push(i);
        } else {
            universe.push(i);
            values.push(i32::from(value.hundredths));
        }
    }
    let mut result = ProgramOptimizationResult { request: request.clone(), universe, excluded_variants,
        unmodeled_variants: "Unlisted variants, Euler consequences, difficult-variation categories, unsupported feature prerequisites, Pair rules and physical skateability are outside this model.",
        equivalence: "Identical ordered candidate variants including evidence; duplicate variants are rejected.",
        proof_scope: "Finite supplied recognized Single Free Skating scenarios and implemented applicable rules only; never full real-world ISU compliance or an official/global maximum.",
        assumptions: "Clean recognition is explicitly assumed for every variant; successful feature qualification is separately assumed where supplied. No execution or official second-half credit is established.",
        sources, applied_rules: vec![], supported_rules: profile.bindings.iter().filter(|b| b.rule != ProfileRule::Scoring).map(|b| profile.binding(b.rule).id).chain(crate::spin_validation::RULE_IDS.iter().copied()).chain(crate::program_validation::RULE_IDS.iter().copied()).chain(["catalog-unfactored-base-value"]).collect(),
        status: OptimizerStatus::Incomplete, enumeration_complete: false, termination_reason: JumpTerminationReason::Interrupted, examined_programs: 0, candidates: vec![] };
    let (mut solver, slots, objective) =
        solver_common::ordered_slots(&values, request.max_elements, request.min_elements);
    let mut deadline = Deadline(end);
    let mut enumerating = false;
    loop {
        if end.is_some_and(|d| Instant::now() >= d) {
            return Ok(result);
        }
        if result.examined_programs >= request.max_proposals {
            result.termination_reason = JumpTerminationReason::SearchLimit;
            return Ok(result);
        }
        let mut brancher = solver.default_brancher();
        let mut resolver = ResolutionResolver::default();
        let outcome = solver.optimise(
            &mut brancher,
            &mut deadline,
            &mut resolver,
            LinearSatUnsat::new(
                OptimisationDirection::Maximise,
                objective,
                |_: &Solver, _: SolutionReference, _: &DefaultBrancher, _: &ResolutionResolver| {
                    if stop_after_incumbent {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            ),
        );
        let status = solver_common::status(&outcome);
        let solution = match outcome {
            OptimisationResult::Optimal(s)
            | OptimisationResult::Satisfiable(s)
            | OptimisationResult::Stopped(s, _) => s,
            OptimisationResult::Unsatisfiable => {
                if enumerating {
                    result.enumeration_complete = true;
                    result.termination_reason = JumpTerminationReason::EnumerationExhausted;
                } else {
                    result.status = OptimizerStatus::InfeasibleProven;
                    result.termination_reason = JumpTerminationReason::Infeasible;
                }
                return Ok(result);
            }
            OptimisationResult::Unknown => return Ok(result),
        };
        result.examined_programs += 1;
        let indices: Vec<_> = slots
            .iter()
            .filter_map(|s| {
                let v = solution.get_integer_value(*s);
                (v > 0).then(|| result.universe[v as usize - 1])
            })
            .collect();
        if let Some(candidate) = checked(registry, request, &indices, ruleset)? {
            if candidate.base_hundredths != solution.get_integer_value(objective) as u32 {
                return Err(E::InconsistentSolverResult);
            }
            for src in &candidate.validation.sources {
                if !result.sources.contains(src) {
                    result.sources.push(src.clone());
                }
            }
            for rule in &candidate.validation.applied_rules {
                if !result.applied_rules.contains(rule) {
                    result.applied_rules.push(rule.clone());
                }
            }
            result.candidates.push(candidate);
            if !enumerating {
                result.status = status;
                if status != OptimizerStatus::OptimalProven {
                    return Ok(result);
                }
                if !request.enumerate_equal_optima {
                    result.termination_reason = JumpTerminationReason::EnumerationDisabled;
                    return Ok(result);
                }
                enumerating = true;
                let tag = solver.new_constraint_tag();
                solver
                    .add_constraint(pumpkin_solver::equals(
                        vec![objective],
                        solution.get_integer_value(objective),
                        tag,
                    ))
                    .post();
            }
            if result.candidates.len() >= request.max_solutions {
                result.termination_reason = JumpTerminationReason::SolutionLimit;
                return Ok(result);
            }
        } else if status != OptimizerStatus::OptimalProven {
            return Ok(result);
        }
        let clause = slots
            .iter()
            .map(|&s| predicate!(s != solution.get_integer_value(s)))
            .collect::<Vec<_>>();
        let tag = solver.new_constraint_tag();
        solver.add_clause(clause, tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopped_search_returns_unproven_incumbent() {
        let profile = adult_profile(AdultCategory::Bronze);
        let request = ProgramOptimizationRequest {
            season: profile.season,
            profile: profile.id,
            scoring_table: profile.scoring_table,
            policy: adult_program_policy(AdultCategory::Bronze),
            variants: vec![ProgramVariant {
                element: PlannedElement::Jump(JumpElement::Solo(ListedJump {
                    kind: Some(JumpKind::ToeLoop),
                    rotations: Some(1),
                })),
                evidence: ElementPlanEvidence::Jump {
                    direct_transitions_assumed: false,
                },
                recognition_assumed: true,
            }],
            exclusions: vec![],
            must_have: vec![],
            min_elements: 1,
            max_elements: 1,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 100,
            max_solutions: 10,
            enumerate_equal_optima: true,
            time_limit: None,
        };
        let result = optimize(&ProfileRegistry::adult(), &request, true).unwrap();
        assert_eq!(result.status, OptimizerStatus::FeasibleUnproven);
        assert_eq!(result.candidates.len(), 1);
        assert!(!result.enumeration_complete);
    }
}
