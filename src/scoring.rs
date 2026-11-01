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
    /// Elapsed milliseconds from the program start; required for jumps.
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
    pub tes_hundredths: Option<i64>,
    pub fall_deduction: Result<Deduction, ProgramScoreError>,
    pub time_deduction: Result<Deduction, ProgramScoreError>,
}

fn adult_source(location: &str) -> SourceReference {
    SourceReference {
        document: "International Adult Figure Skating Competitions Technical Announcement".into(),
        version: "2026-07-16".into(),
        location: location.into(),
    }
}

fn required_duration(category: crate::AdultCategory) -> (u32, &'static str) {
    match category {
        crate::AdultCategory::Bronze => (100, "p. 11"),
        crate::AdultCategory::Silver => (120, "p. 10"),
        crate::AdultCategory::Gold => (170, "p. 9"),
        crate::AdultCategory::Masters => (180, "p. 8"),
    }
}

fn factored_base(hundredths: u16) -> u16 {
    ((u32::from(hundredths) * 11 + 5) / 10) as u16
}

/// Calculate a partial-coverage scenario, not an official panel or segment score.
/// Elements must be in program order; callers supply all recognized elements and falls.
pub fn score_program(
    category: crate::AdultCategory,
    table: ScoringTableId,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
    timing: Option<ProgramTiming>,
) -> ProgramScore {
    let (required, page) = required_duration(category);
    let timing_ok = timing.filter(|t| t.ordinary_timing_confirmed && t.duration_seconds > 0);
    let time_deduction = match timing_ok {
        None => Err(ProgramScoreError::TimingUnknown),
        Some(t) => {
            let deviation = required.abs_diff(t.duration_seconds).saturating_sub(10);
            deviation
                .div_ceil(5)
                .checked_mul(100)
                .map(|hundredths| Deduction {
                    hundredths,
                    source: adult_source(&format!(
                        "p. 6, time deductions; {page}, required duration"
                    )),
                })
                .ok_or(ProgramScoreError::InvalidInput)
        }
    };
    let fall_deduction = match (
        category,
        outside_falls,
        elements.iter().all(|e| {
            e.fall == Some(false)
                || (e.fall == Some(true) && e.recognition.outcome == RecognitionOutcome::Recognized)
        }),
    ) {
        (crate::AdultCategory::Bronze, Some(count), true) => count
            .checked_mul(50)
            .map(|hundredths| Deduction {
                hundredths,
                source: adult_source("p. 11, Bronze fall deductions"),
            })
            .ok_or(ProgramScoreError::InvalidInput),
        (crate::AdultCategory::Bronze, _, _) => Err(ProgramScoreError::UnknownFall),
        _ => Err(ProgramScoreError::UnsupportedCategory),
    };
    let preliminary: Vec<_> = elements
        .iter()
        .map(|e| {
            // The x annotation is a recorded claim, not evidence of eligibility.
            score_element(table, &e.recognition, &[], e.goe, e.fall)
                .map_err(ProgramScoreError::Element)
        })
        .collect();
    let mut jump_indices = Vec::new();
    let mut jump_error = None;
    let mut previous_start = None;
    for (i, (element, score)) in elements.iter().zip(&preliminary).enumerate() {
        if let Ok(score) = score {
            if score.base.notation == "1Lz" {
                if let Some(start) = element.start_ms {
                    if previous_start.is_some_and(|previous| start < previous) {
                        jump_error = Some(ProgramScoreError::InvalidTiming);
                    }
                    previous_start = Some(start);
                }
                match (timing_ok, element.start_ms) {
                    (None, _) | (_, None) => jump_error = Some(ProgramScoreError::TimingUnknown),
                    (Some(t), Some(start))
                        if u64::from(start) > u64::from(t.duration_seconds) * 1000 =>
                    {
                        jump_error = Some(ProgramScoreError::InvalidTiming)
                    }
                    (Some(t), _) if !t.second_half_default_confirmed => {
                        jump_error = Some(ProgramScoreError::ExceptionUnresolved)
                    }
                    (Some(_), Some(start))
                        if u64::from(start) * 2 >= u64::from(required) * 1000 =>
                    {
                        jump_indices.push(i)
                    }
                    _ => {}
                }
            }
        }
    }
    // An unresolved element may itself affect which jumps are the last three.
    if preliminary.iter().any(Result::is_err) && !jump_indices.is_empty() {
        jump_error = Some(ProgramScoreError::TimingUnknown);
    }
    let selected = jump_indices.len().saturating_sub(3);
    let results: Vec<_> = preliminary
        .into_iter()
        .enumerate()
        .map(|(i, result)| {
            let score = result?;
            let jump = score.base.notation == "1Lz";
            if jump {
                if let Some(error) = &jump_error {
                    return Err(error.clone());
                }
            }
            let second_half = jump_indices[selected..].contains(&i);
            let credited = if second_half {
                u32::from(factored_base(score.base.hundredths))
            } else {
                u32::from(score.base.hundredths)
            } as u16;
            let total = credited as i16 + score.goe_hundredths;
            let source = second_half.then(|| SourceReference {
                document: "ISU Sports Rules — Figure Skating".into(),
                version: "2026-06-12".into(),
                location: "p. 9, Rule 353.1(h)(iii)".into(),
            });
            Ok(ProgramElementScore {
                unfactored: score,
                second_half,
                credited_base_hundredths: credited,
                score_hundredths: total,
                second_half_source: source,
            })
        })
        .collect();
    let tes = results
        .iter()
        .map(|r| r.as_ref().map(|s| i64::from(s.score_hundredths)))
        .sum::<Result<i64, _>>()
        .ok();
    ProgramScore {
        elements: results,
        tes_hundredths: tes,
        fall_deduction,
        time_deduction,
    }
}

#[cfg(test)]
mod program_tests {
    #[test]
    fn factored_base_rounds_each_value_to_hundredths() {
        assert_eq!(super::factored_base(35), 39);
        assert_eq!(super::factored_base(34), 37);
    }
}
