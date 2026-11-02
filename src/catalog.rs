//! Immutable Scale of Values data, independent of recognition and scoring support.
use crate::{Level, ScoringTableId, SourceReference};

/// SHA-256 of the official PDF whose cover states "Updated as of 2 June, 2026".
/// The PDF's internal title still names the superseded May edition.
pub const ISU_2786_JUNE_2_2026_SHA256: &str =
    "980ca3f689c280196f1fe4e32843e97010116c8b624eb7871c9f8850561613b7";

/// Applicability of a published table section, not engine support or eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogDiscipline {
    SingleAndPair,
    Pair,
}

/// Published element families, including Pair data that the engine cannot score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogFamily {
    Jump,
    Spin,
    StepSequence,
    Lift,
    TwistLift,
    ThrowJump,
    DeathSpiral,
    PivotFigure,
    PairSpin,
    PairSpinCombination,
    ChoreographicSequence,
    ChoreographicSpin,
    ChoreographicPairSpin,
    ChoreographicPairLift,
}

/// References to Communication 2786 p. 14; these do not implement its rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogNote {
    /// Remark 1: regular GOE steps use 10% of the applicable base value.
    PercentageGoe,
    /// Remark 2: choreographic elements use the separately published GOE scale.
    ChoreographicGoe,
    /// Remark 3: <, e and V use the reduced base value for GOE.
    ReducedBaseGoe,
    /// Remark 4: combination/sequence reference difficulty uses reduced values.
    CombinationReference,
    /// Remark 5: +Rep uses 80% BV but GOE refers to the original BV.
    Repetition,
    /// Remark 6: quint-jump program, combination and repetition restrictions.
    QuintJumps,
    /// Legend: b, q, <, <<, !, e and V; bonus eligibility is data-only.
    Legend,
}

/// Printed page and one-based value-row ordinal within that page.
/// Headers and blank separators are excluded from the row count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogSource {
    pub table: ScoringTableId,
    pub page: u8,
    pub row: u8,
    pub section: &'static str,
}

impl CatalogSource {
    /// Reuse the existing exact-edition document resolver.
    pub fn reference(self, notation: &str) -> SourceReference {
        let (document, version) = match self.table {
            ScoringTableId::Isu2786June2_2026 => ("ISU Communication 2786", "2026-06-02"),
        };
        SourceReference {
            document: document.into(),
            version: version.into(),
            location: format!(
                "p. {}, {}, value row {}, code {}",
                self.page, self.section, self.row, notation
            ),
        }
    }
}

/// A published row. Its presence does not establish scoring/recognition support.
/// No category legality or inferred technical calls belong in this data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogRow {
    /// Exact published code, including level and technical variant.
    pub notation: &'static str,
    pub discipline: CatalogDiscipline,
    pub family: CatalogFamily,
    /// Published B/1..4 level; absent for jumps and fixed-code elements.
    pub level: Option<Level>,
    pub base_hundredths: u16,
    /// GOE -5..+5 in order, in hundredths. Index 5 is zero: the PDF prints
    /// BASE in that position rather than a separate GOE 0 adjustment.
    pub goe_hundredths: [i16; 11],
    pub source: CatalogSource,
    pub notes: &'static [CatalogNote],
}
