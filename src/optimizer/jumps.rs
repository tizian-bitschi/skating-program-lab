//! Bounded, category-aware jump-only optimization.
use super::*;
use crate::{Finding, JumpPart, RotationPreference, SourceReference};
use std::time::Duration;

mod model;

/// An explicit finite domain. Each allowed jump becomes a solo candidate; compounds
/// must be listed explicitly and may contain only allowed, non-excluded jumps.
#[derive(Debug, Clone)]
pub struct JumpOptimizerRequest {
    pub rotation_preference: RotationPreference,
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
    /// Maximum number of solver proposals checked against the shared validator.
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
    UnknownValidation {
        elements: Vec<JumpElement>,
        findings: Vec<Finding>,
    },
    Scoring(BaseValueError),
    InconsistentSolverResult,
}

pub(super) fn jump_code(jump: &ListedJump) -> Result<ComponentCode, JumpOptimizerError> {
    if !jump.rotations.is_some_and(|n| (1..=2).contains(&n)) {
        return Err(JumpOptimizerError::UnsupportedVariant);
    }
    crate::plan_recognition::jump_component(jump).ok_or(JumpOptimizerError::UnsupportedVariant)
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

#[cfg(test)]
fn checked_candidate(
    request: &JumpOptimizerRequest,
    elements: &[JumpElement],
) -> Result<Option<ReferenceCandidate>, JumpOptimizerError> {
    checked_profile_candidate(request, elements, &crate::adult_profile(request.category))
}

fn checked_profile_candidate(
    request: &JumpOptimizerRequest,
    elements: &[JumpElement],
    profile: &crate::RuleProfile,
) -> Result<Option<ReferenceCandidate>, JumpOptimizerError> {
    if !request_allows(request, elements)? {
        return Ok(None);
    }
    let mut program = Program {
        season: Season {
            start_year: 2026,
            end_year: 2027,
        },
        category: request.category,
        ruleset: request.ruleset,
        scoring_table: request.scoring_table,
        sources: profile.jump_sources(),
        elements: elements.iter().cloned().map(PlannedElement::Jump).collect(),
    };
    let validation = crate::validation::validate_elements(&program.elements, profile);
    if validation
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::Violation)
    {
        return Ok(None);
    }
    if !validation.findings.is_empty() {
        return Err(JumpOptimizerError::UnknownValidation {
            elements: elements.to_vec(),
            findings: validation.findings,
        });
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

fn request_allows(
    request: &JumpOptimizerRequest,
    elements: &[JumpElement],
) -> Result<bool, JumpOptimizerError> {
    if elements.len() > request.max_elements || (request.require_nonempty && elements.is_empty()) {
        return Ok(false);
    }
    for element in elements {
        if crate::rotation::conflict(
            &PlannedElement::Jump(element.clone()),
            0,
            request.rotation_preference,
        )
        .is_some()
        {
            return Ok(false);
        }
        if matches!(element, JumpElement::Combination(_)) && !request.combinations.contains(element)
        {
            return Ok(false);
        }
        for jump in parts(element)? {
            if !request.allowed_jumps.contains(jump)
                || request.exclusions.contains(jump)
                || jump
                    .kind
                    .is_some_and(|kind| request.excluded_kinds.contains(&kind))
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Optimize only the supplied finite jump domain, never a complete Free Skating program.
pub fn optimize_jumps(
    request: &JumpOptimizerRequest,
) -> Result<JumpOptimizerResult, JumpOptimizerError> {
    model::optimize(request)
}

/// The same finite-domain solver controls as the legacy request, with profile selection.
#[derive(Debug, Clone)]
pub struct ProfileJumpOptimizerRequest {
    pub rotation_preference: RotationPreference,
    pub profile: crate::ProfileId,
    pub scoring_table: ScoringTableId,
    pub allowed_jumps: Vec<ListedJump>,
    pub combinations: Vec<JumpElement>,
    pub exclusions: Vec<ListedJump>,
    pub excluded_kinds: Vec<JumpKind>,
    pub max_elements: usize,
    pub require_nonempty: bool,
    pub max_search_nodes: usize,
    pub enumerate_equal_optima: bool,
    pub max_solutions: usize,
    pub time_limit: Option<Duration>,
}

impl ProfileJumpOptimizerRequest {
    fn search_controls(&self, profile: &crate::RuleProfile) -> JumpOptimizerRequest {
        // The internal legacy carrier supplies domain controls only; policy comes from profile.
        JumpOptimizerRequest {
            rotation_preference: self.rotation_preference,
            category: AdultCategory::Bronze,
            ruleset: profile.scoring.recognition_ruleset,
            scoring_table: self.scoring_table,
            allowed_jumps: self.allowed_jumps.clone(),
            combinations: self.combinations.clone(),
            exclusions: self.exclusions.clone(),
            excluded_kinds: self.excluded_kinds.clone(),
            max_elements: self.max_elements,
            require_nonempty: self.require_nonempty,
            max_search_nodes: self.max_search_nodes,
            enumerate_equal_optima: self.enumerate_equal_optima,
            max_solutions: self.max_solutions,
            time_limit: self.time_limit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileJumpCandidate {
    pub program: crate::ProfileProgram,
    pub base_hundredths: u16,
    pub coverage: Coverage,
}

#[derive(Debug, Clone)]
pub struct ProfileJumpOptimizerResult {
    pub request: ProfileJumpOptimizerRequest,
    pub universe: Vec<JumpElement>,
    pub omitted_variants: &'static str,
    pub equivalence: &'static str,
    pub proof_scope: &'static str,
    pub objective: OptimizerObjective,
    pub implemented_rules: Vec<&'static str>,
    pub sources: Vec<SourceReference>,
    pub examined_subsections: usize,
    pub status: OptimizerStatus,
    pub enumeration_complete: bool,
    pub termination_reason: JumpTerminationReason,
    pub candidates: Vec<ProfileJumpCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileJumpOptimizerError {
    Selection(crate::ProfileSelectionError),
    Search {
        profile: crate::ProfileId,
        reason: JumpOptimizerError,
    },
}

pub fn optimize_profile_jumps(
    registry: &crate::ProfileRegistry,
    request: &ProfileJumpOptimizerRequest,
) -> Result<ProfileJumpOptimizerResult, ProfileJumpOptimizerError> {
    let profile = registry
        .resolve(&request.profile, request.scoring_table)
        .map_err(ProfileJumpOptimizerError::Selection)?;
    let controls = request.search_controls(profile);
    let result = model::optimize_profile(&controls, profile).map_err(|reason| {
        ProfileJumpOptimizerError::Search {
            profile: profile.id.clone(),
            reason,
        }
    })?;
    Ok(ProfileJumpOptimizerResult {
        request: request.clone(),
        universe: result.universe,
        omitted_variants: result.omitted_variants,
        equivalence: result.equivalence,
        proof_scope: result.proof_scope,
        objective: result.objective,
        implemented_rules: result.implemented_rules,
        sources: result.sources,
        examined_subsections: result.examined_subsections,
        status: result.status,
        enumeration_complete: result.enumeration_complete,
        termination_reason: result.termination_reason,
        candidates: result
            .candidates
            .into_iter()
            .map(|candidate| ProfileJumpCandidate {
                program: crate::ProfileProgram {
                    profile: profile.id.clone(),
                    scoring_table: request.scoring_table,
                    elements: candidate.program.elements,
                },
                base_hundredths: candidate.base_hundredths,
                coverage: candidate.coverage,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_adapter_preserves_both_rotation_preferences() {
        let profile = crate::adult_profile(AdultCategory::Silver);
        for preference in [
            RotationPreference::SameDirection,
            RotationPreference::AllowMixedDirections,
        ] {
            let request = ProfileJumpOptimizerRequest {
                rotation_preference: preference,
                profile: profile.id.clone(),
                scoring_table: profile.scoring_table,
                allowed_jumps: vec![],
                combinations: vec![],
                exclusions: vec![],
                excluded_kinds: vec![],
                max_elements: 4,
                require_nonempty: false,
                max_search_nodes: 100,
                enumerate_equal_optima: false,
                max_solutions: 1,
                time_limit: None,
            };
            assert_eq!(
                request.search_controls(&profile).rotation_preference,
                preference
            );
        }
    }

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
        let universe = [JumpElement::Solo(a.clone()), JumpElement::Solo(lo.clone())];
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
                rotation_preference: RotationPreference::default(),
                category,
                ruleset: BRONZE_RULESET,
                scoring_table: BRONZE_TABLE,
                allowed_jumps: vec![a.clone(), lo.clone()],
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

    #[test]
    fn candidate_check_enforces_every_request_restriction() {
        let a = ListedJump {
            kind: Some(JumpKind::Axel),
            rotations: Some(1),
        };
        let t = ListedJump {
            kind: Some(JumpKind::ToeLoop),
            rotations: Some(1),
        };
        let pair =
            JumpElement::Combination(vec![JumpPart::Jump(a.clone()), JumpPart::Jump(t.clone())]);
        let mut request = JumpOptimizerRequest {
            rotation_preference: RotationPreference::default(),
            category: AdultCategory::Gold,
            ruleset: BRONZE_RULESET,
            scoring_table: BRONZE_TABLE,
            allowed_jumps: vec![a.clone(), t.clone()],
            combinations: vec![pair.clone()],
            exclusions: vec![],
            excluded_kinds: vec![],
            max_elements: 2,
            require_nonempty: false,
            max_search_nodes: 100,
            enumerate_equal_optima: true,
            max_solutions: 20,
            time_limit: None,
        };
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_some());
        request.allowed_jumps.clear();
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_none());
        request.allowed_jumps = vec![a.clone(), t.clone()];
        request.combinations.clear();
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_none());
        request.combinations.push(pair.clone());
        request.exclusions.push(t.clone());
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_none());
        request.exclusions.clear();
        request.excluded_kinds.push(JumpKind::Axel);
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_none());
        request.excluded_kinds.clear();
        request.max_elements = 0;
        assert!(checked_candidate(&request, std::slice::from_ref(&pair))
            .unwrap()
            .is_none());
        request.require_nonempty = true;
        assert!(checked_candidate(&request, &[]).unwrap().is_none());
    }
}
