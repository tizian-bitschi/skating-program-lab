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
        RecognitionOutcome::Ignored => return Ok(BaseValueOutcome::Ignored),
        RecognitionOutcome::NoValue => return Ok(BaseValueOutcome::NoValue),
        RecognitionOutcome::Unknown => return Err(BaseValueError::UnknownRecognition),
        RecognitionOutcome::Recognized => {}
    }
    let details = recognition
        .details
        .as_ref()
        .ok_or(BaseValueError::UnknownDetails)?;
    let notation = details.notation().ok_or(BaseValueError::UnknownCalls)?;
    if details.components().len() != 1 {
        return Err(BaseValueError::Unsupported);
    }
    let (hundredths, location) = match table {
        ScoringTableId::Isu2786June2_2026 => match (notation.as_str(), details.level()) {
            ("1Lz", None) | ("1Lzq", None) | ("1Lz!", None) => (60, "p. 2, Jumps, BASE column"),
            ("1Lz<", None) | ("1Lze", None) => (48, "p. 2, Jumps, BASE column"),
            ("1Lz<<", None) => (0, "p. 2, Jumps, BASE column"),
            ("1Lze<", None) => (36, "p. 2, Jumps, BASE column"),
            ("1A<<", None) => (0, "p. 3, Jumps, BASE column"),
            ("CSSp1", Some(Level::One)) => (230, "p. 8, Spins, BASE column"),
            ("ChSp1", None) => (350, "p. 14, Choreographic Elements, BASE column"),
            _ => return Err(BaseValueError::Unsupported),
        },
    };
    Ok(BaseValueOutcome::Value(BaseValue {
        hundredths,
        table,
        source: SourceReference {
            document: "ISU Communication 2786".into(),
            version: "2026-06-02".into(),
            location: format!("{location}, row {notation}"),
        },
        notation,
    }))
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
    NoScorableValue,
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
    let base = match lookup_base_value(table, recognition, annotations)
        .map_err(ElementScoreError::BaseValue)?
    {
        BaseValueOutcome::Value(value) => value,
        BaseValueOutcome::Ignored | BaseValueOutcome::NoValue => {
            return Err(ElementScoreError::NoScorableValue)
        }
    };
    let per_grade = match base.notation.as_str() {
        "1Lz" => 6,
        "CSSp1" => 23,
        "ChSp1" => 50,
        _ => return Err(ElementScoreError::UnsupportedMarks),
    };
    match fall {
        None => return Err(ElementScoreError::UnknownFall),
        Some(true) if goe > -3 => return Err(ElementScoreError::InconsistentFall),
        _ => {}
    }
    let goe_hundredths = i16::from(goe) * per_grade;
    let score_hundredths = base.hundredths as i16 + goe_hundredths;
    let column = if goe == 0 {
        String::from("BASE (zero adjustment)")
    } else {
        format!("GOE column {goe:+}")
    };
    let goe_source = SourceReference {
        document: "ISU Communication 2786".into(),
        version: "2026-06-02".into(),
        location: format!(
            "{}, {column}",
            base.source.location.replace("BASE column, ", "")
        ),
    };
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
