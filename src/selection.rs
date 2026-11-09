//! Profile-local choices from the immutable SOV, without execution assumptions.
use crate::program_validation::{
    individual_element_restriction, ElementRestriction, EvaluationScope,
};
use crate::*;

/// Personal exclusions of whole codes or semantic jump/spin content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningExclusion {
    /// Excludes this whole intended code, independently of transition evidence.
    ExactElement(String),
    JumpKind(JumpKind),
    ExactJump(ListedJump),
    SpinKind(RegularSpinKind),
}

impl PlanningExclusion {
    pub(crate) fn valid(&self, selection: &Selection<'_>, table: ScoringTableId) -> bool {
        let Self::ExactElement(code) = self else {
            return true;
        };
        let assumption = AssumedRecognition {
            code: code.clone(),
            direct_transitions_assumed: code.ends_with("+SEQ"),
        };
        // Check notation and published value independently of category admission.
        crate::manual_bv::resolve_selection(
            selection,
            table,
            RulesetId::IsuSenior2026_27June12,
            &assumption,
        )
        .2
        .is_some()
    }

    pub(crate) fn excludes(&self, code: &str) -> bool {
        match self {
            Self::ExactElement(excluded) => code == excluded,
            Self::JumpKind(kind) => code.split('+').filter_map(clean_jump).any(|j| j.0 == *kind),
            Self::ExactJump(jump) => {
                code.split('+')
                    .filter_map(clean_jump)
                    .any(|(kind, rotations)| {
                        jump.kind == Some(kind) && jump.rotations == Some(rotations)
                    })
            }
            Self::SpinKind(kind) => crate::assumed_recognition::spin_abbreviation(code)
                .is_some_and(|spin| spin.kind == *kind),
        }
    }
}

/// Individual choices intersected with personal maxima and planning exclusions.
pub fn selectable_elements_with_exclusions(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    limits: &PersonalLimits,
    exclusions: &[PlanningExclusion],
) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
    validate_exclusions(registry, profile, table, exclusions)?;
    Ok(
        selectable_elements_with_limits(registry, profile, table, limits)?
            .into_iter()
            .filter(|choice| !exclusions.iter().any(|e| e.excludes(choice.row.notation)))
            .collect(),
    )
}

/// Complete whole jump choices intersected with maxima, rotation and exclusions.
pub fn selectable_jump_elements_with_exclusions(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    rotation_preference: RotationPreference,
    limits: &PersonalLimits,
    exclusions: &[PlanningExclusion],
) -> Result<Vec<AssumedRecognition>, ProfileSelectionError> {
    validate_exclusions(registry, profile, table, exclusions)?;
    Ok(
        selectable_jump_elements_with_limits(
            registry,
            profile,
            table,
            rotation_preference,
            limits,
        )?
        .into_iter()
        .filter(|choice| !exclusions.iter().any(|e| e.excludes(&choice.code)))
        .collect(),
    )
}

fn validate_exclusions(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    exclusions: &[PlanningExclusion],
) -> Result<(), ProfileSelectionError> {
    let selection = Selection::new(registry, profile, table)?;
    if let Some(index) = exclusions.iter().position(|e| !e.valid(&selection, table)) {
        return Err(ProfileSelectionError {
            profile: profile.clone(),
            kind: ProfileErrorKind::Invalid,
            reason: format!("Malformed exact whole-element exclusion at index {index}."),
        });
    }
    Ok(())
}

/// A published single-element choice. The catalog row identifies its edition,
/// exact code, level (if numbered or Basic), value and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectableElement {
    pub row: &'static CatalogRow,
    pub kind: SelectableElementKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectableElementKind {
    SoloJump { kind: JumpKind, rotations: u8 },
    RegularSpin(SpinAbbreviation),
    StepSequence,
    ChoreographicSequence,
    ChoreographicSpin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementSelectionError {
    Profile(ProfileSelectionError),
    UnknownCode { source: SourceReference },
    UnsupportedCode { source: SourceReference },
    TypeNotSelectable(Box<Finding>),
    LevelNotSelectable(Box<Finding>),
}

impl ElementSelectionError {
    pub(crate) fn reason(&self, code: &str) -> String {
        let (reason, source) = match self {
            Self::Profile(error) => return error.reason.clone(),
            Self::UnknownCode { source } => ("No exact published SOV row", source),
            Self::UnsupportedCode { source } => {
                ("Published code is unsupported for selection", source)
            }
            Self::TypeNotSelectable(f) | Self::LevelNotSelectable(f) => (f.explanation, &f.source),
        };
        format!(
            "SOV code {code:?} is not selectable: {reason} ({}; {}; {}).",
            source.document, source.version, source.location
        )
    }
}

/// List supported individual codes in catalog order for this exact profile.
/// Program counts, required roles and compatibility with other elements are
/// checked separately by program validation.
pub fn selectable_elements(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
    Selection::new(registry, profile, table)?.list()
}

/// List every profile- and preference-permitted atomic planned jump code.
/// `+` intends a same-foot connection; `+SEQ` additionally assumes a qualifying
/// direct step into an Axel. Actual footwork and recognition remain unevaluated.
pub fn selectable_jump_elements(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    rotation_preference: RotationPreference,
) -> Result<Vec<AssumedRecognition>, ProfileSelectionError> {
    planned_jump_elements(
        registry,
        profile,
        table,
        rotation_preference,
        &PersonalLimits::default(),
        &mut || false,
    )
    .map(|choices| choices.expect("noncancellable listing"))
}

/// List choices for a registered profile with an explicit, profile-bound policy.
pub fn selectable_elements_with_policy(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    policy: &ProgramPolicy,
) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
    Selection::for_policy(registry, registry.resolve(profile, table)?, table, policy)?.list()
}

/// Explain why an exact code is unavailable, without substituting another code.
pub fn check_selectable_element(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    code: &str,
) -> Result<SelectableElement, ElementSelectionError> {
    Selection::new(registry, profile, table)
        .map_err(ElementSelectionError::Profile)?
        .check(code)
}

/// Check an exact choice using an explicit, profile-bound policy.
pub fn check_selectable_element_with_policy(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    code: &str,
    policy: &ProgramPolicy,
) -> Result<SelectableElement, ElementSelectionError> {
    let profile = registry
        .resolve(profile, table)
        .map_err(ElementSelectionError::Profile)?;
    Selection::for_policy(registry, profile, table, policy)
        .map_err(ElementSelectionError::Profile)?
        .check(code)
}

pub(crate) struct Selection<'a> {
    registry: &'a ProfileRegistry,
    profile: &'a RuleProfile,
    table: ScoringTableId,
    policy: ProgramPolicy,
    checked: std::cell::RefCell<
        std::collections::BTreeMap<String, Result<SelectableElement, ElementSelectionError>>,
    >,
}

pub(crate) struct ResolvedIndividual {
    pub(crate) option: SelectableElement,
    pub(crate) element: PlannedElement,
    pub(crate) evidence: ElementPlanEvidence,
    pub(crate) value: BaseValue,
    pub(crate) restriction: Option<ElementSelectionError>,
}

impl<'a> Selection<'a> {
    pub(crate) fn new(
        registry: &'a ProfileRegistry,
        id: &ProfileId,
        table: ScoringTableId,
    ) -> Result<Self, ProfileSelectionError> {
        let profile = registry.resolve(id, table)?;
        let policy = registry
            .program_policy(profile, None)?
            .ok_or_else(|| unsupported_profile(profile))?;
        Self::for_policy(registry, profile, table, &policy)
    }

    pub(crate) fn for_policy(
        registry: &'a ProfileRegistry,
        profile: &'a RuleProfile,
        table: ScoringTableId,
        policy: &ProgramPolicy,
    ) -> Result<Self, ProfileSelectionError> {
        if profile.discipline != ProfileDiscipline::Single
            || profile.segment != ProgramSegment::FreeSkating
        {
            return Err(unsupported_profile(profile));
        }
        if policy.spins.profile != profile.id {
            return Err(ProfileSelectionError {
                profile: profile.id.clone(),
                kind: ProfileErrorKind::Invalid,
                reason: "Selection requires a program policy bound to the selected profile.".into(),
            });
        }
        registry.program_policy(profile, Some(policy))?;
        Ok(Self {
            registry,
            profile,
            table,
            policy: policy.clone(),
            checked: std::cell::RefCell::default(),
        })
    }

    fn list(&self) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
        let mut options = Vec::new();
        for row in crate::catalog::shared_rows(self.table) {
            match self.check_row(row) {
                Ok(option) => options.push(option),
                Err(ElementSelectionError::Profile(error)) => return Err(error),
                Err(_) => {}
            }
        }
        Ok(options)
    }

    pub(crate) fn check_assumption(
        &self,
        element: &PlannedElement,
        assumption: &AssumedRecognition,
    ) -> Result<(), ElementSelectionError> {
        if matches!(
            element,
            PlannedElement::Jump(JumpElement::Combination(_) | JumpElement::Sequence(_))
        ) {
            return Ok(());
        }
        self.check(&assumption.code).map(|_| ())
    }

    pub(crate) fn policy(&self) -> &ProgramPolicy {
        &self.policy
    }

    pub(crate) fn check(&self, code: &str) -> Result<SelectableElement, ElementSelectionError> {
        if let Some(result) = self.checked.borrow().get(code) {
            return result.clone();
        }
        let result = self.row(code).and_then(|row| self.check_row(row));
        self.checked
            .borrow_mut()
            .insert(code.into(), result.clone());
        result
    }

    pub(crate) fn row(&self, code: &str) -> Result<&'static CatalogRow, ElementSelectionError> {
        catalog_row(self.table, code).ok_or_else(|| ElementSelectionError::UnknownCode {
            source: self
                .table
                .source_reference(format!("Exact notation lookup: {code}")),
        })
    }

    pub(crate) fn check_row(
        &self,
        row: &'static CatalogRow,
    ) -> Result<SelectableElement, ElementSelectionError> {
        let resolved = self.resolve_row(row)?;
        match resolved.restriction {
            Some(error) => Err(error),
            None => Ok(resolved.option),
        }
    }

    pub(crate) fn resolve_row(
        &self,
        row: &'static CatalogRow,
    ) -> Result<ResolvedIndividual, ElementSelectionError> {
        use ElementSelectionError as Error;
        let unsupported = || Error::UnsupportedCode {
            source: row.source.reference(row.notation),
        };
        let (kind, element) = selection_shape(row).ok_or_else(unsupported)?;
        let (evidence, value) =
            crate::assumed_recognition::resolve_individual_row(row, &element, None)
                .ok_or_else(unsupported)?;
        let report = crate::program_validation::validate_program_scope(
            self.registry,
            &ProfileProgram {
                profile: self.profile.id.clone(),
                scoring_table: self.table,
                elements: vec![element.clone()],
            },
            std::slice::from_ref(&evidence),
            &self.policy,
            EvaluationScope::AssumedRecognition,
        )
        .map_err(Error::Profile)?;
        let restriction = match individual_element_restriction(report.validation.findings) {
            Some(ElementRestriction::Type(finding)) => {
                Some(Error::TypeNotSelectable(Box::new(finding)))
            }
            Some(ElementRestriction::Level(finding)) => {
                Some(Error::LevelNotSelectable(Box::new(finding)))
            }
            None => None,
        };
        Ok(ResolvedIndividual {
            option: SelectableElement { row, kind },
            element,
            evidence,
            value,
            restriction,
        })
    }
}

pub(crate) fn selection_shape(row: &CatalogRow) -> Option<(SelectableElementKind, PlannedElement)> {
    use SelectableElementKind as Kind;
    if row.discipline != CatalogDiscipline::SingleAndPair {
        return None;
    }
    Some(match row.family {
        CatalogFamily::Jump => {
            let (kind, rotations) = clean_jump(row.notation)?;
            (
                Kind::SoloJump { kind, rotations },
                PlannedElement::Jump(JumpElement::Solo(ListedJump {
                    kind: Some(kind),
                    rotations: Some(rotations),
                })),
            )
        }
        CatalogFamily::Spin => (
            Kind::RegularSpin(crate::assumed_recognition::spin_abbreviation(row.notation)?),
            PlannedElement::RegularSpin(RegularSpin {
                entry: None,
                steps: None,
            }),
        ),
        CatalogFamily::StepSequence => (Kind::StepSequence, PlannedElement::StepSequence),
        CatalogFamily::ChoreographicSequence => (
            Kind::ChoreographicSequence,
            PlannedElement::ChoreographicSequence { movements: None },
        ),
        CatalogFamily::ChoreographicSpin => {
            (Kind::ChoreographicSpin, PlannedElement::ChoreographicSpin)
        }
        _ => return None,
    })
}

fn unsupported_profile(profile: &RuleProfile) -> ProfileSelectionError {
    ProfileSelectionError {
        profile: profile.id.clone(),
        kind: ProfileErrorKind::Unsupported,
        reason: "No supported singles free skating selection policy for this profile.".into(),
    }
}

pub(crate) fn clean_jump(code: &str) -> Option<(JumpKind, u8)> {
    let (rotations, suffix) = code.split_at_checked(1)?;
    let kind = match suffix {
        "T" => JumpKind::ToeLoop,
        "S" => JumpKind::Salchow,
        "Lo" => JumpKind::Loop,
        "F" => JumpKind::Flip,
        "Lz" => JumpKind::Lutz,
        "A" => JumpKind::Axel,
        _ => return None,
    };
    Some((kind, rotations.parse().ok()?))
}

/// Rule 610 connections are intentions in this domain; no executed direction is inferred.
/// None means cancellation, never an empty or complete domain.
pub(crate) fn planned_jump_elements(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    rotation_preference: RotationPreference,
    limits: &PersonalLimits,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<Vec<AssumedRecognition>>, ProfileSelectionError> {
    let selected = Selection::new(registry, profile, table)?;
    let policy = &selected.profile.jumps;
    let mut choices = Vec::new();
    let mut components = Vec::new();
    for row in crate::catalog::shared_rows(table) {
        if cancelled() {
            return Ok(None);
        }
        let option = match selected.check_row(row) {
            Ok(option) => option,
            Err(ElementSelectionError::Profile(error)) => return Err(error),
            Err(_) => continue,
        };
        if let SelectableElement {
            kind: SelectableElementKind::SoloJump { rotations, .. },
            ..
        } = option
        {
            let (_, element) = selection_shape(row).expect("selected solo jump");
            let evidence = ElementPlanEvidence::Jump {
                direct_transitions_assumed: false,
            };
            if !limits.conflicts(&element, &evidence, 0).is_empty() {
                continue;
            }
            choices.push(AssumedRecognition::new(row.notation));
            if rotations <= 4 || !crate::senior::selected(selected.profile, ProfileRule::Structure)
            {
                components.push(row.notation);
            }
        }
    }
    for length in 2..=policy.max_compound_length {
        let mut digits = vec![0; length];
        if components.is_empty() {
            break;
        }
        loop {
            if cancelled() {
                return Ok(None);
            }
            let word: Vec<_> = digits.iter().map(|&i| components[i]).collect();
            // An atomic choice must itself fit the program's repetition limit.
            if word.iter().all(|code| {
                (code.starts_with('1')
                    && crate::senior::selected(selected.profile, ProfileRule::Repetition))
                    || word.iter().filter(|other| *other == code).count() <= policy.max_repetitions
            }) {
                for euler_gap in std::iter::once(None)
                    .chain((1..length).filter(|_| policy.max_eulers > 0).map(Some))
                {
                    let mut parts = Vec::new();
                    for (index, code) in word.iter().enumerate() {
                        if euler_gap == Some(index) {
                            parts.push("Eu");
                        }
                        parts.push(code);
                    }
                    let code = parts.join("+");
                    if policy.max_combinations > 0 {
                        choices.push(AssumedRecognition::new(&code));
                    }
                    if policy.max_sequences > 0
                        && word
                            .iter()
                            .enumerate()
                            .skip(1)
                            .any(|(index, code)| code.ends_with('A') && euler_gap != Some(index))
                    {
                        choices.push(AssumedRecognition {
                            code: format!("{code}+SEQ"),
                            direct_transitions_assumed: true,
                        });
                    }
                }
            }
            let mut position = length;
            while position > 0 {
                position -= 1;
                digits[position] += 1;
                if digits[position] < components.len() {
                    break;
                }
                digits[position] = 0;
            }
            if position == 0 && digits[0] == 0 {
                break;
            }
        }
    }
    if rotation_preference == RotationPreference::AllowMixedDirections {
        return Ok(Some(choices));
    }
    let mut filtered = Vec::with_capacity(choices.len());
    for choice in choices {
        if cancelled() {
            return Ok(None);
        }
        let (element, _, _, _) = crate::manual_bv::resolve_selection(
            &selected,
            table,
            selected.profile.scoring.recognition_ruleset,
            &choice,
        );
        if crate::rotation::conflict(&element, 0, rotation_preference).is_none() {
            filtered.push(choice);
        }
    }
    Ok(Some(filtered))
}

/// Profile-permitted choices intersected with personal repertoire maxima.
pub fn selectable_elements_with_limits(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    limits: &PersonalLimits,
) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
    Ok(selectable_elements(registry, profile, table)?
        .into_iter()
        .filter(|s| {
            let Some((_, element)) = selection_shape(s.row) else {
                return false;
            };
            let Some((evidence, _)) =
                crate::assumed_recognition::resolve_individual_row(s.row, &element, None)
            else {
                return false;
            };
            limits.conflicts(&element, &evidence, 0).is_empty()
        })
        .collect())
}

/// Complete atomic jump choices intersected with personal listed rotation maxima.
pub fn selectable_jump_elements_with_limits(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    rotation_preference: RotationPreference,
    limits: &PersonalLimits,
) -> Result<Vec<AssumedRecognition>, ProfileSelectionError> {
    planned_jump_elements(
        registry,
        profile,
        table,
        rotation_preference,
        limits,
        &mut || false,
    )
    .map(|choices| choices.expect("noncancellable listing"))
}

/// Profile and personal repertoire choices with an optional step-sequence maximum.
/// Choreographic elements have no level and are unaffected.
pub fn selectable_elements_with_step_limit(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    limits: &PersonalLimits,
    maximum: Option<Level>,
) -> Result<Vec<SelectableElement>, ProfileSelectionError> {
    Ok(
        selectable_elements_with_limits(registry, profile, table, limits)?
            .into_iter()
            .filter(|s| {
                s.kind != SelectableElementKind::StepSequence
                    || maximum.is_none_or(|maximum| {
                        s.row
                            .level
                            .is_some_and(|level| level.number() <= maximum.number())
                    })
            })
            .collect(),
    )
}

#[cfg(test)]
mod planning_tests {
    use super::*;

    #[test]
    fn source_grammar_for_six_profiles_uses_the_real_manual_path() {
        // Independently transcribed grammar: Adult July16 pp8–11; Rule610/612
        // June12 pp65/71; TP Handbook July15 pp27–29. See docs/specs/planned-jump-domain.md.
        // Literal counts include every word, internal Euler placement, and both intended structures.
        let adult = ProfileRegistry::adult();
        let senior = ProfileRegistry::senior();
        for (registry, profile, count, present, absent) in [
            (
                &adult,
                adult_profile(AdultCategory::Bronze),
                30,
                "1F+1Lo",
                "1Lz+Eu+1S",
            ),
            (
                &adult,
                adult_profile(AdultCategory::Silver),
                849,
                "1Lz+1A+1T+SEQ",
                "2T",
            ),
            (
                &adult,
                adult_profile(AdultCategory::Gold),
                2652,
                "2Lo+Eu+2S+1A+SEQ",
                "2F",
            ),
            (
                &adult,
                adult_profile(AdultCategory::Masters),
                6570,
                "2F+2T+1A+SEQ",
                "3T",
            ),
            (
                &senior,
                senior_profile(SeniorCategory::Women),
                51518,
                "5Lz",
                "5Lz+1T",
            ),
            (
                &senior,
                senior_profile(SeniorCategory::Men),
                51518,
                "4Lz+1A+Eu+4F+SEQ",
                "4Lz+Eu+1A+SEQ",
            ),
        ] {
            let choices = planned_jump_elements(
                registry,
                &profile.id,
                profile.scoring_table,
                RotationPreference::AllowMixedDirections,
                &PersonalLimits::default(),
                &mut || false,
            )
            .unwrap()
            .unwrap();
            assert_eq!(choices.len(), count, "{}", profile.category);
            assert!(choices.iter().any(|c| c.code == present), "{present}");
            assert!(!choices.iter().any(|c| c.code == absent), "{absent}");
            let chosen = choices.iter().find(|c| c.code == present).unwrap().clone();
            let assessment = assess_manual_bv(
                registry,
                &ManualBvRequest {
                    personal_limits: PersonalLimits::default(),
                    rotation_preference: RotationPreference::default(),
                    profile: profile.id,
                    scoring_table: profile.scoring_table,
                    submission: SubmissionState::Draft,
                    elements: vec![chosen],
                },
            )
            .unwrap();
            assert!(assessment.total.exact_hundredths.is_some(), "{present}");
            assert!(
                !assessment
                    .legality
                    .validation
                    .validation
                    .findings
                    .iter()
                    .any(|f| f.kind == FindingKind::Violation && f.element_index.is_some()),
                "{present}: {:?}",
                assessment.legality.validation.validation.findings
            );
        }
    }

    #[test]
    fn cancellation_never_returns_a_partial_repertoire() {
        let mut polls = 0;
        let result = planned_jump_elements(
            &ProfileRegistry::senior(),
            &senior_profile(SeniorCategory::Women).id,
            ScoringTableId::Isu2786June2_2026,
            RotationPreference::default(),
            &PersonalLimits::default(),
            &mut || {
                polls += 1;
                polls == 200
            },
        )
        .unwrap();
        assert!(result.is_none());
        assert_eq!(polls, 200);
    }

    #[test]
    fn cancellation_during_preference_filtering_returns_no_partial_choices() {
        let registry = ProfileRegistry::adult();
        let p = adult_profile(AdultCategory::Silver);
        let mut expansion_polls = 0;
        planned_jump_elements(
            &registry,
            &p.id,
            p.scoring_table,
            RotationPreference::AllowMixedDirections,
            &PersonalLimits::default(),
            &mut || {
                expansion_polls += 1;
                false
            },
        )
        .unwrap()
        .unwrap();
        let mut polls = 0;
        let result = planned_jump_elements(
            &registry,
            &p.id,
            p.scoring_table,
            RotationPreference::SameDirection,
            &PersonalLimits::default(),
            &mut || {
                polls += 1;
                polls > expansion_polls
            },
        )
        .unwrap();
        assert!(result.is_none());
        assert_eq!(polls, expansion_polls + 1);
    }
}
