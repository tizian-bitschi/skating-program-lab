//! Rule 612 Senior Free Skating. Named mechanisms, independent of category labels.
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeniorCategory {
    Women,
    Men,
}

/// One source-bound technical contract shared by both Senior identities.
pub(crate) struct SeniorDefinition {
    pub max_elements: usize,
    pub max_kind_occurrences: usize,
    pub max_repeated_advanced: usize,
    pub max_repeated_quad_or_quint: usize,
    pub combination_revolutions: u16,
}
pub(crate) const DEFINITION: SeniorDefinition = SeniorDefinition {
    max_elements: 11,
    max_kind_occurrences: 3,
    max_repeated_advanced: 2,
    max_repeated_quad_or_quint: 1,
    combination_revolutions: 10,
};

pub(crate) fn advanced_repeat_limit_exceeded<T: Eq>(occurrences: &[(T, u8)]) -> bool {
    let repeated: Vec<_> = occurrences
        .iter()
        .enumerate()
        .filter(|(i, jump)| {
            jump.1 >= 3
                && !occurrences[..*i].contains(jump)
                && occurrences.iter().filter(|other| *other == *jump).count() >= 2
        })
        .map(|(_, jump)| jump)
        .collect();
    repeated.len() > DEFINITION.max_repeated_advanced
        || repeated.iter().filter(|jump| jump.1 >= 4).count()
            > DEFINITION.max_repeated_quad_or_quint
}

pub(crate) fn source(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Sports Rules — Figure Skating".into(),
        version: "2026-06-12".into(),
        location: location.into(),
    }
}

pub(crate) fn selected(profile: &RuleProfile, rule: ProfileRule) -> bool {
    profile.binding(rule).mechanism == RuleMechanism::SeniorJumpPlanV1
}

pub fn senior_profile(category: SeniorCategory) -> RuleProfile {
    let name = match category {
        SeniorCategory::Women => "women",
        SeniorCategory::Men => "men",
    };
    let jump_source =
        source("p. 71, Rule 612, compounds, Euler and repetitions; p. 70, individual jumps");
    let family_source = source("p. 70, Rule 612, Senior well-balanced program");
    let duration_source = source("pp. 50–51, Rule 502, Senior Free Skating duration and tolerance");
    let time_source = source("pp. 11–12, Rule 353.1.n; pp. 50–51, Rule 502");
    let fall_source =
        source("pp. 11–12, Rule 353.1.n, progressive Senior fall deductions; p. 51, Rule 503");
    let bindings = ProfileRule::ALL
        .into_iter()
        .map(|rule| {
            let (id, citation) = match rule {
                ProfileRule::Input => ("senior-jump-input", jump_source.clone()),
                ProfileRule::Eligibility => ("senior-eligibility", family_source.clone()),
                ProfileRule::Count => ("senior-jump-count", family_source.clone()),
                ProfileRule::CompoundCount => ("senior-compound-count", jump_source.clone()),
                ProfileRule::Structure => ("senior-compound-structure", jump_source.clone()),
                ProfileRule::Euler => ("senior-euler", jump_source.clone()),
                ProfileRule::Repetition => ("senior-repetition", jump_source.clone()),
                ProfileRule::Axel => ("senior-axel", family_source.clone()),
                ProfileRule::Transition => (
                    "senior-jump-transition",
                    source("p. 65, Rule 610, jump combinations; p. 71, Rule 612"),
                ),
                ProfileRule::Scoring => ("senior-scoring", jump_source.clone()),
            };
            RuleBinding {
                rule,
                id,
                source: citation,
                mechanism: if rule == ProfileRule::Scoring {
                    RuleMechanism::SeniorScoringV1
                } else {
                    RuleMechanism::SeniorJumpPlanV1
                },
            }
        })
        .collect::<Vec<_>>();
    let mut sources = vec![
        family_source,
        duration_source.clone(),
        time_source.clone(),
        fall_source.clone(),
        source("pp. 71–72, Rule 612, spins, steps and choreography"),
        SourceReference {
            document: "ISU Communication 2786".into(),
            version: "2026-06-02".into(),
            location: "Single SOV rows; linked table edition".into(),
        },
        crate::spin_validation::communication(
            "pp. 2–5, spin and step levels, feature credit and recognition",
        ),
        crate::spin_validation::handbook("pp. 14–25, spins, steps and choreography"),
        SourceReference {
            document: "Technical Panel Handbook, Single Skating".into(),
            version: "2026-07-15".into(),
            location: "pp. 6–25, spin/sequence recognition; pp. 30–33, jump repetition and calls"
                .into(),
        },
        SourceReference {
            document: "Technical Committee Q&A 3".into(),
            version: "2026-10-08".into(),
            location: "p. 1, Q2–4, starred jumps and repetition order".into(),
        },
    ];
    for binding in &bindings {
        if !sources.contains(&binding.source) {
            sources.push(binding.source.clone());
        }
    }
    RuleProfile {
        id: ProfileId { name: format!("isu-senior-2026-27-{name}-free"), version: "2026-06-12".into() },
        discipline: ProfileDiscipline::Single, segment: ProgramSegment::FreeSkating,
        season: Season { start_year: 2026, end_year: 2027 },
        event_scope: "2026/27 ISU Senior Single Free Skating, Rule 612".into(),
        category: name.into(), provenance: ProfileProvenance::Official,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        sources, precedence: "Sports Rules effective June 12; applicable 2026/27 Communications and panel clarifications govern their specific scope. Event replacements require explicit cited overrides; Adult announcements do not apply.".into(),
        bindings, overrides: vec![],
        jumps: JumpPolicy {
            allowed: [JumpKind::ToeLoop, JumpKind::Salchow, JumpKind::Loop, JumpKind::Flip, JumpKind::Lutz, JumpKind::Axel].into_iter().map(|kind| JumpPermission {kind, min_rotations: 1, max_rotations: u8::MAX}).collect(),
            max_jumps: 6, max_compound_length: 3, max_combinations: 2, max_sequences: 1,
            max_compounds: 2, max_three_jump_compounds: 1, max_eulers: 1,
            max_repetitions: 2, require_axel: true, bronze_diagnostics: false,
        },
        families: FamilyPolicy { allowed: vec![ProfileElementFamily::SoloJump, ProfileElementFamily::Combination, ProfileElementFamily::JumpSequence, ProfileElementFamily::RegularSpin, ProfileElementFamily::ChoreographicSpin, ProfileElementFamily::StepSequence, ProfileElementFamily::ChoreographicSequence], required: vec![ProfileElementFamily::ChoreographicSpin, ProfileElementFamily::StepSequence, ProfileElementFamily::ChoreographicSequence] },
        scoring: ProfileScoringPolicy { recognition_ruleset: RulesetId::IsuSenior2026_27June12, duration_seconds: 240, duration_source, time_deduction_source: time_source, fall_hundredths: None, fall_source },
        unsupported_policies: vec![],
    }
}

fn flying_roles() -> Vec<SpinRequirement> {
    [
        RegularSpinKind::Camel,
        RegularSpinKind::Sit,
        RegularSpinKind::Upright,
        RegularSpinKind::Layback,
    ]
    .into_iter()
    .map(|kind| SpinRequirement {
        kind: Some(kind),
        entry: Some(SpinEntry::Flying),
        change_foot: Some(false),
    })
    .collect()
}

pub fn senior_regular_spin_policy(category: SeniorCategory) -> RegularSpinPolicy {
    RegularSpinPolicy {
        profile: senior_profile(category).id,
        source: source("pp. 70–71, Rule 612, three distinct spins, combination and flying roles, six/ten revolutions"),
        max_spins: 3,
        allowed: std::iter::once(SpinRequirement {kind: Some(RegularSpinKind::Combination), entry: None, change_foot: None}).chain(flying_roles()).collect(),
        required: vec![vec![SpinRequirement {kind: Some(RegularSpinKind::Combination), entry: None, change_foot: None}], flying_roles()],
        min_revolutions: 6, min_change_foot_revolutions: 6, max_level: Level::Four,
        positions: Some(SpinPositionRules { source: crate::spin_validation::handbook("pp. 14, 18, spin positions and combination recognition"), min_basic_revolutions: 2, min_spin_revolutions: 3, min_combination_positions: 2, full_value_positions: 3, max_other_position_revolutions: 2 }),
        per_foot: Some(SpinFootRules { source: crate::spin_validation::communication("p. 4, Free Skating change-foot revolutions and basic positions"), min_revolutions: 3, min_basic_revolutions: 2 }),
    }
}

pub fn senior_program_policy(category: SeniorCategory) -> ProgramPolicy {
    ProgramPolicy {
        spins: senior_regular_spin_policy(category),
        max_steps: 1,
        max_choreographic_sequences: 1,
        max_choreographic_spins: 1,
        require_choreographic_spin: true,
        max_step_level: Level::Four,
        step_movement_required: false,
        permitted_step_features: vec![1, 2, 3, 4, 5],
        step_source: source("pp. 70–71, Rule 612, Senior step sequence and full ice surface"),
        choreography_source: source("pp. 70–72, Rule 612, choreographic spin and sequence"),
        feature_source: crate::spin_validation::communication(
            "pp. 2, 4–5, Single spin and step level features and recognition",
        ),
    }
}

/// Resolve ordered repetition credit independently for each element.
/// Definitive zero outcomes remain zero; uncertainty affects advanced jumps only.
pub(crate) fn repetition_context(
    panels: &[Option<PanelRecognition>],
) -> Vec<Option<PanelRecognition>> {
    let mut occurrences: Vec<(String, usize, bool)> = vec![];
    let mut all_jumps = vec![];
    let mut uncertain = false;
    let advanced = |panel: &PanelRecognition| {
        panel.details.as_ref().is_some_and(|d| {
            d.components()
                .iter()
                .any(|c| matches!(c.as_str().as_bytes().first(), Some(b'3' | b'4' | b'5')))
        })
    };
    let nonjump = |panel: &PanelRecognition| {
        panel.details.as_ref().is_some_and(|d| {
            d.components()
                .iter()
                .all(|c| !c.as_str().starts_with(char::is_numeric) && c.as_str() != "Eu")
        })
    };
    for (index, panel) in panels.iter().enumerate() {
        let Some(panel) = panel else {
            uncertain = true;
            continue;
        };
        if panel.outcome != RecognitionOutcome::Recognized {
            // A known non-jump cannot change jump repetition, including a no-value spin.
            uncertain |= !nonjump(panel);
            continue;
        }
        let Some(details) = panel.details.as_ref() else {
            uncertain = true;
            continue;
        };
        if advanced(panel)
            && details.calls().is_none_or(|calls| {
                calls.iter().any(|call| {
                    !matches!(
                        call.call,
                        PanelCall::Repetition
                            | PanelCall::Sequence
                            | PanelCall::Quarter
                            | PanelCall::UnderRotated
                            | PanelCall::Downgraded
                            | PanelCall::UnclearEdge
                            | PanelCall::WrongEdge
                    )
                })
            })
        {
            uncertain = true;
        }
        for code in details.components() {
            if let Some(rotation @ b'1'..=b'5') = code.as_str().as_bytes().first() {
                all_jumps.push((code.as_str()[1..].to_string(), rotation - b'0'));
            }
            if matches!(code.as_str().as_bytes().first(), Some(b'3' | b'4' | b'5')) {
                occurrences.push((code.as_str().into(), index, details.components().len() == 1));
            }
        }
    }
    let invalid_repetition = advanced_repeat_limit_exceeded(&all_jumps)
        || all_jumps.iter().any(|jump| {
            (jump.1 >= 2 && all_jumps.iter().filter(|other| *other == jump).count() > 2)
                || all_jumps.iter().filter(|other| other.0 == jump.0).count()
                    > DEFINITION.max_kind_occurrences
        });
    let mut adjusted = panels.to_vec();
    if uncertain || invalid_repetition {
        for panel in &mut adjusted {
            if panel.as_ref().is_some_and(|p| {
                p.outcome == RecognitionOutcome::Recognized
                    && (advanced(p) || (invalid_repetition && !nonjump(p)))
            }) {
                *panel = None;
            }
        }
        return adjusted;
    }
    for (code, index, solo) in &occurrences {
        let matching: Vec<_> = occurrences.iter().filter(|e| e.0 == *code).collect();
        if *solo && matching.len() == 2 && matching.iter().all(|e| e.2) && matching[1].1 == *index {
            let panel = adjusted[*index]
                .as_mut()
                .expect("known advanced recognition");
            let details = panel.details.as_ref().expect("known advanced details");
            let mut calls = details.calls().expect("known clean calls").to_vec();
            if !calls.iter().any(|c| c.call == PanelCall::Repetition) {
                calls.push(RecordedCall {
                    target: CallTarget::Element,
                    call: PanelCall::Repetition,
                });
                panel.details = Some(
                    RecognitionDetails::new(
                        details.components().to_vec(),
                        details.level(),
                        Some(calls),
                    )
                    .expect("valid repetition call"),
                );
            }
        }
    }
    adjusted
}
