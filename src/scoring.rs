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
