//! Numerical replay of supplied official recognition; no automatic judging.
use super::*;
use crate::{ElementExecutionEvidence, ProfileId, ProfileRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgePanelGrades {
    /// Ordered judge positions. Missing marks are not zero or a smaller panel.
    pub grades: Vec<Option<i8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelGoeScore {
    pub submitted: JudgePanelGrades,
    pub retained_positions: Vec<usize>,
    pub goe_hundredths: i16,
    pub sources: Vec<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelScoreError {
    InvalidGrade,
    UnsupportedPanelConfiguration,
    InvalidInput,
    BaseValue(BaseValueError),
    UnresolvedContext,
    UnsupportedConsequence,
}

fn rule(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Sports Rules — Figure Skating".into(),
        version: "2026-06-12".into(),
        location: location.into(),
    }
}

fn aggregate(row: &TableRow, grades: &JudgePanelGrades) -> Result<PanelGoeScore, PanelScoreError> {
    if grades.grades.is_empty()
        || grades.grades.len() > 9
        || grades.grades.iter().any(Option::is_none)
    {
        return Err(PanelScoreError::UnsupportedPanelConfiguration);
    }
    let columns = row.goe.ok_or(PanelScoreError::UnsupportedConsequence)?;
    let mut values = grades
        .grades
        .iter()
        .enumerate()
        .map(|(position, grade)| {
            let grade = grade.ok_or(PanelScoreError::UnsupportedPanelConfiguration)?;
            if !(-5..=5).contains(&grade) {
                return Err(PanelScoreError::InvalidGrade);
            }
            Ok((columns[(grade + 5) as usize], position))
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort_unstable();
    let retained = if values.len() >= 5 {
        &values[1..values.len() - 1]
    } else {
        &values[..]
    };
    let sum: i32 = retained.iter().map(|(value, _)| i32::from(*value)).sum();
    let count = retained.len() as i32;
    let rounded = sum.signum() * ((sum.abs() * 2 + count) / (count * 2));
    Ok(PanelGoeScore {
        submitted: grades.clone(),
        retained_positions: retained.iter().map(|(_, index)| *index).collect(),
        goe_hundredths: rounded as i16,
        sources: vec![
            row.goe_source("submitted judge GOE columns"),
            rule("p. 9, Rule 353.1(b–e), numerical trimmed mean and two-decimal rounding"),
        ],
    })
}

/// Map each grade through the SOV before trimming and averaging. No grade is inferred.
/// Incomplete arrays are unresolved; a reduced panel must be supplied explicitly.
pub fn aggregate_panel_goe(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    grades: &JudgePanelGrades,
) -> Result<PanelGoeScore, PanelScoreError> {
    match resolve_row(table, recognition, annotations).map_err(PanelScoreError::BaseValue)? {
        ResolvedRow::Value(row) => aggregate(&row, grades),
        _ => Err(PanelScoreError::UnsupportedConsequence),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolElementInput {
    pub recognition: PanelRecognition,
    pub annotations: Vec<ScoringAnnotation>,
    pub judges: Option<JudgePanelGrades>,
    /// Printed x, not independently verified movement timing.
    pub printed_second_half: bool,
    pub source: SourceReference,
    pub execution: Option<ElementExecutionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolElementScore {
    pub table_base: BaseValueOutcome,
    pub credited_base_hundredths: u16,
    pub panel_goe: Option<PanelGoeScore>,
    pub score_hundredths: Option<i32>,
    pub supplied_second_half: bool,
    pub source: SourceReference,
    pub credit_sources: Vec<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolProgramScore {
    /// Every row retains its input location even when its result is unresolved.
    pub sources: Vec<SourceReference>,
    pub elements: Vec<Result<ProtocolElementScore, PanelScoreError>>,
    pub tes_hundredths: Option<i64>,
}

fn replay_row(
    table: ScoringTableId,
    input: &ProtocolElementInput,
    contextual: Option<&PanelRecognition>,
) -> Result<ProtocolElementScore, PanelScoreError> {
    if !input.annotations.is_empty() {
        return Err(PanelScoreError::InvalidInput);
    }
    let panel = contextual.ok_or(PanelScoreError::UnresolvedContext)?;
    if let Some(evidence) = &input.execution {
        // Execution evidence cannot silently override supplied official recognition.
        execution_evidence::resolve(panel, None, Some(evidence))
            .map_err(|_| PanelScoreError::UnsupportedConsequence)?;
    }
    let row = resolve_row(table, panel, &[]).map_err(PanelScoreError::BaseValue)?;
    let mut table_panel = panel.clone();
    if let Some(details) = panel.details.as_ref().filter(|d| {
        d.calls()
            .is_some_and(|calls| calls.iter().any(|call| call.call == PanelCall::Repetition))
    }) {
        table_panel.details =
            Some(without_repetition(details).map_err(PanelScoreError::BaseValue)?);
    }
    let table_base =
        lookup_base_value(table, &table_panel, &[]).map_err(PanelScoreError::BaseValue)?;
    let (base, goe, credit_sources) = match row {
        ResolvedRow::Value(row) => {
            if input.printed_second_half
                && !panel
                    .details
                    .as_ref()
                    .is_some_and(|d| d.components()[0].as_str().starts_with(char::is_numeric))
            {
                return Err(PanelScoreError::InvalidInput);
            }
            let goe = input
                .judges
                .as_ref()
                .map(|grades| aggregate(&row, grades))
                .transpose()?;
            let base = if input.printed_second_half {
                factored_base(row.hundredths)
            } else {
                row.hundredths
            };
            let mut sources = vec![row.source("credited BASE")];
            if input.printed_second_half {
                sources.push(rule(
                    "pp. 9–10, Rule 353.1(h)(iii); supplied printed x, start time unverified",
                ));
            }
            (base, goe, sources)
        }
        ResolvedRow::NoValue | ResolvedRow::Ignored => {
            if input
                .judges
                .as_ref()
                .is_some_and(|g| g.grades.iter().any(Option::is_some))
            {
                return Err(PanelScoreError::InvalidInput);
            }
            (0, None, vec![input.source.clone()])
        }
    };
    let zero = matches!(
        table_base,
        BaseValueOutcome::Ignored | BaseValueOutcome::NoValue
    );
    let total = if zero {
        Some(0)
    } else {
        goe.as_ref()
            .map(|g| i32::from(base) + i32::from(g.goe_hundredths))
    };
    Ok(ProtocolElementScore {
        table_base,
        credited_base_hundredths: base,
        panel_goe: goe,
        score_hundredths: total,
        supplied_second_half: input.printed_second_half,
        source: input.source.clone(),
        credit_sources,
    })
}

/// Replay a complete ordered list of supplied recognition. This is not plan legality.
/// TES excludes PCS and deductions; printed x never verifies actual start time.
pub fn score_protocol_program(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    elements: &[ProtocolElementInput],
) -> Result<ProtocolProgramScore, crate::ProfileSelectionError> {
    let selected = registry.resolve(profile, table)?;
    let panels: Vec<_> = elements
        .iter()
        .map(|e| {
            (e.recognition.ruleset == selected.scoring.recognition_ruleset)
                .then(|| e.recognition.clone())
        })
        .collect();
    // Preserve printed REP and deletions. Context assessment checks eligibility;
    // deriving replacement calls here would change the supplied official record.
    let contextual = panels;
    let results: Vec<_> = elements
        .iter()
        .zip(&contextual)
        .map(|(input, panel)| replay_row(table, input, panel.as_ref()))
        .collect();
    let tes = results
        .iter()
        .map(|r| r.as_ref().ok()?.score_hundredths.map(i64::from))
        .sum();
    Ok(ProtocolProgramScore {
        sources: elements.iter().map(|e| e.source.clone()).collect(),
        elements: results,
        tes_hundredths: tes,
    })
}
