//! Partial, source-backed validation of intended programs.
use crate::{
    AdultCategory, JumpElement, JumpKind, JumpPart, ListedJump, PlannedElement, Program, RulesetId,
    Season, SourceReference,
};

const ELIGIBILITY_RULE: &str = "isu-adult-2026-27-bronze-jump-eligibility";
const COUNT_RULE: &str = "isu-adult-2026-27-bronze-jump-element-count";
const COMBINATION_COUNT_RULE: &str = "isu-adult-2026-27-bronze-combination-count";
const STRUCTURE_RULE: &str = "isu-adult-2026-27-bronze-combination-structure";
const EULER_RULE: &str = "isu-adult-2026-27-bronze-combination-euler";
const REPETITION_RULE: &str = "isu-adult-2026-27-bronze-jump-repetition";
const SCOPE_RULE: &str = "validator-scope";

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
    rule_id: &'static str,
    element_index: Option<usize>,
    component_index: Option<usize>,
    explanation: &'static str,
) -> Finding {
    Finding {
        kind,
        rule_id,
        element_index,
        component_index,
        explanation,
        source: if rule_id == SCOPE_RULE {
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

/// Checks selected Bronze jump rules; an empty list does not establish compliance.
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
            SCOPE_RULE,
            None,
            None,
            "Only Bronze in the 2026/27 Adult ruleset is supported.",
        ));
        return result;
    }
    let mut jump_count = 0;
    let mut combination_count = 0;
    let mut occurrences: Vec<(JumpKind, u8)> = Vec::new();
    let mut repetition_unknown = false;
    let mut unspecified_element = false;
    for (index, element) in program.elements.iter().enumerate() {
        if matches!(element, PlannedElement::Unspecified) {
            unspecified_element = true;
            repetition_unknown = true;
        }
        if let PlannedElement::Jump(jump) = element {
            jump_count += 1;
            match jump {
                JumpElement::Solo(jump) => {
                    check_listed_jump(
                        jump,
                        index,
                        None,
                        &mut occurrences,
                        &mut repetition_unknown,
                        &mut result.findings,
                    );
                }
                JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                    if matches!(jump, JumpElement::Combination(_)) {
                        combination_count += 1;
                        if check_combination(parts, index, &mut result.findings) {
                            repetition_unknown = true;
                        }
                    } else {
                        result.findings.push(finding(
                            FindingKind::Violation,
                            STRUCTURE_RULE,
                            Some(index),
                            None,
                            "Bronze does not allow jump sequences.",
                        ));
                    }
                    for (component_index, part) in parts.iter().enumerate() {
                        if let JumpPart::Jump(jump) = part {
                            check_listed_jump(
                                jump,
                                index,
                                Some(component_index),
                                &mut occurrences,
                                &mut repetition_unknown,
                                &mut result.findings,
                            );
                        }
                    }
                }
            }
        }
    }
    if combination_count > 1 {
        result.findings.push(finding(
            FindingKind::Violation,
            COMBINATION_COUNT_RULE,
            None,
            None,
            "Bronze permits at most one jump combination.",
        ));
    } else if unspecified_element {
        result.findings.push(finding(
            FindingKind::Unknown,
            COMBINATION_COUNT_RULE,
            None,
            None,
            "An unspecified element may be a jump combination.",
        ));
    }
    if repetition_unknown {
        result.findings.push(finding(
            FindingKind::Unknown,
            REPETITION_RULE,
            None,
            None,
            "Incomplete jump or element details prevent a definitive repetition check.",
        ));
    }
    if jump_count > 4 {
        result.findings.push(finding(
            FindingKind::Violation,
            COUNT_RULE,
            None,
            None,
            "Bronze permits at most four jump elements.",
        ));
    }
    result
}

fn check_combination(parts: &[JumpPart], index: usize, findings: &mut Vec<Finding>) -> bool {
    let count = parts
        .iter()
        .filter(|part| matches!(part, JumpPart::Jump(_)))
        .count();
    if count > 2 {
        findings.push(finding(
            FindingKind::Violation,
            STRUCTURE_RULE,
            Some(index),
            None,
            "A Bronze combination permits at most two listed jumps.",
        ));
    } else if count < 2 {
        findings.push(finding(
            FindingKind::Unknown,
            STRUCTURE_RULE,
            Some(index),
            None,
            "Combination jump details are incomplete.",
        ));
    }
    for (component_index, part) in parts.iter().enumerate() {
        match part {
            JumpPart::Euler => findings.push(finding(
                FindingKind::Violation,
                EULER_RULE,
                Some(index),
                Some(component_index),
                "Euler is prohibited in a Bronze combination.",
            )),
            JumpPart::UnspecifiedTransition => findings.push(finding(
                FindingKind::Unknown,
                EULER_RULE,
                Some(index),
                Some(component_index),
                "The unspecified transition may be an Euler.",
            )),
            _ => {}
        }
    }
    count < 2
}

fn check_listed_jump(
    jump: &ListedJump,
    index: usize,
    component_index: Option<usize>,
    occurrences: &mut Vec<(JumpKind, u8)>,
    unknown: &mut bool,
    findings: &mut Vec<Finding>,
) {
    check_jump(jump, index, component_index, findings);
    check_repetition(jump, index, component_index, occurrences, unknown, findings);
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
            ELIGIBILITY_RULE,
            index,
            component_index,
            "Jump kind or a positive rotation count is missing.",
        ));
    }
    if jump.kind == Some(JumpKind::Axel) {
        findings.push(finding(
            FindingKind::Violation,
            ELIGIBILITY_RULE,
            index,
            component_index,
            "Axel jumps are prohibited in Bronze.",
        ));
    }
    if jump.rotations.is_some_and(|count| count >= 2) {
        findings.push(finding(
            FindingKind::Violation,
            ELIGIBILITY_RULE,
            index,
            component_index,
            "Multi-rotation jumps are prohibited in Bronze.",
        ));
    }
}

fn check_repetition(
    jump: &ListedJump,
    index: usize,
    component_index: Option<usize>,
    occurrences: &mut Vec<(JumpKind, u8)>,
    unknown: &mut bool,
    findings: &mut Vec<Finding>,
) {
    if let (Some(kind), Some(rotations @ 1..)) = (jump.kind, jump.rotations) {
        occurrences.push((kind, rotations));
        if occurrences
            .iter()
            .filter(|&&entry| entry == (kind, rotations))
            .count()
            > 2
        {
            findings.push(finding(
                FindingKind::Violation,
                REPETITION_RULE,
                Some(index),
                component_index,
                "A listed jump may occur at most twice in the program.",
            ));
        }
    } else {
        *unknown = true;
    }
}
