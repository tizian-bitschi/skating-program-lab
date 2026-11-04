//! Source-backed, unfactored base values for a deliberately limited catalog.
use crate::{
    CatalogDiscipline, CatalogFamily, PanelCall, PanelRecognition, RecognitionOutcome,
    ScoringAnnotation, ScoringTableId, SourceReference,
};

mod execution_evidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseValue {
    /// Hundredths of a point, without second-half credit.
    pub hundredths: u16,
    pub table: ScoringTableId,
    pub notation: String,
    pub source: SourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseValueOutcome {
    Value(BaseValue),
    Ignored,
    NoValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseValueError {
    UnknownRecognition,
    UnknownDetails,
    UnknownCalls,
    Unsupported,
}

enum JumpUnitKind {
    Combination,
    Sequence,
}

impl JumpUnitKind {
    fn label(&self) -> &'static str {
        match self {
            Self::Combination => "combination",
            Self::Sequence => "sequence",
        }
    }
}

struct ComponentRow {
    row: &'static crate::CatalogRow,
    counted: bool,
}

fn component_sources(rows: &[ComponentRow], column: &str) -> String {
    rows.iter()
        .map(|component| {
            let source = catalog_source(component.row, column);
            if component.counted {
                source.location
            } else {
                format!(
                    "excluded by supplied * (TPH pp. 2, 29): {}",
                    source.location
                )
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

struct JumpUnitShape {
    kind: JumpUnitKind,
    interrupted: Option<usize>,
}

enum RowOrigin {
    Catalog {
        row: &'static crate::CatalogRow,
        legacy_section: Option<&'static str>,
    },
    JumpUnit {
        rows: Vec<ComponentRow>,
        reference: &'static crate::CatalogRow,
        kind: JumpUnitKind,
        interrupted: Option<usize>,
    },
    Repetition {
        rows: Vec<ComponentRow>,
        reference: &'static crate::CatalogRow,
        interrupted: Option<usize>,
    },
}

struct TableRow {
    hundredths: u16,
    goe: Option<[i16; 11]>,
    table: ScoringTableId,
    notation: String,
    origin: RowOrigin,
}

fn catalog_source(row: &crate::CatalogRow, column: &str) -> SourceReference {
    let mut source = row.source.reference(row.notation);
    source.location.push_str(&format!(", {column}"));
    source
}

impl TableRow {
    fn interruption(&self) -> Option<usize> {
        match self.origin {
            RowOrigin::Catalog { .. } => None,
            RowOrigin::JumpUnit { interrupted, .. } | RowOrigin::Repetition { interrupted, .. } => {
                interrupted
            }
        }
    }
    fn source(&self, column: &str) -> SourceReference {
        match &self.origin {
            RowOrigin::Catalog {
                row,
                legacy_section: Some(section),
            } => self
                .table
                .source_reference(format!("{section}, {column}, row {}", row.notation)),
            RowOrigin::Catalog {
                row,
                legacy_section: None,
            } => catalog_source(row, column),
            RowOrigin::JumpUnit { rows, kind, .. } => {
                let components = component_sources(rows, column);
                self.table.source_reference(format!(
                    "p. 14, remark 4, {} sum; component sources: {components}",
                    kind.label()
                ))
            }
            RowOrigin::Repetition {
                rows, reference, ..
            } => {
                let mut source = catalog_source(reference, column);
                source.location.push_str(&format!(
                    "; p. 14, remark 5, 80% BASE rounded to two decimals; component sources: {}",
                    component_sources(rows, column)
                ));
                source
            }
        }
    }
    fn goe_source(&self, column: &str) -> SourceReference {
        match &self.origin {
            RowOrigin::JumpUnit {
                reference, kind, ..
            } => {
                let mut source = catalog_source(reference, column);
                source
                    .location
                    .push_str(&format!("; p. 14, remark 4, {} reference", kind.label()));
                source
            }
            RowOrigin::Repetition { reference, .. } => {
                let mut source = catalog_source(reference, column);
                source
                    .location
                    .push_str("; p. 14, remark 5, original reference");
                source
            }
            RowOrigin::Catalog { .. } => self.source(column),
        }
    }
    fn base(&self) -> BaseValue {
        BaseValue {
            hundredths: self.hundredths,
            table: self.table,
            notation: self.notation.clone(),
            source: self.source("BASE column"),
        }
    }
}

// Catalog presence alone does not expand the supported jump recognition subset.
fn jump_row(
    table: ScoringTableId,
    notation: &str,
    senior: bool,
) -> Option<&'static crate::CatalogRow> {
    let unmarked = notation.trim_end_matches(['q', '<', '!', 'e']);
    if !(senior
        && unmarked == notation
        && matches!(unmarked.as_bytes().first(), Some(b'3' | b'4' | b'5')))
        && !matches!(
            unmarked,
            "1T" | "1S" | "1Lo" | "1F" | "1Lz" | "1A" | "2T" | "2S" | "2Lo" | "2F" | "2Lz" | "2A"
        )
    {
        return None;
    }
    crate::catalog_row(table, notation)
}

fn single_row(
    table: ScoringTableId,
    details: &crate::RecognitionDetails,
    notation: String,
    senior: bool,
) -> Result<TableRow, BaseValueError> {
    if details.components().len() != 1 {
        return Err(BaseValueError::Unsupported);
    }
    let component = details.components()[0].as_str();
    if details.level().is_none() && (component.ends_with("Sp") || component == "StSq") {
        return Err(BaseValueError::UnknownDetails);
    }
    let row = crate::catalog_row(table, &notation).ok_or(BaseValueError::Unsupported)?;
    if row.discipline != CatalogDiscipline::SingleAndPair || row.level != details.level() {
        return Err(BaseValueError::Unsupported);
    }
    let legacy_section = match row.family {
        CatalogFamily::Jump if details.level().is_none() => {
            jump_row(table, &notation, senior).ok_or(BaseValueError::Unsupported)?;
            Some(match row.source.page {
                2 => "p. 2, Jumps",
                3 => "p. 3, Jumps",
                4 => "p. 4, Jumps",
                _ => return Err(BaseValueError::Unsupported),
            })
        }
        CatalogFamily::Spin
        | CatalogFamily::StepSequence
        | CatalogFamily::ChoreographicSpin
        | CatalogFamily::ChoreographicSequence => match notation.as_str() {
            "CSSp1" => Some("p. 8, Spins"),
            "ChSp1" => Some("p. 14, Choreographic Elements"),
            _ => None,
        },
        _ => return Err(BaseValueError::Unsupported),
    };
    Ok(TableRow {
        hundredths: row.base_hundredths,
        goe: Some(row.goe_hundredths),
        table,
        notation,
        origin: RowOrigin::Catalog {
            row,
            legacy_section,
        },
    })
}

fn repetition_row(
    table: ScoringTableId,
    details: &crate::RecognitionDetails,
    notation: String,
    senior: bool,
) -> Result<ResolvedRow, BaseValueError> {
    if details.level().is_some()
        || !details.components()[0]
            .as_str()
            .starts_with(char::is_numeric)
    {
        return Err(BaseValueError::Unsupported);
    }
    let original = crate::RecognitionDetails::new(
        details.components().to_vec(),
        None,
        Some(
            details
                .calls()
                .ok_or(BaseValueError::UnknownCalls)?
                .iter()
                .filter(|entry| entry.call != PanelCall::Repetition)
                .cloned()
                .collect(),
        ),
    )
    .map_err(|_| BaseValueError::Unsupported)?;
    let original_notation = original.notation().ok_or(BaseValueError::UnknownCalls)?;
    let mut resolved = if original.components().len() == 1 {
        single_row(table, &original, original_notation, senior)?
    } else {
        match jump_unit_row(table, &original, original_notation, senior)? {
            ResolvedRow::Value(row) => row,
            outcome => return Ok(outcome),
        }
    };
    let (rows, reference, interrupted) = match resolved.origin {
        RowOrigin::Catalog { row, .. } => (vec![ComponentRow { row, counted: true }], row, None),
        RowOrigin::JumpUnit {
            rows,
            reference,
            interrupted,
            ..
        } => {
            // TPH p. 33: contextual +REP retains the first jump and stars later jumps.
            if rows
                .iter()
                .filter(|component| component.counted && component.row.notation != "Eu")
                .count()
                != 1
                || !rows[0].counted
                || rows[0].row.notation == "Eu"
            {
                return Err(BaseValueError::Unsupported);
            }
            (rows, reference, interrupted)
        }
        RowOrigin::Repetition { .. } => unreachable!("unrepeated input"),
    };
    resolved.hundredths = ((u32::from(resolved.hundredths) * 8 + 5) / 10) as u16;
    resolved.origin = RowOrigin::Repetition {
        rows,
        reference,
        interrupted,
    };
    resolved.notation = notation;
    Ok(ResolvedRow::Value(resolved))
}

fn component_is_starred(calls: &[crate::RecordedCall], index: usize) -> bool {
    calls.iter().any(|entry| {
        entry.target == crate::CallTarget::Component(index) && entry.call == PanelCall::NotCounted
    })
}

fn jump_unit_kind(details: &crate::RecognitionDetails) -> Result<JumpUnitShape, BaseValueError> {
    let components = details.components();
    if !(2..=4).contains(&components.len()) || details.level().is_some() {
        return Err(BaseValueError::Unsupported);
    }
    let calls = details.calls().ok_or(BaseValueError::UnknownCalls)?;
    let structure: Vec<_> = calls
        .iter()
        .filter(|entry| matches!(entry.call, PanelCall::Sequence | PanelCall::Combination))
        .collect();
    if structure.len() > 1
        || calls.iter().any(|entry| {
            !matches!(entry.target, crate::CallTarget::Component(_))
                && !matches!(entry.call, PanelCall::Sequence | PanelCall::Combination)
        })
    {
        return Err(BaseValueError::Unsupported);
    }
    let sequence = structure
        .first()
        .is_some_and(|entry| entry.call == PanelCall::Sequence);
    let mut interrupted_at = None;
    if let Some(entry) = structure.first() {
        let crate::CallTarget::BetweenComponents(index) = entry.target else {
            return Err(BaseValueError::Unsupported);
        };
        if index < components.len()
            && (index..components.len()).all(|i| component_is_starred(calls, i))
        {
            interrupted_at = Some(index);
        }
        if entry.call == PanelCall::Combination {
            if interrupted_at.is_none() {
                return Err(BaseValueError::Unsupported);
            }
        } else if index != components.len()
            && !matches!(components[index].as_str(), "1A" | "2A")
            && interrupted_at.is_none()
        {
            return Err(BaseValueError::Unsupported);
        }
    }
    let eulers = components
        .iter()
        .filter(|code| code.as_str() == "Eu")
        .count();
    if eulers > 1 || !(2..=3).contains(&(components.len() - eulers)) {
        return Err(BaseValueError::Unsupported);
    }
    let mut later_axel = false;
    for (index, code) in components.iter().enumerate() {
        let code = code.as_str();
        if code == "Eu" {
            if index == 0 || index + 1 == components.len() {
                return Err(BaseValueError::Unsupported);
            }
        } else if index > 0 {
            later_axel |= matches!(code, "1A" | "2A");
            if interrupted_at.is_some_and(|boundary| index >= boundary) {
                continue;
            }
            if components[index - 1].as_str() == "Eu" {
                if !matches!(code, "1S" | "2S" | "1F" | "2F") {
                    return Err(BaseValueError::Unsupported);
                }
            } else if matches!(code, "1A" | "2A") && sequence {
                later_axel = true;
            } else if !matches!(code, "1T" | "2T" | "1Lo" | "2Lo") {
                return Err(BaseValueError::Unsupported);
            }
        }
    }
    if sequence && !later_axel {
        return Err(BaseValueError::Unsupported);
    }
    Ok(JumpUnitShape {
        kind: if sequence {
            JumpUnitKind::Sequence
        } else {
            JumpUnitKind::Combination
        },
        interrupted: interrupted_at,
    })
}

fn jump_unit_row(
    table: ScoringTableId,
    details: &crate::RecognitionDetails,
    notation: String,
    senior: bool,
) -> Result<ResolvedRow, BaseValueError> {
    let shape = jump_unit_kind(details)?;
    let calls = details.calls().ok_or(BaseValueError::UnknownCalls)?;
    let mut rows = Vec::new();
    for (index, code) in details.components().iter().enumerate() {
        let component = crate::RecognitionDetails::new(
            vec![code.clone()],
            None,
            Some(
                calls
                    .iter()
                    .filter(|entry| {
                        entry.target == crate::CallTarget::Component(index)
                            && entry.call != PanelCall::NotCounted
                    })
                    .map(|entry| crate::RecordedCall {
                        target: crate::CallTarget::Component(0),
                        call: entry.call,
                    })
                    .collect(),
            ),
        )
        .map_err(|_| BaseValueError::Unsupported)?;
        let component_notation = component.notation().ok_or(BaseValueError::UnknownCalls)?;
        let row = if component_notation == "Eu" {
            crate::catalog_row(table, "Eu")
        } else {
            jump_row(table, &component_notation, senior)
        }
        .ok_or(BaseValueError::Unsupported)?;
        if code.as_str() == "Eu"
            && calls
                .iter()
                .any(|entry| entry.target == crate::CallTarget::Component(index))
        {
            return Err(BaseValueError::Unsupported);
        }
        rows.push(ComponentRow {
            row,
            counted: !component_is_starred(calls, index),
        });
    }
    // Ties retain the first component, preserving reference selection.
    let reference = rows
        .iter()
        .filter(|component| component.counted && component.row.notation != "Eu")
        .map(|component| component.row)
        .reduce(|first, next| {
            if next.base_hundredths > first.base_hundredths {
                next
            } else {
                first
            }
        });
    let Some(reference) = reference else {
        return Ok(ResolvedRow::NoValue);
    };
    Ok(ResolvedRow::Value(TableRow {
        hundredths: rows
            .iter()
            .filter(|component| component.counted)
            .map(|component| component.row.base_hundredths)
            .sum(),
        goe: Some(reference.goe_hundredths),
        table,
        notation,
        origin: RowOrigin::JumpUnit {
            rows,
            reference,
            kind: shape.kind,
            interrupted: shape.interrupted,
        },
    }))
}

fn table_row(
    table: ScoringTableId,
    recognition: &PanelRecognition,
) -> Result<ResolvedRow, BaseValueError> {
    let senior = recognition.ruleset == crate::RulesetId::IsuSenior2026_27June12;
    let details = recognition
        .details
        .as_ref()
        .ok_or(BaseValueError::UnknownDetails)?;
    let notation = details.notation().ok_or(BaseValueError::UnknownCalls)?;
    let calls = details.calls().ok_or(BaseValueError::UnknownCalls)?;
    if calls
        .iter()
        .any(|entry| entry.call == PanelCall::Repetition)
    {
        repetition_row(table, details, notation, senior)
    } else if details.components().len() > 1 {
        jump_unit_row(table, details, notation, senior)
    } else {
        single_row(table, details, notation, senior).map(ResolvedRow::Value)
    }
}

enum ResolvedRow {
    Value(TableRow),
    Ignored,
    NoValue,
}

fn resolve_row(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
) -> Result<ResolvedRow, BaseValueError> {
    if !annotations.is_empty() {
        return Err(BaseValueError::Unsupported);
    }
    match recognition.outcome {
        RecognitionOutcome::Ignored => Ok(ResolvedRow::Ignored),
        RecognitionOutcome::NoValue => Ok(ResolvedRow::NoValue),
        RecognitionOutcome::Unknown => Err(BaseValueError::UnknownRecognition),
        RecognitionOutcome::Recognized => {
            // TPH Singles p. 2: an explicitly starred whole element has no value.
            // Component deletion inside a combination needs separate reference-GOE rules.
            let starred = recognition.details.as_ref().is_some_and(|details| {
                details.components().len() == 1
                    && details.calls().is_some_and(
                        |calls| matches!(calls, [entry] if entry.call == PanelCall::NotCounted),
                    )
            });
            if starred {
                Ok(ResolvedRow::NoValue)
            } else {
                table_row(table, recognition)
            }
        }
    }
}

/// Look up a supplied panel outcome. This does not establish event eligibility.
pub fn lookup_base_value(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
) -> Result<BaseValueOutcome, BaseValueError> {
    Ok(match resolve_row(table, recognition, annotations)? {
        ResolvedRow::Value(row) => BaseValueOutcome::Value(row.base()),
        ResolvedRow::Ignored => BaseValueOutcome::Ignored,
        ResolvedRow::NoValue => BaseValueOutcome::NoValue,
    })
}

/// A single hypothetical judge's unfactored element score, in hundredths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementScore {
    pub base: BaseValue,
    pub goe: i8,
    pub goe_hundredths: i16,
    pub score_hundredths: i16,
    pub goe_source: SourceReference,
    pub consistency_source: SourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementScoreError {
    BaseValue(BaseValueError),
    Ignored,
    NoValue,
    InvalidGoe,
    UnknownFall,
    InconsistentFall,
    UnsupportedMarks,
}

// 2788 pp. 14, 16: add reductions and cap the starting grade for multiple errors.
// Multiple q signs use their own -3..-4 row, not a separate -2 per q.
struct GoeLimits {
    maximum: i8,
    fall_maximum: i8,
    downgraded: bool,
    handbook_source: bool,
}

fn goe_limits(
    calls: &[crate::RecordedCall],
    execution: Option<&execution_evidence::Adjustment>,
) -> GoeLimits {
    let mut reduction: i16 = 0;
    let mut errors = 0;
    let mut quarters = 0;
    let mut capped = false;
    let mut downgraded = false;
    for entry in calls {
        let minimum = match entry.call {
            PanelCall::Quarter => {
                quarters += 1;
                0
            }
            PanelCall::UnderRotated => 2,
            PanelCall::UnclearEdge => 1,
            PanelCall::WrongEdge => {
                capped = true;
                2
            }
            PanelCall::Downgraded => {
                capped = true;
                downgraded = true;
                3
            }
            PanelCall::ReducedSpinValue
            | PanelCall::Repetition
            | PanelCall::Sequence
            | PanelCall::Combination
            | PanelCall::NotCounted => continue,
        };
        errors += 1;
        reduction += minimum;
    }
    reduction += match quarters {
        0 => 0,
        1 => 2,
        _ => 3,
    };
    if let Some(extra) = execution {
        reduction += extra.minimum_reduction;
        errors += extra.errors;
    }
    let fall_count = execution.map_or(1, |extra| i16::from(extra.fall_count));
    // Judges Handbook pp. 2, 6: any error excludes +5; a single q starts at most +3.
    let mut starting_max = if capped || errors > 1 {
        2
    } else if quarters == 1 {
        3
    } else if errors > 0 {
        4
    } else {
        5
    };
    if let Some(extra) = execution {
        starting_max = starting_max.min(extra.starting_cap);
    }
    GoeLimits {
        maximum: (starting_max - reduction).max(-5) as i8,
        fall_maximum: (starting_max.min(2) - reduction - 5 * fall_count).max(-5) as i8,
        downgraded,
        handbook_source: (errors == 1
            && !capped
            && !execution.is_some_and(|extra| extra.errors > 0))
            || execution.is_some_and(|extra| extra.handbook_source),
    }
}

/// Scores supported single elements with an explicit fall observation.
/// This does not validate event eligibility or other execution errors.
pub fn score_element(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    goe: i8,
    fall: Option<bool>,
) -> Result<ElementScore, ElementScoreError> {
    score_element_internal(table, recognition, annotations, goe, fall, None)
}

/// Score using separately supplied complete observations of additional execution errors.
/// Unknown evidence preserves the existing incomplete outcomes; no panel calls are inferred.
pub fn score_element_with_execution(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    goe: i8,
    fall: Option<bool>,
    execution: &crate::ElementExecutionEvidence,
) -> Result<ElementScore, ElementScoreError> {
    let resolved = execution_evidence::resolve(recognition, fall, Some(execution))?;
    score_element_internal(
        table,
        recognition,
        annotations,
        goe,
        resolved.fall,
        resolved.adjustment.as_ref(),
    )
}

fn score_element_internal(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    goe: i8,
    fall: Option<bool>,
    execution: Option<&execution_evidence::Adjustment>,
) -> Result<ElementScore, ElementScoreError> {
    if !(-5..=5).contains(&goe) {
        return Err(ElementScoreError::InvalidGoe);
    }
    let row =
        match resolve_row(table, recognition, annotations).map_err(ElementScoreError::BaseValue)? {
            ResolvedRow::Ignored => return Err(ElementScoreError::Ignored),
            ResolvedRow::NoValue => return Err(ElementScoreError::NoValue),
            ResolvedRow::Value(row) => row,
        };
    // A structural interruption does not identify its execution error/cumulative GOE cap.
    if row.interruption().is_some_and(|boundary| {
        !execution.is_some_and(|extra| extra.break_positions.contains(&boundary))
    }) {
        return Err(ElementScoreError::UnsupportedMarks);
    }
    let goe_values = row.goe.ok_or(ElementScoreError::UnsupportedMarks)?;
    let calls = recognition
        .details
        .as_ref()
        .and_then(|details| details.calls())
        .ok_or(ElementScoreError::BaseValue(BaseValueError::UnknownCalls))?;
    let limits = goe_limits(calls, execution);
    if goe > limits.maximum {
        return Err(ElementScoreError::UnsupportedMarks);
    }
    match fall {
        None => return Err(ElementScoreError::UnknownFall),
        Some(true) if goe > limits.fall_maximum => return Err(ElementScoreError::InconsistentFall),
        _ => {}
    }
    let goe_hundredths = goe_values[(goe + 5) as usize];
    let score_hundredths = row.hundredths as i16 + goe_hundredths;
    let column = if goe == 0 {
        String::from("BASE (zero adjustment)")
    } else {
        format!("GOE column {goe:+}")
    };
    let goe_source = row.goe_source(&column);
    let base = row.base();
    let consistency_source = SourceReference {
        document: if limits.handbook_source { "ISU Handbook for Referees and Judges" } else { "ISU Communication 2788" }.into(),
        version: "2026-05-19".into(),
        location: if execution.is_some_and(|extra| extra.handbook_source) {
            "pp. 2, 5–6, execution errors, full revolutions, Euler and maximum starting GOE; Communication 2788 pp. 14, 16, call and fall reductions"
        } else if limits.handbook_source {
            "pp. 2, 6, maximum starting GOE for errors and q; Communication 2788 pp. 14, 16, call and fall reductions"
        } else if limits.downgraded {
            "pp. 14, 16, downgrade and fall reductions and maximum starting GOE"
        } else {
            "pp. 14, 16, call and fall reductions and maximum starting GOE"
        }
        .into(),
    };
    Ok(ElementScore {
        base,
        goe,
        goe_hundredths,
        score_hundredths,
        goe_source,
        consistency_source,
    })
}

/// Facts required for the supported, uninterrupted timing scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramTiming {
    pub duration_seconds: u32,
    /// True only when interruption and incomplete-program consequences have been ruled out.
    pub ordinary_timing_confirmed: bool,
    /// True only after checking applicable event notices and Adult exceptions.
    pub second_half_default_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramElement {
    pub recognition: PanelRecognition,
    pub annotations: Vec<ScoringAnnotation>,
    pub goe: i8,
    pub fall: Option<bool>,
    /// Elapsed milliseconds from the program start; required for jumps and late programs.
    pub start_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramElementScore {
    pub unfactored: ElementScore,
    pub second_half: bool,
    pub credited_base_hundredths: u16,
    pub score_hundredths: i16,
    pub second_half_source: Option<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramScoreError {
    Element(ElementScoreError),
    TimingUnknown,
    InvalidTiming,
    ExceptionUnresolved,
    UnsupportedCategory,
    UnknownFall,
    InvalidInput,
    RulesetMismatch,
    TimingConsequenceUnsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduction {
    pub hundredths: u32,
    pub source: SourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramScore {
    pub elements: Vec<Result<ProgramElementScore, ProgramScoreError>>,
    /// None means at least one element is unresolved, not a zero score.
    /// Ignored and no-value outcomes are represented in `elements` and contribute zero.
    pub tes_hundredths: Option<i64>,
    pub fall_deduction: Result<Deduction, ProgramScoreError>,
    pub time_deduction: Result<Deduction, ProgramScoreError>,
}

fn factored_base(hundredths: u16) -> u16 {
    ((u32::from(hundredths) * 11 + 5) / 10) as u16
}

fn time_deduction(
    timing: Option<ProgramTiming>,
    required: u32,
    source: &SourceReference,
) -> Result<Deduction, ProgramScoreError> {
    let timing = timing
        .filter(|t| t.ordinary_timing_confirmed && t.duration_seconds > 0)
        .ok_or(ProgramScoreError::TimingUnknown)?;
    let deviation = required
        .abs_diff(timing.duration_seconds)
        .saturating_sub(10);
    let hundredths = deviation
        .div_ceil(5)
        .checked_mul(100)
        .ok_or(ProgramScoreError::InvalidInput)?;
    Ok(Deduction {
        hundredths,
        source: source.clone(),
    })
}

fn fall_deduction(
    policy: &crate::ProfileScoringPolicy,
    falls_confirmed: bool,
    outside_falls: Option<u32>,
) -> Result<Deduction, ProgramScoreError> {
    let per_fall = policy
        .fall_hundredths
        .ok_or(ProgramScoreError::UnsupportedCategory)?;
    if !falls_confirmed {
        return Err(ProgramScoreError::UnknownFall);
    }
    let count = outside_falls.ok_or(ProgramScoreError::UnknownFall)?;
    let hundredths = count
        .checked_mul(per_fall)
        .ok_or(ProgramScoreError::InvalidInput)?;
    Ok(Deduction {
        hundredths,
        source: policy.fall_source.clone(),
    })
}

// None means that the recognition does not establish whether the element is a jump.
fn is_jump(element: &ProgramElement) -> Option<bool> {
    if element.recognition.outcome != RecognitionOutcome::Recognized {
        return None;
    }
    let details = element.recognition.details.as_ref()?;
    Some(
        details
            .components()
            .iter()
            .any(|c| c.as_str().starts_with(char::is_numeric)),
    )
}

fn element_score(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    element: &ProgramElement,
    timing: Option<ProgramTiming>,
    required: u32,
    observed_fall: Option<bool>,
    execution: Option<&execution_evidence::Adjustment>,
) -> Result<ElementScore, ProgramScoreError> {
    if element.recognition.ruleset != ruleset {
        return Err(ProgramScoreError::RulesetMismatch);
    }
    let timing = timing
        .filter(|t| t.ordinary_timing_confirmed && t.duration_seconds > 0)
        .ok_or(ProgramScoreError::TimingUnknown)?;
    // Rule 502: thirty seconds below the lower limit means no marks.
    if timing.duration_seconds <= required.saturating_sub(40) {
        return Err(ProgramScoreError::TimingConsequenceUnsupported);
    }
    if let Some(start) = element.start_ms {
        if u64::from(start) > u64::from(timing.duration_seconds) * 1000 {
            return Err(ProgramScoreError::InvalidTiming);
        }
        if u64::from(start) > u64::from(required + 10) * 1000 {
            return Err(ProgramScoreError::TimingConsequenceUnsupported);
        }
    } else if timing.duration_seconds > required + 10 {
        return Err(ProgramScoreError::TimingUnknown);
    }
    unfactored_element_score(ruleset, table, element, observed_fall, execution)
}

fn unfactored_element_score(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    element: &ProgramElement,
    observed_fall: Option<bool>,
    execution: Option<&execution_evidence::Adjustment>,
) -> Result<ElementScore, ProgramScoreError> {
    if element.recognition.ruleset != ruleset {
        return Err(ProgramScoreError::RulesetMismatch);
    }
    // A recorded x does not determine eligibility; each caller supplies its own evidence.
    let annotations = if is_jump(element) == Some(true)
        && element.annotations == [ScoringAnnotation::SecondHalfCredit]
    {
        &[][..]
    } else {
        element.annotations.as_slice()
    };
    score_element_internal(
        table,
        &element.recognition,
        annotations,
        element.goe,
        observed_fall,
        execution,
    )
    .map_err(ProgramScoreError::Element)
}

fn sum_element_scores(results: &[Result<ProgramElementScore, ProgramScoreError>]) -> Option<i64> {
    results
        .iter()
        .map(|result| match result {
            Ok(score) => Ok(i64::from(score.score_hundredths)),
            Err(ProgramScoreError::Element(
                ElementScoreError::Ignored | ElementScoreError::NoValue,
            )) => Ok(0),
            Err(error) => Err(error),
        })
        .sum::<Result<i64, _>>()
        .ok()
        .filter(|_| !results.is_empty())
}

fn second_half_selection(
    elements: &[ProgramElement],
    preliminary: &[Result<ElementScore, ProgramScoreError>],
    timing: Option<ProgramTiming>,
    required: u32,
) -> Result<Vec<usize>, ProgramScoreError> {
    let mut indices = Vec::new();
    for (i, (element, score)) in elements.iter().zip(preliminary).enumerate() {
        if matches!(
            score,
            Err(ProgramScoreError::Element(ElementScoreError::Ignored))
        ) {
            continue;
        }
        if matches!(
            score,
            Err(ProgramScoreError::Element(ElementScoreError::NoValue))
        ) {
            let before_half = element
                .start_ms
                .is_some_and(|start| u64::from(start) * 2 < u64::from(required) * 1000);
            let known_nonjump = element.recognition.details.as_ref().is_some_and(|details| {
                details
                    .components()
                    .iter()
                    .all(|c| !c.as_str().starts_with(char::is_numeric))
            });
            if before_half || known_nonjump {
                continue;
            }
            return Err(ProgramScoreError::TimingUnknown);
        }
        if score.is_err() {
            if is_jump(element) != Some(false) {
                return Err(ProgramScoreError::TimingUnknown);
            }
            continue;
        }
        if is_jump(element) != Some(true) {
            continue;
        }
        let timing = timing
            .filter(|t| t.ordinary_timing_confirmed && t.duration_seconds > 0)
            .ok_or(ProgramScoreError::TimingUnknown)?;
        if !timing.second_half_default_confirmed {
            return Err(ProgramScoreError::ExceptionUnresolved);
        }
        let start = element.start_ms.ok_or(ProgramScoreError::TimingUnknown)?;
        if u64::from(start) * 2 >= u64::from(required) * 1000 {
            indices.push(i);
        }
    }
    let first = indices.len().saturating_sub(3);
    Ok(indices[first..].to_vec())
}

fn apply_credit(score: ElementScore, second_half: bool) -> ProgramElementScore {
    let credited_base_hundredths = if second_half {
        factored_base(score.base.hundredths)
    } else {
        score.base.hundredths
    };
    let score_hundredths = credited_base_hundredths as i16 + score.goe_hundredths;
    let second_half_source = second_half.then(|| SourceReference {
        document: "ISU Sports Rules — Figure Skating".into(),
        version: "2026-06-12".into(),
        location: "p. 9, Rule 353.1(h)(iii)".into(),
    });
    ProgramElementScore {
        unfactored: score,
        second_half,
        credited_base_hundredths,
        score_hundredths,
        second_half_source,
    }
}

/// Timestamp-free single-judge what-if calculation. A boundary is an assumption, not timing evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningScore {
    pub second_half_boundary: Option<usize>,
    pub elements: Vec<Result<ProgramElementScore, ProgramScoreError>>,
    /// A hypothetical sum, never an official TES.
    pub hypothetical_hundredths: Option<i64>,
}

/// The boundary is the index of the first element assumed to be in the second half.
pub fn score_planning_scenario(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    elements: &[ProgramElement],
    second_half_boundary: Option<usize>,
) -> Result<PlanningScore, ProgramScoreError> {
    score_planning_internal(ruleset, table, elements, second_half_boundary, None)
}

/// Timestamp-free scenario with execution evidence aligned to the ordered elements.
pub fn score_planning_scenario_with_execution(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    elements: &[ProgramElement],
    second_half_boundary: Option<usize>,
    execution: &[crate::ElementExecutionEvidence],
) -> Result<PlanningScore, ProgramScoreError> {
    score_planning_internal(
        ruleset,
        table,
        elements,
        second_half_boundary,
        Some(execution),
    )
}

fn score_planning_internal(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    elements: &[ProgramElement],
    second_half_boundary: Option<usize>,
    execution: Option<&[crate::ElementExecutionEvidence]>,
) -> Result<PlanningScore, ProgramScoreError> {
    if execution.is_some_and(|values| values.len() != elements.len()) {
        return Err(ProgramScoreError::InvalidInput);
    }
    if second_half_boundary.is_some_and(|boundary| boundary > elements.len()) {
        return Err(ProgramScoreError::InvalidInput);
    }
    let preliminary: Vec<_> = elements
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let resolved = execution_evidence::resolve(
                &element.recognition,
                element.fall,
                execution.map(|values| &values[index]),
            )
            .map_err(ProgramScoreError::Element)?;
            unfactored_element_score(
                ruleset,
                table,
                element,
                resolved.fall,
                resolved.adjustment.as_ref(),
            )
        })
        .collect();
    let mut certain_later = 0usize;
    let mut possible_later = 0usize;
    let mut credit = vec![Ok(false); elements.len()];
    if let Some(boundary) = second_half_boundary {
        for i in (boundary..elements.len()).rev() {
            let element = &elements[i];
            let score = &preliminary[i];
            if matches!(
                score,
                Err(ProgramScoreError::Element(ElementScoreError::Ignored))
            ) {
                continue;
            }
            if score.is_ok() && is_jump(element) == Some(true) {
                credit[i] = if certain_later >= 3 {
                    Ok(false)
                } else if possible_later < 3 {
                    Ok(true)
                } else {
                    Err(ProgramScoreError::TimingUnknown)
                };
                certain_later += 1;
                possible_later += 1;
            } else if score.is_err() && is_jump(element) != Some(false) {
                // A no-value or unresolved element can affect selection unless known to be a non-jump.
                let known_nonjump = element.recognition.details.as_ref().is_some_and(|details| {
                    details
                        .components()
                        .iter()
                        .all(|c| !c.as_str().starts_with(char::is_numeric))
                });
                if !known_nonjump || element.recognition.outcome == RecognitionOutcome::Unknown {
                    possible_later += 1;
                }
            }
        }
    }
    let results: Vec<_> = preliminary
        .into_iter()
        .zip(credit)
        .map(|(score, second_half)| Ok(apply_credit(score?, second_half?)))
        .collect();
    let hypothetical_hundredths = sum_element_scores(&results);
    Ok(PlanningScore {
        second_half_boundary,
        elements: results,
        hypothetical_hundredths,
    })
}

/// Calculate a partial-coverage scenario, not an official panel or segment score.
/// Elements must be in program order; callers supply all recognized elements and falls.
pub fn score_program(
    ruleset: crate::RulesetId,
    category: crate::AdultCategory,
    table: ScoringTableId,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
    timing: Option<ProgramTiming>,
) -> ProgramScore {
    score_program_internal(
        ruleset,
        &crate::adult_profile(category).scoring,
        table,
        elements,
        outside_falls,
        timing,
        None,
    )
}

/// Timed scenario with execution evidence aligned to the ordered elements.
/// A length mismatch rejects the input with InvalidInput and no exact total/deductions.
pub fn score_program_with_execution(
    ruleset: crate::RulesetId,
    category: crate::AdultCategory,
    table: ScoringTableId,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
    timing: Option<ProgramTiming>,
    execution: &[crate::ElementExecutionEvidence],
) -> ProgramScore {
    score_program_internal(
        ruleset,
        &crate::adult_profile(category).scoring,
        table,
        elements,
        outside_falls,
        timing,
        Some(execution),
    )
}

pub(crate) fn score_program_internal(
    ruleset: crate::RulesetId,
    policy: &crate::ProfileScoringPolicy,
    table: ScoringTableId,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
    timing: Option<ProgramTiming>,
    execution: Option<&[crate::ElementExecutionEvidence]>,
) -> ProgramScore {
    if execution.is_some_and(|values| values.len() != elements.len()) {
        return ProgramScore {
            elements: vec![Err(ProgramScoreError::InvalidInput); elements.len()],
            tes_hundredths: None,
            fall_deduction: Err(ProgramScoreError::InvalidInput),
            time_deduction: Err(ProgramScoreError::InvalidInput),
        };
    }
    let required = policy.duration_seconds;
    let mut previous_start = None;
    let mut falls_confirmed = true;
    let preliminary: Vec<_> = elements
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let invalid_timing = if let Some(start) = element.start_ms {
                let invalid = previous_start.is_some_and(|previous| start < previous);
                previous_start =
                    Some(previous_start.map_or(start, |previous: u32| previous.max(start)));
                invalid
            } else {
                false
            };
            let resolved = execution_evidence::resolve(
                &element.recognition,
                element.fall,
                execution.map(|values| &values[index]),
            );
            if resolved.is_err() {
                falls_confirmed = false;
            }
            let resolved = resolved.map_err(ProgramScoreError::Element)?;
            falls_confirmed &= resolved.fall == Some(false)
                || (resolved.fall == Some(true)
                    && element.recognition.outcome == RecognitionOutcome::Recognized);
            if invalid_timing {
                return Err(ProgramScoreError::InvalidTiming);
            }
            element_score(
                ruleset,
                table,
                element,
                timing,
                required,
                resolved.fall,
                resolved.adjustment.as_ref(),
            )
        })
        .collect();
    let selection = second_half_selection(elements, &preliminary, timing, required);
    let results: Vec<_> = preliminary
        .into_iter()
        .enumerate()
        .map(|(i, score)| {
            let score = score?;
            let second_half = if is_jump(&elements[i]) == Some(true) {
                selection.as_ref().map_err(Clone::clone)?.contains(&i)
            } else {
                false
            };
            Ok(apply_credit(score, second_half))
        })
        .collect();
    let tes_hundredths = sum_element_scores(&results).filter(|_| {
        timing.is_some_and(|t| {
            t.ordinary_timing_confirmed && t.duration_seconds > required.saturating_sub(40)
        })
    });
    ProgramScore {
        elements: results,
        tes_hundredths,
        fall_deduction: fall_deduction(policy, falls_confirmed, outside_falls),
        time_deduction: time_deduction(timing, required, &policy.time_deduction_source),
    }
}

#[cfg(test)]
mod program_tests {
    #[test]
    fn factored_base_rounds_each_value_to_hundredths() {
        assert_eq!(super::factored_base(35), 39);
        assert_eq!(super::factored_base(34), 37);
    }

    #[test]
    fn mismatched_recognition_ruleset_prevents_exact_tes() {
        use super::*;
        let element = ProgramElement {
            recognition: PanelRecognition {
                ruleset: crate::RulesetId::TestOther,
                source: None,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Recognized,
                details: None,
            },
            annotations: vec![],
            goe: 0,
            fall: Some(false),
            start_ms: Some(1_000),
        };
        let result = score_program(
            crate::RulesetId::IsuAdult2026_27July16,
            crate::AdultCategory::Bronze,
            ScoringTableId::Isu2786June2_2026,
            &[element],
            Some(0),
            Some(ProgramTiming {
                duration_seconds: 100,
                ordinary_timing_confirmed: true,
                second_half_default_confirmed: true,
            }),
        );
        assert_eq!(result.elements[0], Err(ProgramScoreError::RulesetMismatch));
        assert_eq!(result.tes_hundredths, None);
    }
}
