//! Partial, source-backed validation of intended programs.
use crate::{
    AdultCategory, JumpElement, JumpKind, JumpPart, ListedJump, PlannedElement, Program, RulesetId,
    Season, SourceReference,
};

const RULE: &str = "isu-adult-2026-27-bronze-jump-eligibility";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    Violation,
    Unknown,
    Unsupported,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Partial,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    pub rule_id: &'static str,
    /// Zero-based program element index; None denotes program-level scope.
    pub element_index: Option<usize>,
    pub explanation: &'static str,
    pub source: SourceReference,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub coverage: Coverage,
    pub findings: Vec<Finding>,
}

fn finding(kind: FindingKind, element_index: Option<usize>, explanation: &'static str) -> Finding {
    Finding {
        kind,
        rule_id: RULE,
        element_index,
        explanation,
        source: SourceReference {
            document: "International Adult Figure Skating Competitions Technical Announcement"
                .into(),
            version: "2026-07-16".into(),
            location: "p. 11, Bronze Free Skating, jumps".into(),
        },
    }
}

/// Checks Bronze jump eligibility only; an empty finding list does not establish compliance.
pub fn validate_program(program: &Program) -> ValidationResult {
    let mut result = ValidationResult {
        coverage: Coverage::Partial,
        findings: Vec::new(),
    };
    if program.ruleset != RulesetId::IsuAdult2026_27July16
        || program.category != AdultCategory::Bronze
        || program.season
            != (Season {
                start_year: 2026,
                end_year: 2027,
            })
    {
        result.findings.push(finding(
            FindingKind::Unsupported,
            None,
            "Only Bronze in the 2026/27 Adult ruleset is supported.",
        ));
        return result;
    }
    for (index, element) in program.elements.iter().enumerate() {
        if let PlannedElement::Jump(jump) = element {
            match jump {
                JumpElement::Solo(jump) => check_jump(jump, index, &mut result.findings),
                JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                    for part in parts {
                        if let JumpPart::Jump(jump) = part {
                            check_jump(jump, index, &mut result.findings);
                        }
                    }
                }
            }
        }
    }
    result
}

fn check_jump(jump: &ListedJump, index: usize, findings: &mut Vec<Finding>) {
    let index = Some(index);
    if jump.kind.is_none() || jump.rotations.is_none() || jump.rotations == Some(0) {
        findings.push(finding(
            FindingKind::Unknown,
            index,
            "Jump kind or a positive rotation count is missing.",
        ));
    }
    if jump.rotations.is_some_and(|count| count >= 2)
        || (jump.kind == Some(JumpKind::Axel) && jump.rotations == Some(1))
    {
        findings.push(finding(
            FindingKind::Violation,
            index,
            "Bronze permits only single jumps other than Axel.",
        ));
    }
}
