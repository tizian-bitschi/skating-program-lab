use skating_program_lab::{CatalogSource, ScoringTableId, ISU_2786_JUNE_2_2026_SHA256};
use std::collections::{BTreeMap, BTreeSet};

const MANIFEST: &str = include_str!("fixtures/isu-2786-2026-06-02-manifest.tsv");

#[test]
fn source_inventory_accounts_for_every_printed_value_row() {
    // Counted against rendered pages, including graphic-only 3A labels on p. 4.
    let page_counts = [55, 52, 54, 54, 50, 55, 55, 45, 45, 52, 54, 40, 14];
    let family_counts = BTreeMap::from([
        ("Jump", 215),
        ("Spin", 180),
        ("StepSequence", 5),
        ("Lift", 45),
        ("TwistLift", 40),
        ("ThrowJump", 92),
        ("DeathSpiral", 28),
        ("PivotFigure", 1),
        ("PairSpin", 5),
        ("PairSpinCombination", 10),
        ("ChoreographicSequence", 1),
        ("ChoreographicSpin", 1),
        ("ChoreographicPairSpin", 1),
        ("ChoreographicPairLift", 1),
    ]);
    let mut seen = BTreeSet::new();
    let mut pages = [0; 13];
    let mut families = BTreeMap::new();
    for line in MANIFEST.lines().skip(1) {
        let cells: Vec<_> = line.split('\t').collect();
        assert_eq!(cells.len(), 11, "{line}");
        assert_eq!(cells[0], "isu-2786-2026-06-02");
        let page: usize = cells[1].parse().unwrap();
        assert!((2..=14).contains(&page));
        pages[page - 2] += 1;
        assert_eq!(cells[2].parse::<usize>().unwrap(), pages[page - 2]);
        assert!(!cells[3].is_empty());
        assert!(matches!(cells[4], "SingleAndPair" | "Pair"));
        *families.entry(cells[5]).or_insert(0) += 1;
        assert!(seen.insert(cells[6]), "duplicate code {}", cells[6]);
        assert!(matches!(cells[7], "-" | "B" | "1" | "2" | "3" | "4"));
        assert_eq!(cells[8], "11", "BASE + ten GOE columns: {line}");
        assert!(!cells[9].is_empty());
        assert_eq!(cells[10], "render-cross-checked");
    }
    assert_eq!(pages, page_counts);
    assert_eq!(families, family_counts);
    assert_eq!(seen.len(), 625);
    for code in ["Eu", "3A", "3Ab", "3Aq", "3Aqb", "5Lz!<<", "PiF1", "ChPLi1"] {
        assert!(seen.contains(code), "missing easily omitted row {code}");
    }
}

#[test]
fn catalog_provenance_resolves_only_the_preserved_june_edition() {
    let source = CatalogSource {
        table: ScoringTableId::Isu2786June2_2026,
        page: 13,
        row: 35,
        section: "Pivot Figure",
    }
    .reference("PiF1");
    assert_eq!(source.document, "ISU Communication 2786");
    assert_eq!(source.version, "2026-06-02");
    assert_eq!(
        source.location,
        "p. 13, Pivot Figure, value row 35, code PiF1"
    );
    assert!(source.official_url().unwrap().contains("Update-260602"));
    assert_eq!(
        ISU_2786_JUNE_2_2026_SHA256,
        "980ca3f689c280196f1fe4e32843e97010116c8b624eb7871c9f8850561613b7"
    );
}
