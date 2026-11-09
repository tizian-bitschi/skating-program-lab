//! Shared structural encoding of explicit intended elements. No qualification or
//! panel decision is inferred here; callers retain their own supported domains.
use crate::*;

pub(crate) fn jump_component(jump: &ListedJump) -> Option<ComponentCode> {
    let suffix = match jump.kind? {
        JumpKind::Waltz => return ComponentCode::try_from("1A").ok(),
        JumpKind::ToeLoop => "T",
        JumpKind::Salchow => "S",
        JumpKind::Loop => "Lo",
        JumpKind::Flip => "F",
        JumpKind::Lutz => "Lz",
        JumpKind::Axel => "A",
    };
    ComponentCode::try_from(format!("{}{suffix}", jump.rotations?).as_str()).ok()
}

pub(crate) fn intended_details(
    planned: &PlannedElement,
    evidence: &ElementPlanEvidence,
) -> Option<RecognitionDetails> {
    let mut level = None;
    let mut calls = vec![];
    let codes = match (planned, evidence) {
        (PlannedElement::Jump(jump), ElementPlanEvidence::Jump { .. }) => match jump {
            JumpElement::Solo(j) => {
                if j.kind == Some(JumpKind::Waltz) {
                    calls.push(RecordedCall {
                        target: CallTarget::Component(0),
                        call: PanelCall::Downgraded,
                    });
                }
                vec![jump_component(j)?]
            }
            JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                let codes: Option<Vec<_>> = parts
                    .iter()
                    .map(|p| match p {
                        JumpPart::Jump(j) => jump_component(j),
                        JumpPart::Euler => ComponentCode::try_from("Eu").ok(),
                        _ => None,
                    })
                    .collect();
                let codes = codes?;
                for (index, part) in parts.iter().enumerate() {
                    if matches!(part, JumpPart::Jump(j) if j.kind == Some(JumpKind::Waltz)) {
                        calls.push(RecordedCall {
                            target: CallTarget::Component(index),
                            call: PanelCall::Downgraded,
                        });
                    }
                }
                if matches!(jump, JumpElement::Sequence(_)) {
                    calls.push(RecordedCall {
                        target: CallTarget::BetweenComponents(codes.len()),
                        call: PanelCall::Sequence,
                    });
                }
                codes
            }
        },
        (PlannedElement::RegularSpin(_), ElementPlanEvidence::Spin { spin, .. }) => {
            let a = spin.abbreviation?;
            let name = match a.kind {
                RegularSpinKind::Camel => "CSp",
                RegularSpinKind::Sit => "SSp",
                RegularSpinKind::Upright => "USp",
                RegularSpinKind::Layback => "LSp",
                RegularSpinKind::Combination => "CoSp",
            };
            let code = format!(
                "{}{}{name}",
                if a.entry == SpinEntry::Flying {
                    "F"
                } else {
                    ""
                },
                if a.change_foot { "C" } else { "" }
            );
            level = Some(spin.planned_level?);
            vec![ComponentCode::try_from(code.as_str()).ok()?]
        }
        (PlannedElement::StepSequence, ElementPlanEvidence::Step { level: target, .. }) => {
            level = Some(*target);
            vec![ComponentCode::try_from("StSq").ok()?]
        }
        (
            PlannedElement::ChoreographicSequence { .. },
            ElementPlanEvidence::ChoreographicSequence { .. },
        ) => vec![ComponentCode::try_from("ChSq1").ok()?],
        (PlannedElement::ChoreographicSpin, ElementPlanEvidence::ChoreographicSpin { .. }) => {
            vec![ComponentCode::try_from("ChSp1").ok()?]
        }
        _ => return None,
    };
    RecognitionDetails::new(codes, level, Some(calls)).ok()
}

pub(crate) fn matches_intended(
    planned: &PlannedElement,
    evidence: &ElementPlanEvidence,
    panel: &PanelRecognition,
) -> bool {
    let (Some(intended), Some(supplied)) = (intended_details(planned, evidence), &panel.details)
    else {
        return false;
    };
    let is_sequence = |d: &RecognitionDetails| {
        d.calls()
            .is_some_and(|calls| calls.iter().any(|c| c.call == PanelCall::Sequence))
    };
    intended.components() == supplied.components()
        && intended.level() == supplied.level()
        && is_sequence(&intended) == is_sequence(supplied)
}
