//! Hypothetical final GOE for an already selected program.
use crate::*;
use std::collections::BTreeMap;

/// Optional changes to the hypothetical scoring inputs at a zero-based index.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GoeScenarioOverride {
    /// The uniform grade applies if this is absent.
    pub final_goe: Option<i8>,
    /// Supplied calls and outcomes. Recognized components and level must match the selection.
    pub recognition: Option<PanelRecognition>,
    pub fall: Option<bool>,
    pub execution: Option<ElementExecutionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoeScenarioElementError {
    Base(ManualBvError),
    RecognitionMismatch,
    ContextUnavailable,
    Scoring(ProgramScoreError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoeScenarioPreview {
    pub scoring_table: ScoringTableId,
    pub ruleset: RulesetId,
    pub second_half_boundary: Option<usize>,
    /// In selection order. Ok(None) means a grade was not supplied.
    pub elements: Vec<Result<Option<ProgramElementScore>, GoeScenarioElementError>>,
    /// Absent if no element received a grade. Explicit ignored and no-value
    /// outcomes contribute zero and remain visible in the element results.
    pub total: Option<AssessmentAggregate>,
}

/// A selected program and its BV assessment, ready for repeated GOE previews.
pub struct GoeScenarioPlan {
    program: ProfileProgram,
    evidence: Vec<ElementPlanEvidence>,
    assessment: ManualBvAssessment,
    ruleset: RulesetId,
    senior: bool,
}

impl GoeScenarioPlan {
    /// Resolve selections, contextual BV and composition once.
    pub fn prepare(
        registry: &ProfileRegistry,
        request: &ManualBvRequest,
    ) -> Result<Self, ProfileSelectionError> {
        let profile = registry.resolve(&request.profile, request.scoring_table)?;
        let (assessment, program, evidence) =
            crate::manual_bv::assess_selections(registry, profile, request)?;
        Ok(Self {
            program,
            evidence,
            assessment,
            ruleset: profile.scoring.recognition_ruleset,
            senior: profile.binding(ProfileRule::Scoring).mechanism
                == RuleMechanism::SeniorScoringV1,
        })
    }

    pub fn assessment(&self) -> &ManualBvAssessment {
        &self.assessment
    }

    fn known_nonjump(&self, index: usize) -> bool {
        !matches!(
            self.program.elements[index],
            PlannedElement::Jump(_) | PlannedElement::Unspecified
        )
    }
}

/// Preview final numeric GOE for an already selected program. An absent uniform
/// grade leaves elements without overrides ungraded. Without supplied
/// recognition or execution evidence the scenario assumes clean execution
/// and no fall, unless an explicit fall assumption is supplied.
///
/// The optional boundary is the first element assumed to be in the second half.
/// It does not establish actual timing. This preview never applies deductions.
pub fn preview_goe_scenario(
    plan: &GoeScenarioPlan,
    uniform_goe: Option<i8>,
    overrides: &BTreeMap<usize, GoeScenarioOverride>,
    second_half_boundary: Option<usize>,
) -> Result<GoeScenarioPreview, ProfileSelectionError> {
    use GoeScenarioElementError as Error;

    let len = plan.program.elements.len();
    if second_half_boundary.is_some_and(|boundary| boundary > len)
        || overrides
            .last_key_value()
            .is_some_and(|(&index, _)| index >= len)
    {
        return Err(ProfileSelectionError {
            profile: plan.program.profile.clone(),
            kind: ProfileErrorKind::Invalid,
            reason: "GOE override index or second-half boundary is outside the program".into(),
        });
    }
    let table = plan.program.scoring_table;
    let panels = scenario_recognition(plan, overrides);
    let bases: Vec<_> = panels
        .iter()
        .map(|panel| {
            lookup_base_value(table, panel.as_ref().map_err(Clone::clone)?, &[])
                .map_err(|error| scoring_error(ElementScoreError::BaseValue(error)))
        })
        .collect();
    let eligibility: Vec<_> = panels
        .iter()
        .zip(&bases)
        .enumerate()
        .map(|(index, (panel, base))| {
            if plan.known_nonjump(index) || matches!(base, Ok(BaseValueOutcome::Ignored)) {
                return Some(false);
            }
            let panel = panel.as_ref().ok()?;
            if panel.outcome == RecognitionOutcome::Unknown {
                return None;
            }
            let details = panel.details.as_ref()?;
            let jump = details
                .components()
                .iter()
                .any(|c| c.as_str().starts_with(char::is_numeric));
            (!jump || matches!(base, Ok(BaseValueOutcome::Value(_)))).then_some(jump)
        })
        .collect();
    let mut has_grade = false;
    let elements: Vec<_> = panels
        .iter()
        .zip(bases)
        .zip(crate::scoring::planning_credit(
            &eligibility,
            second_half_boundary,
        ))
        .enumerate()
        .map(|(index, ((panel, base), second_half))| {
            let inputs = overrides.get(&index);
            let grade = inputs.and_then(|inputs| inputs.final_goe).or(uniform_goe);
            has_grade |= grade.is_some();
            score_scenario_element(table, panel.as_ref(), base, second_half, grade, inputs)
        })
        .collect();
    let total = has_grade.then(|| {
        crate::assessment::aggregate(elements.iter().map(|element| match element {
            Ok(Some(score)) => Some(i64::from(score.score_hundredths)),
            Err(Error::Scoring(ProgramScoreError::Element(
                ElementScoreError::Ignored | ElementScoreError::NoValue,
            ))) => Some(0),
            _ => None,
        }))
    });
    Ok(GoeScenarioPreview {
        scoring_table: table,
        ruleset: plan.ruleset,
        second_half_boundary,
        elements,
        total,
    })
}

fn scenario_recognition(
    plan: &GoeScenarioPlan,
    overrides: &BTreeMap<usize, GoeScenarioOverride>,
) -> Vec<Result<PanelRecognition, GoeScenarioElementError>> {
    use GoeScenarioElementError as Error;

    let ruleset = plan.ruleset;
    let panels: Vec<_> = plan
        .program
        .elements
        .iter()
        .zip(&plan.evidence)
        .zip(&plan.assessment.elements)
        .enumerate()
        .map(|(index, ((planned, evidence), assessment))| {
            let base = assessment
                .credited_base
                .as_ref()
                .map_err(|err| Error::Base(err.clone()));
            if let Some(panel) = overrides.get(&index).and_then(|o| o.recognition.as_ref()) {
                if matches!(base, Err(Error::Base(ManualBvError::Selection { .. })))
                    || panel.outcome == RecognitionOutcome::Recognized
                {
                    base?;
                }
                if panel.ruleset != ruleset
                    || panel.planned_element.is_some_and(|i| i != index)
                    || (panel.outcome == RecognitionOutcome::Recognized
                        && panel.details.is_some()
                        && !crate::plan_recognition::matches_intended(planned, evidence, panel))
                {
                    return Err(Error::RecognitionMismatch);
                }
                return Ok(panel.clone());
            }
            Ok(PanelRecognition {
                ruleset,
                source: None,
                planned_element: Some(index),
                observed_element: None,
                outcome: match base? {
                    BaseValueOutcome::Value(_) => RecognitionOutcome::Recognized,
                    BaseValueOutcome::Ignored => RecognitionOutcome::Ignored,
                    BaseValueOutcome::NoValue => RecognitionOutcome::NoValue,
                },
                details: crate::plan_recognition::intended_details(planned, evidence),
            })
        })
        .collect();
    if !plan.senior {
        return panels;
    }
    let contextual = crate::senior::repetition_context_with_plan(
        &panels
            .iter()
            .map(|p| p.as_ref().ok().cloned())
            .collect::<Vec<_>>(),
        |index| plan.known_nonjump(index),
    );
    panels
        .into_iter()
        .zip(contextual)
        .map(|(panel, contextual)| panel.and(contextual.ok_or(Error::ContextUnavailable)))
        .collect()
}

fn scoring_error(error: ElementScoreError) -> GoeScenarioElementError {
    GoeScenarioElementError::Scoring(ProgramScoreError::Element(error))
}

fn score_scenario_element(
    table: ScoringTableId,
    panel: Result<&PanelRecognition, &GoeScenarioElementError>,
    base: Result<BaseValueOutcome, GoeScenarioElementError>,
    second_half: Result<bool, ProgramScoreError>,
    grade: Option<i8>,
    inputs: Option<&GoeScenarioOverride>,
) -> Result<Option<ProgramElementScore>, GoeScenarioElementError> {
    if let Some(grade) = grade {
        crate::scoring::validate_final_goe(grade).map_err(scoring_error)?;
    }
    match base? {
        BaseValueOutcome::Ignored => return Err(scoring_error(ElementScoreError::Ignored)),
        BaseValueOutcome::NoValue => return Err(scoring_error(ElementScoreError::NoValue)),
        BaseValueOutcome::Value(_) => {}
    }
    let Some(grade) = grade else { return Ok(None) };
    let panel = panel.map_err(Clone::clone)?;
    let fall = inputs.and_then(|inputs| inputs.fall).or_else(|| {
        inputs
            .is_none_or(|inputs| inputs.recognition.is_none() && inputs.execution.is_none())
            .then_some(false)
    });
    let score = match inputs.and_then(|inputs| inputs.execution.as_ref()) {
        Some(execution) => score_element_with_execution(table, panel, &[], grade, fall, execution),
        None => score_element(table, panel, &[], grade, fall),
    }
    .map_err(scoring_error)?;
    Ok(Some(crate::scoring::apply_credit(
        score,
        second_half.map_err(GoeScenarioElementError::Scoring)?,
    )))
}
