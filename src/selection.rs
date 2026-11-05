//! Profile-local choices from the immutable SOV, without execution assumptions.
use crate::program_validation::EvaluationScope;
use crate::*;

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
    let selection = Selection::new(registry, profile, table)?;
    let mut options = Vec::new();
    for row in crate::catalog::shared_rows(table) {
        match selection.check_row(row) {
            Ok(option) => options.push(option),
            Err(ElementSelectionError::Profile(error)) => return Err(error),
            Err(_) => {}
        }
    }
    Ok(options)
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

pub(crate) struct Selection<'a> {
    registry: &'a ProfileRegistry,
    profile: &'a RuleProfile,
    table: ScoringTableId,
    policy: ProgramPolicy,
}

impl<'a> Selection<'a> {
    pub(crate) fn new(
        registry: &'a ProfileRegistry,
        id: &ProfileId,
        table: ScoringTableId,
    ) -> Result<Self, ProfileSelectionError> {
        let profile = registry.resolve(id, table)?;
        let policy = crate::profiles::resolve_program_policy(profile, None)?
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
        crate::profiles::resolve_program_policy(profile, Some(policy))?;
        Ok(Self {
            registry,
            profile,
            table,
            policy: policy.clone(),
        })
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

    pub(crate) fn check(&self, code: &str) -> Result<SelectableElement, ElementSelectionError> {
        self.check_row(catalog_row(self.table, code).ok_or_else(|| {
            ElementSelectionError::UnknownCode {
                source: self
                    .table
                    .source_reference(format!("Exact notation lookup: {code}")),
            }
        })?)
    }

    fn check_row(
        &self,
        row: &'static CatalogRow,
    ) -> Result<SelectableElement, ElementSelectionError> {
        use ElementSelectionError as Error;
        use SelectableElementKind as Kind;
        let unsupported = || Error::UnsupportedCode {
            source: row.source.reference(row.notation),
        };
        if row.discipline != CatalogDiscipline::SingleAndPair {
            return Err(unsupported());
        }
        let (kind, element) = match row.family {
            CatalogFamily::Jump => {
                let (kind, rotations) = clean_jump(row.notation).ok_or_else(unsupported)?;
                (
                    Kind::SoloJump { kind, rotations },
                    PlannedElement::Jump(JumpElement::Solo(ListedJump {
                        kind: Some(kind),
                        rotations: Some(rotations),
                    })),
                )
            }
            CatalogFamily::Spin => (
                Kind::RegularSpin(
                    crate::assumed_recognition::spin_abbreviation(row.notation)
                        .ok_or_else(unsupported)?,
                ),
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
            _ => return Err(unsupported()),
        };
        let (evidence, _) = crate::assumed_recognition::resolve_individual_row(row, &element, None)
            .ok_or_else(unsupported)?;
        let report = crate::program_validation::validate_program_scope(
            self.registry,
            &ProfileProgram {
                profile: self.profile.id.clone(),
                scoring_table: self.table,
                elements: vec![element],
            },
            &[evidence],
            &self.policy,
            EvaluationScope::AssumedRecognition,
        )
        .map_err(Error::Profile)?;
        let is_level = |f: &Finding| {
            matches!(
                f.rule_id,
                "spin-level-cap" | crate::program_validation::ASSUMED_STEP_LEVEL_CAP
            )
        };
        // Type restrictions explain the rejection even if the level also exceeds its cap.
        if let Some(finding) = report
            .validation
            .findings
            .into_iter()
            .filter(|f| {
                f.kind == FindingKind::Violation
                    && (f.element_index.is_some()
                        || f.rule_id == crate::program_validation::FAMILY_CAP_RULE)
            })
            .min_by_key(is_level)
        {
            return Err(if is_level(&finding) {
                Error::LevelNotSelectable(Box::new(finding))
            } else {
                Error::TypeNotSelectable(Box::new(finding))
            });
        }
        Ok(SelectableElement { row, kind })
    }
}

fn unsupported_profile(profile: &RuleProfile) -> ProfileSelectionError {
    ProfileSelectionError {
        profile: profile.id.clone(),
        kind: ProfileErrorKind::Unsupported,
        reason: "No supported singles free skating selection policy for this profile.".into(),
    }
}

fn clean_jump(code: &str) -> Option<(JumpKind, u8)> {
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
