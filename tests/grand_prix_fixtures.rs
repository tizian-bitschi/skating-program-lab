use std::collections::{BTreeMap, BTreeSet};

const SOURCES: &str = include_str!("fixtures/grand-prix-2026-27/sources.tsv");
const PROGRAMS: &str = include_str!("fixtures/grand-prix-2026-27/programs.tsv");
const ELEMENTS: &str = include_str!("fixtures/grand-prix-2026-27/elements.tsv");
const COMPONENTS: &str = include_str!("fixtures/grand-prix-2026-27/components.tsv");
const DEDUCTIONS: &str = include_str!("fixtures/grand-prix-2026-27/deductions.tsv");
const REVIEWED: &str = include_str!("fixtures/grand-prix-2026-27/reviewed-source.tsv");

const SOURCE_SCHEMA: &str =
    "source\tevent\tindex_url\tsegment\tsegment_id\tpdf_url\tretrieved\tsha256\tprinted\tpages";
const PROGRAM_SCHEMA: &str = "source\tpage\trank\tname\tnation\tstarting_number\tsegment_score\ttes\tpcs\tdeductions\tbase_total\telement_total";
const ELEMENT_SCHEMA: &str = "source\tpage\trank\trow\tcode\tinfo\tbase\tfactor\tgoe\tj1\tj2\tj3\tj4\tj5\tj6\tj7\tj8\tj9\treferee\tscore\tsupport";
const COMPONENT_SCHEMA: &str =
    "source\tpage\trank\trow\tname\tfactor\tj1\tj2\tj3\tj4\tj5\tj6\tj7\tj8\tj9\treferee\tscore";
const DEDUCTION_SCHEMA: &str = "source\tpage\trank\trow\tamount\tcount\treason";
const REVIEW_SCHEMA: &str = "source\tpage\trank\tkind\tprinted_fields";

const JUDGES: [&str; 9] = ["j1", "j2", "j3", "j4", "j5", "j6", "j7", "j8", "j9"];

#[derive(Debug)]
struct Row<'a> {
    fields: Vec<(&'static str, &'a str)>,
}

impl<'a> Row<'a> {
    fn get(&self, name: &str) -> &'a str {
        self.fields
            .iter()
            .find(|(field, _)| *field == name)
            .unwrap_or_else(|| panic!("unknown field {name}"))
            .1
    }

    fn grades(&self) -> [&'a str; 9] {
        JUDGES.map(|name| self.get(name))
    }
}

fn rows<'a>(text: &'a str, schema: &'static str) -> Vec<Row<'a>> {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(schema), "TSV header/schema mismatch");
    let columns: Vec<_> = schema.split('\t').collect();
    lines
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            assert_eq!(cells.len(), columns.len(), "column count: {line}");
            Row {
                fields: columns.iter().copied().zip(cells).collect(),
            }
        })
        .collect()
}

// Printed signed hundredths only: no float parsing, rounding or production scorer.
fn decimal(text: &str) -> Result<i64, &'static str> {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = unsigned.split_once('.').ok_or("missing decimal point")?;
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() != 2
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("expected exact printed hundredths");
    }
    let value = whole
        .parse::<i64>()
        .ok()
        .and_then(|w| w.checked_mul(100))
        .and_then(|w| fraction.parse::<i64>().ok().and_then(|f| w.checked_add(f)))
        .ok_or("decimal overflow")?;
    Ok(if text.starts_with('-') { -value } else { value })
}

fn cents(text: &str, location: &str) -> i64 {
    decimal(text).unwrap_or_else(|error| panic!("{location}: {text:?}: {error}"))
}

fn identity<'a>(row: &Row<'a>) -> (&'a str, &'a str, &'a str) {
    (row.get("source"), row.get("page"), row.get("rank"))
}

#[test]
fn exact_decimal_contract_rejects_lossy_or_missing_values() {
    for bad in [
        "",
        "-",
        "1",
        "1.1",
        "1.001",
        "1e2",
        "NaN",
        "+1.00",
        " 1.00",
        "1. 0",
        "99999999999999999999.00",
    ] {
        assert!(decimal(bad).is_err(), "accepted {bad:?}");
    }
    assert_eq!(decimal("-0.01"), Ok(-1));
    assert_eq!(decimal("181.78"), Ok(18178));
}

#[test]
fn schema_rejects_swapped_columns_in_every_fixture_file() {
    for (text, schema) in [
        (SOURCES, SOURCE_SCHEMA),
        (PROGRAMS, PROGRAM_SCHEMA),
        (ELEMENTS, ELEMENT_SCHEMA),
        (COMPONENTS, COMPONENT_SCHEMA),
        (DEDUCTIONS, DEDUCTION_SCHEMA),
        (REVIEWED, REVIEW_SCHEMA),
    ] {
        rows(text, schema);
        let mut columns: Vec<_> = schema.split('\t').collect();
        let position = columns.iter().position(|c| *c == "j1").unwrap_or(0);
        columns.swap(position, position + 1);
        let corrupted = text.replacen(schema, &columns.join("\t"), 1);
        assert!(std::panic::catch_unwind(|| rows(&corrupted, schema)).is_err());
    }
}

// Reviewed inventory of these observed codes, separate from production capabilities.
// "supported" covers bounded recognized-row representation, never Senior adjudication.
const SUPPORT_MATRIX: [(&str, &str); 2] = [
    ("supported", "1F 1T 2A* 2A+2Aq+2T+SEQ 2Lz CCoSp2 CCoSp3 CCoSp4 CCoSp4V ChSp1 ChSq1 FCSp3 FCSp4 FSSp3 FSSp4 StSq2 StSq3 StSq4"),
    ("requires #78/#77", "2A+3Tq 2A+Eu+3F+2A+SEQ 3A 3A+Eu+2S+2A+SEQ 3A< 3A<* 3Aq 3Aq+3Tq 3F 3F! 3F!+2A+2A+SEQ 3F!+2T 3F!+3T+2A+SEQ 3F+2T< 3F+3A<+2A+SEQ 3F<< 3Lo 3Lo+2A+2A+SEQ 3Lo+2A+2A<+SEQ 3Lo< 3Lz 3Lz!+2A+2A+SEQ 3Lz+3T 3Lz+Eu+3S 3Lz+Eu+3S< 3Lz< 3Lz<<+REP 3S 3Sq 3T 3T+Eu+3S+2A+SEQ 4F!< 4Lz<+3T 4Lzq 4S 4S< 4T 4T+2T 4T+3T 4T+REP 4T< 4T<+REP"),
];

fn check_support(e: &Row<'_>) {
    let code = e.get("code");
    let expected = SUPPORT_MATRIX
        .iter()
        .find(|(_, codes)| codes.split_whitespace().any(|observed| observed == code))
        .unwrap_or_else(|| panic!("unreviewed coverage code {code} at {:?}", identity(e)))
        .0;
    assert_eq!(
        e.get("support"),
        expected,
        "{:?} element {} {code}, field support",
        identity(e),
        e.get("row")
    );
}

#[test]
fn every_support_label_matches_the_reviewed_coverage_matrix() {
    let elements = rows(ELEMENTS, ELEMENT_SCHEMA);
    let observed: BTreeSet<_> = elements.iter().map(|e| e.get("code")).collect();
    let mut inventory = BTreeSet::new();
    for (_, codes) in SUPPORT_MATRIX {
        for code in codes.split_whitespace() {
            assert!(inventory.insert(code), "duplicate coverage code {code}");
        }
    }
    assert_eq!(observed, inventory, "coverage matrix drift");
    for e in elements {
        check_support(&e);
    }
}

#[test]
fn coverage_rejects_mislabeled_triples_quads_marked_compounds_and_spins() {
    for code in ["3A", "4T", "2A+3Tq", "3F!+2T", "3Lz!+2A+2A+SEQ", "CCoSp4V"] {
        let mut e = rows(ELEMENTS, ELEMENT_SCHEMA)
            .into_iter()
            .find(|e| e.get("code") == code)
            .unwrap();
        let label = if e.get("support") == "supported" {
            "requires #78/#77"
        } else {
            "supported"
        };
        e.fields
            .iter_mut()
            .find(|(name, _)| *name == "support")
            .unwrap()
            .1 = label;
        assert!(
            std::panic::catch_unwind(|| check_support(&e)).is_err(),
            "missed {code}"
        );
    }
}

fn factored_component(score: &str, factor: &str, location: &str) -> i64 {
    let score = cents(score, location);
    let factor = cents(factor, location);
    assert!(
        (0..=1000).contains(&score) && (0..=1000).contains(&factor),
        "{location}"
    );
    // Each nonnegative component is factored and rounded to hundredths before summing.
    (score * factor + 50) / 100
}

fn check_pcs(programs: &str, components: &str) {
    let components = rows(components, COMPONENT_SCHEMA);
    for p in rows(programs, PROGRAM_SCHEMA) {
        let key = identity(&p);
        let location = format!("{key:?} {}", p.get("name"));
        let selected: Vec<_> = components.iter().filter(|c| identity(c) == key).collect();
        assert_eq!(selected.len(), 3, "{location}");
        let mut total = 0;
        for (i, c) in selected.iter().enumerate() {
            let loc = format!("{location}, component {} {}", c.get("row"), c.get("name"));
            assert_eq!(c.get("row").parse::<usize>().unwrap(), i + 1, "{loc}");
            assert_eq!(
                c.get("name"),
                ["Composition", "Presentation", "Skating Skills"][i],
                "{loc}"
            );
            for value in c.grades() {
                assert!((0..=1000).contains(&cents(value, &loc)), "{loc}");
            }
            assert_eq!(c.get("referee"), "", "{loc}");
            total += factored_component(c.get("score"), c.get("factor"), &loc);
        }
        assert_eq!(
            total,
            cents(p.get("pcs"), &location),
            "factored PCS: {location}"
        );
    }
}

#[test]
fn factored_pcs_matches_every_program_with_component_rounding() {
    check_pcs(PROGRAMS, COMPONENTS);
    // Canada M p. 6, rank 11: rounding the sum instead gives 62.90.
    let kim =
        ["6.11", "6.32", "6.46"].map(|score| factored_component(score, "3.33", "Hyungyeom KIM"));
    assert_eq!(kim, [2035, 2105, 2151]);
    assert_eq!(kim.iter().sum::<i64>(), 6291);
    // Two further source-grounded one-cent rounding cases.
    assert_eq!(
        ["8.14", "8.18", "8.11"]
            .map(|s| factored_component(s, "3.33", "Aleksandr SELEVKO"))
            .iter()
            .sum::<i64>(),
        8136
    );
    assert_eq!(
        ["5.29", "4.82", "6.29"]
            .map(|s| factored_component(s, "2.67", "Seojin YOUN"))
            .iter()
            .sum::<i64>(),
        4378
    );
}

#[test]
#[should_panic(expected = "factored PCS: (\"gpcan2026-M\", \"6\", \"11\") Hyungyeom KIM")]
fn pcs_rejects_rounding_only_after_summing_components() {
    let corrupted = PROGRAMS.replacen("121.43\t59.52\t62.91", "121.43\t59.52\t62.90", 1);
    check_pcs(&corrupted, COMPONENTS);
}

#[test]
fn official_source_editions_and_selected_programs_are_pinned() {
    let expected = [
        (
            "gpfra2026-M",
            "ff61bf7603ce88a94d1794b1caaff09869eb524995d7e6a7cde95f52b041be78",
            "25.10.2026 15:03",
        ),
        (
            "gpfra2026-W",
            "7501c3305802e0ef42a56e616d2dd709d4bd844199346746d18284e789917c9b",
            "24.10.2026 21:22",
        ),
        (
            "gpcan2026-M",
            "f95e8e2ee7f820780ab76d9b15ef326d9620be3e10d9fb1d856b79799023a961",
            "31.10.2026 18:34",
        ),
        (
            "gpcan2026-W",
            "0d475f51aaff8001213cb44bac462db41d936f513bac1cc11aa8fc89e44345a1",
            "01.11.2026 13:55",
        ),
    ];
    let sources = rows(SOURCES, SOURCE_SCHEMA);
    assert_eq!(sources.len(), 4);
    let programs = rows(PROGRAMS, PROGRAM_SCHEMA);
    assert_eq!(programs.len(), 12);
    let mut people = BTreeSet::new();
    let mut starts = BTreeSet::new();
    for (source, digest, printed) in expected {
        let s = sources.iter().find(|s| s.get("source") == source).unwrap();
        let (event, gender) = source.split_once('-').unwrap();
        assert_eq!(
            s.get("index_url"),
            format!("https://results.isu.org/results/season2627/{event}/")
        );
        assert_eq!(
            s.get("segment"),
            if gender == "M" {
                "MEN FREE SKATING"
            } else {
                "WOMEN FREE SKATING"
            }
        );
        assert_eq!(
            s.get("segment_id"),
            format!("FSK{gender}SINGLES-----------FNL-000100--")
        );
        assert_eq!(
            s.get("pdf_url"),
            format!(
                "{}{}_JudgesDetailsperSkater.pdf",
                s.get("index_url"),
                s.get("segment_id")
            )
        );
        assert_eq!(s.get("retrieved"), "2026-11-04");
        assert_eq!(s.get("sha256"), digest, "source edition: {source}");
        assert_eq!(s.get("printed"), printed, "report creation: {source}");
        assert_eq!(s.get("pages"), "6");
        let selected: Vec<_> = programs
            .iter()
            .filter(|p| p.get("source") == source)
            .collect();
        assert_eq!(selected.len(), 3, "{source}");
        for p in selected {
            assert!((1..=6).contains(&p.get("page").parse::<usize>().unwrap()));
            assert!(
                people.insert((p.get("name"), p.get("nation"))),
                "duplicate skater: {p:?}"
            );
            assert!(
                starts.insert((source, p.get("starting_number"))),
                "duplicate start: {p:?}"
            );
        }
    }
}

fn check_parity(elements: &str) {
    let mut manifest = BTreeMap::new();
    for r in rows(REVIEWED, REVIEW_SCHEMA) {
        let values: Vec<_> = r.get("printed_fields").split('|').collect();
        let ordinal = if r.get("kind") == "program" {
            "0"
        } else {
            values[0]
        };
        assert!(
            manifest
                .insert((identity(&r), r.get("kind"), ordinal), values)
                .is_none(),
            "duplicate source location: {r:?}"
        );
    }
    let mut count = 0;
    for (kind, text, schema) in [
        ("program", PROGRAMS, PROGRAM_SCHEMA),
        ("element", elements, ELEMENT_SCHEMA),
        ("component", COMPONENTS, COMPONENT_SCHEMA),
        ("deduction", DEDUCTIONS, DEDUCTION_SCHEMA),
    ] {
        for r in rows(text, schema) {
            let ordinal = if kind == "program" { "0" } else { r.get("row") };
            let key = (identity(&r), kind, ordinal);
            let original = manifest
                .remove(&key)
                .unwrap_or_else(|| panic!("missing/duplicate location {key:?}"));
            let facts: Vec<_> = r
                .fields
                .iter()
                .skip(3)
                .filter(|(name, _)| *name != "support")
                .collect();
            assert_eq!(original.len(), facts.len(), "{key:?}");
            for ((field, actual), expected) in facts.into_iter().zip(original) {
                assert_eq!(*actual, expected, "{key:?}, field {field}");
            }
            count += 1;
        }
    }
    assert!(
        manifest.is_empty(),
        "source rows silently omitted: {manifest:?}"
    );
    assert_eq!(count, 187);
}

#[test]
fn every_printed_field_matches_separately_reviewed_source_manifest() {
    check_parity(ELEMENTS);
}

#[test]
#[should_panic(expected = "field j1")]
fn parity_detects_grade_column_corruption_with_location() {
    let corrupted = ELEMENTS.replacen("1.90\t2\t2", "1.90\t1\t2", 1);
    check_parity(&corrupted);
}

#[test]
fn arithmetic_inventory_and_missing_grades_are_explicit() {
    let programs = rows(PROGRAMS, PROGRAM_SCHEMA);
    let elements = rows(ELEMENTS, ELEMENT_SCHEMA);
    let components = rows(COMPONENTS, COMPONENT_SCHEMA);
    let deductions = rows(DEDUCTIONS, DEDUCTION_SCHEMA);
    let keys: BTreeSet<_> = programs.iter().map(|p| identity(p)).collect();
    assert_eq!(keys.len(), 12);
    for r in elements.iter().chain(&components).chain(&deductions) {
        assert!(keys.contains(&identity(r)), "orphan row: {r:?}");
    }
    for p in &programs {
        let key = identity(p);
        let location = format!("{key:?} {}", p.get("name"));
        let selected: Vec<_> = elements.iter().filter(|e| identity(e) == key).collect();
        assert_eq!(selected.len(), 11, "{location}");
        let mut bv = 0;
        let mut tes = 0;
        for (i, e) in selected.iter().enumerate() {
            let loc = format!("{location}, element {} {}", e.get("row"), e.get("code"));
            assert_eq!(e.get("row").parse::<usize>().unwrap(), i + 1, "{loc}");
            assert!(matches!(e.get("factor"), "" | "x"), "{loc}");
            assert_eq!(e.get("referee"), "", "Ref. column is blank: {loc}");
            let base = cents(e.get("base"), &loc);
            let goe = cents(e.get("goe"), &loc);
            let score = cents(e.get("score"), &loc);
            assert_eq!(base + goe, score, "BASE + GOE: {loc}");
            bv += base;
            tes += score;
            let missing = e.grades().iter().filter(|g| **g == "-").count();
            if missing > 0 {
                assert_eq!(missing, 9, "{loc}");
                assert!(e.get("code").contains('*'), "{loc}");
                assert_eq!((base, goe, score), (0, 0, 0), "{loc}");
            } else {
                for grade in &e.grades() {
                    assert!((-5..=5).contains(&grade.parse::<i8>().unwrap()), "{loc}");
                }
            }
        }
        assert_eq!(
            bv,
            cents(p.get("base_total"), &location),
            "BASE subtotal: {location}"
        );
        assert_eq!(tes, cents(p.get("tes"), &location), "TES: {location}");
        assert_eq!(
            tes,
            cents(p.get("element_total"), &location),
            "element footer: {location}"
        );
        let mut deduction_total = 0;
        for (i, d) in deductions.iter().filter(|d| identity(d) == key).enumerate() {
            assert_eq!(d.get("row").parse::<usize>().unwrap(), i + 1, "{location}");
            assert!(cents(d.get("amount"), &location) < 0, "{location}");
            if !d.get("count").is_empty() {
                assert!(d.get("count").parse::<usize>().unwrap() > 0, "{location}");
            }
            deduction_total += cents(d.get("amount"), &location);
        }
        assert_eq!(
            deduction_total,
            cents(p.get("deductions"), &location),
            "deduction rows: {location}"
        );
        assert_eq!(
            tes + cents(p.get("pcs"), &location) + deduction_total,
            cents(p.get("segment_score"), &location),
            "segment total, deductions separate from TES: {location}"
        );
    }
    for code in [
        "4T+2T",
        "3A",
        "2A*",
        "4T+REP",
        "3Lz<<+REP",
        "CCoSp2",
        "FCSp3",
        "CCoSp4V",
        "3Aq+3Tq",
        "3F!+2T",
    ] {
        assert!(
            elements.iter().any(|e| e.get("code") == code),
            "coverage missing {code}"
        );
    }
    assert_eq!(
        elements
            .iter()
            .filter(|e| e.get("code").contains('*'))
            .count(),
        2
    );
    assert!(elements
        .iter()
        .any(|e| e.get("factor") == "x" && e.get("code").contains("+SEQ")));
}
