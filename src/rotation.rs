//! Personal connection preferences, independent of recognition and competition rules.
use crate::{JumpElement, JumpKind, JumpPart, PlannedElement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationPreference {
    #[default]
    SameDirection,
    AllowMixedDirections,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationPreferenceConflict {
    pub element_index: usize,
    /// Sorted original JumpPart positions, including an intervening Euler.
    pub component_indices: Vec<usize>,
}

pub(crate) fn conflict(
    element: &PlannedElement,
    element_index: usize,
    preference: RotationPreference,
) -> Option<RotationPreferenceConflict> {
    if preference == RotationPreference::AllowMixedDirections {
        return None;
    }
    let (parts, sequence) = match element {
        PlannedElement::Jump(JumpElement::Combination(parts)) => (parts, false),
        PlannedElement::Jump(JumpElement::Sequence(parts)) => (parts, true),
        _ => return None,
    };
    // Missing kinds and unmodeled footwork retain their existing evidence diagnostics.
    if parts.iter().any(|p| match p {
        JumpPart::Jump(j) => j.kind.is_none(),
        JumpPart::Euler => false,
        _ => true,
    }) {
        return None;
    }
    let listed: Vec<_> = parts
        .iter()
        .enumerate()
        .filter_map(|(i, part)| {
            if let JumpPart::Jump(jump) = part {
                Some((i, jump.kind.unwrap()))
            } else {
                None
            }
        })
        .collect();
    let mut indices = std::collections::BTreeSet::new();
    for pair in listed.windows(2) {
        let (previous, _) = pair[0];
        let (next, kind) = pair[1];
        let gap = &parts[previous + 1..next];
        // Rule 610 and TP Handbook 2026/27 p.28: normal BO landings, Euler BI
        // landings, same-foot turn to Axel, or a direct sequence step into Axel.
        let compatible = match gap {
            [] => {
                matches!(kind, JumpKind::ToeLoop | JumpKind::Loop)
                    || (sequence && matches!(kind, JumpKind::Axel | JumpKind::Waltz))
            }
            [JumpPart::Euler] => {
                matches!(
                    kind,
                    JumpKind::Salchow | JumpKind::Flip | JumpKind::Axel | JumpKind::Waltz
                )
            }
            _ => true, // Malformed Euler placement belongs to structure validation.
        };
        if !compatible {
            indices.extend(previous..=next);
        }
    }
    (!indices.is_empty()).then(|| RotationPreferenceConflict {
        element_index,
        component_indices: indices.into_iter().collect(),
    })
}

pub(crate) fn conflicts<'a>(
    elements: impl Iterator<Item = &'a PlannedElement>,
    preference: RotationPreference,
) -> Vec<RotationPreferenceConflict> {
    elements
        .enumerate()
        .filter_map(|(i, e)| conflict(e, i, preference))
        .collect()
}
