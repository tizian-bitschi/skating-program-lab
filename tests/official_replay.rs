use skating_program_lab::*;
use std::collections::BTreeMap;

#[path = "support/protocol_recognition.rs"]
mod protocol_recognition;
use protocol_recognition::recognition;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const ELEMENTS: &str = include_str!("fixtures/grand-prix-2026-27/elements.tsv");
const PROGRAMS: &str = include_str!("fixtures/grand-prix-2026-27/programs.tsv");
const REVIEWED: &str = include_str!("fixtures/grand-prix-2026-27/reviewed-source.tsv");
const SOURCES: &str = include_str!("fixtures/grand-prix-2026-27/sources.tsv");
const SOV: &str = include_str!("fixtures/isu-2786-2026-06-02-source-values.tsv");
const DEDUCTIONS: &str = include_str!("fixtures/grand-prix-2026-27/deductions.tsv");

type Record = BTreeMap<String, String>;

fn records(text: &str) -> Vec<Record> {
    let mut lines = text.lines().filter(|s| !s.starts_with('#'));
    let header: Vec<_> = lines.next().unwrap().split('\t').collect();
    lines
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            assert_eq!(cells.len(), header.len(), "{line}");
            header
                .iter()
                .zip(cells)
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .collect()
}

fn cents(text: &str) -> i64 {
    let (whole, fraction) = text.trim_start_matches('-').split_once('.').unwrap();
    assert_eq!(fraction.len(), 2);
    let n = whole.parse::<i64>().unwrap() * 100 + fraction.parse::<i64>().unwrap();
    if text.starts_with('-') {
        -n
    } else {
        n
    }
}

fn same_program(a: &Record, b: &Record) -> bool {
    ["source", "page", "rank"].iter().all(|k| a[*k] == b[*k])
}

#[derive(Debug)]
struct Difference {
    field: String,
    expected: String,
    actual: String,
    delta_hundredths: Option<i64>,
}

#[derive(Debug)]
struct RowReport {
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
        self.differences.is_empty() && self.missing.is_empty() && self.score.is_some()
    }
    fn number(&mut self, field: &str, expected: i64, actual: Option<i64>) {
        if actual != Some(expected) {
            self.differences.push(Difference {
                field: field.into(),
                expected: expected.to_string(),
                actual: format!("{actual:?}"),
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
fn reviewed_element(program: &Record, index: usize) -> Record {
    let manifest = records(REVIEWED);
    let row = manifest
        .iter()
        .find(|r| {
            same_program(r, program)
                && r["kind"] == "element"
                && r["printed_fields"].split('|').next() == Some(&(index + 1).to_string())
        })
        .unwrap();
    let header: Vec<_> = ELEMENTS
        .lines()
        .next()
        .unwrap()
        .split('\t')
        .skip(3)
        .take(17)
        .collect();
    header
        .into_iter()
        .zip(row["printed_fields"].split('|'))
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
}

fn replay(program: &Record, elements: &[Record]) -> ReplayReport {
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
            index + 1
        );
        let mut report = RowReport {
            location: format!("{location} / {}", source["pdf_url"]),
            raw_code: row["code"].clone(),
            canonical: None,
            table_rows: vec![],
            outcome: None,
            differences: vec![],
            missing: vec![],
            score: None,
        };
        let expected = reviewed_element(program, index);
        for (field, value) in &expected {
            if row.get(field) != Some(value) {
                report.differences.push(Difference {
                    field: field.clone(),
                    expected: value.clone(),
                    actual: row.get(field).cloned().unwrap_or_default(),
                    delta_hundredths: None,
                });
            }
        }
        // Refuse unknown tokens before invoking the bounded fixture adapter.
        let known = row["code"].split('+').all(|token| {
            matches!(token, "SEQ" | "REP")
                || sov.iter().any(|s| s["code"] == token.trim_end_matches('*'))
        });
        let panel = if known {
            let mut panel = recognition(&row["code"]);
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
            Err(e) => report.missing.push(format!("BV/GOE/score: {e:?}")),
            Ok(score) => {
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
            assert_ne!(d.expected, d.actual);
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
    assert!(report.rows[0].differences.iter().any(|d| d.field == "row"));
    let mut elements = program_elements(&program);
    elements.remove(0);
    assert!(replay(&program, &elements).exact_tes.is_none());
}

#[test]
fn complete_source_linked_manual_envelope_keeps_movement_evidence_missing() {
    let program = records(PROGRAMS)
        .into_iter()
        .find(|p| p["name"] == "Ami NAKAI")
        .unwrap();
    let source = records(SOURCES)
        .into_iter()
        .find(|s| s["source"] == program["source"])
        .unwrap();
    let entries: Vec<_> = program_elements(&program)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let mut panel = recognition(&row["code"]);
            panel.planned_element = Some(index);
            panel.source = Some(SourceReference {
                document: source["pdf_url"].clone(),
                version: source["sha256"].clone(),
                location: format!("Ami NAKAI / page 1 rank 1 element {}", index + 1),
            });
            // A hypothetical family envelope only. Notation supplies no intended
            // jump choreography, transition, spin entry, feature order or start time.
            let (planned, evidence) = if row["code"].starts_with(char::is_numeric) {
                (
                    PlannedElement::Jump(JumpElement::Solo(ListedJump {
                        kind: None,
                        rotations: None,
                    })),
                    ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false,
                    },
                )
            } else if row["code"] == "StSq4" {
                (
                    PlannedElement::StepSequence,
                    ElementPlanEvidence::Step {
                        level: Level::Four,
                        clean_difficult_turns: None,
                        full_ice_coverage: None,
                        movement: None,
                        qualified_features: None,
                        qualification_assumed: false,
                    },
                )
            } else if row["code"] == "ChSq1" {
                (
                    PlannedElement::ChoreographicSequence { movements: None },
                    ElementPlanEvidence::ChoreographicSequence {
                        movements: None,
                        clearly_visible: None,
                    },
                )
            } else if row["code"] == "ChSp1" {
                (
                    PlannedElement::ChoreographicSpin,
                    ElementPlanEvidence::ChoreographicSpin {
                        consecutive_revolutions_per_foot: None,
                    },
                )
            } else {
                (
                    PlannedElement::RegularSpin(RegularSpin {
                        entry: None,
                        steps: None,
                    }),
                    ElementPlanEvidence::Spin {
                        spin: RegularSpinEvidence {
                            panel: Some(panel.clone()),
                            ..Default::default()
                        },
                        local_feature_qualification_assumed: false,
                    },
                )
            };
            ManualProgramEntry {
                planned,
                evidence,
                intended_recognition: None,
                scenario: Some(ManualElementScenario {
                    recognition: panel,
                    annotations: vec![],
                    final_goe: None,
                    fall: None,
                    execution: None,
                }),
            }
        })
        .collect();
    let registry = ProfileRegistry::senior();
    let request = ManualProgramRequest {
        profile: senior_profile(SeniorCategory::Women).id,
        scoring_table: TABLE,
        policy: senior_program_policy(SeniorCategory::Women),
        submission: SubmissionState::Complete,
        entries,
        request_hypothetical_total: false,
        second_half_boundary: None,
    };
    let assessment = assess_manual_program(&registry, &request).unwrap();
    let legality = assess_program_legality(
        &registry,
        &ProfileProgram {
            profile: request.profile.clone(),
            scoring_table: TABLE,
            elements: request.entries.iter().map(|e| e.planned.clone()).collect(),
        },
        &request
            .entries
            .iter()
            .map(|e| e.evidence.clone())
            .collect::<Vec<_>>(),
        &request.policy,
        SubmissionState::Complete,
    )
    .unwrap();
    assert_eq!(assessment.elements.len(), 11);
    assert_eq!(assessment.request, request);
    assert_eq!(assessment.validation, legality.validation);
    assert_eq!(assessment.status, AssessmentStatus::Indeterminate);
    assert!(assessment.hypothetical_total.is_none());
    assert!(legality
        .rules
        .iter()
        .any(|r| r.state == RuleCoverageState::MissingEvidence));
    assert!(legality.rules.iter().all(|r| !r.source.location.is_empty()));
    for entry in &assessment.request.entries {
        let scenario = entry.scenario.as_ref().unwrap();
        assert!(scenario.final_goe.is_none() && scenario.execution.is_none());
        assert!(scenario
            .recognition
            .source
            .as_ref()
            .unwrap()
            .location
            .contains("Ami NAKAI"));
    }
}
