//! Immutable Scale of Values data, independent of recognition and scoring support.
use crate::{Level, ScoringTableId, SourceReference};

mod pair;
mod shared;

/// Exact published-code lookup in the selected immutable edition.
/// Unknown notation returns `None`; this does not infer calls or engine support.
pub fn catalog_row(table: ScoringTableId, notation: &str) -> Option<&'static CatalogRow> {
    match table {
        ScoringTableId::Isu2786June2_2026 => shared::ROWS
            .iter()
            .chain(pair::ROWS)
            .find(|row| row.notation == notation),
    }
}

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
        self.table.source_reference(format!(
            "p. {}, {}, value row {}, code {}",
            self.page, self.section, self.row, notation
        ))
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

// Compact typed initializers keep the printed data visible without repeated field names.
const fn row(
    notation: &'static str,
    family: CatalogFamily,
    level: Option<Level>,
    base_hundredths: u16,
    goe_hundredths: [i16; 11],
    source: (u8, u8, &'static str),
    notes: &'static [CatalogNote],
) -> CatalogRow {
    let discipline = match family {
        CatalogFamily::Jump
        | CatalogFamily::Spin
        | CatalogFamily::StepSequence
        | CatalogFamily::ChoreographicSequence
        | CatalogFamily::ChoreographicSpin => CatalogDiscipline::SingleAndPair,
        CatalogFamily::Lift
        | CatalogFamily::TwistLift
        | CatalogFamily::ThrowJump
        | CatalogFamily::DeathSpiral
        | CatalogFamily::PivotFigure
        | CatalogFamily::PairSpin
        | CatalogFamily::PairSpinCombination
        | CatalogFamily::ChoreographicPairSpin
        | CatalogFamily::ChoreographicPairLift => CatalogDiscipline::Pair,
    };
    CatalogRow {
        notation,
        discipline,
        family,
        level,
        base_hundredths,
        goe_hundredths,
        source: CatalogSource {
            table: ScoringTableId::Isu2786June2_2026,
            page: source.0,
            row: source.1,
            section: source.2,
        },
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn embedded_rows_have_no_extra_or_duplicate_codes() {
        assert_eq!((shared::ROWS.len(), pair::ROWS.len()), (402, 223));
        let rows: Vec<_> = shared::ROWS.iter().chain(pair::ROWS).collect();
        let codes: BTreeSet<_> = rows.iter().map(|row| row.notation).collect();
        assert_eq!(codes.len(), 625);
        assert_eq!(codes.len(), rows.len());
        for row in rows {
            assert_eq!(row.source.table, ScoringTableId::Isu2786June2_2026);
            assert_eq!(row.goe_hundredths[5], 0);
        }
    }
}
