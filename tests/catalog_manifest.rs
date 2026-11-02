use skating_program_lab::{CatalogSource, ScoringTableId, ISU_2786_JUNE_2_2026_SHA256};
use std::collections::{BTreeMap, BTreeSet};

const MANIFEST: &str = include_str!("fixtures/isu-2786-2026-06-02-manifest.tsv");
const SOURCE_CODES: &str = include_str!("fixtures/isu-2786-2026-06-02-source-codes.txt");

#[test]
fn source_inventory_accounts_for_every_printed_value_row() {
    check_manifest(MANIFEST);
}

fn check_manifest(manifest: &str) {
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
    let expected: Vec<_> = SOURCE_CODES
        .lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(|line| {
            let group: Vec<_> = line.split('|').collect();
            assert_eq!(group.len(), 4);
            group[3]
                .split_whitespace()
                .map(|code| (group[0], group[1], group[2], code))
                .collect::<Vec<_>>()
        })
        .collect();
    let lines: Vec<_> = manifest.lines().skip(1).collect();
    assert_eq!(expected.len(), 625);
    assert_eq!(lines.len(), expected.len());
    for (line, (expected_page, family, discipline, code)) in lines.into_iter().zip(expected) {
        let cells: Vec<_> = line.split('\t').collect();
        assert_eq!(cells.len(), 11, "{line}");
        assert_eq!(cells[0], "isu-2786-2026-06-02");
        assert_eq!(cells[1], expected_page, "{line}");
        assert_eq!(cells[4], discipline, "{line}");
        assert_eq!(cells[5], family, "{line}");
        assert_eq!(cells[6], code, "{line}");
        assert_eq!(cells[7], expected_level(code, family), "{line}");
        assert_eq!(cells[9], expected_notes(code, family), "{line}");
        let page: usize = cells[1].parse().unwrap();
        assert!((2..=14).contains(&page));
        pages[page - 2] += 1;
        assert_eq!(cells[2].parse::<usize>().unwrap(), pages[page - 2]);
        assert_eq!(cells[3], expected_section(page, pages[page - 2], code));
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

fn expected_section(page: usize, row: usize, code: &str) -> String {
    // Boundaries read from the printed section headers, independently of catalog data.
    match (page, row) {
        (2..=5, _) => "Jumps",
        (6, 1..=20) => {
            "Spin in one position and no change of foot (upright, layback, camel or sit)"
        }
        (6, _) | (7, 1..=10) => "Flying Spin (any position - upright, layback, camel or sit)",
        (7, _) | (8, 1..=35) => "Spins with change of foot",
        (8, _) => "Spin Combination with change of position and no change of foot",
        (9, 1..=20) => "Spin Combination with change of position and change of foot",
        (9, 21..=25) => "Step Sequence",
        (9, _) => return format!("Lifts / Group {}", &code[..1]),
        (10, 1..=25) => return format!("Lifts / Group {}", &code[..2]),
        (10, _) | (11, 1..=20) => "Twist Lifts",
        (11..=12, _) | (13, 1..=6) => "Throw Jumps",
        (13, 7..=26) => "Death Spirals",
        (13, 27..=34) => "Advanced Novice Death Spirals eligible for bonus (b)",
        (13, 35) => "Pivot Figure",
        (13, _) => "Pair Spins",
        (14, 1..=10) => "Pair Spin Combination",
        (14, 11) => "Choreographic Step Sequence",
        (14, 12) => "Choreographic Spin",
        (14, 13) => "Choreographic Pair Spin",
        (14, 14) => "Choreographic Pair Lift",
        _ => panic!("unexpected page/row {page}/{row}"),
    }
    .into()
}

fn expected_level<'a>(code: &'a str, family: &str) -> &'a str {
    match family {
        "Spin"
        | "StepSequence"
        | "Lift"
        | "TwistLift"
        | "DeathSpiral"
        | "PairSpin"
        | "PairSpinCombination" => {
            let unmarked = code.trim_end_matches(['V', 'b', '<']);
            &unmarked[unmarked.len() - 1..]
        }
        _ => "-",
    }
}

fn expected_notes(code: &str, family: &str) -> String {
    // Marker parsing uses family boundaries: the q in StSq is not a call.
    let markers = match family {
        "Jump" if code != "Eu" => {
            let jump = code.trim_start_matches(|c: char| c.is_ascii_digit());
            ["Lo", "Lz", "T", "S", "F", "A"]
                .into_iter()
                .find_map(|name| jump.strip_prefix(name))
                .unwrap()
        }
        "ThrowJump" => code.split_once("Th").unwrap().1,
        "TwistLift" if code.ends_with("<<") => "<<",
        "Spin" | "PairSpinCombination" if code.ends_with('V') => "V",
        "DeathSpiral" if code.ends_with('b') => "b",
        _ => "",
    };
    let mut notes = vec![if family.starts_with("Choreographic") {
        "2"
    } else {
        "1"
    }];
    // Remark 3 lists <, e and V, while << is separately defined in the legend.
    if markers.contains('e')
        || markers.contains('V')
        || (markers.contains('<') && !markers.contains("<<"))
    {
        notes.push("3");
    }
    if family == "Jump" {
        notes.extend(["4", "5"]);
        if code.starts_with('5') {
            notes.push("6");
        }
    }
    if !markers.is_empty() {
        notes.push("legend");
    }
    notes.join(",")
}

#[test]
fn inventory_validation_rejects_source_and_metadata_corruption() {
    for (before, after) in [
        ("\tJumps\t", "\tWrong section\t"),
        ("\tStSq4\t", "\tStSq5\t"),
        ("\t1,legend\t", "\tINVALID_NOTE\t"),
        ("\tStSq4\t4\t", "\tStSq4\t3\t"),
        ("\tSingleAndPair\tStepSequence\t", "\tPair\tStepSequence\t"),
        ("\tStepSequence\tStSq4\t", "\tSpin\tStSq4\t"),
        (
            "\t1T<<\t-\t11\t1,4,5,legend\t",
            "\t1T<<\t-\t11\t1,3,4,5,legend\t",
        ),
        ("\tStSq4\t4\t11\t1\t", "\tStSq4\t4\t11\t1,legend\t"),
    ] {
        assert!(
            MANIFEST.contains(before),
            "mutation must change the fixture"
        );
        let mutated = MANIFEST.replacen(before, after, 1);
        assert!(
            std::panic::catch_unwind(|| check_manifest(&mutated)).is_err(),
            "{after}"
        );
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
