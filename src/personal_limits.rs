//! Personal repertoire maxima, independent of competition eligibility and scoring.
use crate::*;

/// Listed rotation classes: Single Axel means 1A, not its physical turn count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JumpRotationClass {
    Single = 1,
    Double,
    Triple,
    Quadruple,
    Quintuple,
}

impl TryFrom<u8> for JumpRotationClass {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Single),
            2 => Ok(Self::Double),
            3 => Ok(Self::Triple),
            4 => Ok(Self::Quadruple),
            5 => Ok(Self::Quintuple),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpRotationLimit {
    pub kind: JumpKind,
    pub maximum: JumpRotationClass,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpinLevelLimit {
    pub abbreviation: SpinAbbreviation,
    pub maximum: Level,
}

/// Omitted keys add no restriction. Repeated keys intersect (the lowest maximum wins).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PersonalLimits {
    pub jump_rotations: Vec<JumpRotationLimit>,
    pub spin_levels: Vec<SpinLevelLimit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalLimitConflictKind {
    JumpRotation {
        kind: JumpKind,
        requested: u8,
        maximum: JumpRotationClass,
    },
    SpinLevel {
        abbreviation: SpinAbbreviation,
        requested: Level,
        maximum: Level,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalLimitConflict {
    pub element_index: usize,
    /// Original JumpPart position; solo jumps and spins have no component index.
    pub component_index: Option<usize>,
    pub kind: PersonalLimitConflictKind,
}

impl PersonalLimits {
    pub(crate) fn conflicts(
        &self,
        element: &PlannedElement,
        evidence: &ElementPlanEvidence,
        element_index: usize,
    ) -> Vec<PersonalLimitConflict> {
        let mut conflicts = vec![];
        let mut jump = |j: &ListedJump, component_index| {
            if let (Some(kind), Some(requested)) = (j.kind, j.rotations) {
                if let Some(maximum) = self
                    .jump_rotations
                    .iter()
                    .filter(|l| l.kind == kind)
                    .map(|l| l.maximum)
                    .min()
                {
                    if requested > maximum as u8 {
                        conflicts.push(PersonalLimitConflict {
                            element_index,
                            component_index,
                            kind: PersonalLimitConflictKind::JumpRotation {
                                kind,
                                requested,
                                maximum,
                            },
                        });
                    }
                }
            }
        };
        match element {
            PlannedElement::Jump(JumpElement::Solo(j)) => jump(j, None),
            PlannedElement::Jump(
                JumpElement::Combination(parts) | JumpElement::Sequence(parts),
            ) => {
                for (i, part) in parts.iter().enumerate() {
                    if let JumpPart::Jump(j) = part {
                        jump(j, Some(i));
                    }
                }
            }
            PlannedElement::RegularSpin(_) => {
                if let ElementPlanEvidence::Spin { spin, .. } = evidence {
                    if let (Some(abbreviation), Some(requested)) =
                        (spin.abbreviation, spin.planned_level)
                    {
                        if let Some(maximum) = self
                            .spin_levels
                            .iter()
                            .filter(|l| l.abbreviation == abbreviation)
                            .map(|l| l.maximum)
                            .min_by_key(|l| l.number())
                        {
                            if requested.number() > maximum.number() {
                                conflicts.push(PersonalLimitConflict {
                                    element_index,
                                    component_index: None,
                                    kind: PersonalLimitConflictKind::SpinLevel {
                                        abbreviation,
                                        requested,
                                        maximum,
                                    },
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        conflicts
    }
}
