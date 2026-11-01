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
            ("1Lz<<", None) => (0, "p. 2, Jumps", Some([0; 11])),
            ("1Lze<", None) => (36, "p. 2, Jumps", None),
            ("1A<<", None) => (0, "p. 3, Jumps", Some([0; 11])),
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

fn adult_source(location: &str) -> SourceReference {
    SourceReference {
        document: "International Adult Figure Skating Competitions Technical Announcement".into(),
        version: "2026-07-16".into(),
        location: location.into(),
    }
}

fn required_duration(
    ruleset: crate::RulesetId,
    category: crate::AdultCategory,
) -> (u32, &'static str) {
    match ruleset {
        crate::RulesetId::IsuAdult2026_27July16 => match category {
            crate::AdultCategory::Bronze => (100, "p. 11"),
            crate::AdultCategory::Silver => (120, "p. 10"),
            crate::AdultCategory::Gold => (170, "p. 9"),
            crate::AdultCategory::Masters => (180, "p. 8"),
        },
        #[cfg(test)]
        crate::RulesetId::TestOther => unreachable!("test identity has no scoring rules"),
    }
}

fn factored_base(hundredths: u16) -> u16 {
    ((u32::from(hundredths) * 11 + 5) / 10) as u16
}

fn time_deduction(
    timing: Option<ProgramTiming>,
    required: u32,
    page: &str,
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
        source: adult_source(&format!("p. 6, time deductions; {page}, required duration")),
    })
}

fn fall_deduction(
    category: crate::AdultCategory,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
) -> Result<Deduction, ProgramScoreError> {
    if category != crate::AdultCategory::Bronze {
        return Err(ProgramScoreError::UnsupportedCategory);
    }
    if !elements.iter().all(|e| {
        e.fall == Some(false)
            || (e.fall == Some(true) && e.recognition.outcome == RecognitionOutcome::Recognized)
    }) {
        return Err(ProgramScoreError::UnknownFall);
    }
    let count = outside_falls.ok_or(ProgramScoreError::UnknownFall)?;
    let hundredths = count
        .checked_mul(50)
        .ok_or(ProgramScoreError::InvalidInput)?;
    Ok(Deduction {
        hundredths,
        source: adult_source("p. 11, Bronze fall deductions"),
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
    // A single recorded x is only meaningful for a jump; timing determines credit.
    let annotations = if is_jump(element) == Some(true)
        && element.annotations == [ScoringAnnotation::SecondHalfCredit]
    {
        &[][..]
    } else {
        element.annotations.as_slice()
    };
    score_element(
        table,
        &element.recognition,
        annotations,
        element.goe,
        element.fall,
    )
    .map_err(ProgramScoreError::Element)
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
            Err(ProgramScoreError::Element(
                ElementScoreError::Ignored | ElementScoreError::NoValue
            ))
        ) {
            continue;
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
    let (required, page) = required_duration(ruleset, category);
    let mut previous_start = None;
    let preliminary: Vec<_> = elements
        .iter()
        .map(|e| {
            if let Some(start) = e.start_ms {
                let inconsistent = previous_start.is_some_and(|previous| start < previous);
                previous_start =
                    Some(previous_start.map_or(start, |previous: u32| previous.max(start)));
                if inconsistent {
                    return Err(ProgramScoreError::InvalidTiming);
                }
            }
            element_score(ruleset, table, e, timing, required)
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
    let tes_hundredths = results
        .iter()
        .map(|r| match r {
            Ok(score) => Ok(i64::from(score.score_hundredths)),
            Err(ProgramScoreError::Element(
                ElementScoreError::Ignored | ElementScoreError::NoValue,
            )) => Ok(0),
            Err(error) => Err(error),
        })
        .sum::<Result<i64, _>>()
        .ok()
        .filter(|_| !elements.is_empty())
        .filter(|_| {
            timing.is_some_and(|t| {
                t.ordinary_timing_confirmed && t.duration_seconds > required.saturating_sub(40)
            })
        });
    ProgramScore {
        elements: results,
        tes_hundredths,
        fall_deduction: fall_deduction(category, elements, outside_falls),
        time_deduction: time_deduction(timing, required, page),
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
