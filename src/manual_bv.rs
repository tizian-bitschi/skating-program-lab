//! Base-value assessment of ordered selections under a single profile and SOV.
use crate::assumed_recognition::assumed_credit;
use crate::program_validation::{validate_program_scope_with_credit, EvaluationScope};
use crate::selection::{clean_jump, Selection};
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBvRequest {
    pub rotation_preference: RotationPreference,
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub submission: SubmissionState,
    /// Exact clean SOV codes or compound codes such as 2T+2Lo and 2F+1A+SEQ.
    pub elements: Vec<AssumedRecognition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualBvError {
    /// The requested code or a component is outside the supported selections.
    Selection {
        error: ElementSelectionError,
        component_index: Option<usize>,
    },
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
    pub rotation_conflicts: Vec<RotationPreferenceConflict>,
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
    let profile = registry.resolve(&request.profile, request.scoring_table)?;
    assess_selections(registry, profile, request).map(|(assessment, _, _)| assessment)
}

pub(crate) fn assess_selections(
    registry: &ProfileRegistry,
    profile: &RuleProfile,
    request: &ManualBvRequest,
) -> Result<(ManualBvAssessment, ProfileProgram, Vec<ElementPlanEvidence>), ProfileSelectionError> {
    let selection = Selection::new(registry, &request.profile, request.scoring_table)?;
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
    let (validation, credit_blocks) =
        validate_program_scope_with_credit(registry, &program, &evidence, policy, scope)?;
    let credits = assumed_credit(
        profile,
        &program.elements,
        &evidence,
        &credit_blocks,
        values.iter().map(|(table_base, _)| table_base.as_ref()),
    );
    let elements: Vec<_> = values
        .into_iter()
        .zip(credits)
        .enumerate()
        .map(|(element_index, ((table_base, restriction), credit))| {
            let element = &program.elements[element_index];
            let credited_base = if let Some((mut error, component_index)) = restriction {
                match &mut error {
                    ElementSelectionError::TypeNotSelectable(finding)
                    | ElementSelectionError::LevelNotSelectable(finding) => {
                        finding.element_index = Some(element_index);
                        finding.component_index = component_index;
                    }
                    _ => {}
                }
                Err(ManualBvError::Selection {
                    error,
                    component_index,
                })
            } else if request.elements[element_index].direct_transitions_assumed
                && !matches!(element, PlannedElement::Jump(JumpElement::Sequence(_)))
            {
                Err(ManualBvError::ContextUnavailable)
            } else {
                credit.map_err(|_| ManualBvError::ContextUnavailable)
            };
            ManualBvElement {
                element_index,
                table_base: table_base.or_else(|| {
                    listed_compound_sum(
                        request.scoring_table,
                        &request.elements[element_index].code,
                        element,
                    )
                }),
                credited_base,
            }
        })
        .collect();
    let total = crate::assessment::aggregate(
        elements
            .iter()
            .map(|element| crate::assessment::base_number(element.credited_base.as_ref().ok()?)),
    );
    let legality = crate::legality::report_program_legality(
        registry,
        &program,
        &evidence,
        policy,
        request.submission,
        validation,
        scope,
    )?;
    Ok((
        ManualBvAssessment {
            rotation_conflicts: crate::rotation::conflicts(
                program.elements.iter(),
                request.rotation_preference,
            ),
            elements,
            total,
            legality,
        },
        program,
        evidence,
    ))
}

// Preserve the listed sum even when the compound shape cannot be credited.
fn listed_compound_sum(
    table: ScoringTableId,
    code: &str,
    element: &PlannedElement,
) -> Option<BaseValue> {
    let PlannedElement::Jump(JumpElement::Combination(parts) | JumpElement::Sequence(parts)) =
        element
    else {
        return None;
    };
    let rows: Option<Vec<_>> = parts
        .iter()
        .map(|part| match part {
            JumpPart::Euler => catalog_row(table, "Eu"),
            JumpPart::Jump(jump) => catalog_row(
                table,
                crate::plan_recognition::jump_component(jump)?.as_str(),
            ),
            _ => None,
        })
        .collect();
    let rows = rows?;
    let locations: Vec<_> = rows
        .iter()
        .map(|row| row.source.reference(row.notation).location)
        .collect();
    Some(BaseValue {
        hundredths: rows
            .iter()
            .try_fold(0u16, |sum, row| sum.checked_add(row.base_hundredths))?,
        table,
        notation: code.into(),
        source: table.source_reference(format!("Listed component rows: {}", locations.join("; "))),
    })
}

type ResolvedSelection = (
    PlannedElement,
    ElementPlanEvidence,
    Option<BaseValue>,
    Option<(ElementSelectionError, Option<usize>)>,
);

pub(crate) fn resolve_selection(
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
        return match selection.resolve_row(row) {
            Ok(resolved) => (
                resolved.element,
                resolved.evidence,
                Some(resolved.value),
                resolved.restriction.map(|error| (error, None)),
            ),
            Err(error) => (
                PlannedElement::Unspecified,
                jump_evidence,
                None,
                Some((error, None)),
            ),
        };
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
            selection.check(code).err().map(|error| (error, None)),
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
            if let Err(error) = selection.check(component) {
                restriction.get_or_insert((error, Some(index)));
            } else if jump.is_none() {
                restriction.get_or_insert_with(|| {
                    (
                        ElementSelectionError::UnsupportedCode {
                            source: table.source_reference(format!("Jump component: {component}")),
                        },
                        Some(index),
                    )
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
