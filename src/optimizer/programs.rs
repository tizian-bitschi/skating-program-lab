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
pub(super) fn score(
    variant: &ProgramVariant,
    table: ScoringTableId,
    ruleset: RulesetId,
) -> Result<BaseValue, ProgramOptimizationError> {
    use ProgramOptimizationError as E;
    let unsupported = || E::InvalidRequest("unsupported plan/evidence/recognition variant");
    if !variant.recognition_assumed {
        return Err(unsupported());
    }
    let check_jump = |jump: &ListedJump| {
        if ruleset == RulesetId::IsuSenior2026_27June12 {
            crate::plan_recognition::jump_component(jump).ok_or_else(unsupported)
        } else {
            super::jumps::jump_code(jump).map_err(|_| unsupported())
        }
    };
    // Keep the optimizer's finite clean subset separate from structural encoding.
    match (&variant.element, &variant.evidence) {
        (
            PlannedElement::Jump(jump),
            ElementPlanEvidence::Jump {
                direct_transitions_assumed,
            },
        ) => match jump {
            JumpElement::Solo(j) => {
                check_jump(j)?;
            }
            JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                if !(2..=3).contains(&parts.len())
                    || (matches!(jump, JumpElement::Sequence(_)) && !direct_transitions_assumed)
                {
                    return Err(unsupported());
                }
                for part in parts {
                    let JumpPart::Jump(j) = part else {
                        return Err(unsupported());
                    };
                    check_jump(j)?;
                }
            }
        },
        (PlannedElement::RegularSpin(_), ElementPlanEvidence::Spin { spin, .. })
            if spin.panel.is_some() =>
        {
            return Err(unsupported())
        }
        _ => {}
    }
    let details = crate::plan_recognition::intended_details(&variant.element, &variant.evidence)
        .ok_or_else(unsupported)?;
    let panel = PanelRecognition {
        ruleset,
        source: None,
        planned_element: None,
        observed_element: None,
        outcome: RecognitionOutcome::Recognized,
        details: Some(details),
    };
    match lookup_base_value(table, &panel, &[]).map_err(E::Scoring)? {
        BaseValueOutcome::Value(v) => Ok(v),
        _ => Err(unsupported()),
    }
}

pub(super) fn excluded(
    variant: &ProgramVariant,
    exclusion: &PersonalExclusion,
    index: usize,
) -> bool {
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

/// Normalize only set-valued evidence; position/attempt order remains significant.
pub(super) fn normalize_variant(mut variant: ProgramVariant) -> ProgramVariant {
    if let ElementPlanEvidence::Step {
        qualified_features: Some(features),
        ..
    } = &mut variant.evidence
    {
        features.sort_unstable();
        features.dedup();
    }
    if matches!(
        variant.element,
        PlannedElement::Jump(JumpElement::Solo(_) | JumpElement::Combination(_))
    ) {
        if let ElementPlanEvidence::Jump {
            direct_transitions_assumed,
        } = &mut variant.evidence
        {
            *direct_transitions_assumed = false;
        }
    }
    variant
}

pub(super) struct PreparedVariant {
    pub value: BaseValue,
    pub sources: Vec<SourceReference>,
    pub local_findings: Vec<Finding>,
}

/// Shared preflight for generation and optimizer input. Whole-program obligations
/// remain in the full validator; callers decide how to handle local violations.
pub(super) fn prepare_variant(
    registry: &ProfileRegistry,
    profile_id: &ProfileId,
    table: ScoringTableId,
    policy: &ProgramPolicy,
    variant: &ProgramVariant,
) -> Result<PreparedVariant, ProgramOptimizationError> {
    let profile = registry
        .resolve(profile_id, table)
        .map_err(ProgramOptimizationError::Profile)?;
    let value = score(variant, table, profile.scoring.recognition_ruleset)?;
    prepare_with_base(registry, profile_id, table, policy, variant, value)
}

/// A fixed original's base comes from its validated manual recognition, not a clean reconstruction.
pub(super) fn prepare_with_base(
    registry: &ProfileRegistry,
    profile_id: &ProfileId,
    table: ScoringTableId,
    policy: &ProgramPolicy,
    variant: &ProgramVariant,
    value: BaseValue,
) -> Result<PreparedVariant, ProgramOptimizationError> {
    let profile = registry
        .resolve(profile_id, table)
        .map_err(ProgramOptimizationError::Profile)?;
    let program = ProfileProgram {
        profile: profile_id.clone(),
        scoring_table: table,
        elements: vec![variant.element.clone()],
    };
    let report = validate_supported_program(
        registry,
        &program,
        std::slice::from_ref(&variant.evidence),
        policy,
    )
    .map_err(ProgramOptimizationError::Profile)?;
    let local_findings = report
        .validation
        .findings
        .into_iter()
        .filter(|f| {
            crate::program_validation::candidate_finding_scope(f, profile)
                == crate::program_validation::CandidateFindingScope::Element
        })
        .collect();
    let mut sources = report.sources;
    if !sources.contains(&value.source) {
        sources.push(value.source.clone());
    }
    Ok(PreparedVariant {
        value,
        sources,
        local_findings,
    })
}

fn checked(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    indices: &[usize],
    ruleset: RulesetId,
    anchors: Option<&AnchoredPlacement>,
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
        .map(|i| variant_base(request, *i, ruleset, anchors))
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
    // At most eleven u16 catalog values: <=720885, exact in u32.
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
    optimize_anchored(registry, request, stop_after_incumbent, None)
}

pub(super) struct AnchoredPlacement {
    pub first_anchor: usize,
    pub count: usize,
    pub boundaries: Vec<usize>,
    pub absolute: Vec<(usize, usize)>,
    pub equivalence_keys: Vec<usize>,
    pub original_values: Vec<BaseValue>,
}

fn variant_base(
    request: &ProgramOptimizationRequest,
    index: usize,
    ruleset: RulesetId,
    anchors: Option<&AnchoredPlacement>,
) -> Result<BaseValue, ProgramOptimizationError> {
    if let Some(a) = anchors.filter(|a| index >= a.first_anchor) {
        return Ok(a.original_values[index - a.first_anchor].clone());
    }
    score(&request.variants[index], request.scoring_table, ruleset)
}

pub(super) fn optimize_anchored(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    stop_after_incumbent: bool,
    anchors: Option<&AnchoredPlacement>,
) -> Result<ProgramOptimizationResult, ProgramOptimizationError> {
    use ProgramOptimizationError as E;
    let start = Instant::now();
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?;
    if request.season != profile.season
        || request.max_elements == 0
        || request.max_elements > crate::profiles::program_search_limit(profile)
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
        if request.variants[..i].contains(variant) && !anchors.is_some_and(|a| i >= a.first_anchor)
        {
            return Err(E::InvalidRequest("duplicate candidate variants"));
        }
        let value =
            variant_base(request, i, ruleset, anchors).map_err(|_| E::UnsupportedVariant(i))?;
        let prepared = prepare_with_base(
            registry,
            &request.profile,
            request.scoring_table,
            &request.policy,
            variant,
            value,
        )
        .map_err(|e| match e {
            ProgramOptimizationError::Profile(_) => e,
            _ => E::UnsupportedVariant(i),
        })?;
        let unresolved: Vec<_> = prepared
            .local_findings
            .into_iter()
            .filter(|f| f.kind != FindingKind::Violation)
            .collect();
        if !unresolved.is_empty() {
            return Err(E::UnresolvedValidation(unresolved));
        }
        let value = prepared.value;
        for src in prepared.sources {
            if !sources.contains(&src) {
                sources.push(src);
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
    // Mandatory variants constrain the search as well as the final shared check.
    for group in &request.must_have {
        let clause = slots
            .iter()
            .flat_map(|&slot| {
                result
                    .universe
                    .iter()
                    .enumerate()
                    .filter(|(_, variant)| group.contains(variant))
                    .map(move |(code, _)| predicate!(slot == code as i32 + 1))
            })
            .collect::<Vec<_>>();
        let tag = solver.new_constraint_tag();
        solver.add_clause(clause, tag);
    }
    if let Some(a) = anchors {
        result.assumptions = "Original recognition, calls and qualification assumptions are supplied and retained; additions use explicit clean recognition and supplied feature qualification. No official recognition, execution, GOE or second-half credit is inferred.";
        solver_common::anchor_slots(&mut solver, &slots, &result.universe, a);
    }
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
        if let Some(candidate) = checked(registry, request, &indices, ruleset, anchors)? {
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
            if !anchors.is_some_and(|a| {
                result.candidates.iter().any(|c| {
                    c.variant_indices
                        .iter()
                        .map(|i| a.equivalence_keys[*i])
                        .eq(candidate
                            .variant_indices
                            .iter()
                            .map(|i| a.equivalence_keys[*i]))
                })
            }) {
                result.candidates.push(candidate);
            }
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
