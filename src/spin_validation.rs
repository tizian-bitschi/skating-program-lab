//! Additive planned-spin checks. Execution consequences remain conditional.
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegularSpinKind {
    Camel,
    Sit,
    Upright,
    Layback,
    Combination,
}
impl RegularSpinKind {
    fn basic_position(self) -> Option<SpinPosition> {
        match self {
            Self::Camel => Some(SpinPosition::Camel),
            Self::Sit => Some(SpinPosition::Sit),
            Self::Upright | Self::Layback => Some(SpinPosition::Upright),
            Self::Combination => None,
        }
    }
}

/// An intended abbreviation, without a level or panel marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpinAbbreviation {
    pub kind: RegularSpinKind,
    pub entry: SpinEntry,
    pub change_foot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinFeature {
    DifficultVariation,
    JumpChangeOfFoot,
    JumpWithinSpin,
    DifficultPositionChange,
    DifficultEntrance,
    DifficultExit,
    ChangeOfEdge,
    BothDirections,
    IncreaseOfSpeed,
    EightRevolutions,
    DifficultFlyingEntry,
    DifficultBlade,
    Windmills,
    LaybackPositionChange,
    BiellmannAfterLayback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureAttemptOutcome {
    Unknown,
    Unsuccessful,
    Successful,
}

/// Ordered caller assertions, not awarded level features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinFeatureAttempt {
    pub feature: SpinFeature,
    /// Zero-based position step; its foot is determined by preceding foot changes.
    pub position_step: usize,
    pub outcome: FeatureAttemptOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegularSpinEvidence {
    pub abbreviation: Option<SpinAbbreviation>,
    pub planned_level: Option<Level>,
    /// None is unknown; Some(empty) explicitly states no attempts.
    pub feature_attempts: Option<Vec<SpinFeatureAttempt>>,
    /// Supplied independently; never used to fill gaps in the intended plan.
    pub panel: Option<PanelRecognition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSpinEvidence {
    pub element_index: usize,
    pub spin: RegularSpinEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgramValidationEvidence {
    pub spins: Vec<IndexedSpinEvidence>,
}

/// A supported planned role; None leaves that attribute unrestricted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinRequirement {
    pub kind: Option<RegularSpinKind>,
    pub entry: Option<SpinEntry>,
    pub change_foot: Option<bool>,
}
impl SpinRequirement {
    fn matches(&self, spin: SpinAbbreviation) -> bool {
        self.kind.is_none_or(|v| v == spin.kind)
            && self.entry.is_none_or(|v| v == spin.entry)
            && self.change_foot.is_none_or(|v| v == spin.change_foot)
    }
}

/// Explicit policy paired with an exact profile identity. It contains no SOV values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegularSpinPolicy {
    pub profile: ProfileId,
    pub source: SourceReference,
    /// Includes choreographic spins; their recognition is outside this slice.
    pub max_spins: usize,
    pub allowed: Vec<SpinRequirement>,
    /// Each group requires at least one matching alternative.
    pub required: Vec<Vec<SpinRequirement>>,
    pub min_revolutions: u16,
    pub min_change_foot_revolutions: u16,
    pub max_level: Level,
}

pub fn adult_regular_spin_policy(category: AdultCategory) -> RegularSpinPolicy {
    use RegularSpinKind::*;
    use SpinEntry::*;
    let one_position = |entry| {
        [Camel, Sit, Upright, Layback].map(|kind| SpinRequirement {
            kind: Some(kind),
            entry,
            change_foot: None,
        })
    };
    let role = |kind, entry, change_foot| SpinRequirement {
        kind,
        entry,
        change_foot,
    };
    let (page, max_spins, min, changed, level, allowed, required) = match category {
        AdultCategory::Bronze => (
            11,
            2,
            3,
            6,
            Level::One,
            one_position(Some(NonFlying)).to_vec(),
            vec![],
        ),
        AdultCategory::Silver => (
            10,
            2,
            4,
            6,
            Level::Two,
            vec![role(None, None, None)],
            vec![one_position(Some(NonFlying)).to_vec()],
        ),
        AdultCategory::Gold => (
            9,
            3,
            4,
            8,
            Level::Three,
            vec![
                role(Some(Combination), Some(NonFlying), Some(true)),
                role(None, Some(Flying), None),
            ],
            vec![
                vec![role(Some(Combination), Some(NonFlying), Some(true))],
                vec![role(None, Some(Flying), None)],
            ],
        ),
        AdultCategory::Masters => (
            8,
            3,
            5,
            8,
            Level::Four,
            vec![
                role(Some(Combination), Some(NonFlying), None),
                role(None, Some(Flying), Some(false)),
            ],
            vec![
                vec![role(Some(Combination), Some(NonFlying), None)],
                vec![role(None, Some(Flying), Some(false))],
            ],
        ),
    };
    RegularSpinPolicy {
        profile: adult_profile(category).id,
        source: SourceReference {
            document: "International Adult Figure Skating Competitions Technical Announcement"
                .into(),
            version: "2026-07-16".into(),
            location: format!("p. {page}, Free Skating, spins and level cap; p. 4, precedence"),
        },
        max_spins,
        allowed,
        required,
        min_revolutions: min,
        min_change_foot_revolutions: changed,
        max_level: level,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationFamily {
    Jumps,
    RegularSpins,
    SpinFeatures,
    StepsAndChoreography,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyCoverage {
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationCoverage {
    pub family: ValidationFamily,
    pub coverage: FamilyCoverage,
    pub explanation: &'static str,
}

/// Findings never establish complete compliance, even when the list is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceValidationReport {
    pub profile: ProfileId,
    pub sources: Vec<SourceReference>,
    pub validation: ValidationResult,
    pub coverage: Vec<ValidationCoverage>,
}

fn level_number(level: Level) -> u8 {
    match level {
        Level::Basic => 0,
        Level::One => 1,
        Level::Two => 2,
        Level::Three => 3,
        Level::Four => 4,
    }
}
fn communication(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Communication 2788".into(),
        version: "2026-05-19".into(),
        location: location.into(),
    }
}
fn handbook(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Technical Panel Handbook, Single Skating".into(),
        version: "2026-07-15".into(),
        location: location.into(),
    }
}
fn input_source() -> SourceReference {
    SourceReference {
        document: "Skating Program Lab input contract".into(),
        version: "issue-48-slice-1".into(),
        location: "indexed regular-spin evidence".into(),
    }
}

/// Legacy program selection plus additive evidence. No source/season fallback.
pub fn validate_program_with_evidence(
    program: &Program,
    evidence: &ProgramValidationEvidence,
) -> Result<EvidenceValidationReport, ProfileSelectionError> {
    let profile = adult_profile(program.category);
    if program.ruleset != RulesetId::IsuAdult2026_27July16 || program.season != profile.season {
        return Err(ProfileSelectionError {
            profile: profile.id,
            kind: ProfileErrorKind::Unsupported,
            reason: "only the selected 2026/27 Adult ruleset and season are supported".into(),
        });
    }
    validate_profile_program_with_evidence(
        &ProfileRegistry::adult(),
        &ProfileProgram {
            profile: profile.id,
            scoring_table: program.scoring_table,
            elements: program.elements.clone(),
        },
        evidence,
        None,
    )
}

/// Explicit custom policy is required for non-built-in profiles. A supplied policy
/// is an asserted rule definition; this function does not authenticate its source.
pub fn validate_profile_program_with_evidence(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    evidence: &ProgramValidationEvidence,
    policy: Option<&RegularSpinPolicy>,
) -> Result<EvidenceValidationReport, ProfileSelectionError> {
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    let built_in = [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ]
    .into_iter()
    .find(|c| adult_profile(*c) == *profile);
    let default = built_in.map(adult_regular_spin_policy);
    let policy = policy.or(default.as_ref());
    if let Some(p) = policy {
        if p.profile != profile.id
            || p.source.document.trim().is_empty()
            || p.source.version.trim().is_empty()
            || p.source.location.trim().is_empty()
        {
            return Err(ProfileSelectionError {
                profile: profile.id.clone(),
                kind: ProfileErrorKind::Invalid,
                reason:
                    "spin policy needs the selected profile identity and a complete source citation"
                        .into(),
            });
        }
    }
    let mut check = SpinValidator {
        findings: crate::validation::validate_elements(&program.elements, profile).findings,
    };
    check.evidence_targets(&program.elements, evidence);
    if let Some(policy) = policy {
        check.program(&program.elements, evidence, policy);
    } else {
        check.add(
            FindingKind::Unsupported,
            "spin-policy",
            None,
            None,
            "No explicit regular-spin policy is supplied for this profile.",
            &input_source(),
        );
    }
    let mut sources = profile.sources.clone();
    for source in check
        .findings
        .iter()
        .map(|f| &f.source)
        .chain(policy.map(|p| &p.source))
    {
        if !sources.contains(source) {
            sources.push(source.clone());
        }
    }
    Ok(EvidenceValidationReport {
        profile: profile.id.clone(), sources,
        validation: ValidationResult { coverage: Coverage::Partial, findings: check.findings },
        coverage: vec![
            ValidationCoverage { family: ValidationFamily::Jumps, coverage: FamilyCoverage::Partial,
                explanation: "Legacy jump checks; unresolved transitions and panel consequences remain conditional." },
            ValidationCoverage { family: ValidationFamily::RegularSpins, coverage: if policy.is_some() { FamilyCoverage::Partial } else { FamilyCoverage::Unsupported },
                explanation: "Planned roles, abbreviation, revolution and position checks only; execution and panel calls are not inferred." },
            ValidationCoverage { family: ValidationFamily::SpinFeatures, coverage: FamilyCoverage::Unsupported,
                explanation: "Typed ordered assertions only; feature eligibility, reuse and level derivation are deferred to slice 2." },
            ValidationCoverage { family: ValidationFamily::StepsAndChoreography, coverage: FamilyCoverage::Unsupported,
                explanation: "Recognition, movement requirements and category caps are deferred to slice 3." },
        ],
    })
}

struct SpinValidator {
    findings: Vec<Finding>,
}
impl SpinValidator {
    fn add(
        &mut self,
        kind: FindingKind,
        rule_id: &'static str,
        element_index: Option<usize>,
        component_index: Option<usize>,
        explanation: &'static str,
        source: &SourceReference,
    ) {
        self.findings.push(Finding {
            kind,
            rule_id,
            element_index,
            component_index,
            explanation,
            source: source.clone(),
        });
    }
    fn evidence_targets(
        &mut self,
        elements: &[PlannedElement],
        evidence: &ProgramValidationEvidence,
    ) {
        for (i, item) in evidence.spins.iter().enumerate() {
            if !matches!(
                elements.get(item.element_index),
                Some(PlannedElement::RegularSpin(_))
            ) || evidence.spins[..i]
                .iter()
                .any(|e| e.element_index == item.element_index)
            {
                self.add(
                    FindingKind::Violation,
                    "spin-evidence-target",
                    Some(item.element_index),
                    None,
                    "Spin evidence must target a regular spin exactly once.",
                    &input_source(),
                );
            }
        }
    }
    fn program(
        &mut self,
        elements: &[PlannedElement],
        evidence: &ProgramValidationEvidence,
        policy: &RegularSpinPolicy,
    ) {
        let mut known = vec![];
        let mut unknown = elements
            .iter()
            .any(|e| matches!(e, PlannedElement::Unspecified));
        let count = elements
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    PlannedElement::RegularSpin(_) | PlannedElement::ChoreographicSpin
                )
            })
            .count();
        if count > policy.max_spins {
            self.add(
                FindingKind::Violation,
                "spin-count",
                None,
                None,
                "The planned spin count exceeds the profile limit, including choreographic spins.",
                &policy.source,
            );
        } else if count
            + elements
                .iter()
                .filter(|e| matches!(e, PlannedElement::Unspecified))
                .count()
            > policy.max_spins
        {
            self.add(
                FindingKind::Unknown,
                "spin-count",
                None,
                None,
                "Unspecified elements may exceed the spin limit.",
                &policy.source,
            );
        }
        for (index, element) in elements.iter().enumerate() {
            if let PlannedElement::RegularSpin(spin) = element {
                let items: Vec<_> = evidence
                    .spins
                    .iter()
                    .filter(|e| e.element_index == index)
                    .collect();
                let data = (items.len() == 1).then(|| &items[0].spin);
                let abbreviation = data.and_then(|d| d.abbreviation);
                if let Some(abbreviation) = abbreviation {
                    if known.contains(&abbreviation) {
                        self.add(FindingKind::Violation, "spin-distinct-abbreviation", Some(index), None,
                            "Planned spins must have different abbreviations; changing only the level does not change the abbreviation.", &policy.source);
                    }
                    known.push(abbreviation);
                } else {
                    unknown = true;
                }
                self.spin(index, spin, data, policy);
            }
        }
        for required in &policy.required {
            if !known.iter().any(|s| required.iter().any(|r| r.matches(*s))) {
                self.add(
                    if unknown {
                        FindingKind::Unknown
                    } else {
                        FindingKind::Violation
                    },
                    "spin-required-role",
                    None,
                    None,
                    "A required planned regular-spin role is missing or unspecified.",
                    &policy.source,
                );
            }
        }
    }

    fn spin(
        &mut self,
        index: usize,
        spin: &RegularSpin,
        data: Option<&RegularSpinEvidence>,
        policy: &RegularSpinPolicy,
    ) {
        let abbreviation = data.and_then(|d| d.abbreviation);
        let source = &policy.source;
        self.add(FindingKind::Unknown, "spin-execution-recognition", Some(index), None,
            "Plan checks are conditional on execution; visible flying jump, uninterrupted positions, spin centres and final panel recognition are not established by a plan.",
            &handbook("pp. 14, 24–25, spin recognition, foot changes and V"));
        if spin.entry.is_some_and(|entry| {
            !policy
                .allowed
                .iter()
                .any(|r| r.entry.is_none_or(|allowed| allowed == entry))
        }) {
            self.add(
                FindingKind::Violation,
                "spin-eligibility",
                Some(index),
                None,
                "The planned entry is prohibited by every allowed spin role.",
                source,
            );
        }
        match abbreviation {
            None => self.add(
                FindingKind::Unknown,
                "spin-abbreviation",
                Some(index),
                None,
                "The intended spin abbreviation/type is missing.",
                source,
            ),
            Some(a) => {
                if !policy.allowed.iter().any(|r| r.matches(a)) {
                    self.add(FindingKind::Violation, "spin-eligibility", Some(index), None, "The planned spin type, entry or foot-change role is prohibited by this profile.", source);
                }
                if spin.entry.is_some_and(|e| e != a.entry) {
                    self.add(
                        FindingKind::Violation,
                        "spin-evidence-entry",
                        Some(index),
                        None,
                        "The intended abbreviation and plan disagree about flying entry.",
                        &input_source(),
                    );
                }
            }
        }
        if spin.entry.is_none() {
            self.add(FindingKind::Unknown, "spin-entry", Some(index), None, "The planned entry is unspecified; abbreviation alone does not supply movement evidence.", source);
        }
        match data.and_then(|d| d.planned_level) {
            None => self.add(FindingKind::Unknown, "spin-level-cap", Some(index), None, "The target level is unspecified; no recognized level is inferred.", source),
            Some(level) if level_number(level) > level_number(policy.max_level) => self.add(FindingKind::Unknown, "spin-level-cap", Some(index), None,
                "The target exceeds the profile level cap; additional features are ignored, not a prohibition of the spin.", source),
            _ => {}
        }
        let attempts = data.and_then(|d| d.feature_attempts.as_deref());
        if attempts.is_none() {
            self.add(FindingKind::Unknown, "spin-feature-evidence", Some(index), None, "Typed feature attempts are unspecified; free-text descriptions are not confirmed features.", &communication("p. 2, spin features"));
        }
        if let Some(attempts) = attempts {
            let mut previous = None;
            for attempt in attempts {
                if !spin.steps.as_ref().is_some_and(|s| {
                    matches!(
                        s.get(attempt.position_step),
                        Some(SpinStep::Position { .. })
                    )
                }) || previous.is_some_and(|p| p > attempt.position_step)
                {
                    self.add(
                        FindingKind::Violation,
                        "spin-feature-target",
                        Some(index),
                        Some(attempt.position_step),
                        "Feature attempts need valid position-step targets in program order.",
                        &input_source(),
                    );
                }
                previous = Some(attempt.position_step);
            }
            if !attempts.is_empty() {
                self.add(FindingKind::Unsupported, "spin-feature-credit", Some(index), None, "Feature assertions are stored in order; success does not establish eligibility, reuse credit or a level in this slice.", &communication("p. 2, feature restrictions and first attempt/first success"));
            }
        }
        if let Some(panel) = data.and_then(|d| d.panel.as_ref()) {
            if panel.planned_element != Some(index)
                || panel.ruleset != RulesetId::IsuAdult2026_27July16
            {
                self.add(FindingKind::Violation, "spin-panel-target", Some(index), None, "Supplied panel recognition must identify this plan element and the supported recognition ruleset.", &input_source());
            }
            self.add(FindingKind::Unsupported, "spin-panel-reconciliation", Some(index), None, "Supplied panel recognition is retained separately; reconciliation with the intended plan is not implemented.", &input_source());
        }
        let Some(steps) = &spin.steps else {
            self.add(
                FindingKind::Unknown,
                "spin-rotations",
                Some(index),
                None,
                "Ordered positions, foot changes and revolutions are unspecified.",
                source,
            );
            return;
        };
        let changes = steps
            .iter()
            .filter(|s| matches!(s, SpinStep::ChangeFoot))
            .count();
        if abbreviation.is_some_and(|a| a.change_foot != (changes > 0)) {
            self.add(
                FindingKind::Violation,
                "spin-evidence-foot-change",
                Some(index),
                None,
                "The intended abbreviation and ordered steps disagree about change of foot.",
                &input_source(),
            );
        }
        if changes > 1 {
            self.add(FindingKind::Unsupported, "spin-multiple-foot-changes", Some(index), None, "Multiple foot changes are permitted in Free Skating; per-foot recognition of these plans is outside this slice.", &handbook("p. 24, multiple changes of foot in Free Skating"));
        }
        let mut feet = vec![(0u64, false, false)]; // revolutions, unknown count, qualifying basic position
        let mut positions = vec![];
        let mut unknown_position = false;
        for (step_index, step) in steps.iter().enumerate() {
            match step {
                SpinStep::ChangeFoot => feet.push((0, false, false)),
                SpinStep::Position {
                    position,
                    revolutions,
                    ..
                } => {
                    let foot = feet.last_mut().unwrap();
                    if let Some(n) = revolutions {
                        foot.0 += u64::from(*n);
                    } else {
                        foot.1 = true;
                    }
                    if position.is_none() || revolutions.is_none() {
                        unknown_position = true;
                    }
                    if let (Some(position), Some(2..)) = (position, revolutions) {
                        foot.2 |= abbreviation
                            .and_then(|a| a.kind.basic_position())
                            .is_none_or(|expected| expected == *position);
                        if !positions.contains(position) {
                            positions.push(*position);
                        }
                    }
                    if position.is_none() || revolutions.is_none() {
                        self.add(
                            FindingKind::Unknown,
                            "spin-position-evidence",
                            Some(index),
                            Some(step_index),
                            "A position or its consecutive revolution count is missing.",
                            source,
                        );
                    }
                }
            }
        }
        let minimum = if changes > 0 {
            policy.min_change_foot_revolutions
        } else {
            policy.min_revolutions
        };
        if let Some(a) = abbreviation {
            let expected = a.kind.basic_position();
            if expected.is_some_and(|p| !positions.contains(&p)) {
                self.add(FindingKind::Unknown, "spin-position-type", Some(index), None,
                    "The intended one-position type lacks a qualifying matching basic position; recognition or fallback is not inferred.", source);
            }
        }
        let total: u64 = feet.iter().map(|f| f.0).sum();
        if total < u64::from(minimum) {
            self.add(if feet.iter().any(|f| f.1) { FindingKind::Unknown } else { FindingKind::Violation }, "spin-minimum-revolutions", Some(index), None,
                "The plan does not establish the profile minimum total revolutions; this does not determine a panel call.", source);
        }
        if positions.is_empty() {
            self.add(FindingKind::Unknown, "spin-basic-position", Some(index), None,
                if unknown_position { "Missing position evidence prevents checking a basic position held for two consecutive revolutions." }
                else { "If executed as planned, no basic position is held for two consecutive revolutions: conditional no-value consequence; panel identification remains separate." }, source);
        }
        if changes == 1 {
            for &(count, missing, basic) in &feet {
                if count < 3
                    || (!basic
                        && abbreviation.is_some_and(|a| a.kind != RegularSpinKind::Combination))
                {
                    self.add(FindingKind::Unknown, "spin-per-foot-revolutions", Some(index), None,
                        if missing || unknown_position { "Incomplete evidence prevents checking the per-foot revolution/position requirement." }
                        else { "If executed as planned, the change-foot spin lacks three revolutions on a foot, or a one-position spin lacks two basic-position revolutions there: conditional V consequence, not a supplied panel call." },
                        &communication("p. 4, Free Skating change-foot revolutions and basic positions"));
                }
            }
        }
        if abbreviation.is_some_and(|a| a.kind == RegularSpinKind::Combination)
            && positions.len() < 3
        {
            self.add(FindingKind::Unknown, "spin-combination-positions", Some(index), None,
                if unknown_position { "Missing position evidence prevents checking combination recognition and full-value requirements." }
                else if positions.len() == 2 { "If executed as planned and otherwise recognized, only two basic positions qualify: conditional V consequence; full value requires three." }
                else { "The plan lacks two qualifying basic positions for combination recognition; the panel fallback is not inferred." }, source);
        }
        // A concluding upright position can be a permitted wind-up. Do not infer
        // combination recognition or reject a one-position abbreviation from it.
    }
}
