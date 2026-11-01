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
    /// Zero-based index in the combination or sequence parts, including transitions.
    pub component_index: Option<usize>,
    pub explanation: &'static str,
    pub source: SourceReference,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub coverage: Coverage,
    pub findings: Vec<Finding>,
}

fn finding(
    kind: FindingKind,
    element_index: Option<usize>,
    component_index: Option<usize>,
    explanation: &'static str,
) -> Finding {
    Finding {
        kind,
        rule_id: if kind == FindingKind::Unsupported {
            "validator-scope"
        } else {
            RULE
        },
        element_index,
        component_index,
        explanation,
        source: if kind == FindingKind::Unsupported {
            SourceReference {
                document: "Skating Program Lab architecture".into(),
                version: "issue-13".into(),
                location: "Current state, validator scope".into(),
            }
        } else {
            SourceReference {
                document: "International Adult Figure Skating Competitions Technical Announcement"
                    .into(),
                version: "2026-07-16".into(),
                location: "p. 11, Bronze Free Skating, jumps".into(),
            }
        },
    }
}

/// Checks Bronze jump eligibility and element count; an empty list does not establish compliance.
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
            None,
            "Only Bronze in the 2026/27 Adult ruleset is supported.",
        ));
        return result;
    }
    let mut jump_count = 0;
    for (index, element) in program.elements.iter().enumerate() {
        if let PlannedElement::Jump(jump) = element {
            jump_count += 1;
            match jump {
                JumpElement::Solo(jump) => check_jump(jump, index, None, &mut result.findings),
                JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                    for (component_index, part) in parts.iter().enumerate() {
                        if let JumpPart::Jump(jump) = part {
                            check_jump(jump, index, Some(component_index), &mut result.findings);
                        }
                    }
                }
            }
        }
    }
    if jump_count > 4 {
        let mut count_finding = finding(
            FindingKind::Violation,
            None,
            None,
            "Bronze permits at most four jump elements.",
        );
        count_finding.rule_id = "isu-adult-2026-27-bronze-jump-element-count";
        result.findings.push(count_finding);
    }
    result
}

fn check_jump(
    jump: &ListedJump,
    index: usize,
    component_index: Option<usize>,
    findings: &mut Vec<Finding>,
) {
    let index = Some(index);
    if jump.kind.is_none() || jump.rotations.is_none() || jump.rotations == Some(0) {
        findings.push(finding(
            FindingKind::Unknown,
            index,
            component_index,
            "Jump kind or a positive rotation count is missing.",
        ));
    }
    if jump.kind == Some(JumpKind::Axel) {
        findings.push(finding(
            FindingKind::Violation,
            index,
            component_index,
            "Axel jumps are prohibited in Bronze.",
        ));
    }
    if jump.rotations.is_some_and(|count| count >= 2) {
        findings.push(finding(
            FindingKind::Violation,
            index,
            component_index,
            "Multi-rotation jumps are prohibited in Bronze.",
        ));
    }
}
