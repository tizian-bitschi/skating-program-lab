//! Stage-aware coverage of technical plan rules. No panel outcome is inferred.
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCoverageState {
    Evaluated,
    Conditional,
    MissingEvidence,
    Unsupported,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCoverageEntry {
    pub rule_id: &'static str,
    pub source: SourceReference,
    pub element_index: Option<usize>,
    pub component_index: Option<usize>,
    pub state: RuleCoverageState,
    pub reason: &'static str,
    /// Exact selected replacement, including its scope and justification.
    pub event_override: Option<RuleOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLegalityReport {
    pub validation: EvidenceValidationReport,
    pub submission: SubmissionState,
    pub rules: Vec<RuleCoverageEntry>,
    /// Indices into validation.validation.findings; only definite draft omissions.
    /// The underlying findings are retained for compatibility with final checks.
    pub repairable_absences: Vec<usize>,
}

/// Evaluate the shared plan validator and inventory checks that it cannot finish.
/// Evaluated denotes one checked rule, including violations, never full legality.
pub fn assess_program_legality(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    evidence: &[ElementPlanEvidence],
    policy: &ProgramPolicy,
    submission: SubmissionState,
) -> Result<ProgramLegalityReport, ProfileSelectionError> {
    let validation = validate_supported_program(registry, program, evidence, policy)?;
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    let repairable_absences = if submission == SubmissionState::Draft {
        crate::program_validation::completion_findings(
            &validation,
            profile,
            program,
            policy,
            evidence,
        )
    } else {
        vec![]
    };
    let mut rules = Vec::new();
    for applied in &validation.applied_rules {
        let findings: Vec<_> = validation
            .validation
            .findings
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.rule_id == applied.rule_id
                    && f.element_index == applied.element_index
                    && f.component_index == applied.component_index
                    && f.source == applied.source
            })
            .collect();
        let (state, reason) = if applied.rule_id == crate::validation::SEQUENCE_DIRECT_STEP_RULE
            && applied.element_index.is_some_and(|i| {
                matches!(
                    evidence[i],
                    ElementPlanEvidence::Jump {
                        direct_transitions_assumed: false
                    }
                )
            }) {
            (
                RuleCoverageState::MissingEvidence,
                "The direct step into the Axel has not been supplied as an explicit assumption.",
            )
        } else if crate::program_validation::assumption_rule(applied.rule_id) {
            (RuleCoverageState::Conditional, "Checked against supplied qualification or transition assumptions; actual execution and panel recognition are not established.")
        } else {
            (
                RuleCoverageState::Evaluated,
                "Checked against the supplied technical plan; see findings for violations.",
            )
        };
        let entry = RuleCoverageEntry {
            rule_id: applied.rule_id,
            source: applied.source.clone(),
            element_index: applied.element_index,
            component_index: applied.component_index,
            state,
            reason,
            event_override: None,
        };
        if findings.is_empty() {
            rules.push(entry);
        } else {
            // Each finding survives independently: missing evidence must never
            // hide a confirmed violation of the very same rule instance.
            for (i, finding) in findings {
                let (state, reason) = match finding.kind {
                    FindingKind::Violation if repairable_absences.contains(&i) => (RuleCoverageState::Conditional, "A definite draft omission has capacity for insertion; final checks retain the violation."),
                    FindingKind::Violation => (RuleCoverageState::Evaluated, finding.explanation),
                    FindingKind::Unknown => (RuleCoverageState::MissingEvidence, finding.explanation),
                    FindingKind::Unsupported => (RuleCoverageState::Unsupported, finding.explanation),
                };
                rules.push(RuleCoverageEntry {
                    state,
                    reason,
                    ..entry.clone()
                });
            }
        }
    }
    // Inventory is separate from the execution trace: absent families and rules
    // blocked by missing structure still need an explicit coverage entry.
    for rule in ProfileRule::ALL {
        if rule == ProfileRule::Scoring {
            continue;
        }
        let binding = profile.binding(rule);
        if rules.iter().any(|r| r.rule_id == binding.id) {
            continue;
        }
        let not_applicable = (rule == ProfileRule::Axel && !profile.jumps.require_axel)
            || (matches!(rule, ProfileRule::Input | ProfileRule::Eligibility)
                && !program
                    .elements
                    .iter()
                    .any(|e| matches!(e, PlannedElement::Jump(_))))
            || (matches!(rule, ProfileRule::Transition | ProfileRule::Structure)
                && !program.elements.iter().any(|e| {
                    matches!(
                        e,
                        PlannedElement::Jump(
                            JumpElement::Combination(_) | JumpElement::Sequence(_)
                        )
                    )
                }));
        inventory(
            &mut rules,
            binding.id,
            &binding.source,
            None,
            if not_applicable {
                RuleCoverageState::NotApplicable
            } else {
                RuleCoverageState::MissingEvidence
            },
            if not_applicable {
                "No applicable requirement or matching element in this selected plan."
            } else {
                "The evaluator did not establish this selected profile rule; inspect missing plan evidence."
            },
        );
    }
    let spins: Vec<_> = program
        .elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| matches!(e, PlannedElement::RegularSpin(_)).then_some(i))
        .collect();
    for id in ["spin-count", "spin-required-role"] {
        inventory(
            &mut rules,
            id,
            &policy.spins.source,
            None,
            if id == "spin-required-role" && policy.spins.required.is_empty() {
                RuleCoverageState::NotApplicable
            } else {
                RuleCoverageState::MissingEvidence
            },
            "The explicit spin policy has no such obligation, or it was not evaluated.",
        );
    }
    for &index in &spins {
        for id in [
            "spin-abbreviation",
            "spin-eligibility",
            "spin-entry",
            "spin-level-cap",
            "spin-distinct-abbreviation",
            "spin-minimum-revolutions",
        ] {
            inventory(
                &mut rules,
                id,
                &policy.spins.source,
                Some(index),
                RuleCoverageState::MissingEvidence,
                "Missing spin structure prevents this check.",
            );
        }
        for (ids, source) in [
            (
                &[
                    "spin-position-type",
                    "spin-basic-position",
                    "spin-recognition-minimum",
                    "spin-combination-positions",
                ][..],
                policy.spins.positions.as_ref().map(|p| &p.source),
            ),
            (
                &["spin-per-foot-revolutions"][..],
                policy.spins.per_foot.as_ref().map(|p| &p.source),
            ),
        ] {
            for id in ids {
                let data = match &evidence[index] {
                    ElementPlanEvidence::Spin { spin, .. } => spin,
                    _ => unreachable!(),
                };
                let not_applicable =
                    (matches!(*id, "spin-combination-positions" | "spin-position-type")
                        && data.abbreviation.is_some_and(|a| {
                            crate::spin_validation::position_type_rule(a.kind) != *id
                        }))
                        || (*id == "spin-per-foot-revolutions"
                            && data.abbreviation.is_some_and(|a| !a.change_foot));
                inventory(&mut rules, id, source.unwrap_or(&policy.spins.source), Some(index),
                    if not_applicable { RuleCoverageState::NotApplicable } else if source.is_some() { RuleCoverageState::MissingEvidence } else { RuleCoverageState::Unsupported },
                    "Position/foot checks require both explicit policy opt-in and sufficient ordered evidence.");
            }
        }
    }
    for (family, ids, source) in [
        (
            ProfileElementFamily::RegularSpin,
            &[
                "program-spin-level-credit",
                "program-feature-once-per-program",
                "program-feature-per-foot",
                "program-feature-qualification",
                "program-feature-position",
            ][..],
            &policy.feature_source,
        ),
        (
            ProfileElementFamily::StepSequence,
            &[
                "program-step-recognition",
                "program-step-feature-dependency",
                "program-step-level-credit",
            ][..],
            &policy.step_source,
        ),
        (
            ProfileElementFamily::ChoreographicSequence,
            &["program-choreographic-sequence"][..],
            &policy.choreography_source,
        ),
        (
            ProfileElementFamily::ChoreographicSpin,
            &["program-choreographic-spin"][..],
            &policy.choreography_source,
        ),
    ] {
        let indices: Vec<_> = program
            .elements
            .iter()
            .enumerate()
            .filter_map(|(i, e)| (ProfileElementFamily::of(e) == Some(family)).then_some(i))
            .collect();
        for id in ids {
            if indices.is_empty() {
                inventory(&mut rules, id, source, None, RuleCoverageState::NotApplicable, "No matching element in this plan; family eligibility and obligations are checked separately.");
            } else {
                for &i in &indices {
                    inventory(&mut rules, id, source, Some(i),
                        if family == ProfileElementFamily::RegularSpin { RuleCoverageState::NotApplicable } else { RuleCoverageState::MissingEvidence },
                        "No applicable feature attempt, or evidence needed by this family check is missing.");
                }
            }
        }
    }
    // Adult-only regulatory gaps must never be inferred from a category label.
    if [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ]
    .iter()
    .any(|c| *profile == adult_profile(*c))
    {
        for (id, source, state, reason) in [
            ("program-duration", &profile.scoring.duration_source, RuleCoverageState::MissingEvidence, "No actual program duration is supplied; timing and incomplete-program consequences remain unevaluated."),
            ("program-performance-panel", &policy.feature_source, RuleCoverageState::MissingEvidence, "Plans do not establish actual edges, uninterrupted execution, musical fit, panel identification or fallback recognition."),
            ("program-waltz-recognition", &profile.binding(ProfileRule::Eligibility).source, RuleCoverageState::Unsupported, "The planned listed-jump model cannot represent a Waltz jump and its category-specific counting consequences."),
        ] {
            inventory(&mut rules, id, source, None, state, reason);
        }
        for index in spins {
            inventory(&mut rules, "program-variation-categories", &policy.feature_source, Some(index), RuleCoverageState::Unsupported,
                "Difficult-variation categories, two-variation quota, category reuse and dependent feature reuse require typed variation evidence; generic feature assertions do not establish them.");
        }
    }
    for entry in &mut rules {
        entry.event_override = profile
            .overrides
            .iter()
            .find(|o| o.replacement.id == entry.rule_id)
            .cloned();
    }
    Ok(ProgramLegalityReport {
        validation,
        submission,
        rules,
        repairable_absences,
    })
}

fn inventory(
    rules: &mut Vec<RuleCoverageEntry>,
    id: &'static str,
    source: &SourceReference,
    index: Option<usize>,
    state: RuleCoverageState,
    reason: &'static str,
) {
    if !rules
        .iter()
        .any(|r| r.rule_id == id && r.element_index == index)
    {
        rules.push(RuleCoverageEntry {
            rule_id: id,
            source: source.clone(),
            element_index: index,
            component_index: None,
            state,
            reason,
            event_override: None,
        });
    }
}
