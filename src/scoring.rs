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

// ISU Communication 2786, updated 2026-06-02, pp. 2-4: BASE and GOE columns.
fn jump_row(notation: &str) -> Option<(u16, &'static str, [i16; 11])> {
    match notation {
        "1T" | "1S" | "1Tq" | "1Sq" | "1Lo<" | "1F<" | "1Fe" | "1Feq" | "1F!<" => Some((
            40,
            "p. 2, Jumps",
            [-20, -16, -12, -8, -4, 0, 4, 8, 12, 16, 20],
        )),
        "1Lo" | "1F" | "1Loq" | "1Fq" | "1F!" | "1F!q" => Some((
            50,
            "p. 2, Jumps",
            [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25],
        )),
        "1Lz" | "1Lzq" | "1Lz!" | "1Lz!q" => Some((
            60,
            "p. 2, Jumps",
            [-30, -24, -18, -12, -6, 0, 6, 12, 18, 24, 30],
        )),
        "1T<" | "1S<" => Some((
            32,
            "p. 2, Jumps",
            [-16, -13, -10, -6, -3, 0, 3, 6, 10, 13, 16],
        )),
        "1Lz<" | "1Lze" | "1Lzeq" | "1Lz!<" => Some((
            48,
            "p. 2, Jumps",
            [-24, -19, -14, -10, -5, 0, 5, 10, 14, 19, 24],
        )),
        "1T<<" | "1S<<" | "1Lo<<" | "1F<<" | "1Lz<<" | "1Fe<<" | "1Lze<<" | "1F!<<" | "1Lz!<<" => {
            Some((0, "p. 2, Jumps", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]))
        }
        "1Fe<" => Some((
            30,
            "p. 2, Jumps",
            [-15, -12, -9, -6, -3, 0, 3, 6, 9, 12, 15],
        )),
        "1Lze<" => Some((
            36,
            "p. 2, Jumps",
            [-18, -14, -11, -7, -4, 0, 4, 7, 11, 14, 18],
        )),
        "1A" | "1Aq" => Some((
            110,
            "p. 2, Jumps",
            [-55, -44, -33, -22, -11, 0, 11, 22, 33, 44, 55],
        )),
        "2T" | "2S" | "2Tq" | "2Sq" => Some((
            130,
            "p. 2, Jumps",
            [-65, -52, -39, -26, -13, 0, 13, 26, 39, 52, 65],
        )),
        "2Lo" | "2Loq" => Some((
            170,
            "p. 2, Jumps",
            [-85, -68, -51, -34, -17, 0, 17, 34, 51, 68, 85],
        )),
        "2F" | "2Fq" => Some((
            180,
            "p. 2, Jumps",
            [-90, -72, -54, -36, -18, 0, 18, 36, 54, 72, 90],
        )),
        "2Lz" | "2Lzq" => Some((
            210,
            "p. 2, Jumps",
            [-105, -84, -63, -42, -21, 0, 21, 42, 63, 84, 105],
        )),
        "1A<" => Some((
            88,
            "p. 2, Jumps",
            [-44, -35, -26, -18, -9, 0, 9, 18, 26, 35, 44],
        )),
        "2T<" | "2S<" => Some((
            104,
            "p. 2, Jumps",
            [-52, -42, -31, -21, -10, 0, 10, 21, 31, 42, 52],
        )),
        "2Lo<" => Some((
            136,
            "p. 2, Jumps",
            [-68, -54, -41, -27, -14, 0, 14, 27, 41, 54, 68],
        )),
        "2F<" => Some((
            144,
            "p. 2, Jumps",
            [-72, -58, -43, -29, -14, 0, 14, 29, 43, 58, 72],
        )),
        "2Lz<" => Some((
            168,
            "p. 2, Jumps",
            [-84, -67, -50, -34, -17, 0, 17, 34, 50, 67, 84],
        )),
        "1A<<" => Some((0, "p. 3, Jumps", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        "2T<<" | "2S<<" | "2Fe<<" => Some((
            40,
            "p. 3, Jumps",
            [-20, -16, -12, -8, -4, 0, 4, 8, 12, 16, 20],
        )),
        "2Lo<<" | "2F<<" | "2F!<<" => Some((
            50,
            "p. 3, Jumps",
            [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25],
        )),
        "2Lz<<" | "2Lz!<<" => Some((
            60,
            "p. 3, Jumps",
            [-30, -24, -18, -12, -6, 0, 6, 12, 18, 24, 30],
        )),
        "2Fe" | "2Feq" | "2F!<" => Some((
            144,
            "p. 3, Jumps",
            [-72, -58, -43, -29, -14, 0, 14, 29, 43, 58, 72],
        )),
        "2Fe<" => Some((
            108,
            "p. 3, Jumps",
            [-54, -43, -32, -22, -11, 0, 11, 22, 32, 43, 54],
        )),
        "2Lze" | "2Lzeq" | "2Lz!<" => Some((
            168,
            "p. 3, Jumps",
            [-84, -67, -50, -34, -17, 0, 17, 34, 50, 67, 84],
        )),
        "2Lze<" => Some((
            126,
            "p. 3, Jumps",
            [-63, -50, -38, -25, -13, 0, 13, 25, 38, 50, 63],
        )),
        "2Lze<<" => Some((
            48,
            "p. 3, Jumps",
            [-24, -19, -14, -10, -5, 0, 5, 10, 14, 19, 24],
        )),
        "2F!" | "2F!q" => Some((
            180,
            "p. 3, Jumps",
            [-90, -72, -54, -36, -18, 0, 18, 36, 54, 72, 90],
        )),
        "2Lz!" | "2Lz!q" => Some((
            210,
            "p. 3, Jumps",
            [-105, -84, -63, -42, -21, 0, 21, 42, 63, 84, 105],
        )),
        "2A" | "2Aq" => Some((
            330,
            "p. 3, Jumps",
            [-165, -132, -99, -66, -33, 0, 33, 66, 99, 132, 165],
        )),
        "2A<" => Some((
            264,
            "p. 3, Jumps",
            [-132, -106, -79, -53, -26, 0, 26, 53, 79, 106, 132],
        )),
        "2A<<" => Some((
            110,
            "p. 4, Jumps",
            [-55, -44, -33, -22, -11, 0, 11, 22, 33, 44, 55],
        )),
        _ => None,
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
            (name, None) if jump_row(name).is_some() => {
                let (base, section, goe) = jump_row(name).expect("matched jump row");
                (base, section, Some(goe))
            }
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

/// Scores supported single elements with an explicit fall observation.
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
    let details = recognition
        .details
        .as_ref()
        .expect("table row requires details");
    let marks = if row.section.ends_with("Jumps") {
        row.notation
            .strip_prefix(details.components()[0].as_str())
            .unwrap_or("")
    } else {
        ""
    };
    if (marks.contains('e') || marks.contains('!')) && (marks.contains('<') || marks.contains('q'))
    {
        return Err(ElementScoreError::UnsupportedMarks);
    }
    // Communication 2788 pp. 14, 16: reductions and maximum starting GOE.
    let max_grade = match marks {
        "q" | "<" => 3,
        "!" => 4,
        "e" => 0,
        "<<" => -1,
        _ => 5,
    };
    if goe > max_grade {
        return Err(ElementScoreError::UnsupportedMarks);
    }
    match fall {
        None => return Err(ElementScoreError::UnknownFall),
        Some(true)
            if goe
                > match marks {
                    "" => -3,
                    "!" => -4,
                    _ => -5,
                } =>
        {
            return Err(ElementScoreError::InconsistentFall);
        }
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
        location: if row.notation.ends_with("<<") {
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
    unfactored_element_score(ruleset, table, element)
}

fn unfactored_element_score(
    ruleset: crate::RulesetId,
    table: ScoringTableId,
    element: &ProgramElement,
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
    score_element(
        table,
        &element.recognition,
        annotations,
        element.goe,
        element.fall,
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
    if second_half_boundary.is_some_and(|boundary| boundary > elements.len()) {
        return Err(ProgramScoreError::InvalidInput);
    }
    let preliminary: Vec<_> = elements
        .iter()
        .map(|element| unfactored_element_score(ruleset, table, element))
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
    let tes_hundredths = sum_element_scores(&results).filter(|_| {
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
