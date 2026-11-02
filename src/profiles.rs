//! Immutable, explicitly selected rule policy. Catalog values remain separate.
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileId {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileDiscipline {
    Single,
    Pair,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSegment {
    FreeSkating,
    ShortProgram,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileProvenance {
    Official,
    Synthetic,
}

/// Each mechanism is a named implementation version, not an expression language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleMechanism {
    JumpPlanV1,
    AdultScoringV1,
    Unsupported(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileRule {
    Input,
    Eligibility,
    Count,
    CompoundCount,
    Structure,
    Euler,
    Repetition,
    Axel,
    Transition,
    Scoring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleBinding {
    pub rule: ProfileRule,
    pub id: &'static str,
    pub mechanism: RuleMechanism,
    pub source: SourceReference,
}

/// A scoped replacement of one base binding. Chained overrides are rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleOverride {
    pub replaces: &'static str,
    pub replacement: RuleBinding,
    pub event_scope: String,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpPermission {
    pub kind: JumpKind,
    pub min_rotations: u8,
    pub max_rotations: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileElementFamily {
    SoloJump,
    Combination,
    JumpSequence,
    RegularSpin,
    ChoreographicSpin,
    StepSequence,
    ChoreographicSequence,
}
impl ProfileElementFamily {
    pub(crate) fn of(element: &PlannedElement) -> Option<Self> {
        Some(match element {
            PlannedElement::Jump(JumpElement::Solo(_)) => Self::SoloJump,
            PlannedElement::Jump(JumpElement::Combination(_)) => Self::Combination,
            PlannedElement::Jump(JumpElement::Sequence(_)) => Self::JumpSequence,
            PlannedElement::RegularSpin(_) => Self::RegularSpin,
            PlannedElement::ChoreographicSpin => Self::ChoreographicSpin,
            PlannedElement::StepSequence => Self::StepSequence,
            PlannedElement::ChoreographicSequence { .. } => Self::ChoreographicSequence,
            PlannedElement::Unspecified => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyPolicy {
    pub allowed: Vec<ProfileElementFamily>,
    pub required: Vec<ProfileElementFamily>,
}

/// Limits have the semantics of the named JumpPlanV1 evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpPolicy {
    pub allowed: Vec<JumpPermission>,
    pub max_jumps: usize,
    pub max_compound_length: usize,
    pub max_combinations: usize,
    pub max_sequences: usize,
    pub max_compounds: usize,
    pub max_three_jump_compounds: usize,
    pub max_eulers: usize,
    pub max_repetitions: usize,
    pub require_axel: bool,
    /// Compatibility diagnostics only; never selects rules.
    pub(crate) bronze_diagnostics: bool,
}

impl JumpPolicy {
    pub(crate) fn prohibited(&self, jump: &ListedJump) -> bool {
        if let Some(kind) = jump.kind {
            !self.allowed.iter().any(|p| {
                p.kind == kind
                    && jump
                        .rotations
                        .is_none_or(|n| n == 0 || (p.min_rotations..=p.max_rotations).contains(&n))
            })
        } else if let Some(n) = jump.rotations {
            n > 0
                && !self
                    .allowed
                    .iter()
                    .any(|p| (p.min_rotations..=p.max_rotations).contains(&n))
        } else {
            false
        }
    }
}

/// Non-jump recognition/feature eligibility is not implemented by JumpPlanV1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedPolicy {
    pub family: ProfileElementFamily,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    pub permitted_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileScoringPolicy {
    pub recognition_ruleset: RulesetId,
    pub duration_seconds: u32,
    pub duration_source: SourceReference,
    pub time_deduction_source: SourceReference,
    /// None preserves unsupported fall deductions rather than assuming zero.
    pub fall_hundredths: Option<u32>,
    pub fall_source: SourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleProfile {
    pub id: ProfileId,
    pub discipline: ProfileDiscipline,
    pub segment: ProgramSegment,
    pub season: Season,
    pub event_scope: String,
    pub category: String,
    pub provenance: ProfileProvenance,
    pub scoring_table: ScoringTableId,
    pub sources: Vec<SourceReference>,
    pub precedence: String,
    pub bindings: Vec<RuleBinding>,
    pub overrides: Vec<RuleOverride>,
    pub jumps: JumpPolicy,
    pub families: FamilyPolicy,
    pub scoring: ProfileScoringPolicy,
    pub unsupported_policies: Vec<UnsupportedPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileErrorKind {
    Unknown,
    Invalid,
    Unsupported,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSelectionError {
    pub profile: ProfileId,
    pub kind: ProfileErrorKind,
    pub reason: String,
}
impl std::fmt::Display for ProfileSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}: {}",
            self.profile.name, self.profile.version, self.reason
        )
    }
}
impl std::error::Error for ProfileSelectionError {}

impl RuleProfile {
    fn error(&self, kind: ProfileErrorKind, reason: &str) -> ProfileSelectionError {
        ProfileSelectionError {
            profile: self.id.clone(),
            kind,
            reason: reason.into(),
        }
    }
    pub(crate) fn binding(&self, rule: ProfileRule) -> &RuleBinding {
        self.overrides
            .iter()
            .find(|o| o.replacement.rule == rule)
            .map(|o| &o.replacement)
            .unwrap_or_else(|| {
                self.bindings
                    .iter()
                    .find(|b| b.rule == rule)
                    .expect("registered binding")
            })
    }
    pub(crate) fn supported(&self, table: ScoringTableId) -> Result<(), ProfileSelectionError> {
        if self.discipline != ProfileDiscipline::Single
            || self.segment != ProgramSegment::FreeSkating
        {
            return Err(self.error(
                ProfileErrorKind::Unsupported,
                "discipline or segment evaluator is unsupported",
            ));
        }
        if table != self.scoring_table {
            return Err(self.error(
                ProfileErrorKind::Unsupported,
                "SOV edition does not match the selected profile",
            ));
        }
        if !self.unsupported_policies.is_empty()
            || self
                .bindings
                .iter()
                .any(|b| matches!(b.mechanism, RuleMechanism::Unsupported(_)))
            || self
                .overrides
                .iter()
                .any(|o| matches!(o.replacement.mechanism, RuleMechanism::Unsupported(_)))
        {
            return Err(self.error(
                ProfileErrorKind::Unsupported,
                "profile requires an unsupported rule mechanism",
            ));
        }
        Ok(())
    }
    #[cfg(feature = "optimizer")]
    pub(crate) fn jump_bindings(&self) -> Vec<&RuleBinding> {
        self.bindings
            .iter()
            .filter(|b| {
                b.rule != ProfileRule::Input
                    && b.rule != ProfileRule::Scoring
                    && (b.rule != ProfileRule::Axel || self.jumps.require_axel)
            })
            .map(|b| self.binding(b.rule))
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileRegistry {
    profiles: Vec<RuleProfile>,
}
fn citation_valid(source: &SourceReference) -> bool {
    [&source.document, &source.version, &source.location]
        .iter()
        .all(|s| !s.trim().is_empty())
}
impl ProfileRegistry {
    pub fn adult() -> Self {
        let mut registry = Self::default();
        for category in [
            AdultCategory::Bronze,
            AdultCategory::Silver,
            AdultCategory::Gold,
            AdultCategory::Masters,
        ] {
            registry
                .register(adult_profile(category))
                .expect("built-in profile");
        }
        registry
    }
    /// Validate and retain the exact definition; registration never overwrites.
    pub fn register(&mut self, profile: RuleProfile) -> Result<(), ProfileSelectionError> {
        let fail = |reason| profile.error(ProfileErrorKind::Invalid, reason);
        if self.profiles.iter().any(|p| p.id == profile.id) {
            return Err(fail("profile identity already registered"));
        }
        if [
            &profile.id.name,
            &profile.id.version,
            &profile.event_scope,
            &profile.category,
            &profile.precedence,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || profile.season.end_year != profile.season.start_year.saturating_add(1)
            || profile.sources.is_empty()
            || profile.sources.iter().any(|s| !citation_valid(s))
        {
            return Err(fail(
                "identity, scope, season or exact sources are missing or invalid",
            ));
        }
        if profile.families.allowed.is_empty()
            || profile
                .families
                .required
                .iter()
                .any(|f| !profile.families.allowed.contains(f))
        {
            return Err(fail("required families must be allowed"));
        }
        let policy = &profile.jumps;
        if policy.max_compound_length < 2
            || policy.max_compound_length > 3
            || policy.max_repetitions == 0
            || policy.max_combinations > policy.max_compounds
            || policy.max_sequences > policy.max_compounds
            || policy.max_three_jump_compounds > policy.max_compounds
            || policy.allowed.is_empty()
            || policy
                .allowed
                .iter()
                .any(|p| p.min_rotations == 0 || p.min_rotations > p.max_rotations)
            || policy
                .allowed
                .iter()
                .enumerate()
                .any(|(i, p)| policy.allowed[..i].iter().any(|q| q.kind == p.kind))
        {
            return Err(fail("invalid jump policy bounds or permissions"));
        }
        if profile.scoring.duration_seconds == 0
            || [
                &profile.scoring.duration_source,
                &profile.scoring.time_deduction_source,
                &profile.scoring.fall_source,
            ]
            .iter()
            .any(|s| !citation_valid(s) || !profile.sources.contains(s))
            || profile
                .unsupported_policies
                .iter()
                .any(|p| matches!((p.min_level,p.max_level), (Some(a),Some(b)) if a>b))
        {
            return Err(fail("invalid scoring or unsupported-policy metadata"));
        }
        let required = [
            ProfileRule::Input,
            ProfileRule::Eligibility,
            ProfileRule::Count,
            ProfileRule::CompoundCount,
            ProfileRule::Structure,
            ProfileRule::Euler,
            ProfileRule::Repetition,
            ProfileRule::Axel,
            ProfileRule::Transition,
            ProfileRule::Scoring,
        ];
        if profile.bindings.len() != required.len()
            || required
                .iter()
                .any(|r| profile.bindings.iter().filter(|b| b.rule == *r).count() != 1)
        {
            return Err(fail("each supported rule requires exactly one binding"));
        }
        let valid_binding = |b: &RuleBinding| {
            !b.id.trim().is_empty()
                && citation_valid(&b.source)
                && profile.sources.contains(&b.source)
                && match b.mechanism {
                    RuleMechanism::JumpPlanV1 => b.rule != ProfileRule::Scoring,
                    RuleMechanism::AdultScoringV1 => b.rule == ProfileRule::Scoring,
                    RuleMechanism::Unsupported(_) => true,
                }
        };
        if profile.bindings.iter().any(|b| !valid_binding(b))
            || profile
                .bindings
                .iter()
                .enumerate()
                .any(|(i, b)| profile.bindings[..i].iter().any(|q| q.id == b.id))
        {
            return Err(fail("invalid, conflicting or uncited rule binding"));
        }
        for (i, replacement) in profile.overrides.iter().enumerate() {
            if !valid_binding(&replacement.replacement)
                || replacement.justification.trim().is_empty()
                || replacement.event_scope != profile.event_scope
                || !profile
                    .bindings
                    .iter()
                    .any(|b| b.id == replacement.replaces && b.rule == replacement.replacement.rule)
                || profile.overrides[..i].iter().any(|o| {
                    o.replaces == replacement.replaces
                        || o.replacement.id == replacement.replacement.id
                })
                || profile
                    .bindings
                    .iter()
                    .any(|b| b.id == replacement.replacement.id)
                || profile
                    .overrides
                    .iter()
                    .any(|o| o.replaces == replacement.replacement.id)
            {
                return Err(fail("ambiguous, cyclic or out-of-scope override"));
            }
        }
        self.profiles.push(profile);
        Ok(())
    }
    /// Exact identity lookup. Unsupported definitions remain inspectable.
    pub fn get(&self, id: &ProfileId) -> Result<&RuleProfile, ProfileSelectionError> {
        self.profiles
            .iter()
            .find(|p| p.id == *id)
            .ok_or_else(|| ProfileSelectionError {
                profile: id.clone(),
                kind: ProfileErrorKind::Unknown,
                reason: "unknown profile identity/version".into(),
            })
    }
    pub fn resolve(
        &self,
        id: &ProfileId,
        table: ScoringTableId,
    ) -> Result<&RuleProfile, ProfileSelectionError> {
        let profile = self.get(id)?;
        profile.supported(table)?;
        Ok(profile)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProgram {
    pub profile: ProfileId,
    pub scoring_table: ScoringTableId,
    pub elements: Vec<PlannedElement>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileValidationResult {
    pub profile: ProfileId,
    pub validation: ValidationResult,
}
pub fn validate_profile_program(
    registry: &ProfileRegistry,
    program: &ProfileProgram,
) -> Result<ProfileValidationResult, ProfileSelectionError> {
    let profile = registry.resolve(&program.profile, program.scoring_table)?;
    Ok(ProfileValidationResult {
        profile: profile.id.clone(),
        validation: crate::validation::validate_elements(&program.elements, profile),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProgramScore {
    pub profile: ProfileId,
    pub score: ProgramScore,
}

/// Uses the shared numerical scorer, with explicit profile timing and deductions.
/// This does not infer category eligibility from supplied panel recognition.
#[allow(clippy::too_many_arguments)]
pub fn score_profile_program(
    registry: &ProfileRegistry,
    id: &ProfileId,
    table: ScoringTableId,
    elements: &[ProgramElement],
    outside_falls: Option<u32>,
    timing: Option<ProgramTiming>,
    execution: Option<&[ElementExecutionEvidence]>,
) -> Result<ProfileProgramScore, ProfileSelectionError> {
    let profile = registry.resolve(id, table)?;
    if elements
        .iter()
        .any(|e| e.recognition.ruleset != profile.scoring.recognition_ruleset)
    {
        return Err(profile.error(
            ProfileErrorKind::Unsupported,
            "recognition ruleset does not match profile compatibility binding",
        ));
    }
    Ok(ProfileProgramScore {
        profile: id.clone(),
        score: crate::scoring::score_program_internal(
            profile.scoring.recognition_ruleset,
            &profile.scoring,
            table,
            elements,
            outside_falls,
            timing,
            execution,
        ),
    })
}

/// The sole legacy category-to-profile adapter. New categories use registration.
pub fn adult_profile(category: AdultCategory) -> RuleProfile {
    let (name, page, cap, length, duration, axel, bronze) = match category {
        AdultCategory::Bronze => ("bronze", "p. 11", 4, 2, 100, false, true),
        AdultCategory::Silver => ("silver", "p. 10", 4, 3, 120, false, false),
        AdultCategory::Gold => ("gold", "p. 9", 5, 3, 170, false, false),
        AdultCategory::Masters => ("masters", "p. 8", 5, 3, 180, true, false),
    };
    let source = |location: String| SourceReference {
        document: "International Adult Figure Skating Competitions Technical Announcement".into(),
        version: "2026-07-16".into(),
        location,
    };
    let category_source = source(format!(
        "{page}, {} Free Skating, jumps",
        match category {
            AdultCategory::Bronze => "Bronze",
            AdultCategory::Silver => "Silver",
            AdultCategory::Gold => "Gold",
            AdultCategory::Masters => "Masters",
        }
    ));
    let duration_source = source(page.into());
    let time_source = source(format!("p. 6, time deductions; {page}, required duration"));
    let fall_source = source("p. 11, Bronze fall deductions".into());
    let mut bindings = crate::validation::adult_bindings(category, &category_source);
    bindings.push(RuleBinding {
        rule: ProfileRule::Scoring,
        id: "isu-adult-2026-27-scoring",
        mechanism: RuleMechanism::AdultScoringV1,
        source: duration_source.clone(),
    });
    let mut sources = vec![
        category_source,
        duration_source.clone(),
        time_source.clone(),
        fall_source.clone(),
    ];
    sources.extend([
        SourceReference {
            document: "ISU Communication 2786".into(),
            version: "2026-06-02".into(),
            location: "Single SOV rows; linked table edition".into(),
        },
        SourceReference {
            document: "ISU Communication 2788".into(),
            version: "2026-05-19".into(),
            location: "pp. 12, 14, 16, GOE limits and reductions".into(),
        },
        SourceReference {
            document: "ISU Handbook for Referees and Judges".into(),
            version: "2026-05-19".into(),
            location: "pp. 2-6, GOE and calls".into(),
        },
        SourceReference {
            document: "Technical Panel Handbook, Single Skating".into(),
            version: "2026-07-15".into(),
            location: "pp. 2, 6, 25, 30-33, recognition and calls".into(),
        },
        SourceReference {
            document: "Technical Committee Q&A 3".into(),
            version: "2026-10-08".into(),
            location: "p. 1 Q2-4, starred jumps".into(),
        },
    ]);
    for binding in &bindings {
        if !sources.contains(&binding.source) {
            sources.push(binding.source.clone());
        }
    }
    let allowed = [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
        JumpKind::Axel,
    ]
    .into_iter()
    .filter(|kind| !bronze || *kind != JumpKind::Axel)
    .map(|kind| JumpPermission {
        kind,
        min_rotations: 1,
        max_rotations: match category {
            AdultCategory::Masters => 2,
            AdultCategory::Gold
                if matches!(kind, JumpKind::ToeLoop | JumpKind::Salchow | JumpKind::Loop) =>
            {
                2
            }
            _ => 1,
        },
    })
    .collect();
    RuleProfile {
        id: ProfileId { name: format!("isu-adult-2026-27-{name}"), version: "2026-07-16".into() },
        discipline: ProfileDiscipline::Single, segment: ProgramSegment::FreeSkating,
        season: Season { start_year: 2026, end_year: 2027 },
        event_scope: "2026/27 ISU International Adult competitions covered by the July 16 announcement".into(),
        category: name.into(), provenance: ProfileProvenance::Official,
        scoring_table: ScoringTableId::Isu2786June2_2026, sources,
        precedence: "Adult announcement p. 4: announcement prevails within scope unless a pertinent Communication contains special Adult comments; unresolved conflicts remain unsupported".into(),
        bindings, overrides: vec![],
        families: FamilyPolicy { allowed: vec![ProfileElementFamily::SoloJump, ProfileElementFamily::Combination, ProfileElementFamily::JumpSequence, ProfileElementFamily::RegularSpin, ProfileElementFamily::ChoreographicSpin, ProfileElementFamily::StepSequence, ProfileElementFamily::ChoreographicSequence], required: vec![] },
        jumps: JumpPolicy { allowed, max_jumps: cap, max_compound_length: length, max_combinations: if bronze {1} else {2}, max_sequences: usize::from(!bronze), max_compounds: if bronze {1} else {2}, max_three_jump_compounds: usize::from(!bronze), max_eulers: usize::from(!bronze), max_repetitions: 2, require_axel: axel, bronze_diagnostics: bronze },
        scoring: ProfileScoringPolicy { recognition_ruleset: RulesetId::IsuAdult2026_27July16, duration_seconds: duration, duration_source, time_deduction_source: time_source, fall_hundredths: if bronze {Some(50)} else {None}, fall_source },
        unsupported_policies: vec![],
    }
}
