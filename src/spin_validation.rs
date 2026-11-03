//! Additive planned-spin checks. Execution consequences remain conditional.
use crate::*;

pub(crate) const RULE_IDS: &[&str] = &[
    "spin-abbreviation",
    "spin-basic-position",
    "spin-combination-positions",
    "spin-count",
    "spin-distinct-abbreviation",
    "spin-eligibility",
    "spin-entry",
    "spin-evidence-entry",
    "spin-evidence-foot-change",
    "spin-evidence-target",
    "spin-execution-recognition",
    "spin-feature-credit",
    "spin-feature-evidence",
    "spin-feature-target",
    "spin-layback-position",
    "spin-level-cap",
    "spin-minimum-revolutions",
    "spin-multiple-foot-changes",
    "spin-panel-reconciliation",
    "spin-panel-target",
    "spin-per-foot-policy",
    "spin-per-foot-revolutions",
    "spin-policy",
    "spin-position-evidence",
    "spin-position-policy",
    "spin-position-type",
    "spin-recognition-minimum",
    "spin-required-role",
    "spin-rotations",
    "spin-wind-up-target",
];

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

/// None is missing structure; false is a contradicted target or attempt order.
pub(crate) fn feature_target_validity(
    spin: &RegularSpin,
    attempt: &SpinFeatureAttempt,
    previous: Option<usize>,
) -> Option<bool> {
    if previous.is_some_and(|p| p > attempt.position_step) {
        return Some(false);
    }
    spin.steps.as_ref().map(|steps| {
        matches!(
            steps.get(attempt.position_step),
            Some(SpinStep::Position { .. })
        )
    })
}

/// Input contradictions differ from category eligibility and program-count rules.
pub(crate) fn contradicts_spin_evidence(finding: &Finding) -> bool {
    matches!(
        finding.rule_id,
        "spin-evidence-target"
            | "spin-evidence-entry"
            | "spin-evidence-foot-change"
            | "spin-wind-up-target"
            | "spin-feature-target"
            | "spin-panel-target"
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegularSpinEvidence {
    pub abbreviation: Option<SpinAbbreviation>,
    pub planned_level: Option<Level>,
    /// None is unknown; Some(empty) explicitly states no attempts.
    pub feature_attempts: Option<Vec<SpinFeatureAttempt>>,
    /// Supplied independently; never used to fill gaps in the intended plan.
    pub panel: Option<PanelRecognition>,
    /// Explicit final upright wind-up step, asserting no additional feature there.
    pub final_upright_wind_up: Option<usize>,
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
    pub positions: Option<SpinPositionRules>,
    pub per_foot: Option<SpinFootRules>,
}

/// Explicit opt-in to the source-defined planned-position mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinPositionRules {
    pub source: SourceReference,
    pub min_basic_revolutions: u16,
    pub min_spin_revolutions: u16,
    pub min_combination_positions: usize,
    pub full_value_positions: usize,
    pub max_other_position_revolutions: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinFootRules {
    pub source: SourceReference,
    pub min_revolutions: u16,
    pub min_basic_revolutions: u16,
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
    pub applied_rules: Vec<AppliedRule>,
    pub validation: ValidationResult,
    pub coverage: Vec<ValidationCoverage>,
}

pub(crate) fn communication(location: &str) -> SourceReference {
    SourceReference {
        document: "ISU Communication 2788".into(),
        version: "2026-05-19".into(),
        location: location.into(),
    }
}
pub(crate) fn handbook(location: &str) -> SourceReference {
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
    validate_profile_program_with_assumptions(
        registry,
        program,
        evidence,
        policy,
        &[],
        FeatureCreditEvaluation::Deferred,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeatureCreditEvaluation {
    Deferred,
    ProgramWide,
}

pub(crate) fn validate_profile_program_with_assumptions(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
    evidence: &ProgramValidationEvidence,
    policy: Option<&RegularSpinPolicy>,
    assumed_direct_sequences: &[usize],
    feature_credit: FeatureCreditEvaluation,
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
        let sources = std::iter::once(&p.source)
            .chain(p.positions.as_ref().map(|r| &r.source))
            .chain(p.per_foot.as_ref().map(|r| &r.source));
        if p.profile != profile.id
            || sources.into_iter().any(|s| {
                s.document.trim().is_empty()
                    || s.version.trim().is_empty()
                    || s.location.trim().is_empty()
            })
            || p.positions.as_ref().is_some_and(|r| {
                r.min_basic_revolutions == 0
                    || r.min_spin_revolutions == 0
                    || r.min_combination_positions < 2
                    || r.full_value_positions < r.min_combination_positions
            })
            || p.per_foot
                .as_ref()
                .is_some_and(|r| r.min_revolutions == 0 || r.min_basic_revolutions == 0)
        {
            return Err(ProfileSelectionError { profile: profile.id.clone(), kind: ProfileErrorKind::Invalid,
                reason: "spin rules need the selected profile identity, complete source bindings and positive coherent recognition thresholds".into() });
        }
    }
    let (jump_validation, applied_rules) = crate::validation::validate_elements_traced(
        &program.elements,
        profile,
        assumed_direct_sequences,
    );
    let mut check = SpinValidator {
        findings: jump_validation.findings,
        applied_rules,
        feature_credit,
    };
    let indexed = check.index_evidence(&program.elements, evidence);
    if let Some(policy) = policy {
        check.program(
            &program.elements,
            &indexed,
            policy,
            profile.scoring.recognition_ruleset,
        );
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
        .chain(check.applied_rules.iter().map(|r| &r.source))
        .chain(policy.map(|p| &p.source))
        .chain(policy.and_then(|p| p.positions.as_ref().map(|r| &r.source)))
        .chain(policy.and_then(|p| p.per_foot.as_ref().map(|r| &r.source)))
    {
        if !sources.contains(source) {
            sources.push(source.clone());
        }
    }
    Ok(EvidenceValidationReport {
        profile: profile.id.clone(), sources, applied_rules: check.applied_rules,
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
    applied_rules: Vec<AppliedRule>,
    feature_credit: FeatureCreditEvaluation,
}
impl SpinValidator {
    fn apply(&mut self, id: &'static str, index: Option<usize>, source: &SourceReference) {
        crate::validation::record_rule(&mut self.applied_rules, id, index, None, source);
    }
    fn add(
        &mut self,
        kind: FindingKind,
        rule_id: &'static str,
        element_index: Option<usize>,
        component_index: Option<usize>,
        explanation: &'static str,
        source: &SourceReference,
    ) {
        crate::validation::record_rule(
            &mut self.applied_rules,
            rule_id,
            element_index,
            component_index,
            source,
        );
        self.findings.push(Finding {
            kind,
            rule_id,
            element_index,
            component_index,
            explanation,
            source: source.clone(),
        });
    }
    fn index_evidence<'a>(
        &mut self,
        elements: &[PlannedElement],
        evidence: &'a ProgramValidationEvidence,
    ) -> Vec<Option<&'a RegularSpinEvidence>> {
        let mut indexed = vec![None; elements.len()];
        let mut seen = vec![false; elements.len()];
        for item in &evidence.spins {
            self.apply(
                "spin-evidence-target",
                Some(item.element_index),
                &input_source(),
            );
            let index = item.element_index;
            if !matches!(elements.get(index), Some(PlannedElement::RegularSpin(_))) {
                self.add(
                    FindingKind::Violation,
                    "spin-evidence-target",
                    Some(index),
                    None,
                    "Spin evidence must target a regular spin exactly once.",
                    &input_source(),
                );
            } else if seen[index] {
                indexed[index] = None;
                self.add(
                    FindingKind::Violation,
                    "spin-evidence-target",
                    Some(index),
                    None,
                    "Spin evidence must target a regular spin exactly once.",
                    &input_source(),
                );
            } else {
                indexed[index] = Some(&item.spin);
                seen[index] = true;
            }
        }
        indexed
    }
    fn program(
        &mut self,
        elements: &[PlannedElement],
        evidence: &[Option<&RegularSpinEvidence>],
        policy: &RegularSpinPolicy,
        recognition_ruleset: RulesetId,
    ) {
        self.apply("spin-count", None, &policy.source);
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
                let data = evidence[index];
                let abbreviation = data.and_then(|d| d.abbreviation);
                if let Some(abbreviation) = abbreviation {
                    self.apply("spin-distinct-abbreviation", Some(index), &policy.source);
                    if known.contains(&abbreviation) {
                        self.add(FindingKind::Violation, "spin-distinct-abbreviation", Some(index), None,
                            "Planned spins must have different abbreviations; changing only the level does not change the abbreviation.", &policy.source);
                    }
                    known.push(abbreviation);
                } else {
                    unknown = true;
                }
                self.spin(index, spin, data, policy, recognition_ruleset);
            }
        }
        for required in &policy.required {
            self.apply("spin-required-role", None, &policy.source);
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
        recognition_ruleset: RulesetId,
    ) {
        self.add(FindingKind::Unknown, "spin-execution-recognition", Some(index), None,
            "Plan checks are conditional on execution; visible flying jump, uninterrupted positions, spin centres and final panel recognition are not established by a plan.", &input_source());
        self.check_abbreviation(index, spin, data, policy);
        self.check_wind_up_target(index, spin, data);
        self.check_feature_targets(index, spin, data);
        if self.feature_credit == FeatureCreditEvaluation::Deferred
            && data
                .and_then(|d| d.feature_attempts.as_ref())
                .is_some_and(|a| !a.is_empty())
        {
            self.add(FindingKind::Unsupported, "spin-feature-credit", Some(index), None,
                "Feature assertions are stored in order; success does not establish eligibility, reuse credit or a level in this slice.", &input_source());
        }
        self.check_panel(index, data, recognition_ruleset);
        let Some(steps) = &spin.steps else {
            self.add(
                FindingKind::Unknown,
                "spin-rotations",
                Some(index),
                None,
                "Ordered positions, foot changes and revolutions are unspecified.",
                &policy.source,
            );
            return;
        };
        let summary = SpinSummary::new(steps);
        self.check_revolutions(index, data, policy, &summary);
        self.check_positions(index, steps, data, policy.positions.as_ref(), &summary);
        self.check_feet(index, data, policy.per_foot.as_ref(), &summary);
    }

    fn check_abbreviation(
        &mut self,
        index: usize,
        spin: &RegularSpin,
        data: Option<&RegularSpinEvidence>,
        policy: &RegularSpinPolicy,
    ) {
        let source = &policy.source;
        for id in [
            "spin-eligibility",
            "spin-abbreviation",
            "spin-entry",
            "spin-level-cap",
        ] {
            self.apply(id, Some(index), source);
        }
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
        match data.and_then(|d| d.abbreviation) {
            None => self.add(
                FindingKind::Unknown,
                "spin-abbreviation",
                Some(index),
                None,
                "The intended spin abbreviation/type is missing.",
                source,
            ),
            Some(a) => {
                self.apply("spin-evidence-entry", Some(index), &input_source());
                if !policy.allowed.iter().any(|r| r.matches(a)) {
                    self.add(FindingKind::Violation, "spin-eligibility", Some(index), None,
                        "The planned spin type, entry or foot-change role is prohibited by this profile.", source);
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
            self.add(FindingKind::Unknown, "spin-entry", Some(index), None,
                "The planned entry is unspecified; abbreviation alone does not supply movement evidence.", source);
        }
        match data.and_then(|d| d.planned_level) {
            None => self.add(FindingKind::Unknown, "spin-level-cap", Some(index), None,
                "The target level is unspecified; no recognized level is inferred.", source),
            Some(level) if level.number() > policy.max_level.number() => self.add(FindingKind::Unknown, "spin-level-cap", Some(index), None,
                "The target exceeds the profile level cap; additional features are ignored, not a prohibition of the spin.", source),
            _ => {}
        }
    }

    fn check_wind_up_target(
        &mut self,
        index: usize,
        spin: &RegularSpin,
        data: Option<&RegularSpinEvidence>,
    ) {
        let Some(target) = data.and_then(|d| d.final_upright_wind_up) else {
            return;
        };
        self.apply("spin-wind-up-target", Some(index), &input_source());
        match &spin.steps {
            None => self.add(
                FindingKind::Unknown,
                "spin-wind-up-target",
                Some(index),
                Some(target),
                "Missing steps prevent checking the final upright wind-up target.",
                &input_source(),
            ),
            Some(steps)
                if target.checked_add(1) != Some(steps.len())
                    || !matches!(
                        steps.get(target),
                        Some(SpinStep::Position {
                            position: Some(SpinPosition::Upright),
                            ..
                        })
                    ) =>
            {
                self.add(
                    FindingKind::Violation,
                    "spin-wind-up-target",
                    Some(index),
                    Some(target),
                    "A final upright wind-up must target the final upright position step.",
                    &input_source(),
                )
            }
            _ => {}
        }
    }

    fn check_feature_targets(
        &mut self,
        index: usize,
        spin: &RegularSpin,
        data: Option<&RegularSpinEvidence>,
    ) {
        self.apply("spin-feature-evidence", Some(index), &input_source());
        let Some(attempts) = data.and_then(|d| d.feature_attempts.as_deref()) else {
            self.add(FindingKind::Unknown, "spin-feature-evidence", Some(index), None,
                "Typed feature attempts are unspecified; free-text descriptions are not confirmed features.", &input_source());
            return;
        };
        let mut previous = None;
        for attempt in attempts {
            self.apply("spin-feature-target", Some(index), &input_source());
            if feature_target_validity(spin, attempt, previous) == Some(false) {
                self.add(
                    FindingKind::Violation,
                    "spin-feature-target",
                    Some(index),
                    Some(attempt.position_step),
                    "Feature attempts need valid position-step targets in program order.",
                    &input_source(),
                );
            } else if feature_target_validity(spin, attempt, previous).is_none() {
                self.add(
                    FindingKind::Unknown,
                    "spin-feature-target",
                    Some(index),
                    Some(attempt.position_step),
                    "Missing steps prevent verification of the feature target.",
                    &input_source(),
                );
            }
            previous = Some(attempt.position_step);
        }
    }

    fn check_panel(
        &mut self,
        index: usize,
        data: Option<&RegularSpinEvidence>,
        recognition_ruleset: RulesetId,
    ) {
        if let Some(panel) = data.and_then(|d| d.panel.as_ref()) {
            self.apply("spin-panel-target", Some(index), &input_source());
            if panel.planned_element != Some(index) || panel.ruleset != recognition_ruleset {
                self.add(FindingKind::Violation, "spin-panel-target", Some(index), None,
                    "Supplied panel recognition must identify this plan element and the selected profile recognition ruleset.", &input_source());
            }
            self.add(FindingKind::Unsupported, "spin-panel-reconciliation", Some(index), None,
                "Supplied panel recognition is retained separately; reconciliation with the intended plan is not implemented.", &input_source());
        }
    }

    fn check_revolutions(
        &mut self,
        index: usize,
        data: Option<&RegularSpinEvidence>,
        policy: &RegularSpinPolicy,
        summary: &SpinSummary,
    ) {
        self.apply("spin-minimum-revolutions", Some(index), &policy.source);
        if data.and_then(|d| d.abbreviation).is_some() {
            self.apply("spin-evidence-foot-change", Some(index), &input_source());
        }
        let changed = summary.feet.len() > 1;
        if data
            .and_then(|d| d.abbreviation)
            .is_some_and(|a| a.change_foot != changed)
        {
            self.add(
                FindingKind::Violation,
                "spin-evidence-foot-change",
                Some(index),
                None,
                "The intended abbreviation and ordered steps disagree about change of foot.",
                &input_source(),
            );
        }
        let minimum = if changed {
            policy.min_change_foot_revolutions
        } else {
            policy.min_revolutions
        };
        if summary.total() < u64::from(minimum) {
            self.add(if summary.missing_count() { FindingKind::Unknown } else { FindingKind::Violation },
                "spin-minimum-revolutions", Some(index), None,
                "The plan does not establish the profile minimum total revolutions; this does not determine a panel call.", &policy.source);
        }
    }

    fn check_positions(
        &mut self,
        index: usize,
        steps: &[SpinStep],
        data: Option<&RegularSpinEvidence>,
        rules: Option<&SpinPositionRules>,
        summary: &SpinSummary,
    ) {
        let Some(rules) = rules else {
            self.add(
                FindingKind::Unsupported,
                "spin-position-policy",
                Some(index),
                None,
                "This profile supplies no source-bound position/recognition rules.",
                &input_source(),
            );
            return;
        };
        for id in [
            "spin-position-evidence",
            "spin-recognition-minimum",
            "spin-basic-position",
        ] {
            self.apply(id, Some(index), &rules.source);
        }
        let mut positions = vec![];
        for (step_index, step) in steps.iter().enumerate() {
            if let SpinStep::Position {
                position,
                revolutions,
                ..
            } = step
            {
                if position.is_none() || revolutions.is_none() {
                    self.add(
                        FindingKind::Unknown,
                        "spin-position-evidence",
                        Some(index),
                        Some(step_index),
                        "A position or its consecutive revolution count is missing.",
                        &rules.source,
                    );
                }
                if let (Some(p), Some(n)) = (position, revolutions) {
                    if *n >= rules.min_basic_revolutions && !positions.contains(p) {
                        positions.push(*p);
                    }
                }
            }
        }
        if summary.total() < u64::from(rules.min_spin_revolutions) && !summary.missing_count() {
            self.add(FindingKind::Unknown, "spin-recognition-minimum", Some(index), None,
                "If executed as planned, the movement has too few total revolutions to be identified as a spin; this is conditional skating-movement identification, not a prohibited spin or an inferred panel call.", &rules.source);
            return;
        }
        if positions.is_empty() {
            self.add(FindingKind::Unknown, "spin-basic-position", Some(index), None,
                if summary.unknown_position { "Missing position evidence prevents checking a qualifying basic position." }
                else { "If executed as planned and identified as a spin, no basic position qualifies: conditional no-value consequence; panel identification remains separate." }, &rules.source);
        }
        let abbreviation = data.and_then(|d| d.abbreviation);
        if let Some(a) = abbreviation {
            if a.kind == RegularSpinKind::Layback {
                self.add(FindingKind::Unknown, "spin-layback-position", Some(index), None,
                    "Upright is only the basic-position class; specific Layback posture is not represented by this plan and remains unknown.", &rules.source);
            }
            self.apply(
                if a.kind.basic_position().is_some() {
                    "spin-position-type"
                } else {
                    "spin-combination-positions"
                },
                Some(index),
                &rules.source,
            );
            if let Some(expected) = a.kind.basic_position() {
                if !positions.contains(&expected) {
                    self.add(FindingKind::Unknown, "spin-position-type", Some(index), None,
                        "The intended one-position type lacks a qualifying matching basic position; recognition or fallback is not inferred.", &rules.source);
                }
                self.check_one_position(index, steps, data, expected, rules);
            } else if positions.len() < rules.full_value_positions {
                self.add(FindingKind::Unknown, "spin-combination-positions", Some(index), None,
                    if summary.unknown_position { "Missing position evidence prevents checking combination recognition and full-value requirements." }
                    else if positions.len() >= rules.min_combination_positions { "If executed as planned and otherwise recognized, too few basic positions qualify for full value: conditional V consequence." }
                    else if positions.len() == 1 { "If the planned combination is clearly visibly attempted and only one basic position qualifies, conditional no-value follows; the actual panel recognition or fallback is not inferred." }
                    else { "No qualifying basic position is established; conditional no-value if identified as a spin, with panel fallback kept separate." }, &rules.source);
            }
        }
    }

    fn check_one_position(
        &mut self,
        index: usize,
        steps: &[SpinStep],
        data: Option<&RegularSpinEvidence>,
        expected: SpinPosition,
        rules: &SpinPositionRules,
    ) {
        let wind_up = data.and_then(|d| d.final_upright_wind_up);
        for (step_index, step) in steps.iter().enumerate() {
            if let SpinStep::Position {
                position: Some(p),
                revolutions: Some(n),
                feature_attempts,
            } = step
            {
                if *p == expected || *n <= rules.max_other_position_revolutions {
                    continue;
                }
                let explicit_wind_up = wind_up == Some(step_index)
                    && step_index + 1 == steps.len()
                    && *p == SpinPosition::Upright;
                let no_features = data
                    .and_then(|d| d.feature_attempts.as_ref())
                    .is_some_and(|a| !a.iter().any(|f| f.position_step == step_index))
                    && feature_attempts.as_ref().is_some_and(|a| a.is_empty());
                if explicit_wind_up && no_features {
                    continue;
                }
                self.add(FindingKind::Unknown, "spin-position-type", Some(index), Some(step_index),
                    if *p == SpinPosition::Upright && step_index + 1 == steps.len() { "An additional upright position needs explicit final-wind-up and no-additional-feature evidence; one-position recognition remains unknown." }
                    else { "Another basic position exceeds the permitted hold for the declared one-position type; the plan is inconsistent with that type, and panel reclassification is not inferred." }, &rules.source);
            }
        }
    }

    fn check_feet(
        &mut self,
        index: usize,
        data: Option<&RegularSpinEvidence>,
        rules: Option<&SpinFootRules>,
        summary: &SpinSummary,
    ) {
        if summary.feet.len() == 1 {
            return;
        }
        let Some(rules) = rules else {
            self.add(
                FindingKind::Unsupported,
                "spin-per-foot-policy",
                Some(index),
                None,
                "This profile supplies no source-bound per-foot requirements.",
                &input_source(),
            );
            return;
        };
        if summary.feet.len() > 2 {
            self.add(FindingKind::Unsupported, "spin-multiple-foot-changes", Some(index), None,
                "Per-foot checks for multiple changes of foot are outside this mechanism; no prohibition is inferred.", &rules.source);
            return;
        }
        self.apply("spin-per-foot-revolutions", Some(index), &rules.source);
        let expected = data
            .and_then(|d| d.abbreviation)
            .and_then(|a| a.kind.basic_position());
        for foot in &summary.feet {
            let basic = expected.is_none_or(|p| {
                foot.positions
                    .iter()
                    .any(|&(position, n)| position == p && n >= rules.min_basic_revolutions)
            });
            if foot.revolutions < u64::from(rules.min_revolutions) || !basic {
                self.add(FindingKind::Unknown, "spin-per-foot-revolutions", Some(index), Some(foot.start_step),
                    if foot.missing_count || foot.unknown_position { "Incomplete evidence prevents checking this foot's revolution/position requirement." }
                    else { "If executed as planned, this foot lacks required revolutions or the intended one-position hold: conditional V consequence, not a supplied panel call." }, &rules.source);
            }
        }
    }
}

#[derive(Default)]
struct FootSummary {
    start_step: usize,
    revolutions: u64,
    missing_count: bool,
    unknown_position: bool,
    positions: Vec<(SpinPosition, u16)>,
}
struct SpinSummary {
    feet: Vec<FootSummary>,
    unknown_position: bool,
}
impl SpinSummary {
    fn new(steps: &[SpinStep]) -> Self {
        let mut feet = vec![FootSummary::default()];
        for (index, step) in steps.iter().enumerate() {
            match step {
                SpinStep::ChangeFoot => feet.push(FootSummary {
                    start_step: if index + 1 < steps.len() {
                        index + 1
                    } else {
                        index
                    },
                    ..FootSummary::default()
                }),
                SpinStep::Position {
                    position,
                    revolutions,
                    ..
                } => {
                    let foot = feet.last_mut().unwrap();
                    if let Some(n) = revolutions {
                        foot.revolutions += u64::from(*n);
                    } else {
                        foot.missing_count = true;
                    }
                    foot.unknown_position |= position.is_none() || revolutions.is_none();
                    if let (Some(p), Some(n)) = (position, revolutions) {
                        foot.positions.push((*p, *n));
                    }
                }
            }
        }
        let unknown_position = feet.iter().any(|f| f.unknown_position);
        Self {
            feet,
            unknown_position,
        }
    }
    fn total(&self) -> u64 {
        self.feet.iter().map(|f| f.revolutions).sum()
    }
    fn missing_count(&self) -> bool {
        self.feet.iter().any(|f| f.missing_count)
    }
}
