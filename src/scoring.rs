//! Source-backed, unfactored base values for a deliberately limited catalog.
use crate::{
    Level, PanelRecognition, RecognitionOutcome, ScoringAnnotation, ScoringTableId, SourceReference,
};

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

struct TableRow {
    hundredths: u16,
    goe: Option<[i16; 11]>,
    table: ScoringTableId,
    notation: String,
    document: &'static str,
    version: &'static str,
    section: &'static str,
}

impl TableRow {
    fn source(&self, column: &str) -> SourceReference {
        SourceReference {
            document: self.document.into(),
            version: self.version.into(),
            location: format!("{}, {column}, row {}", self.section, self.notation),
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

fn table_row(
    table: ScoringTableId,
    recognition: &PanelRecognition,
) -> Result<TableRow, BaseValueError> {
    let details = recognition
        .details
        .as_ref()
        .ok_or(BaseValueError::UnknownDetails)?;
    let notation = details.notation().ok_or(BaseValueError::UnknownCalls)?;
    if details.components().len() != 1 {
        return Err(BaseValueError::Unsupported);
    }
    let (hundredths, section, goe) = match table {
        ScoringTableId::Isu2786June2_2026 => match (notation.as_str(), details.level()) {
            ("1Lz", None) => (
                60,
                "p. 2, Jumps",
                Some([-30, -24, -18, -12, -6, 0, 6, 12, 18, 24, 30]),
            ),
            ("1Lzq", None) | ("1Lz!", None) => (60, "p. 2, Jumps", None),
            ("1Lz<", None) | ("1Lze", None) => (48, "p. 2, Jumps", None),
            ("1Lz<<", None) => (0, "p. 2, Jumps", None),
            ("1Lze<", None) => (36, "p. 2, Jumps", None),
            ("1A<<", None) => (0, "p. 3, Jumps", None),
            ("CSSp1", Some(Level::One)) => (
                230,
                "p. 8, Spins",
                Some([-115, -92, -69, -46, -23, 0, 23, 46, 69, 92, 115]),
            ),
            ("ChSp1", None) => (
                350,
                "p. 14, Choreographic Elements",
                Some([-250, -200, -150, -100, -50, 0, 50, 100, 150, 200, 250]),
            ),
            _ => return Err(BaseValueError::Unsupported),
        },
    };
    let (document, version) = match table {
        ScoringTableId::Isu2786June2_2026 => ("ISU Communication 2786", "2026-06-02"),
    };
    Ok(TableRow {
        hundredths,
        goe,
        table,
        notation,
        document,
        version,
        section,
    })
}

/// Look up a supplied panel outcome. This does not establish event eligibility.
pub fn lookup_base_value(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
) -> Result<BaseValueOutcome, BaseValueError> {
    if !annotations.is_empty() {
        return Err(BaseValueError::Unsupported);
    }
    match recognition.outcome {
        RecognitionOutcome::Ignored => Ok(BaseValueOutcome::Ignored),
        RecognitionOutcome::NoValue => Ok(BaseValueOutcome::NoValue),
        RecognitionOutcome::Unknown => Err(BaseValueError::UnknownRecognition),
        RecognitionOutcome::Recognized => Ok(BaseValueOutcome::Value(
            table_row(table, recognition)?.base(),
        )),
    }
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

/// Scores only individually reviewed unmarked elements, with an explicit fall observation.
/// This does not validate event eligibility or other execution errors.
pub fn score_element(
    table: ScoringTableId,
    recognition: &PanelRecognition,
    annotations: &[ScoringAnnotation],
    goe: i8,
    fall: Option<bool>,
) -> Result<ElementScore, ElementScoreError> {
    if !(-5..=5).contains(&goe) {
        return Err(ElementScoreError::InvalidGoe);
    }
    match lookup_base_value(table, recognition, annotations)
        .map_err(ElementScoreError::BaseValue)?
    {
        BaseValueOutcome::Ignored => return Err(ElementScoreError::Ignored),
        BaseValueOutcome::NoValue => return Err(ElementScoreError::NoValue),
        BaseValueOutcome::Value(_) => {}
    }
    let row = table_row(table, recognition).map_err(ElementScoreError::BaseValue)?;
    let goe_values = row.goe.ok_or(ElementScoreError::UnsupportedMarks)?;
    match fall {
        None => return Err(ElementScoreError::UnknownFall),
        Some(true) if goe > -3 => return Err(ElementScoreError::InconsistentFall),
        _ => {}
    }
    let goe_hundredths = goe_values[(goe + 5) as usize];
    let score_hundredths = row.hundredths as i16 + goe_hundredths;
    let column = if goe == 0 {
        String::from("BASE (zero adjustment)")
    } else {
        format!("GOE column {goe:+}")
    };
    let goe_source = row.source(&column);
    let base = row.base();
    let consistency_source = SourceReference {
        document: "ISU Communication 2788".into(),
        version: "2026-05-19".into(),
        location: "pp. 14, 16, fall reduction and maximum starting GOE".into(),
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
