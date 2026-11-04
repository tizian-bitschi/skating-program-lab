use skating_program_lab::*;
use std::collections::BTreeMap;

#[path = "support/protocol_recognition.rs"]
mod protocol_recognition;
use protocol_recognition::{recognition, try_recognition};

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const ELEMENTS: &str = include_str!("fixtures/grand-prix-2026-27/elements.tsv");
const PROGRAMS: &str = include_str!("fixtures/grand-prix-2026-27/programs.tsv");
const REVIEWED: &str = include_str!("fixtures/grand-prix-2026-27/reviewed-source.tsv");
const SOURCES: &str = include_str!("fixtures/grand-prix-2026-27/sources.tsv");
const SOV: &str = include_str!("fixtures/isu-2786-2026-06-02-source-values.tsv");
const DEDUCTIONS: &str = include_str!("fixtures/grand-prix-2026-27/deductions.tsv");

type Record = BTreeMap<String, String>;

#[path = "support/fixture.rs"]
mod fixture;
use fixture::{cents as checked_cents, rows};

fn records(text: &str) -> Vec<Record> {
    let schema = text.lines().find(|s| !s.starts_with('#')).unwrap();
    let data = text
        .lines()
        .filter(|s| !s.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    rows(&data, schema)
        .into_iter()
        .map(|r| {
            r.fields
                .iter()
                .map(|(k, _)| (k.to_string(), r.get(k).to_string()))
                .collect()
        })
        .collect()
}

fn cents(text: &str) -> i64 {
    checked_cents(text, "official replay amount")
}

fn same_program(a: &Record, b: &Record) -> bool {
    ["source", "page", "rank"].iter().all(|k| a[*k] == b[*k])
}

#[derive(Debug)]
struct Difference {
    field: String,
    expected: String,
    actual: Option<String>,
    delta_hundredths: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
enum SupportState {
    Exact,
    Mismatch,
    Unsupported,
    Unreproducible,
}

#[derive(Debug)]
struct RowReport {
    support: SupportState,
    location: String,
    raw_code: String,
    canonical: Option<String>,
    table_rows: Vec<String>,
    outcome: Option<BaseValueOutcome>,
    differences: Vec<Difference>,
    missing: Vec<String>,
    score: Option<i64>,
}

impl RowReport {
    fn exact(&self) -> bool {
        self.support == SupportState::Exact
            && self.differences.is_empty()
            && self.missing.is_empty()
            && self.score.is_some()
    }
    fn number(&mut self, field: &str, expected: i64, actual: Option<i64>) {
        if actual != Some(expected) {
            self.differences.push(Difference {
                field: field.into(),
                expected: expected.to_string(),
                actual: actual.map(|n| n.to_string()),
                delta_hundredths: actual.map(|n| n - expected),
            });
        }
    }
}

#[derive(Debug)]
struct ReplayReport {
    skater: String,
    group: String,
    rows: Vec<RowReport>,
    published_tes: i64,
    exact_tes: Option<i64>,
    verified_subtotal: i64,
    deductions: i64,
    program_gaps: Vec<String>,
}

// The reviewed transcription is an independent source oracle, never scorer output.
fn reviewed_element(manifest: &str, program: &Record, ordinal: &str) -> Option<Record> {
    let manifest = records(manifest);
    let row = manifest.iter().find(|r| {
        same_program(r, program)
            && r["kind"] == "element"
            && r["printed_fields"].split('|').next() == Some(ordinal)
    })?;
    let header: Vec<_> = ELEMENTS
        .lines()
        .next()
        .unwrap()
        .split('\t')
        .skip(3)
        .take(17)
        .collect();
    Some(
        header
            .into_iter()
            .zip(row["printed_fields"].split('|'))
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
    )
}

fn replay(program: &Record, elements: &[Record]) -> ReplayReport {
    replay_with_manifest(program, elements, REVIEWED)
}

fn replay_with_manifest(program: &Record, elements: &[Record], manifest: &str) -> ReplayReport {
    let source = records(SOURCES)
        .into_iter()
        .find(|s| s["source"] == program["source"])
        .unwrap();
    let category = if program["source"].ends_with('W') {
        SeniorCategory::Women
    } else {
        SeniorCategory::Men
    };
    let mut reports = vec![];
    let mut inputs = vec![];
    let sov = records(SOV);
    for (index, row) in elements.iter().enumerate() {
        let location = format!(
            "{} / {} / {} / page {} rank {} element {}",
            source["event"],
            source["segment"],
            program["name"],
            program["page"],
            program["rank"],
            row["row"]
        );
        let mut report = RowReport {
            support: SupportState::Unreproducible,
            location: format!("{location} / {}", source["pdf_url"]),
            raw_code: row["code"].clone(),
            canonical: None,
            table_rows: vec![],
            outcome: None,
            differences: vec![],
            missing: vec![],
            score: None,
        };
        let expected = reviewed_element(manifest, program, &row["row"]);
        if index > 0
            && elements[index - 1]["row"].parse::<usize>().ok() >= row["row"].parse::<usize>().ok()
        {
            report.differences.push(Difference {
                field: "order".into(),
                expected: "strictly increasing source ordinals".into(),
                actual: Some(row["row"].clone()),
                delta_hundredths: None,
            });
        }
        if expected.is_none() {
            report
                .missing
                .push("source manifest: no matching element ordinal".into());
        }
        for (field, value) in expected.iter().flatten() {
            if row.get(field) != Some(value) {
                report.differences.push(Difference {
                    field: field.clone(),
                    expected: value.clone(),
                    actual: row.get(field).cloned(),
                    delta_hundredths: None,
                });
            }
        }
        // Refuse unknown tokens before invoking the bounded fixture adapter.
        let known = row["code"].split('+').all(|token| {
            matches!(token, "SEQ" | "REP")
                || sov.iter().any(|s| s["code"] == token.trim_end_matches('*'))
        });
        let panel = if let Some(mut panel) = known.then(|| try_recognition(&row["code"])).flatten()
        {
            report.canonical = panel.details.as_ref().and_then(|d| d.notation());
            panel.source = Some(SourceReference {
                document: source["pdf_url"].clone(),
                version: source["sha256"].clone(),
                location: location.clone(),
            });
            panel
        } else {
            report
                .missing
                .push("code: unsupported recognition/call; no canonical mapping".into());
            PanelRecognition {
                ruleset: RulesetId::IsuSenior2026_27June12,
                source: None,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Unknown,
                details: None,
            }
        };
        inputs.push(ProtocolElementInput {
            recognition: panel,
            annotations: vec![],
            judges: Some(JudgePanelGrades {
                grades: (1..=9)
                    .map(|j| row[&format!("j{j}")].parse().ok())
                    .collect(),
            }),
            printed_second_half: row["factor"] == "x",
            source: SourceReference {
                document: source["pdf_url"].clone(),
                version: source["sha256"].clone(),
                location,
            },
            execution: None,
        });
        reports.push(report);
    }
    let scored = score_protocol_program(
        &ProfileRegistry::senior(),
        &senior_profile(category).id,
        TABLE,
        &inputs,
    )
    .unwrap();
    for ((row, report), actual) in elements.iter().zip(&mut reports).zip(scored.elements) {
        match actual {
            Err(e) => {
                report.support = SupportState::Unsupported;
                report.missing.push(format!("BV/GOE/score: {e:?}"));
                for (field, printed) in [
                    ("credited BASE", "base"),
                    ("GOE", "goe"),
                    ("score", "score"),
                ] {
                    report.number(field, cents(&row[printed]), None);
                }
            }
            Ok(score) => {
                report.support = SupportState::Exact;
                report.outcome = Some(score.table_base.clone());
                // Each component's table BASE comes from the independently extracted SOV.
                // Whole stars remain NoValue; zero is not a fabricated table row.
                if let BaseValueOutcome::Value(base) = &score.table_base {
                    let mut expected_base = 0;
                    for token in row["code"]
                        .split('+')
                        .filter(|s| !matches!(*s, "SEQ" | "REP") && !s.ends_with('*'))
                    {
                        if let Some(s) = sov.iter().find(|s| s["code"] == token) {
                            expected_base += s["BASE"].parse::<i64>().unwrap();
                            report.table_rows.push(format!(
                                "Communication 2786 June 2 p. {} row {} {}",
                                s["page"], s["row"], s["code"]
                            ));
                        } else {
                            report
                                .missing
                                .push(format!("table BASE: missing SOV row {token}"));
                        }
                    }
                    report.number(
                        "table BASE",
                        expected_base,
                        Some(i64::from(base.hundredths)),
                    );
                    if base.notation
                        != report
                            .canonical
                            .clone()
                            .unwrap_or_default()
                            .replace("+REP", "")
                    {
                        report.missing.push(format!(
                            "canonical/SOV mapping: {} vs {}",
                            report.canonical.as_deref().unwrap_or("unknown"),
                            base.notation
                        ));
                    }
                }
                report.number(
                    "credited BASE",
                    cents(&row["base"]),
                    Some(i64::from(score.credited_base_hundredths)),
                );
                report.number(
                    "GOE",
                    cents(&row["goe"]),
                    score
                        .panel_goe
                        .as_ref()
                        .map(|g| i64::from(g.goe_hundredths))
                        .or_else(|| {
                            matches!(
                                score.table_base,
                                BaseValueOutcome::NoValue | BaseValueOutcome::Ignored
                            )
                            .then_some(0)
                        }),
                );
                report.number(
                    "score",
                    cents(&row["score"]),
                    score.score_hundredths.map(i64::from),
                );
                report.score = score.score_hundredths.map(i64::from);
                if !report.missing.is_empty() || report.score.is_none() {
                    report.support = SupportState::Unreproducible;
                } else if !report.differences.is_empty() {
                    report.support = SupportState::Mismatch;
                }
            }
        }
    }
    let mut gaps = vec![];
    if elements.len() != 11 {
        gaps.push(format!(
            "element count: expected 11, actual {}",
            elements.len()
        ));
    }
    let published_tes = cents(&program["tes"]);
    let subtotal = reports
        .iter()
        .filter(|r| r.exact())
        .filter_map(|r| r.score)
        .sum();
    let exact_tes = (gaps.is_empty()
        && reports.iter().all(RowReport::exact)
        && scored.tes_hundredths == Some(published_tes))
    .then_some(subtotal);
    if scored.tes_hundredths != Some(published_tes) {
        gaps.push(format!(
            "TES: expected {published_tes}, actual {:?}",
            scored.tes_hundredths
        ));
    }
    let deductions = records(DEDUCTIONS)
        .iter()
        .filter(|r| same_program(r, program))
        .map(|r| cents(&r["amount"]))
        .sum();
    if deductions != cents(&program["deductions"]) {
        gaps.push("deductions: source sum mismatch".into());
    }
    ReplayReport {
        skater: program["name"].clone(),
        group: program["source"].clone(),
        rows: reports,
        published_tes,
        exact_tes,
        verified_subtotal: subtotal,
        deductions,
        program_gaps: gaps,
    }
}

fn program_elements(program: &Record) -> Vec<Record> {
    records(ELEMENTS)
        .into_iter()
        .filter(|r| same_program(r, program))
        .collect()
}

#[test]
fn all_twelve_official_programs_have_independently_verified_exact_tes() {
    let mut groups = BTreeMap::<String, usize>::new();
    let mut count = 0;
    for program in records(PROGRAMS) {
        let report = replay(&program, &program_elements(&program));
        assert_eq!(
            report.exact_tes,
            Some(cents(&program["tes"])),
            "{report:#?}"
        );
        assert!(report.program_gaps.is_empty(), "{report:#?}");
        assert_eq!(report.verified_subtotal, report.published_tes);
        assert_eq!(report.deductions, cents(&program["deductions"]));
        println!(
            "{} | {} | exact | TES {} | subtotal {} | deductions {}",
            report.group,
            report.skater,
            report.published_tes,
            report.verified_subtotal,
            report.deductions
        );
        for r in &report.rows {
            assert!(!r.raw_code.is_empty() && r.canonical.is_some());
            if matches!(r.outcome, Some(BaseValueOutcome::Value(_))) {
                assert!(!r.table_rows.is_empty());
            }
        }
        count += report.rows.len();
        *groups.entry(report.group).or_default() += 1;
    }
    assert_eq!(groups.len(), 4);
    assert!(groups.values().all(|n| *n == 3));
    assert_eq!(count, 132);
}

#[test]
fn mutations_cannot_keep_an_exact_total_even_if_arithmetic_is_unchanged() {
    let program = records(PROGRAMS).remove(0);
    for (field, value) in [
        ("goe", "1.91"),
        ("j1", "3"),
        ("code", "4T+3T"),
        ("code", "4Tz"),
        ("code", "1Tw1"),
        ("code", "ChPLi1"),
        ("factor", "x"),
    ] {
        let mut elements = program_elements(&program);
        elements[0].insert(field.into(), value.into());
        let report = replay(&program, &elements);
        assert!(report.exact_tes.is_none(), "{report:#?}");
        let row = &report.rows[0];
        assert!(row.location.contains("Adam SIAO HIM FA") && row.location.contains("element 1"));
        assert!(
            row.differences.iter().any(|d| d.field == field),
            "{report:#?}"
        );
        for d in &row.differences {
            assert_ne!(Some(&d.expected), d.actual.as_ref());
            if let Some(delta) = d.delta_hundredths {
                assert_ne!(delta, 0);
            }
        }
        assert!(report.verified_subtotal < report.published_tes);
    }
    let mut elements = program_elements(&program);
    elements.swap(0, 1);
    let report = replay(&program, &elements);
    assert!(report.exact_tes.is_none());
    assert!(report
        .rows
        .iter()
        .any(|r| r.differences.iter().any(|d| d.field == "order")));
    let mut elements = program_elements(&program);
    elements.remove(0);
    let report = replay(&program, &elements);
    assert!(report.exact_tes.is_none());
    assert_eq!(report.verified_subtotal, report.published_tes - 1270);
    let mut elements = program_elements(&program);
    let mut extra = elements[0].clone();
    extra.insert("row".into(), "12".into());
    elements.push(extra);
    let report = replay(&program, &elements);
    assert!(report.exact_tes.is_none());
    assert!(report.rows[11]
        .missing
        .iter()
        .any(|m| m.contains("manifest")));
    assert!(report.rows[11].location.contains("element 12"));
}

#[path = "support/manual_replay.rs"]
mod manual_replay;

#[test]
fn unchanged_trimmed_goe_still_fails_source_verification() {
    let program = records(PROGRAMS).remove(0);
    let baseline = replay(&program, &program_elements(&program));
    let mut elements = program_elements(&program);
    elements[0].insert("j8".into(), "4".into());
    let report = replay(&program, &elements);
    assert_eq!(report.rows[0].score, baseline.rows[0].score);
    assert!(report.exact_tes.is_none());
    assert_eq!(report.rows[0].support, SupportState::Mismatch);
    assert!(report.rows[0].differences.iter().any(|d| d.field == "j8"));
}

#[test]
fn absent_reference_and_unknown_positions_keep_verified_subtotals() {
    let program = records(PROGRAMS).remove(0);
    let elements = program_elements(&program);
    let missing_reference = REVIEWED
        .lines()
        .filter(|line| !line.starts_with("gpfra2026-M\t1\t1\telement\t1|"))
        .collect::<Vec<_>>()
        .join("\n");
    let report = replay_with_manifest(&program, &elements, &missing_reference);
    assert!(report.exact_tes.is_none());
    assert_eq!(report.rows[0].support, SupportState::Unreproducible);
    assert_eq!(report.verified_subtotal, report.published_tes - 1270);
    let mut elements = elements;
    elements[0].insert("row".into(), "99".into());
    let report = replay(&program, &elements);
    assert!(report.exact_tes.is_none());
    assert!(report.rows[0]
        .missing
        .iter()
        .any(|m| m.contains("manifest")));
    assert!(report.rows[0].location.contains("element 99"));
    elements[0].insert("code".into(), "ChPLi1".into());
    let report = replay(&program, &elements);
    assert_eq!(report.rows[0].support, SupportState::Unsupported);
    for field in ["credited BASE", "GOE", "score"] {
        assert!(report.rows[0]
            .differences
            .iter()
            .any(|d| d.field == field && d.actual.is_none() && !d.expected.is_empty()));
    }
}
