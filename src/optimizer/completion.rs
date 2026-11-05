//! Additive immutable-subsequence completion using the shared ordered Pumpkin search.
use super::programs::{self, AnchoredPlacement};
use crate::*;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEntry {
    pub id: String,
    pub entry: ManualProgramEntry,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionInsertionPolicy {
    /// Original boundaries 0..=original.len(); multiple insertions per gap are allowed.
    pub boundaries: Vec<usize>,
    /// (original ID, zero-based completed index). Relative order remains mandatory.
    pub absolute_positions: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct ProgramCompletionRequest {
    pub season: Season,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub policy: ProgramPolicy,
    /// All original entries survive. Unlocked entries may be edited by the caller before submission.
    pub original: Vec<CompletionEntry>,
    pub variants: Vec<ProgramVariant>,
    /// Optional output of #65; must exactly match variants and selected identity.
    pub generated_domain: Option<CandidateGenerationResult>,
    pub exclusions: Vec<PersonalExclusion>,
    /// Element predicates only: Variant indices have no meaning for original anchors.
    pub global_exclusions: Vec<PersonalExclusion>,
    pub insertion: CompletionInsertionPolicy,
    pub min_additions: usize,
    pub max_additions: usize,
    pub objective: OptimizerObjective,
    pub max_proposals: usize,
    pub max_solutions: usize,
    pub enumerate_equal_optima: bool,
    pub time_limit: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionProvenance {
    Original {
        id: String,
        original_index: usize,
        locked: bool,
    },
    Inserted {
        variant_index: usize,
        boundary: usize,
        ordinal: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedProgramCandidate {
    pub program: ProfileProgram,
    pub evidence: Vec<ElementPlanEvidence>,
    pub provenance: Vec<CompletionProvenance>,
    pub per_element: Vec<BaseValue>,
    pub existing_base_hundredths: u32,
    pub added_base_hundredths: u32,
    pub total_base_hundredths: u32,
    pub assessment: ManualProgramAssessment,
    pub validation: EvidenceValidationReport,
}

#[derive(Debug, Clone)]
pub struct ProgramCompletionResult {
    pub request: ProgramCompletionRequest,
    pub original_assessment: ManualProgramAssessment,
    pub status: OptimizerStatus,
    pub enumeration_complete: bool,
    pub termination_reason: JumpTerminationReason,
    pub examined_programs: usize,
    pub sources: Vec<SourceReference>,
    pub applied_rules: Vec<AppliedRule>,
    pub proof_scope: &'static str,
    pub assumptions: &'static str,
    pub unmodeled_variants: &'static str,
    pub supported_rules: Vec<&'static str>,
    /// Generator coverage is independent of proof over supplied variants. None denotes a manual finite domain.
    pub domain_completeness: Option<CandidateDomainCompleteness>,
    pub candidates: Vec<CompletedProgramCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramCompletionError {
    InvalidRequest(&'static str),
    Profile(ProfileSelectionError),
    GlobalExclusionConflict(Vec<String>),
    ImmutableViolation(Box<ManualProgramAssessment>),
    IndeterminateOriginal(Box<ManualProgramAssessment>),
    Optimization(ProgramOptimizationError),
    InconsistentAssessment,
}

fn manual(
    request: &ProgramCompletionRequest,
    entries: Vec<ManualProgramEntry>,
    submission: SubmissionState,
) -> ManualProgramRequest {
    ManualProgramRequest {
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        policy: request.policy.clone(),
        entries,
        submission,
        request_hypothetical_total: false,
        second_half_boundary: None,
    }
}

fn variant(entry: &ManualProgramEntry) -> ProgramVariant {
    ProgramVariant {
        element: entry.planned.clone(),
        evidence: entry.evidence.clone(),
        recognition_assumed: entry.intended_recognition.is_some(),
    }
}

fn inserted_entry(v: &ProgramVariant, ruleset: RulesetId, assumed: bool) -> ManualProgramEntry {
    ManualProgramEntry {
        planned: v.element.clone(),
        evidence: v.evidence.clone(),
        scenario: None,
        intended_recognition: (!assumed).then(|| PanelRecognition {
            ruleset,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: crate::plan_recognition::intended_details(&v.element, &v.evidence),
        }),
    }
}

fn validate_request(
    request: &ProgramCompletionRequest,
    profile: &RuleProfile,
) -> Result<Vec<(usize, usize)>, ProgramCompletionError> {
    use ProgramCompletionError as E;
    let n = request.original.len();
    let limit = crate::profiles::program_search_limit(profile);
    if request.season != profile.season
        || n > limit
        || request.max_additions > limit.saturating_sub(n)
        || request.min_additions > request.max_additions
        || n + request.max_additions == 0
        || request.variants.len() + n > 1000
        || request.insertion.boundaries.iter().any(|b| *b > n)
        || request
            .original
            .iter()
            .enumerate()
            .any(|(i, e)| e.id.is_empty() || request.original[..i].iter().any(|p| p.id == e.id))
    {
        return Err(E::InvalidRequest(
            "invalid identity, bounds, anchor IDs or insertion boundaries",
        ));
    }
    let mut absolute = vec![];
    for (id, index) in &request.insertion.absolute_positions {
        let anchor = request
            .original
            .iter()
            .position(|e| &e.id == id)
            .ok_or(E::InvalidRequest("unknown absolute-position anchor"))?;
        if *index >= n + request.max_additions
            || absolute.iter().any(|(a, p)| *a == anchor || *p == *index)
        {
            return Err(E::InvalidRequest(
                "duplicate or out-of-range absolute position",
            ));
        }
        absolute.push((anchor, *index));
    }
    absolute.sort_unstable();
    if absolute
        .windows(2)
        .any(|p| p[1].1 < p[0].1 + p[1].0 - p[0].0)
    {
        return Err(E::InvalidRequest(
            "absolute positions contradict anchor order",
        ));
    }
    if let Some(domain) = &request.generated_domain {
        if domain.variants != request.variants
            || domain.audit.profile != request.profile
            || domain.audit.scoring_table != request.scoring_table
        {
            return Err(E::InvalidRequest("generated domain does not match request"));
        }
    }
    if request.global_exclusions.iter().any(|x| {
        matches!(
            x,
            PersonalExclusion::Variant(_) | PersonalExclusion::Unsupported(_)
        )
    }) {
        return Err(E::InvalidRequest(
            "global exclusions require supported element predicates",
        ));
    }
    if request
        .exclusions
        .iter()
        .any(|x| matches!(x, PersonalExclusion::Variant(i) if *i >= request.variants.len()))
    {
        return Err(E::InvalidRequest("addition exclusion index outside domain"));
    }
    Ok(absolute)
}

fn check_global_conflicts(
    request: &ProgramCompletionRequest,
    assumed: bool,
) -> Result<(), ProgramCompletionError> {
    use ProgramCompletionError as E;
    let conflicts: Vec<_> = request
        .original
        .iter()
        .filter(|e| {
            request
                .global_exclusions
                .iter()
                .any(|x| programs::excluded_in_scope(&variant(&e.entry), x, usize::MAX, assumed))
        })
        .map(|e| e.id.clone())
        .collect();
    if !conflicts.is_empty() {
        return Err(E::GlobalExclusionConflict(conflicts));
    }
    Ok(())
}

fn prepare_originals(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
    profile: &RuleProfile,
    assumed_values: Option<&[BaseValue]>,
) -> Result<(ManualProgramAssessment, Vec<BaseValue>), ProgramCompletionError> {
    use ProgramCompletionError as E;
    let assessment = crate::assessment::assess_manual_scope(
        registry,
        &manual(
            request,
            request.original.iter().map(|e| e.entry.clone()).collect(),
            SubmissionState::Draft,
        ),
        assumed_values,
    )
    .map_err(E::Profile)?;
    if assessment.status == AssessmentStatus::ConfirmedViolation {
        return Err(E::ImmutableViolation(Box::new(assessment)));
    }
    let unresolved = assessment.validation.validation.findings.iter().any(|f| {
        f.kind != FindingKind::Violation
            && crate::program_validation::candidate_finding_scope(f, profile)
                == crate::program_validation::CandidateFindingScope::Element
    });
    let values: Option<Vec<_>> = assessment
        .elements
        .iter()
        .map(|e| match &e.intended_base {
            Ok(BaseValueOutcome::Value(value)) => Some(value.clone()),
            _ => None,
        })
        .collect();
    match values {
        Some(values) if !unresolved => Ok((assessment, values)),
        _ => Err(E::IndeterminateOriginal(Box::new(assessment))),
    }
}

fn prepare_search(
    request: &ProgramCompletionRequest,
    profile: &RuleProfile,
    original_values: Vec<BaseValue>,
    absolute: Vec<(usize, usize)>,
    start: Instant,
    assumed: bool,
) -> (ProgramOptimizationRequest, AnchoredPlacement) {
    let n = request.original.len();
    // Exclusions select additions only. Original variants use separate solver identities.
    let mut variants = request.variants.clone();
    let mut exclusions = request.exclusions.clone();
    exclusions.extend(request.global_exclusions.clone());
    for (i, v) in request.variants.iter().enumerate() {
        if exclusions
            .iter()
            .any(|x| programs::excluded_in_scope(v, x, i, assumed))
        {
            // Convert predicates to addition-only indices before appending originals.
            if !exclusions.contains(&PersonalExclusion::Variant(i)) {
                exclusions.push(PersonalExclusion::Variant(i));
            }
        }
    }
    exclusions.retain(|x| {
        matches!(
            x,
            PersonalExclusion::Variant(_) | PersonalExclusion::Unsupported(_)
        )
    });
    variants.extend(request.original.iter().map(|e| variant(&e.entry)));
    let remaining_time = request
        .time_limit
        .map(|limit| limit.saturating_sub(start.elapsed()));
    let search_request = ProgramOptimizationRequest {
        season: request.season,
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        policy: request.policy.clone(),
        variants,
        exclusions,
        must_have: vec![],
        min_elements: n + request.min_additions,
        max_elements: n + request.max_additions,
        objective: request.objective,
        equivalence: ProgramEquivalence::OrderedVariants,
        max_proposals: request.max_proposals,
        max_solutions: request.max_solutions,
        enumerate_equal_optima: request.enumerate_equal_optima,
        time_limit: remaining_time,
    };
    let mut identities: Vec<_> = request
        .variants
        .iter()
        .map(|v| inserted_entry(v, profile.scoring.recognition_ruleset, assumed))
        .collect();
    identities.extend(request.original.iter().map(|e| e.entry.clone()));
    for entry in &mut identities {
        entry.reindex_recognition(0);
    }
    let equivalence_keys = identities
        .iter()
        .enumerate()
        .map(|(i, e)| {
            // Original IDs identify fixed placements even when an addition looks identical.
            if i >= request.variants.len() {
                i
            } else {
                identities[..request.variants.len()]
                    .iter()
                    .position(|other| other == e)
                    .unwrap()
            }
        })
        .collect();
    let placement = AnchoredPlacement {
        first_anchor: request.variants.len(),
        count: n,
        boundaries: request.insertion.boundaries.clone(),
        absolute,
        equivalence_keys,
        original_values,
    };
    (search_request, placement)
}

fn completed_candidate(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
    ruleset: RulesetId,
    candidate: ProgramCandidate,
    assumed: bool,
) -> Result<CompletedProgramCandidate, ProgramCompletionError> {
    use ProgramCompletionError as E;
    let mut entries = vec![];
    let mut provenance = vec![];
    let mut boundary = 0;
    let mut ordinal = 0;
    let mut existing_base_hundredths = 0;
    for (index, (&v, value)) in candidate
        .variant_indices
        .iter()
        .zip(&candidate.per_element)
        .enumerate()
    {
        let mut entry = if v >= request.variants.len() {
            let original_index = v - request.variants.len();
            let original = &request.original[original_index];
            existing_base_hundredths += u32::from(value.hundredths);
            provenance.push(CompletionProvenance::Original {
                id: original.id.clone(),
                original_index,
                locked: original.locked,
            });
            boundary += 1;
            ordinal = 0;
            original.entry.clone()
        } else {
            provenance.push(CompletionProvenance::Inserted {
                variant_index: v,
                boundary,
                ordinal,
            });
            ordinal += 1;
            inserted_entry(&request.variants[v], ruleset, assumed)
        };
        entry.reindex_recognition(index);
        entries.push(entry);
    }
    let assessment = crate::assessment::assess_manual_scope(
        registry,
        &manual(request, entries, SubmissionState::Complete),
        assumed.then_some(candidate.per_element.as_slice()),
    )
    .map_err(E::Profile)?;
    if assessment.intended_base.exact_hundredths != Some(i64::from(candidate.base_hundredths))
        || assessment.status == AssessmentStatus::ConfirmedViolation
    {
        return Err(E::InconsistentAssessment);
    }
    Ok(CompletedProgramCandidate {
        program: candidate.program,
        evidence: candidate.evidence,
        provenance,
        per_element: candidate.per_element,
        existing_base_hundredths,
        added_base_hundredths: candidate.base_hundredths - existing_base_hundredths,
        total_base_hundredths: candidate.base_hundredths,
        assessment,
        validation: candidate.validation,
    })
}

/// Complete a finite, explicitly assumed plan; never establishes official recognition.
pub fn complete_program(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
) -> Result<ProgramCompletionResult, ProgramCompletionError> {
    complete(registry, request, None)
}

/// Complete the ordered program with explicit SOV selections for originals and
/// addition variants, each in the same order as the corresponding input list.
pub fn complete_program_with_assumed_recognition(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
    original_recognition: &[AssumedRecognition],
    variant_recognition: &[AssumedRecognition],
) -> Result<ProgramCompletionResult, ProgramCompletionError> {
    use ProgramCompletionError as E;
    if request.original.len() != original_recognition.len() {
        return Err(E::InvalidRequest("supply one SOV selection per original"));
    }
    if request
        .exclusions
        .iter()
        .chain(&request.global_exclusions)
        .any(|e| matches!(e, PersonalExclusion::SpinFeature(_)))
    {
        return Err(E::InvalidRequest(
            "spin feature exclusions require detailed evidence",
        ));
    }
    let original_request = request;
    let mut request = request.clone();
    let ruleset = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?
        .scoring
        .recognition_ruleset;
    let mut values = programs::resolve_variants(
        request.scoring_table,
        &mut request.variants,
        variant_recognition,
        ruleset,
    )
    .map_err(E::Optimization)?;
    for (original, recognition) in request.original.iter_mut().zip(original_recognition) {
        if original.entry.recognitions().next().is_some() {
            return Err(E::InvalidRequest(
                "assumed completion originals cannot carry panel recognition",
            ));
        }
        let (evidence, value) = recognition
            .resolve(
                request.scoring_table,
                &original.entry.planned,
                ruleset,
                Some(&original.entry.evidence),
            )
            .ok_or(E::InvalidRequest(
                "original SOV code is unsupported or does not match the plan",
            ))?;
        original.entry.evidence = evidence;
        values.push(value);
    }
    let mut result = complete(registry, &request, Some(&values))?;
    result.assumptions = programs::ASSUMED_SCOPE;
    result.request = original_request.clone();
    for (entry, original) in result
        .original_assessment
        .request
        .entries
        .iter_mut()
        .zip(&original_request.original)
    {
        *entry = original.entry.clone();
    }
    for candidate in &mut result.candidates {
        for (entry, provenance) in candidate
            .assessment
            .request
            .entries
            .iter_mut()
            .zip(&candidate.provenance)
        {
            if let CompletionProvenance::Original { original_index, .. } = provenance {
                *entry = original_request.original[*original_index].entry.clone();
            }
        }
    }
    Ok(result)
}

fn complete(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
    assumed_values: Option<&[BaseValue]>,
) -> Result<ProgramCompletionResult, ProgramCompletionError> {
    use ProgramCompletionError as E;
    let start = Instant::now();
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?;
    let absolute = validate_request(request, profile)?;
    let assumed = assumed_values.is_some();
    check_global_conflicts(request, assumed)?;
    let original_values = assumed_values.map(|values| &values[request.variants.len()..]);
    let (original_assessment, values) =
        prepare_originals(registry, request, profile, original_values)?;
    let (search_request, placement) =
        prepare_search(request, profile, values, absolute, start, assumed);
    let search = programs::optimize_anchored(
        registry,
        &search_request,
        false,
        Some(&placement),
        assumed_values,
    )
    .map_err(E::Optimization)?;
    let candidates = search
        .candidates
        .into_iter()
        .map(|c| {
            completed_candidate(
                registry,
                request,
                profile.scoring.recognition_ruleset,
                c,
                assumed,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ProgramCompletionResult {
        request: request.clone(),
        original_assessment,
        status: search.status,
        enumeration_complete: search.enumeration_complete,
        termination_reason: search.termination_reason,
        examined_programs: search.examined_programs,
        sources: search.sources,
        applied_rules: search.applied_rules,
        proof_scope: search.proof_scope,
        assumptions: search.assumptions,
        unmodeled_variants: search.unmodeled_variants,
        supported_rules: search.supported_rules,
        domain_completeness: request
            .generated_domain
            .as_ref()
            .map(|d| d.audit.completeness.clone()),
        candidates,
    })
}
