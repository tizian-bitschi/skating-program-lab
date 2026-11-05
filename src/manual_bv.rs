//! Base-value assessment of ordered selections under a single profile and SOV.
use crate::assumed_recognition::{contextual_base, resolve_individual_row, senior_context};
use crate::program_validation::{validate_program_scope, EvaluationScope};
use crate::selection::{clean_jump, selection_shape, Selection};
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBvRequest {
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub submission: SubmissionState,
    /// Exact clean SOV codes or compound codes such as 2T+2Lo and 2F+1A+SEQ.
    pub elements: Vec<AssumedRecognition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualBvError {
    /// The requested code or a component is outside the supported selections.
    Selection(ElementSelectionError),
    /// The selected shape has no supported calculation or lacks a required assumption.
    ContextUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBvElement {
    pub element_index: usize,
    /// The exact row, or the source-backed sum of compound rows, before contextual credit.
    pub table_base: Option<BaseValue>,
    /// Credit under the selected assumptions. No technical panel decision is inferred.
    pub credited_base: Result<BaseValueOutcome, ManualBvError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBvAssessment {
    pub elements: Vec<ManualBvElement>,
    pub total: AssessmentAggregate,
    pub legality: ProgramLegalityReport,
}

/// Assess base value and covered composition rules from exact selections.
/// A calculable value does not make a program legal. GOE and timing are absent.
pub fn assess_manual_bv(
    registry: &ProfileRegistry,
    request: &ManualBvRequest,
) -> Result<ManualBvAssessment, ProfileSelectionError> {
    let selection = Selection::new(registry, &request.profile, request.scoring_table)?;
    let profile = registry.resolve(&request.profile, request.scoring_table)?;
    let mut program = ProfileProgram {
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        elements: Vec::with_capacity(request.elements.len()),
    };
    let mut evidence = Vec::with_capacity(request.elements.len());
    let mut values = Vec::with_capacity(request.elements.len());
    for assumption in &request.elements {
        let (element, data, table_base, restriction) = resolve_selection(
            &selection,
            request.scoring_table,
            profile.scoring.recognition_ruleset,
            assumption,
        );
        program.elements.push(element);
        evidence.push(data);
        values.push((table_base, restriction));
    }
    let policy = selection.policy();
    let scope = EvaluationScope::AssumedRecognition;
    let validation = validate_program_scope(registry, &program, &evidence, policy, scope)?;
    let senior = (profile.binding(ProfileRule::Scoring).mechanism
        == RuleMechanism::SeniorScoringV1)
        .then(|| senior_context(&program.elements, &evidence));
    let senior_jump_limit = senior.is_some()
        && validation.validation.findings.iter().any(|f| {
            f.kind == FindingKind::Violation && f.rule_id == profile.binding(ProfileRule::Count).id
        });
    let senior_compound_limit = senior.is_some()
        && validation.validation.findings.iter().any(|f| {
            f.kind == FindingKind::Violation
                && f.element_index.is_none()
                && [
                    ProfileRule::CompoundCount,
                    ProfileRule::Structure,
                    ProfileRule::Euler,
                ]
                .iter()
                .any(|&rule| f.rule_id == profile.binding(rule).id)
        });
    let elements: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(element_index, (table_base, restriction))| {
            let element = &program.elements[element_index];
            let credited_base = if let Some(mut error) = restriction {
                match &mut error {
                    ElementSelectionError::TypeNotSelectable(finding)
                    | ElementSelectionError::LevelNotSelectable(finding) => {
                        finding.element_index = Some(element_index);
                    }
                    _ => {}
                }
                Err(ManualBvError::Selection(error))
            } else if sequence_assumption_unresolved(element, &request.elements[element_index])
                || (senior_jump_limit && matches!(element, PlannedElement::Jump(_)))
                || (senior_compound_limit
                    && matches!(
                        element,
                        PlannedElement::Jump(
                            JumpElement::Combination(_) | JumpElement::Sequence(_)
                        )
                    ))
            {
                Err(ManualBvError::ContextUnavailable)
            } else {
                table_base
                    .as_ref()
                    .ok_or(ManualBvError::ContextUnavailable)
                    .and_then(|base| {
                        contextual_base(
                            base,
                            senior.as_ref().map(|context| &context[element_index]),
                        )
                        .map_err(|_| ManualBvError::ContextUnavailable)
                    })
            };
            ManualBvElement {
                element_index,
                table_base,
                credited_base,
            }
        })
        .collect();
    let total = crate::assessment::aggregate(elements.iter().map(|element| {
        match element.credited_base.as_ref().ok()? {
            BaseValueOutcome::Value(base) => Some(i64::from(base.hundredths)),
            BaseValueOutcome::Ignored | BaseValueOutcome::NoValue => Some(0),
        }
    }));
    let legality = crate::legality::report_program_legality(
        registry,
        &program,
        &evidence,
        policy,
        request.submission,
        validation,
        scope,
    )?;
    Ok(ManualBvAssessment {
        elements,
        total,
        legality,
    })
}

fn sequence_assumption_unresolved(element: &PlannedElement, choice: &AssumedRecognition) -> bool {
    matches!(element, PlannedElement::Jump(JumpElement::Sequence(_)))
        != choice.direct_transitions_assumed
}

type ResolvedSelection = (
    PlannedElement,
    ElementPlanEvidence,
    Option<BaseValue>,
    Option<ElementSelectionError>,
);

fn resolve_selection(
    selection: &Selection<'_>,
    table: ScoringTableId,
    ruleset: RulesetId,
    assumption: &AssumedRecognition,
) -> ResolvedSelection {
    let code = assumption.code.as_str();
    let jump_evidence = ElementPlanEvidence::Jump {
        direct_transitions_assumed: false,
    };
    if let Some(row) = catalog_row(table, code) {
        let error = selection.check_row(row).err();
        if let Some((_, element)) = selection_shape(row) {
            if let Some((evidence, base)) = resolve_individual_row(row, &element, None) {
                return (element, evidence, Some(base), error);
            }
        }
        return (PlannedElement::Unspecified, jump_evidence, None, error);
    }

    let (components, sequence) = match code.strip_suffix("+SEQ") {
        Some(components) => (components, true),
        None => (code, false),
    };
    if !components.contains('+') {
        return (
            PlannedElement::Unspecified,
            jump_evidence,
            None,
            selection.check(code).err(),
        );
    }
    let mut restriction = None;
    let parts = components
        .split('+')
        .enumerate()
        .map(|(index, component)| {
            if component == "Eu" && catalog_row(table, component).is_some() {
                return JumpPart::Euler;
            }
            let jump = clean_jump(component);
            if let Err(mut error) = selection.check(component) {
                if let ElementSelectionError::TypeNotSelectable(finding) = &mut error {
                    finding.component_index = Some(index);
                }
                restriction.get_or_insert(error);
            } else if jump.is_none() {
                restriction.get_or_insert_with(|| ElementSelectionError::UnsupportedCode {
                    source: table.source_reference(format!("Jump component: {component}")),
                });
            }
            let (kind, rotations) = jump.unzip();
            JumpPart::Jump(ListedJump { kind, rotations })
        })
        .collect();
    let jump = if sequence {
        JumpElement::Sequence(parts)
    } else {
        JumpElement::Combination(parts)
    };
    let element = PlannedElement::Jump(jump);
    let evidence = ElementPlanEvidence::Jump {
        direct_transitions_assumed: sequence && assumption.direct_transitions_assumed,
    };
    let base = assumption
        .resolve(table, &element, ruleset, None)
        .map(|(_, base)| base);
    (element, evidence, base, restriction)
}
