//! Partial, source-backed validation of intended programs.
use crate::{
    AdultCategory, JumpElement, JumpKind, JumpPart, ListedJump, PlannedElement, Program, RulesetId,
    Season, SourceReference,
};

pub(crate) const ELIGIBILITY_RULE: &str = "isu-adult-2026-27-bronze-jump-eligibility";
pub(crate) const COUNT_RULE: &str = "isu-adult-2026-27-bronze-jump-element-count";
pub(crate) const COMBINATION_COUNT_RULE: &str = "isu-adult-2026-27-bronze-combination-count";
pub(crate) const STRUCTURE_RULE: &str = "isu-adult-2026-27-bronze-combination-structure";
pub(crate) const EULER_RULE: &str = "isu-adult-2026-27-bronze-combination-euler";
pub(crate) const REPETITION_RULE: &str = "isu-adult-2026-27-bronze-jump-repetition";
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

pub(crate) fn bronze_jump_source() -> SourceReference {
    SourceReference {
        document: "International Adult Figure Skating Competitions Technical Announcement".into(),
        version: "2026-07-16".into(),
        location: "p. 11, Bronze Free Skating, jumps".into(),
    }
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
            bronze_jump_source()
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
    if program.category != AdultCategory::Bronze {
        return validate_other_categories(program);
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
    for (i, pair) in parts.windows(2).enumerate() {
        if let [JumpPart::Jump(_), JumpPart::Jump(next)] = pair {
            if !matches!(next.kind, Some(JumpKind::ToeLoop | JumpKind::Loop)) {
                findings.push(finding(
                    FindingKind::Unknown,
                    STRUCTURE_RULE,
                    Some(index),
                    Some(i + 1),
                    "The direct transition is outside the verified combination subset.",
                ));
            }
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

fn validate_other_categories(program: &Program) -> ValidationResult {
    let category = program.category;
    let (page, name, max_jumps) = match category {
        AdultCategory::Silver => (10, "Silver", 4),
        AdultCategory::Gold => (9, "Gold", 5),
        AdultCategory::Masters => (8, "Masters", 5),
        AdultCategory::Bronze => unreachable!(),
    };
    let mut findings = Vec::new();
    let mut occurrences = Vec::new();
    let mut repetition_findings = Vec::new();
    let mut repetition_unknown = false;
    let mut count = 0;
    let mut compounds: Vec<(usize, bool, usize, bool)> = Vec::new();
    let mut eulers = 0;
    let mut axel = false;
    let mut unknown_axel = false;
    let mut unspecified = false;
    let mut add = |kind, rule, index, component, explanation| {
        findings.push(finding(kind, rule, index, component, explanation));
    };
    for (index, element) in program.elements.iter().enumerate() {
        if matches!(element, PlannedElement::Unspecified) {
            unspecified = true;
            repetition_unknown = true;
            unknown_axel = true;
        }
        let PlannedElement::Jump(jump) = element else {
            continue;
        };
        count += 1;
        let parts: Vec<(Option<usize>, &ListedJump)> = match jump {
            JumpElement::Solo(j) => vec![(None, j)],
            JumpElement::Combination(parts) | JumpElement::Sequence(parts) => {
                let sequence = matches!(jump, JumpElement::Sequence(_));
                let listed: Vec<_> = parts
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| match p {
                        JumpPart::Jump(j) => Some((Some(i), j)),
                        _ => None,
                    })
                    .collect();
                let has_euler = parts.iter().any(|p| matches!(p, JumpPart::Euler));
                compounds.push((index, sequence, listed.len(), has_euler));
                if listed.len() < 2 {
                    repetition_unknown = true;
                    unknown_axel = true;
                    add(
                        FindingKind::Unknown,
                        STRUCTURE_RULE,
                        Some(index),
                        None,
                        "Compound jump details are incomplete.",
                    );
                } else if listed.len() > 3 {
                    add(
                        FindingKind::Violation,
                        STRUCTURE_RULE,
                        Some(index),
                        None,
                        "A combination or sequence permits at most three listed jumps.",
                    );
                }
                for (i, part) in parts.iter().enumerate() {
                    match part {
                        JumpPart::Euler => {
                            eulers += 1;
                            if i == 0
                                || i + 1 == parts.len()
                                || !matches!(parts.get(i.wrapping_sub(1)), Some(JumpPart::Jump(_)))
                                || !matches!(parts.get(i + 1), Some(JumpPart::Jump(_)))
                            {
                                add(
                                    FindingKind::Violation,
                                    EULER_RULE,
                                    Some(index),
                                    Some(i),
                                    "Euler must be between listed jumps.",
                                );
                            } else {
                                add(FindingKind::Unknown, EULER_RULE, Some(index), Some(i), "The plan does not establish the foot transition after Euler or the panel consequence.");
                            }
                        }
                        JumpPart::UnspecifiedTransition | JumpPart::OtherTransition(_) => add(
                            FindingKind::Unknown,
                            STRUCTURE_RULE,
                            Some(index),
                            Some(i),
                            "The transition and its recognition are not established.",
                        ),
                        _ => {}
                    }
                }
                if sequence && listed.len() >= 2 {
                    let later = &listed[1..];
                    if !later.iter().any(|(_, j)| j.kind == Some(JumpKind::Axel)) {
                        let unknown = later.iter().any(|(_, j)| j.kind.is_none());
                        add(if unknown { FindingKind::Unknown } else { FindingKind::Violation }, STRUCTURE_RULE, Some(index), None, "A sequence requires an Axel-type second or third jump with a direct step.");
                    } else {
                        add(FindingKind::Unknown, STRUCTURE_RULE, Some(index), None, "The plan does not establish the direct step into the Axel or panel recognition.");
                    }
                }
                if !sequence && listed.len() >= 2 {
                    for pair in listed.windows(2) {
                        let (Some(a), Some(b)) = (pair[0].0, pair[1].0) else {
                            unreachable!()
                        };
                        if b == a + 1 {
                            match pair[1].1.kind {
                                Some(JumpKind::ToeLoop | JumpKind::Loop) => {},
                                _ => add(FindingKind::Unknown, STRUCTURE_RULE, Some(index), Some(b), "The direct transition is outside the verified combination subset."),
                            }
                        }
                    }
                }
                listed
            }
        };
        for (component, jump) in parts {
            if jump.kind == Some(JumpKind::Axel) && jump.rotations.is_some_and(|n| n > 0) {
                axel = true;
            }
            if jump.kind.is_none() || jump.rotations.is_none_or(|n| n == 0) {
                unknown_axel = true;
                add(
                    FindingKind::Unknown,
                    ELIGIBILITY_RULE,
                    Some(index),
                    component,
                    "Jump kind or a positive rotation count is missing.",
                );
            }
            let forbidden = match (category, jump.kind, jump.rotations) {
                (_, _, Some(n)) if n >= 3 => true,
                (AdultCategory::Silver, _, Some(2)) => true,
                (
                    AdultCategory::Gold,
                    Some(JumpKind::Flip | JumpKind::Lutz | JumpKind::Axel),
                    Some(2),
                ) => true,
                _ => false,
            };
            if forbidden {
                add(
                    FindingKind::Violation,
                    ELIGIBILITY_RULE,
                    Some(index),
                    component,
                    "This jump is prohibited in the selected category.",
                );
            }
            check_repetition(
                jump,
                index,
                component,
                &mut occurrences,
                &mut repetition_unknown,
                &mut repetition_findings,
            );
        }
    }
    if count > max_jumps {
        add(
            FindingKind::Violation,
            COUNT_RULE,
            None,
            None,
            "The program exceeds the category's jump element limit.",
        );
    } else if unspecified {
        add(
            FindingKind::Unknown,
            COUNT_RULE,
            None,
            None,
            "An unspecified element may exceed the jump element limit.",
        );
    }
    let sequences = compounds
        .iter()
        .filter(|(_, sequence, _, _)| *sequence)
        .count();
    let combinations = compounds.len() - sequences;
    if combinations > 2 || sequences > 1 || compounds.len() > 2 {
        add(
            FindingKind::Violation,
            COMBINATION_COUNT_RULE,
            None,
            None,
            "At most two combinations, or one combination and one sequence, are permitted.",
        );
    } else if unspecified {
        add(
            FindingKind::Unknown,
            COMBINATION_COUNT_RULE,
            None,
            None,
            "An unspecified element may exceed the combination or sequence limit.",
        );
    }
    if compounds.iter().filter(|(_, _, n, _)| *n == 3).count() > 1 {
        add(
            FindingKind::Violation,
            STRUCTURE_RULE,
            None,
            None,
            "Only one combination or sequence may have three listed jumps.",
        );
    }
    if eulers > 1 {
        add(
            FindingKind::Violation,
            EULER_RULE,
            None,
            None,
            "Euler may be executed only once per program.",
        );
    }
    if sequences > 0 {
        for (index, sequence, _, euler) in &compounds {
            if !sequence && *euler {
                add(
                    FindingKind::Violation,
                    EULER_RULE,
                    Some(*index),
                    None,
                    "A combination performed with a sequence must not use Euler.",
                );
            }
        }
    }
    if repetition_unknown {
        add(
            FindingKind::Unknown,
            REPETITION_RULE,
            None,
            None,
            "Incomplete jump or element details prevent a definitive repetition check.",
        );
    }
    if category == AdultCategory::Masters && !axel {
        add(
            if unknown_axel {
                FindingKind::Unknown
            } else {
                FindingKind::Violation
            },
            "isu-adult-2026-27-masters-axel-required",
            None,
            None,
            "Masters requires an Axel-type jump.",
        );
    }
    findings.extend(repetition_findings);
    for item in &mut findings {
        item.source.location = format!("p. {page}, {name} Free Skating, jumps");
        item.rule_id = match (category, item.rule_id) {
            (AdultCategory::Silver, ELIGIBILITY_RULE) => {
                "isu-adult-2026-27-silver-jump-eligibility"
            }
            (AdultCategory::Gold, ELIGIBILITY_RULE) => "isu-adult-2026-27-gold-jump-eligibility",
            (AdultCategory::Masters, ELIGIBILITY_RULE) => {
                "isu-adult-2026-27-masters-jump-eligibility"
            }
            (_, COUNT_RULE) => "isu-adult-2026-27-jump-element-count",
            (_, COMBINATION_COUNT_RULE) => "isu-adult-2026-27-combination-count",
            (_, STRUCTURE_RULE) => "isu-adult-2026-27-combination-structure",
            (_, EULER_RULE) => "isu-adult-2026-27-combination-euler",
            (_, REPETITION_RULE) => "isu-adult-2026-27-jump-repetition",
            (_, other) => other,
        };
    }
    ValidationResult {
        coverage: Coverage::Partial,
        findings,
    }
}
