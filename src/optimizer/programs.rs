//! Finite ordered mixed-element optimization using the existing Pumpkin backend.
use super::{
    solver_common::{self, Deadline},
    OptimizerObjective, OptimizerStatus,
};
use crate::program_validation::{
    validate_program_scope, validate_program_scope_with_credit, EvaluationScope,
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

/// The scoring objective selected by the program search entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSearchObjective {
    UnfactoredBaseValueHundredths,
    CreditedBaseValueHundredths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCandidate {
    pub program: ProfileProgram,
    pub variant_indices: Vec<usize>,
    pub evidence: Vec<ElementPlanEvidence>,
    /// Ordered input values before program-context credit.
    pub listed_per_element: Vec<BaseValue>,
    /// Ordered values used for the objective.
    pub per_element: Vec<BaseValue>,
    pub base_hundredths: u32,
    pub validation: EvidenceValidationReport,
}

#[derive(Debug, Clone)]
pub struct ProgramOptimizationResult {
    pub request: ProgramOptimizationRequest,
    pub effective_objective: ProgramSearchObjective,
    pub universe: Vec<usize>,
    pub excluded_variants: Vec<usize>,
    pub unmodeled_variants: &'static str,
    pub equivalence: &'static str,
    pub proof_scope: &'static str,
    pub assumptions: &'static str,
    pub sources: Vec<SourceReference>,
    /// Rules available in the active evaluation scope, whether or not this plan needs them.
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

pub(super) fn excluded_in_scope(
    variant: &ProgramVariant,
    exclusion: &PersonalExclusion,
    index: usize,
    assumed: bool,
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
                || (assumed
                    && matches!(&variant.evidence, ElementPlanEvidence::Spin { spin, .. } if spin.abbreviation.is_some_and(|a| a.entry == SpinEntry::Flying)))
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
    prepare_with_base(
        registry,
        profile_id,
        table,
        policy,
        variant,
        value,
        EvaluationScope::EvidenceBased,
    )
}

/// A fixed original's base comes from its validated manual recognition, not a clean reconstruction.
pub(super) fn prepare_with_base(
    registry: &ProfileRegistry,
    profile_id: &ProfileId,
    table: ScoringTableId,
    policy: &ProgramPolicy,
    variant: &ProgramVariant,
    value: BaseValue,
    scope: EvaluationScope,
) -> Result<PreparedVariant, ProgramOptimizationError> {
    let profile = registry
        .resolve(profile_id, table)
        .map_err(ProgramOptimizationError::Profile)?;
    let program = ProfileProgram {
        profile: profile_id.clone(),
        scoring_table: table,
        elements: vec![variant.element.clone()],
    };
    let report = validate_program_scope(
        registry,
        &program,
        std::slice::from_ref(&variant.evidence),
        policy,
        scope,
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

fn credited_values(
    profile: &RuleProfile,
    program: &ProfileProgram,
    evidence: &[ElementPlanEvidence],
    credit_blocks: &crate::validation::JumpCreditBlocks,
    listed_values: &[BaseValue],
    validation: &mut EvidenceValidationReport,
) -> Result<Vec<BaseValue>, ProgramOptimizationError> {
    crate::assumed_recognition::assumed_credit(
        profile,
        &program.elements,
        evidence,
        credit_blocks,
        listed_values.iter().map(Some),
    )
    .into_iter()
    .zip(listed_values)
    .enumerate()
    .map(|(index, (credit, listed))| {
        let credited = match credit {
            Ok(BaseValueOutcome::Value(value)) => value,
            Err(error) => return Err(ProgramOptimizationError::Scoring(error)),
            _ => {
                return Err(ProgramOptimizationError::Scoring(
                    BaseValueError::Unsupported,
                ))
            }
        };
        // Assumed recognition resolves exact clean table values. The only
        // supported contextual adjustment is a non-increasing repetition credit.
        if credited.hundredths > listed.hundredths {
            return Err(ProgramOptimizationError::InconsistentSolverResult);
        }
        if credited != *listed {
            let repetition = profile.binding(ProfileRule::Repetition);
            crate::validation::record_rule(
                &mut validation.applied_rules,
                repetition.id,
                Some(index),
                None,
                &repetition.source,
            );
            if !validation.sources.contains(&repetition.source) {
                validation.sources.push(repetition.source.clone());
            }
        }
        Ok(credited)
    })
    .collect()
}

#[cfg(test)]
#[path = "proposal_diagnostics.rs"]
mod proposal_diagnostics;

fn checked(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    indices: &[usize],
    universe: &[(usize, BaseValue)],
    raw_upper: u32,
    scope: EvaluationScope,
) -> Result<Option<ProgramCandidate>, ProgramOptimizationError> {
    if indices.len() < request.min_elements
        || request
            .must_have
            .iter()
            .any(|group| !indices.iter().any(|i| group.contains(i)))
    {
        #[cfg(test)]
        proposal_diagnostics::record_rejection(
            proposal_diagnostics::Rejection::ModelRequirements,
            std::iter::empty(),
        );
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
    let (mut validation, credit_blocks) =
        validate_program_scope_with_credit(registry, &program, &evidence, &request.policy, scope)
            .map_err(ProgramOptimizationError::Profile)?;
    let violations = validation
        .validation
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Violation);
    if violations.clone().next().is_some() {
        #[cfg(test)]
        proposal_diagnostics::record_rejection(
            proposal_diagnostics::Rejection::Violation,
            violations,
        );
        return Ok(None);
    }
    // Execution remains a declared conditional assumption, not an established fact.
    let unresolved = validation
        .validation
        .findings
        .iter()
        .filter(|f| f.rule_id != "spin-execution-recognition");
    if unresolved.clone().next().is_some() {
        #[cfg(test)]
        proposal_diagnostics::record_rejection(
            proposal_diagnostics::Rejection::Unresolved,
            unresolved.clone(),
        );
        return if scope == EvaluationScope::AssumedRecognition {
            Err(ProgramOptimizationError::UnresolvedValidation(
                unresolved.cloned().collect(),
            ))
        } else {
            Ok(None)
        };
    }
    let listed_values = indices
        .iter()
        .map(|i| {
            universe[universe
                .binary_search_by_key(i, |(index, _)| *index)
                .expect("solver index belongs to the universe")]
            .1
            .clone()
        })
        .collect::<Vec<_>>();
    if listed_values
        .iter()
        .map(|v| u32::from(v.hundredths))
        .sum::<u32>()
        != raw_upper
    {
        return Err(ProgramOptimizationError::InconsistentSolverResult);
    }
    let per_element = if scope == EvaluationScope::AssumedRecognition {
        let profile = registry
            .resolve(&request.profile, request.scoring_table)
            .map_err(ProgramOptimizationError::Profile)?;
        credited_values(
            profile,
            &program,
            &evidence,
            &credit_blocks,
            &listed_values,
            &mut validation,
        )?
    } else {
        listed_values.clone()
    };
    for (index, value) in listed_values.iter().enumerate() {
        crate::validation::record_rule(
            &mut validation.applied_rules,
            "catalog-unfactored-base-value",
            Some(index),
            None,
            &value.source,
        );
        for source in [&value.source, &per_element[index].source] {
            if !validation.sources.contains(source) {
                validation.sources.push(source.clone());
            }
        }
    }
    // At most eleven u16 catalog values: <=720885, exact in u32.
    let base_hundredths = per_element.iter().map(|v| u32::from(v.hundredths)).sum();
    Ok(Some(ProgramCandidate {
        program,
        variant_indices: indices.to_vec(),
        evidence,
        listed_per_element: listed_values,
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
    optimize_anchored(registry, request, Instant::now(), None, None)
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
    start: Instant,
    anchors: Option<&AnchoredPlacement>,
    assumed_values: Option<&[BaseValue]>,
) -> Result<ProgramOptimizationResult, ProgramOptimizationError> {
    use ProgramOptimizationError as E;
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
    let mut universe_values = vec![];
    let scope = if assumed_values.is_some() {
        EvaluationScope::AssumedRecognition
    } else {
        EvaluationScope::EvidenceBased
    };
    let mut sources = profile.sources.clone();
    // Verify all requested variants before exclusions; unsupported inputs are never hidden.
    for (i, variant) in request.variants.iter().enumerate() {
        if assumed_values.is_none()
            && request.variants[..i].contains(variant)
            && !anchors.is_some_and(|a| i >= a.first_anchor)
        {
            return Err(E::InvalidRequest("duplicate candidate variants"));
        }
        let value = if let Some(values) = assumed_values {
            values[i].clone()
        } else {
            variant_base(request, i, ruleset, anchors).map_err(|_| E::UnsupportedVariant(i))?
        };
        let prepared = prepare_with_base(
            registry,
            &request.profile,
            request.scoring_table,
            &request.policy,
            variant,
            value,
            scope,
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
        if request
            .exclusions
            .iter()
            .any(|x| excluded_in_scope(variant, x, i, assumed_values.is_some()))
        {
            excluded_variants.push(i);
        } else {
            universe.push(i);
            values.push(i32::from(value.hundredths));
            universe_values.push((i, value));
        }
    }
    if assumed_values.is_some() {
        let addition_count = anchors.map_or(request.variants.len(), |a| a.first_anchor);
        for (i, variant) in request.variants[..addition_count].iter().enumerate() {
            if request.variants[..i].contains(variant) {
                return Err(E::InvalidRequest("duplicate candidate variants"));
            }
        }
    }
    let effective_objective = match scope {
        EvaluationScope::AssumedRecognition => ProgramSearchObjective::CreditedBaseValueHundredths,
        EvaluationScope::EvidenceBased => ProgramSearchObjective::UnfactoredBaseValueHundredths,
    };
    let mut result = ProgramOptimizationResult { request: request.clone(), effective_objective, universe, excluded_variants,
        unmodeled_variants: "Unlisted variants, Euler consequences, difficult-variation categories, unsupported feature prerequisites, Pair rules and physical skateability are outside this model.",
        equivalence: "Identical ordered candidate variants including evidence; duplicate variants are rejected.",
        proof_scope: "Finite supplied recognized Single Free Skating scenarios and implemented applicable rules only; never full real-world ISU compliance or an official/global maximum.",
        assumptions: "Clean recognition is explicitly assumed for every variant; successful feature qualification is separately assumed where supplied. No execution or official second-half credit is established.",
        sources, applied_rules: vec![], supported_rules: profile.bindings.iter().filter(|b| b.rule != ProfileRule::Scoring).map(|b| profile.binding(b.rule).id).chain(crate::spin_validation::supported_rules(scope)).chain(crate::program_validation::supported_rules(scope)).chain(["catalog-unfactored-base-value"]).collect(),
        status: OptimizerStatus::Incomplete, enumeration_complete: false, termination_reason: JumpTerminationReason::Interrupted, examined_programs: 0, candidates: vec![] };
    if assumed_values.is_some() {
        result.proof_scope = "Maximum credited BV over the supplied ordered domain and implemented construction rules; no GOE, PCS, official second-half credit, regulatory completeness or global maximum.";
    }
    let mut deadline = Deadline(end);
    #[cfg(test)]
    proposal_diagnostics::record_preflight(start.elapsed());
    if deadline.expired() {
        return Ok(result);
    }
    #[cfg(test)]
    let model_start = Instant::now();
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
    #[cfg(test)]
    proposal_diagnostics::record_model(model_start.elapsed());
    // Pumpkin maximizes the remaining raw table sum. Each credited total is
    // bounded above by that sum, so removing one checked word cannot hide a
    // better plan. Keep searching while a remaining word could improve or tie.
    let mut best = None;
    let mut omitted_ties = false;
    loop {
        if deadline.expired() {
            break;
        }
        if result.examined_programs >= request.max_proposals {
            result.termination_reason = JumpTerminationReason::SearchLimit;
            break;
        }
        let mut brancher = solver.default_brancher();
        let mut resolver = ResolutionResolver::default();
        // Check one assignment at a time until the shared validator accepts one.
        let seek_incumbent = best.is_none();
        #[cfg(test)]
        let solver_start = Instant::now();
        let outcome = solver.optimise(
            &mut brancher,
            &mut deadline,
            &mut resolver,
            LinearSatUnsat::new(
                OptimisationDirection::Maximise,
                objective,
                |_: &Solver, _: SolutionReference, _: &DefaultBrancher, _: &ResolutionResolver| {
                    if seek_incumbent {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            ),
        );
        #[cfg(test)]
        proposal_diagnostics::record_solver(solver_start.elapsed());
        let status = solver_common::status(&outcome);
        let solution = match outcome {
            OptimisationResult::Optimal(s)
            | OptimisationResult::Satisfiable(s)
            | OptimisationResult::Stopped(s, _) => s,
            OptimisationResult::Unsatisfiable => {
                if best.is_some() {
                    result.status = OptimizerStatus::OptimalProven;
                    if omitted_ties {
                        result.termination_reason = JumpTerminationReason::SolutionLimit;
                    } else if request.enumerate_equal_optima {
                        result.enumeration_complete = true;
                        result.termination_reason = JumpTerminationReason::EnumerationExhausted;
                    } else {
                        result.termination_reason = JumpTerminationReason::EnumerationDisabled;
                    }
                } else {
                    result.status = OptimizerStatus::InfeasibleProven;
                    result.termination_reason = JumpTerminationReason::Infeasible;
                }
                return Ok(result);
            }
            OptimisationResult::Unknown => break,
        };
        result.examined_programs += 1;
        let indices: Vec<_> = slots
            .iter()
            .filter_map(|s| {
                let v = solution.get_integer_value(*s);
                (v > 0).then(|| result.universe[v as usize - 1])
            })
            .collect();
        let upper = solution.get_integer_value(objective) as u32;
        #[cfg(test)]
        let check_start = Instant::now();
        let candidate = checked(registry, request, &indices, &universe_values, upper, scope)?;
        #[cfg(test)]
        proposal_diagnostics::record_check(
            check_start.elapsed(),
            start.elapsed(),
            candidate.is_some(),
        );
        if let Some(candidate) = candidate {
            let credited = candidate.base_hundredths;
            if best.is_none_or(|b| credited > b) {
                best = Some(credited);
                result.status = OptimizerStatus::FeasibleUnproven;
                omitted_ties = false;
                result.candidates.clear();
                result.applied_rules.clear();
                // Below this floor a word cannot affect the requested result.
                let floor = credited as i32 + i32::from(!request.enumerate_equal_optima);
                let tag = solver.new_constraint_tag();
                solver.add_clause([predicate!(objective >= floor)], tag);
            }
            if best == Some(credited)
                && (request.enumerate_equal_optima || result.candidates.is_empty())
                && !anchors.is_some_and(|a| {
                    result.candidates.iter().any(|c| {
                        c.variant_indices
                            .iter()
                            .map(|i| a.equivalence_keys[*i])
                            .eq(candidate
                                .variant_indices
                                .iter()
                                .map(|i| a.equivalence_keys[*i]))
                    })
                })
            {
                if result.candidates.len() < request.max_solutions {
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
                } else {
                    // Search may still find a better score and discard these ties.
                    omitted_ties = true;
                }
            }
        }
        if status != OptimizerStatus::OptimalProven && !seek_incumbent {
            break;
        }
        if status == OptimizerStatus::OptimalProven && best == Some(upper) {
            result.status = OptimizerStatus::OptimalProven;
            if !request.enumerate_equal_optima {
                result.termination_reason = JumpTerminationReason::EnumerationDisabled;
                return Ok(result);
            }
            if result.candidates.len() >= request.max_solutions {
                result.termination_reason = JumpTerminationReason::SolutionLimit;
                return Ok(result);
            }
        }
        let clause = slots
            .iter()
            .map(|&s| predicate!(s != solution.get_integer_value(s)))
            .collect::<Vec<_>>();
        let tag = solver.new_constraint_tag();
        solver.add_clause(clause, tag);
    }
    if result.status == OptimizerStatus::FeasibleUnproven && result.candidates.len() > 1 {
        result.candidates.truncate(1);
        result.applied_rules = result.candidates[0].validation.applied_rules.clone();
    }
    Ok(result)
}

/// Optimize the selected SOV variants in the same order as the request domain.
/// Evidence fields are replaced by structural identities derived from the selections.
pub fn optimize_program_with_assumed_recognition(
    registry: &ProfileRegistry,
    request: &ProgramOptimizationRequest,
    recognition: &[AssumedRecognition],
) -> Result<ProgramOptimizationResult, ProgramOptimizationError> {
    let start = Instant::now();
    if request
        .exclusions
        .iter()
        .any(|e| matches!(e, PersonalExclusion::SpinFeature(_)))
    {
        return Err(ProgramOptimizationError::UnsupportedRestriction);
    }
    let mut request = request.clone();
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(ProgramOptimizationError::Profile)?;
    let values = resolve_variants(
        request.scoring_table,
        &mut request.variants,
        recognition,
        profile.scoring.recognition_ruleset,
    )?;
    let mut result = optimize_anchored(registry, &request, start, None, Some(&values))?;
    result.assumptions = ASSUMED_SCOPE;
    Ok(result)
}

pub(super) const ASSUMED_SCOPE: &str =
    "Exact supplied SOV recognition under covered program construction rules. Execution and panel recognition are unverified.";

pub(super) fn resolve_variants(
    table: ScoringTableId,
    variants: &mut [ProgramVariant],
    recognition: &[AssumedRecognition],
    ruleset: RulesetId,
) -> Result<Vec<BaseValue>, ProgramOptimizationError> {
    if variants.len() != recognition.len() {
        return Err(ProgramOptimizationError::InvalidRequest(
            "supply one SOV selection per variant",
        ));
    }
    variants
        .iter_mut()
        .zip(recognition)
        .enumerate()
        .map(|(i, (variant, assumption))| {
            let (evidence, value) = assumption
                .resolve(table, &variant.element, ruleset, Some(&variant.evidence))
                .ok_or(ProgramOptimizationError::UnsupportedVariant(i))?;
            variant.evidence = evidence;
            variant.recognition_assumed = true;
            Ok(value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn bronze_solos() -> ProgramOptimizationRequest {
        let profile = adult_profile(AdultCategory::Bronze);
        ProgramOptimizationRequest {
            season: profile.season,
            profile: profile.id,
            scoring_table: profile.scoring_table,
            policy: adult_program_policy(AdultCategory::Bronze),
            variants: [JumpKind::ToeLoop, JumpKind::Salchow]
                .map(|kind| ProgramVariant {
                    element: PlannedElement::Jump(JumpElement::Solo(ListedJump {
                        kind: Some(kind),
                        rotations: Some(1),
                    })),
                    evidence: ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false,
                    },
                    recognition_assumed: true,
                })
                .to_vec(),
            exclusions: vec![],
            must_have: vec![],
            min_elements: 1,
            max_elements: 1,
            objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
            equivalence: ProgramEquivalence::OrderedVariants,
            max_proposals: 1,
            max_solutions: 10,
            enumerate_equal_optima: true,
            time_limit: None,
        }
    }

    #[test]
    fn proposal_cap_retains_first_verified_adult_program_before_proof() {
        let mut request = bronze_solos();
        let registry = ProfileRegistry::adult();
        let codes = ["1T", "1S"].map(AssumedRecognition::new);
        let stopped =
            optimize_program_with_assumed_recognition(&registry, &request, &codes).unwrap();
        assert_eq!(stopped.status, OptimizerStatus::FeasibleUnproven);
        assert_eq!(
            stopped.termination_reason,
            JumpTerminationReason::SearchLimit
        );
        assert_eq!(stopped.examined_programs, 1);
        assert_eq!(stopped.candidates.len(), 1);
        assert_eq!(stopped.candidates[0].base_hundredths, 40);
        assert!(!stopped.enumeration_complete);

        request.max_proposals = 10;
        let proven =
            optimize_program_with_assumed_recognition(&registry, &request, &codes).unwrap();
        assert_eq!(proven.status, OptimizerStatus::OptimalProven);
        assert!(proven.enumeration_complete);
        let mut words: Vec<_> = proven
            .candidates
            .iter()
            .map(|candidate| (candidate.variant_indices.clone(), candidate.base_hundredths))
            .collect();
        words.sort_unstable();
        assert_eq!(words, [(vec![0], 40), (vec![1], 40)]);
    }

    #[test]
    fn complete_bronze_solo_fixture_matches_literal_ordered_words_and_optima() {
        use std::collections::BTreeMap;

        let mut request = bronze_solos();
        request.min_elements = 1;
        request.max_elements = 3;
        request.max_proposals = 100;
        let registry = ProfileRegistry::adult();
        let codes = ["1T", "1S"].map(AssumedRecognition::new);
        let result =
            optimize_program_with_assumed_recognition(&registry, &request, &codes).unwrap();
        let profile = registry
            .resolve(&request.profile, request.scoring_table)
            .unwrap();
        let universe: Vec<_> = resolve_variants(
            request.scoring_table,
            &mut request.variants,
            &codes,
            profile.scoring.recognition_ruleset,
        )
        .unwrap()
        .into_iter()
        .enumerate()
        .collect();

        // Bronze permits each listed jump at most twice. Both singles are 0.40 BV.
        let mut expected = BTreeMap::new();
        let mut verified = BTreeMap::new();
        for length in 1..=3 {
            let bv = 40 * length as u32;
            for bits in 0..(1 << length) {
                let word: Vec<_> = (0..length).map(|i| (bits >> i) & 1).collect();
                if length < 3 || (bits != 0 && bits != 7) {
                    expected.insert(word.clone(), bv);
                }
                if let Some(candidate) = checked(
                    &registry,
                    &request,
                    &word,
                    &universe,
                    bv,
                    EvaluationScope::AssumedRecognition,
                )
                .unwrap()
                {
                    verified.insert(word, candidate.base_hundredths);
                }
            }
        }
        assert_eq!(verified, expected);
        expected.retain(|_, score| *score == 120);
        assert_eq!(result.status, OptimizerStatus::OptimalProven);
        assert!(result.enumeration_complete);
        let returned = result.candidates.len();
        let actual: BTreeMap<_, _> = result
            .candidates
            .into_iter()
            .map(|c| (c.variant_indices, c.base_hundredths))
            .collect();
        assert_eq!(returned, actual.len(), "duplicate ordered optimum");
        assert_eq!(actual, expected);
    }
}
