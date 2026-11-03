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

fn inserted_entry(v: &ProgramVariant, ruleset: RulesetId) -> ManualProgramEntry {
    ManualProgramEntry {
        planned: v.element.clone(),
        evidence: v.evidence.clone(),
        scenario: None,
        intended_recognition: Some(PanelRecognition {
            ruleset,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: crate::plan_recognition::intended_details(&v.element, &v.evidence),
        }),
    }
}

fn reindex(entry: &mut ManualProgramEntry, index: usize) {
    for panel in entry
        .intended_recognition
        .iter_mut()
        .chain(entry.scenario.iter_mut().map(|s| &mut s.recognition))
    {
        if panel.planned_element.is_some() {
            panel.planned_element = Some(index);
        }
    }
    if let ElementPlanEvidence::Spin { spin, .. } = &mut entry.evidence {
        if let Some(panel) = &mut spin.panel {
            if panel.planned_element.is_some() {
                panel.planned_element = Some(index);
            }
        }
    }
}

/// Complete a finite, explicitly assumed plan; never establishes official recognition.
pub fn complete_program(
    registry: &ProfileRegistry,
    request: &ProgramCompletionRequest,
) -> Result<ProgramCompletionResult, ProgramCompletionError> {
    use ProgramCompletionError as E;
    let start = Instant::now();
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(E::Profile)?;
    let n = request.original.len();
    if request.season != profile.season
        || n > 10
        || request.max_additions > 10 - n
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
    let conflicts: Vec<_> = request
        .original
        .iter()
        .filter(|e| {
            request
                .global_exclusions
                .iter()
                .any(|x| programs::excluded(&variant(&e.entry), x, usize::MAX))
        })
        .map(|e| e.id.clone())
        .collect();
    if !conflicts.is_empty() {
        return Err(E::GlobalExclusionConflict(conflicts));
    }
    let original_assessment = assess_manual_program(
        registry,
        &manual(
            request,
            request.original.iter().map(|e| e.entry.clone()).collect(),
            SubmissionState::Draft,
        ),
    )
    .map_err(E::Profile)?;
    if original_assessment.status == AssessmentStatus::ConfirmedViolation {
        return Err(E::ImmutableViolation(Box::new(original_assessment)));
    }
    // Missing program roles can be repaired. Missing element recognition/evidence cannot.
    if original_assessment
        .elements
        .iter()
        .any(|e| !matches!(e.intended_base, Ok(BaseValueOutcome::Value(_))))
    {
        return Err(E::IndeterminateOriginal(Box::new(original_assessment)));
    }
    for original in &request.original {
        let v = variant(&original.entry);
        if programs::prepare_variant(
            registry,
            &request.profile,
            request.scoring_table,
            &request.policy,
            &v,
        )
        .is_err()
        {
            return Err(E::IndeterminateOriginal(Box::new(original_assessment)));
        }
    }
    // Exclusions select additions only. Original variants use separate solver identities.
    let mut variants = request.variants.clone();
    let mut exclusions = request.exclusions.clone();
    exclusions.extend(request.global_exclusions.clone());
    for (i, v) in request.variants.iter().enumerate() {
        if exclusions.iter().any(|x| programs::excluded(v, x, i)) {
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
        .map(|v| inserted_entry(v, profile.scoring.recognition_ruleset))
        .collect();
    identities.extend(request.original.iter().map(|e| e.entry.clone()));
    for entry in &mut identities {
        reindex(entry, 0);
    }
    let equivalence_keys = identities
        .iter()
        .map(|e| identities.iter().position(|other| other == e).unwrap())
        .collect();
    let placement = AnchoredPlacement {
        first_anchor: request.variants.len(),
        count: n,
        boundaries: request.insertion.boundaries.clone(),
        absolute,
        equivalence_keys,
    };
    let search = programs::optimize_anchored(registry, &search_request, false, Some(&placement))
        .map_err(E::Optimization)?;
    let mut candidates = vec![];
    for candidate in search.candidates {
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
            let mut entry = if v >= placement.first_anchor {
                let original_index = v - placement.first_anchor;
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
                inserted_entry(&request.variants[v], profile.scoring.recognition_ruleset)
            };
            reindex(&mut entry, index);
            entries.push(entry);
        }
        let assessment = assess_manual_program(
            registry,
            &manual(request, entries, SubmissionState::Complete),
        )
        .map_err(E::Profile)?;
        if assessment.intended_base.exact_hundredths != Some(i64::from(candidate.base_hundredths))
            || assessment.status == AssessmentStatus::ConfirmedViolation
        {
            return Err(E::InconsistentAssessment);
        }
        candidates.push(CompletedProgramCandidate {
            program: candidate.program,
            evidence: candidate.evidence,
            provenance,
            per_element: candidate.per_element,
            existing_base_hundredths,
            added_base_hundredths: candidate.base_hundredths - existing_base_hundredths,
            total_base_hundredths: candidate.base_hundredths,
            assessment,
            validation: candidate.validation,
        });
    }
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
        candidates,
    })
}
