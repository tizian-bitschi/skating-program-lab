use skating_program_lab::*;
use std::collections::BTreeMap;

#[path = "generation_defaults.rs"]
mod generation_defaults;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Small,
    Medium,
    Large,
}
impl std::fmt::Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        })
    }
}

pub const SOURCES: [&str; 4] = ["gpfra2026-W", "gpfra2026-M", "gpcan2026-W", "gpcan2026-M"];
pub const ELEMENTS: &str = include_str!("../fixtures/grand-prix-2026-27/elements.tsv");
pub const PROGRAMS: &str = include_str!("../fixtures/grand-prix-2026-27/programs.tsv");
pub const EDITIONS: &str = include_str!("../fixtures/grand-prix-2026-27/sources.tsv");

pub fn records(text: &str) -> Vec<BTreeMap<&str, &str>> {
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().unwrap().split('\t').collect();
    lines
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            assert_eq!(cells.len(), header.len());
            header.iter().copied().zip(cells).collect()
        })
        .collect()
}

pub fn source_rows(source: &str) -> Vec<BTreeMap<&'static str, &'static str>> {
    let rows: Vec<_> = records(ELEMENTS)
        .into_iter()
        .filter(|r| r["source"] == source && r["rank"] == "1")
        .collect();
    assert_eq!(rows.len(), 11);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row["row"].parse::<usize>().unwrap(), i + 1);
    }
    rows
}

pub fn category(source: &str) -> SeniorCategory {
    assert!(SOURCES.contains(&source));
    if source.ends_with('W') {
        SeniorCategory::Women
    } else {
        SeniorCategory::Men
    }
}

pub fn solo(code: &str) -> Option<ListedJump> {
    let rotations = code.chars().next()?.to_digit(10)? as u8;
    let kind = match &code[1..] {
        "A" => JumpKind::Axel,
        "T" => JumpKind::ToeLoop,
        "S" => JumpKind::Salchow,
        "Lo" => JumpKind::Loop,
        "F" => JumpKind::Flip,
        "Lz" => JumpKind::Lutz,
        _ => return None,
    };
    Some(ListedJump {
        kind: Some(kind),
        rotations: Some(rotations),
    })
}

// Conditional Basic plans: the protocol supplies families, never these motion facts.
pub fn generation(source: &str, scale: Scale) -> CandidateGenerationRequest {
    let profile = senior_profile(category(source));
    let mut g = generation_defaults::request(profile);
    g.solo_jumps = true;
    g.movement_plans = generation_defaults::basic_movements();
    g.jump_kinds = if scale == Scale::Small {
        vec![JumpKind::Axel]
    } else if scale == Scale::Medium {
        vec![JumpKind::ToeLoop, JumpKind::Axel]
    } else {
        vec![
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
            JumpKind::Axel,
        ]
    };
    g.rotations = if scale == Scale::Small {
        vec![2, 3]
    } else if scale == Scale::Medium {
        vec![1, 2]
    } else {
        vec![2, 3, 4]
    };
    g.base_spins = vec![
        SpinAbbreviation {
            kind: RegularSpinKind::Combination,
            entry: SpinEntry::NonFlying,
            change_foot: true,
        },
        SpinAbbreviation {
            kind: if source == "gpfra2026-W" {
                RegularSpinKind::Sit
            } else {
                RegularSpinKind::Camel
            },
            entry: SpinEntry::Flying,
            change_foot: false,
        },
    ];
    // Keep medium additions at exactly three explicit safe planning alternatives.
    if scale == Scale::Medium {
        g.exclusions = vec![PersonalExclusion::Jump(solo("1A").unwrap())];
        g.base_spins.clear();
        g.movement_plans.clear();
    }
    if scale == Scale::Large {
        g.direct_combinations = true;
        g.axel_sequences = true; // No direct-step evidence: omissions must remain audited.
        g.exclusions = vec![PersonalExclusion::Jump(solo("4Lz").unwrap())];
    }
    g
}

pub fn full(
    source: &str,
    variants: Vec<ProgramVariant>,
    slots: usize,
) -> ProgramOptimizationRequest {
    let p = senior_profile(category(source));
    ProgramOptimizationRequest {
        season: p.season,
        profile: p.id,
        scoring_table: p.scoring_table,
        policy: senior_program_policy(category(source)),
        variants,
        exclusions: vec![],
        must_have: vec![],
        min_elements: slots,
        max_elements: slots,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        equivalence: ProgramEquivalence::OrderedVariants,
        max_proposals: 200,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    }
}

pub fn small_full(source: &str, variants: Vec<ProgramVariant>) -> ProgramOptimizationRequest {
    let mut q = full(source, variants, 6);
    q.must_have = vec![(0..q.variants.len())
        .filter(|i| matches!(q.variants[*i].element, PlannedElement::Jump(_)))
        .collect()];
    q.must_have.extend(
        (0..q.variants.len())
            .filter(|i| !matches!(q.variants[*i].element, PlannedElement::Jump(_)))
            .map(|i| vec![i]),
    );
    q.max_proposals = 10_000;
    q
}

fn details(v: &ProgramVariant) -> RecognitionDetails {
    let (code, level) = match (&v.element, &v.evidence) {
        (PlannedElement::Jump(JumpElement::Solo(j)), _) => {
            let suffix = match j.kind.unwrap() {
                JumpKind::Axel => "A",
                JumpKind::ToeLoop => "T",
                JumpKind::Salchow => "S",
                JumpKind::Loop => "Lo",
                JumpKind::Flip => "F",
                JumpKind::Lutz => "Lz",
            };
            (format!("{}{suffix}", j.rotations.unwrap()), None)
        }
        (_, ElementPlanEvidence::Spin { spin, .. }) => {
            let a = spin.abbreviation.unwrap();
            let code = match (a.kind, a.entry, a.change_foot) {
                (RegularSpinKind::Combination, _, true) => "CCoSp",
                (RegularSpinKind::Camel, SpinEntry::Flying, false) => "FCSp",
                (RegularSpinKind::Sit, SpinEntry::Flying, false) => "FSSp",
                _ => panic!("benchmark spin"),
            };
            (code.into(), spin.planned_level)
        }
        (_, ElementPlanEvidence::Step { level, .. }) => ("StSq".into(), Some(*level)),
        (PlannedElement::ChoreographicSequence { .. }, _) => ("ChSq1".into(), None),
        (PlannedElement::ChoreographicSpin, _) => ("ChSp1".into(), None),
        _ => panic!("benchmark anchor"),
    };
    RecognitionDetails::new(
        vec![ComponentCode::try_from(code.as_str()).unwrap()],
        level,
        Some(vec![]),
    )
    .unwrap()
}

pub fn anchor(id: String, v: &ProgramVariant, source: Option<SourceReference>) -> CompletionEntry {
    CompletionEntry {
        id,
        locked: true,
        entry: ManualProgramEntry {
            planned: v.element.clone(),
            evidence: v.evidence.clone(),
            scenario: None,
            intended_recognition: Some(PanelRecognition {
                ruleset: RulesetId::IsuSenior2026_27June12,
                source,
                planned_element: None,
                observed_element: None,
                outcome: RecognitionOutcome::Recognized,
                details: Some(details(v)),
            }),
        },
    }
}

pub fn completion(
    source: &str,
    scale: Scale,
    domain: CandidateGenerationResult,
) -> ProgramCompletionRequest {
    let registry = ProfileRegistry::senior();
    let small = generate_program_candidates(&registry, &generation(source, Scale::Small)).unwrap();
    let roles: Vec<_> = small
        .variants
        .iter()
        .filter(|v| !matches!(v.element, PlannedElement::Jump(_)))
        .cloned()
        .collect();
    assert_eq!(roles.len(), 5);
    let mut original = vec![];
    if scale == Scale::Small {
        original = roles
            .iter()
            .enumerate()
            .map(|(i, v)| anchor(format!("{source}:conditional-role-{i}"), v, None))
            .collect();
    } else {
        let edition = records(EDITIONS)
            .into_iter()
            .find(|r| r["source"] == source)
            .unwrap();
        let mut jumps = 0;
        for row in source_rows(source) {
            let code = row["code"];
            let v = if let Some(j) = solo(code) {
                if jumps == 2 {
                    continue;
                }
                jumps += 1;
                ProgramVariant {
                    element: PlannedElement::Jump(JumpElement::Solo(j)),
                    evidence: ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false,
                    },
                    recognition_assumed: true,
                }
            } else {
                let role = roles.iter().find(|v| match (&v.element, &v.evidence) {
                    (_, ElementPlanEvidence::Spin { spin, .. }) => {
                        if spin.abbreviation.unwrap().entry == SpinEntry::Flying {
                            code.starts_with('F')
                        } else {
                            code.starts_with("CCoSp")
                        }
                    }
                    (PlannedElement::StepSequence, _) => code.starts_with("StSq"),
                    (PlannedElement::ChoreographicSequence { .. }, _) => code == "ChSq1",
                    (PlannedElement::ChoreographicSpin, _) => code == "ChSp1",
                    _ => false,
                });
                let Some(role) = role else {
                    continue;
                };
                role.clone()
            };
            let provenance = SourceReference {
                document: edition["pdf_url"].into(),
                version: edition["sha256"].into(),
                location: format!(
                    "page {} rank 1 row {}; conditional plan abstraction of {} (printed BASE {})",
                    row["page"], row["row"], code, row["base"]
                ),
            };
            original.push(anchor(
                format!("{source}:rank1:row{}", row["row"]),
                &v,
                Some(provenance),
            ));
        }
        assert_eq!(original.len(), 7);
    }
    let p = senior_profile(category(source));
    let n = original.len();
    let additions = if scale == Scale::Large {
        4
    } else if scale == Scale::Medium {
        3
    } else {
        1
    };
    // Manual assessment applies Senior second-solo credit, while this search
    // objective is unfactored. Exclude high solo additions to avoid that known
    // unsupported cross-contract case; compounds retain shared repetition checks.
    let exclusions = if scale == Scale::Large {
        domain.variants.iter().enumerate().filter_map(|(i,v)| {
            matches!(&v.element, PlannedElement::Jump(JumpElement::Solo(j)) if j.rotations.is_some_and(|r| r >= 3)).then_some(PersonalExclusion::Variant(i))
        }).collect()
    } else {
        vec![]
    };
    ProgramCompletionRequest {
        season: p.season,
        profile: p.id,
        scoring_table: p.scoring_table,
        policy: senior_program_policy(category(source)),
        original,
        variants: domain.variants.clone(),
        generated_domain: Some(domain),
        exclusions,
        global_exclusions: vec![],
        insertion: CompletionInsertionPolicy {
            boundaries: if scale == Scale::Small {
                (0..=n).collect()
            } else {
                vec![0, n / 2, n]
            },
            absolute_positions: vec![],
        },
        min_additions: additions,
        max_additions: additions,
        objective: OptimizerObjective::MaximizeUnfactoredBaseValueHundredths,
        max_proposals: 200,
        max_solutions: 1000,
        enumerate_equal_optima: true,
        time_limit: None,
    }
}

pub fn verify_originals(req: &ProgramCompletionRequest, candidate: &CompletedProgramCandidate) {
    let mut next = 0;
    for (index, provenance) in candidate.provenance.iter().enumerate() {
        if let CompletionProvenance::Original {
            id,
            original_index,
            locked,
        } = provenance
        {
            assert_eq!(*original_index, next);
            let original = &req.original[next];
            assert_eq!(id, &original.id);
            assert_eq!(*locked, original.locked);
            let mut expected = original.entry.clone();
            let recognition = expected.intended_recognition.as_mut().unwrap();
            if recognition.planned_element.is_some() {
                recognition.planned_element = Some(index);
            }
            assert_eq!(candidate.assessment.request.entries[index], expected);
            next += 1;
        }
    }
    assert_eq!(next, req.original.len());
    assert_eq!(
        candidate.total_base_hundredths,
        candidate.existing_base_hundredths + candidate.added_base_hundredths
    );
}
