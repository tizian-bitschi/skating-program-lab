use super::*;

// The API requires concrete plan variants. Its placeholder-only applicability
// is not evidence about the protocol: project that limitation into this report.
fn coverage_with_unknown_jump_structure(report: &ProgramLegalityReport) -> Vec<RuleCoverageEntry> {
    report.rules.iter().cloned().map(|mut rule| {
        if matches!(rule.rule_id, "senior-compound-structure" | "senior-jump-transition") {
            rule.state = RuleCoverageState::MissingEvidence;
            rule.reason = "Intended jump structure is unavailable; solo placeholders establish no compound-rule applicability for the supplied recognition.";
        }
        rule
    }).collect()
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
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
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
    let replay_coverage = coverage_with_unknown_jump_structure(&legality);
    for id in ["senior-compound-structure", "senior-jump-transition"] {
        let rules: Vec<_> = replay_coverage.iter().filter(|r| r.rule_id == id).collect();
        assert!(!rules.is_empty(), "{id}: {replay_coverage:#?}");
        assert!(rules
            .iter()
            .all(|r| r.state == RuleCoverageState::MissingEvidence));
        assert!(rules
            .iter()
            .all(|r| r.reason.contains("Intended jump structure")));
    }
    assert_eq!(replay_coverage.len(), legality.rules.len());
    assert!(replay_coverage
        .iter()
        .zip(&legality.rules)
        .all(|(a, b)| a.source == b.source));
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
