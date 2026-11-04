use std::collections::{BTreeMap, BTreeSet};

const SOURCES: &str = include_str!("fixtures/grand-prix-2026-27/sources.tsv");
const PROGRAMS: &str = include_str!("fixtures/grand-prix-2026-27/programs.tsv");
const ELEMENTS: &str = include_str!("fixtures/grand-prix-2026-27/elements.tsv");
const COMPONENTS: &str = include_str!("fixtures/grand-prix-2026-27/components.tsv");
const DEDUCTIONS: &str = include_str!("fixtures/grand-prix-2026-27/deductions.tsv");
const REVIEWED: &str = include_str!("fixtures/grand-prix-2026-27/reviewed-source.tsv");

fn rows(text: &str, columns: usize) -> Vec<Vec<&str>> {
    text.lines()
        .skip(1)
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            assert_eq!(cells.len(), columns, "column count: {line}");
            cells
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

fn identity<'a>(row: &[&'a str]) -> (&'a str, &'a str, &'a str) {
    (row[0], row[1], row[2])
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
    let sources = rows(SOURCES, 10);
    assert_eq!(sources.len(), 4);
    let programs = rows(PROGRAMS, 12);
    assert_eq!(programs.len(), 12);
    let mut people = BTreeSet::new();
    let mut starts = BTreeSet::new();
    for (source, digest, printed) in expected {
        let s = sources.iter().find(|s| s[0] == source).unwrap();
        let (event, gender) = source.split_once('-').unwrap();
        assert_eq!(
            s[2],
            format!("https://results.isu.org/results/season2627/{event}/")
        );
        assert_eq!(
            s[3],
            if gender == "M" {
                "MEN FREE SKATING"
            } else {
                "WOMEN FREE SKATING"
            }
        );
        assert_eq!(s[4], format!("FSK{gender}SINGLES-----------FNL-000100--"));
        assert_eq!(s[5], format!("{}{}_JudgesDetailsperSkater.pdf", s[2], s[4]));
        assert_eq!(s[6], "2026-11-04");
        assert_eq!(s[7], digest, "source edition: {source}");
        assert_eq!(s[8], printed, "report creation: {source}");
        assert_eq!(s[9], "6");
        let selected: Vec<_> = programs.iter().filter(|p| p[0] == source).collect();
        assert_eq!(selected.len(), 3, "{source}");
        for p in selected {
            assert!((1..=6).contains(&p[1].parse::<usize>().unwrap()));
            assert!(people.insert((p[3], p[4])), "duplicate skater: {p:?}");
            assert!(starts.insert((source, p[5])), "duplicate start: {p:?}");
        }
    }
}

fn check_parity(elements: &str) {
    let mut manifest = BTreeMap::new();
    for r in rows(REVIEWED, 5) {
        let values: Vec<_> = r[4].split('|').collect();
        let ordinal = if r[3] == "program" { "0" } else { values[0] };
        assert!(
            manifest
                .insert((identity(&r), r[3], ordinal), values)
                .is_none(),
            "duplicate source location: {r:?}"
        );
    }
    let mut count = 0;
    for (kind, text, columns) in [
        ("program", PROGRAMS, 12),
        ("element", elements, 21),
        ("component", COMPONENTS, 17),
        ("deduction", DEDUCTIONS, 7),
    ] {
        let fields: Vec<_> = text.lines().next().unwrap().split('\t').collect();
        for r in rows(text, columns) {
            let ordinal = if kind == "program" { "0" } else { r[3] };
            let key = (identity(&r), kind, ordinal);
            let original = manifest
                .remove(&key)
                .unwrap_or_else(|| panic!("missing/duplicate location {key:?}"));
            let end = if kind == "element" {
                r.len() - 1
            } else {
                r.len()
            };
            assert_eq!(original.len(), end - 3, "{key:?}");
            for (i, (actual, expected)) in r[3..end].iter().zip(original).enumerate() {
                assert_eq!(*actual, expected, "{key:?}, field {}", fields[i + 3]);
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
    let programs = rows(PROGRAMS, 12);
    let elements = rows(ELEMENTS, 21);
    let components = rows(COMPONENTS, 17);
    let deductions = rows(DEDUCTIONS, 7);
    let keys: BTreeSet<_> = programs.iter().map(|p| identity(p)).collect();
    assert_eq!(keys.len(), 12);
    for r in elements.iter().chain(&components).chain(&deductions) {
        assert!(keys.contains(&identity(r)), "orphan row: {r:?}");
    }
    for p in &programs {
        let key = identity(p);
        let location = format!("{key:?} {}", p[3]);
        let selected: Vec<_> = elements.iter().filter(|e| identity(e) == key).collect();
        assert_eq!(selected.len(), 11, "{location}");
        let mut bv = 0;
        let mut tes = 0;
        for (i, e) in selected.iter().enumerate() {
            let loc = format!("{location}, element {} {}", e[3], e[4]);
            assert_eq!(e[3].parse::<usize>().unwrap(), i + 1, "{loc}");
            assert!(matches!(e[7], "" | "x"), "{loc}");
            assert!(
                matches!(
                    e[20],
                    "supported" | "requires #78/#77" | "recognition-unknown" | "outside-scope"
                ),
                "{loc}"
            );
            assert_eq!(e[18], "", "Ref. column is blank: {loc}");
            let base = cents(e[6], &loc);
            let goe = cents(e[8], &loc);
            let score = cents(e[19], &loc);
            assert_eq!(base + goe, score, "BASE + GOE: {loc}");
            bv += base;
            tes += score;
            let missing = e[9..18].iter().filter(|g| **g == "-").count();
            if missing > 0 {
                assert_eq!(missing, 9, "{loc}");
                assert!(e[4].contains('*'), "{loc}");
                assert_eq!((base, goe, score), (0, 0, 0), "{loc}");
            } else {
                for grade in &e[9..18] {
                    assert!((-5..=5).contains(&grade.parse::<i8>().unwrap()), "{loc}");
                }
            }
        }
        assert_eq!(bv, cents(p[10], &location), "BASE subtotal: {location}");
        assert_eq!(tes, cents(p[7], &location), "TES: {location}");
        assert_eq!(tes, cents(p[11], &location), "element footer: {location}");
        let mut deduction_total = 0;
        for (i, d) in deductions.iter().filter(|d| identity(d) == key).enumerate() {
            assert_eq!(d[3].parse::<usize>().unwrap(), i + 1, "{location}");
            assert!(cents(d[4], &location) < 0, "{location}");
            if !d[5].is_empty() {
                assert!(d[5].parse::<usize>().unwrap() > 0, "{location}");
            }
            deduction_total += cents(d[4], &location);
        }
        assert_eq!(
            deduction_total,
            cents(p[9], &location),
            "deduction rows: {location}"
        );
        assert_eq!(
            tes + cents(p[8], &location) + deduction_total,
            cents(p[6], &location),
            "segment total, deductions separate from TES: {location}"
        );
        let pcs: Vec<_> = components.iter().filter(|c| identity(c) == key).collect();
        assert_eq!(pcs.len(), 3, "{location}");
        for (i, c) in pcs.iter().enumerate() {
            assert_eq!(c[3].parse::<usize>().unwrap(), i + 1, "{location}");
            assert_eq!(c[4], ["Composition", "Presentation", "Skating Skills"][i]);
            for value in &c[5..15] {
                assert!((0..=1000).contains(&cents(value, &location)), "{location}");
            }
            assert_eq!(c[15], "");
            cents(c[16], &location);
        }
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
            elements.iter().any(|e| e[4] == code),
            "coverage missing {code}"
        );
    }
    assert_eq!(elements.iter().filter(|e| e[4].contains('*')).count(), 2);
    assert!(elements
        .iter()
        .any(|e| e[7] == "x" && e[4].contains("+SEQ")));
}
