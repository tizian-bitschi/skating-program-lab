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
    category_source(AdultCategory::Bronze)
}
pub(crate) fn category_source(category: AdultCategory) -> SourceReference {
    crate::adult_profile(category)
        .binding(crate::ProfileRule::Eligibility)
        .source
        .clone()
}
pub(crate) fn adult_bindings(
    category: AdultCategory,
    category_source: &SourceReference,
) -> Vec<crate::RuleBinding> {
    [
        Rule::Input,
        Rule::Eligibility,
        Rule::Count,
        Rule::CompoundCount,
        Rule::Structure,
        Rule::Euler,
        Rule::Repetition,
        Rule::Axel,
        Rule::Transition,
    ]
    .into_iter()
    .map(|rule| crate::RuleBinding {
        rule: rule.key(),
        id: rule.id(category),
        mechanism: crate::RuleMechanism::JumpPlanV1,
        source: rule.source(category_source),
    })
    .collect()
}
#[derive(Clone, Copy)]
enum Rule {
    Input,
    Eligibility,
    Count,
    CompoundCount,
    Structure,
    Euler,
    Repetition,
    Axel,
    Transition,
}
impl Rule {
    fn key(self) -> crate::ProfileRule {
        match self {
            Self::Input => crate::ProfileRule::Input,
            Self::Eligibility => crate::ProfileRule::Eligibility,
            Self::Count => crate::ProfileRule::Count,
            Self::CompoundCount => crate::ProfileRule::CompoundCount,
            Self::Structure => crate::ProfileRule::Structure,
            Self::Euler => crate::ProfileRule::Euler,
            Self::Repetition => crate::ProfileRule::Repetition,
            Self::Axel => crate::ProfileRule::Axel,
            Self::Transition => crate::ProfileRule::Transition,
        }
    }

    fn id(self, category: AdultCategory) -> &'static str {
        match (self, category) {
            (Self::Input, _) => "listed-jump-invalid-rotations",
            (Self::Eligibility, AdultCategory::Bronze) => ELIGIBILITY_RULE,
            (Self::Eligibility, AdultCategory::Silver) => {
                "isu-adult-2026-27-silver-jump-eligibility"
            }
            (Self::Eligibility, AdultCategory::Gold) => "isu-adult-2026-27-gold-jump-eligibility",
            (Self::Eligibility, AdultCategory::Masters) => {
                "isu-adult-2026-27-masters-jump-eligibility"
            }
            (Self::Count, AdultCategory::Bronze) => COUNT_RULE,
            (Self::CompoundCount, AdultCategory::Bronze) => COMBINATION_COUNT_RULE,
            (Self::Structure, AdultCategory::Bronze) => STRUCTURE_RULE,
            (Self::Euler, AdultCategory::Bronze) => EULER_RULE,
            (Self::Repetition, AdultCategory::Bronze) => REPETITION_RULE,
            (Self::Count, _) => "isu-adult-2026-27-jump-element-count",
            (Self::CompoundCount, _) => "isu-adult-2026-27-combination-count",
            (Self::Structure, _) => "isu-adult-2026-27-combination-structure",
            (Self::Euler, _) => "isu-adult-2026-27-combination-euler",
            (Self::Repetition, _) => "isu-adult-2026-27-jump-repetition",
            (Self::Axel, _) => "isu-adult-2026-27-masters-axel-required",
            (Self::Transition, _) => "isu-sports-2026-jump-transition",
        }
    }
    fn source(self, category_source: &SourceReference) -> SourceReference {
        if matches!(self, Self::Input) {
            SourceReference {
                document: "Skating Program Lab input contract".into(),
                version: "issue-39".into(),
                location: "ListedJump, rotation count".into(),
            }
        } else if matches!(self, Self::Transition) {
            SourceReference {
                document: "ISU Sports Rules — Figure Skating".into(),
                version: "2026-06-12".into(),
                location: "p. 65, Rule 610, jump combinations".into(),
            }
        } else {
            category_source.clone()
        }
    }
}
#[derive(Clone, Copy)]
struct Compound {
    index: usize,
    sequence: bool,
    length: usize,
    euler: bool,
}
struct Validator<'a> {
    profile: &'a crate::RuleProfile,
    policy: &'a crate::JumpPolicy,
    findings: Vec<Finding>,
    occurrences: Vec<(JumpKind, u8)>,
    compounds: Vec<Compound>,
    jump_count: usize,
    eulers: usize,
    repetition_unknown: bool,
    unspecified: usize,
    axel: bool,
    axel_unknown: bool,
}
impl<'a> Validator<'a> {
    fn new(profile: &'a crate::RuleProfile) -> Self {
        Self {
            profile,
            policy: &profile.jumps,
            findings: vec![],
            occurrences: vec![],
            compounds: vec![],
            jump_count: 0,
            eulers: 0,
            repetition_unknown: false,
            unspecified: 0,
            axel: false,
            axel_unknown: false,
        }
    }
    fn bronze_diagnostics(&self) -> bool {
        self.policy.bronze_diagnostics
            && self.profile.provenance == crate::ProfileProvenance::Official
    }
    fn add(
        &mut self,
        kind: FindingKind,
        rule: Rule,
        element_index: Option<usize>,
        component_index: Option<usize>,
        explanation: &'static str,
    ) {
        self.findings.push(Finding {
            kind,
            rule_id: self.profile.binding(rule.key()).id,
            element_index,
            component_index,
            explanation,
            source: self.profile.binding(rule.key()).source.clone(),
        });
    }
    fn jump(&mut self, jump: &ListedJump, index: usize, component: Option<usize>) {
        if jump.kind.is_none() || jump.rotations.is_none() {
            self.add(
                FindingKind::Unknown,
                Rule::Eligibility,
                Some(index),
                component,
                "Jump kind or a positive rotation count is missing.",
            );
        }
        if jump.rotations == Some(0) {
            self.add(
                FindingKind::Violation,
                Rule::Input,
                Some(index),
                component,
                "A listed jump must have a positive rotation count.",
            );
        }
        if self.policy.prohibited(jump) {
            let text = if self.bronze_diagnostics() && jump.kind == Some(JumpKind::Axel) {
                "Axel jumps are prohibited in Bronze."
            } else if self.bronze_diagnostics() {
                "Multi-rotation jumps are prohibited in Bronze."
            } else {
                "This jump is prohibited in the selected profile."
            };
            self.add(
                FindingKind::Violation,
                Rule::Eligibility,
                Some(index),
                component,
                text,
            );
        }
        if jump
            .kind
            .is_some_and(|kind| !self.policy.allowed.iter().any(|p| p.kind == kind))
            && jump.rotations.is_some_and(|n| {
                n > 0
                    && !self
                        .policy
                        .allowed
                        .iter()
                        .any(|p| (p.min_rotations..=p.max_rotations).contains(&n))
            })
        {
            self.add(
                FindingKind::Violation,
                Rule::Eligibility,
                Some(index),
                component,
                if self.bronze_diagnostics() {
                    "Multi-rotation jumps are prohibited in Bronze."
                } else {
                    "The rotation count exceeds the selected profile permissions."
                },
            );
        }
        match (jump.kind, jump.rotations) {
            (Some(JumpKind::Axel), Some(1..)) => self.axel = true,
            (_, Some(0)) => {}
            (None, _) | (Some(JumpKind::Axel), None) => self.axel_unknown = true,
            _ => {}
        }
        if let (Some(kind), Some(rotations @ 1..)) = (jump.kind, jump.rotations) {
            self.occurrences.push((kind, rotations));
            if self
                .occurrences
                .iter()
                .filter(|&&entry| entry == (kind, rotations))
                .count()
                > self.policy.max_repetitions
            {
                self.add(
                    FindingKind::Violation,
                    Rule::Repetition,
                    Some(index),
                    component,
                    if self.policy.max_repetitions == 2 {
                        "A listed jump may occur at most twice in the program."
                    } else {
                        "A listed jump exceeds the selected profile repetition limit."
                    },
                );
            }
        } else {
            self.repetition_unknown = true;
        }
    }
    fn transition(&mut self, parts: &[JumpPart], index: usize, sequence: bool) {
        for (i, part) in parts.iter().enumerate() {
            match part {
                JumpPart::Euler => {
                    self.eulers += 1;
                    if self.policy.max_eulers == 0 {
                        self.add(
                            FindingKind::Violation,
                            Rule::Euler,
                            Some(index),
                            Some(i),
                            if self.bronze_diagnostics() {
                                "Euler is prohibited in a Bronze combination."
                            } else {
                                "The selected profile prohibits Euler."
                            },
                        );
                    } else if !parts[..i].iter().any(|p| matches!(p, JumpPart::Jump(_)))
                        || !parts[i + 1..]
                            .iter()
                            .any(|p| matches!(p, JumpPart::Jump(_)))
                    {
                        self.add(
                            FindingKind::Violation,
                            Rule::Euler,
                            Some(index),
                            Some(i),
                            "Euler must be between listed jumps.",
                        );
                    } else {
                        self.add(FindingKind::Unknown, Rule::Euler, Some(index), Some(i), "The plan does not establish the foot transition after Euler or the panel consequence.");
                    }
                }
                JumpPart::UnspecifiedTransition => {
                    self.add(
                        FindingKind::Unknown,
                        Rule::Euler,
                        Some(index),
                        Some(i),
                        "The unspecified transition may be an Euler.",
                    );
                    self.add(
                        FindingKind::Unknown,
                        Rule::Transition,
                        Some(index),
                        Some(i),
                        "The transition and its recognition are not established.",
                    );
                }
                JumpPart::OtherTransition(_) => self.add(
                    FindingKind::Unknown,
                    Rule::Transition,
                    Some(index),
                    Some(i),
                    "The transition and its recognition are not established.",
                ),
                _ => {}
            }
        }
        if !sequence {
            for (i, pair) in parts.windows(2).enumerate() {
                if let [JumpPart::Jump(_), JumpPart::Jump(next)] = pair {
                    if !matches!(next.kind, Some(JumpKind::ToeLoop | JumpKind::Loop)) {
                        self.add(
                            FindingKind::Unknown,
                            Rule::Transition,
                            Some(index),
                            Some(i + 1),
                            "The direct transition is outside the verified combination subset.",
                        );
                    }
                }
            }
        }
    }
    fn compound(&mut self, parts: &[JumpPart], index: usize, sequence: bool) {
        let listed: Vec<_> = parts
            .iter()
            .filter_map(|part| match part {
                JumpPart::Jump(j) => Some(j),
                _ => None,
            })
            .collect();
        if !sequence || self.policy.max_sequences > 0 {
            self.compounds.push(Compound {
                index,
                sequence,
                length: listed.len(),
                euler: parts.iter().any(|p| matches!(p, JumpPart::Euler)),
            });
        }
        if sequence && self.policy.max_sequences == 0 {
            self.add(
                FindingKind::Violation,
                Rule::Structure,
                Some(index),
                None,
                if self.bronze_diagnostics() {
                    "Bronze does not allow jump sequences."
                } else {
                    "The selected profile prohibits jump sequences."
                },
            );
        } else if listed.len() > self.policy.max_compound_length {
            self.add(
                FindingKind::Violation,
                Rule::Structure,
                Some(index),
                None,
                if self.bronze_diagnostics() {
                    "A Bronze combination permits at most two listed jumps."
                } else {
                    "The compound exceeds the selected profile listed-jump limit."
                },
            );
        } else if listed.len() < 2 {
            self.repetition_unknown = true;
            self.axel_unknown = true;
            self.add(
                FindingKind::Unknown,
                Rule::Structure,
                Some(index),
                None,
                if self.bronze_diagnostics() {
                    "Combination jump details are incomplete."
                } else {
                    "Compound jump details are incomplete."
                },
            );
        }
        if !sequence || self.policy.max_sequences > 0 {
            self.transition(parts, index, sequence);
        }
        if sequence && self.policy.max_sequences > 0 && listed.len() >= 2 {
            let later = &listed[1..];
            if !later.iter().any(|j| j.kind == Some(JumpKind::Axel)) {
                self.add(
                    if later.iter().any(|j| j.kind.is_none()) {
                        FindingKind::Unknown
                    } else {
                        FindingKind::Violation
                    },
                    Rule::Structure,
                    Some(index),
                    None,
                    "A sequence requires an Axel-type second or third jump with a direct step.",
                );
            } else {
                self.add(FindingKind::Unknown, Rule::Structure, Some(index), None, "The plan does not establish the direct step into the Axel or panel recognition.");
            }
        }
        for (i, part) in parts.iter().enumerate() {
            if let JumpPart::Jump(jump) = part {
                self.jump(jump, index, Some(i));
            }
        }
    }
    fn program_checks(&mut self) {
        let sequences = self.compounds.iter().filter(|c| c.sequence).count();
        let combinations = self.compounds.len() - sequences;
        if combinations > self.policy.max_combinations
            || sequences > self.policy.max_sequences
            || self.compounds.len() > self.policy.max_compounds
        {
            self.add(
                FindingKind::Violation,
                Rule::CompoundCount,
                None,
                None,
                if self.bronze_diagnostics() {
                    "Bronze permits at most one jump combination."
                } else {
                    "The program exceeds the selected profile combination, sequence or compound limit."
                },
            );
        } else if self.unspecified > 0
            && (combinations + self.unspecified > self.policy.max_combinations
                || self.compounds.len() + self.unspecified > self.policy.max_compounds
                || (self.policy.max_sequences > 0
                    && sequences + self.unspecified > self.policy.max_sequences))
        {
            self.add(
                FindingKind::Unknown,
                Rule::CompoundCount,
                None,
                None,
                "An unspecified element may be a jump combination.",
            );
        }
        if self.policy.max_compound_length >= 3
            && self.compounds.iter().filter(|c| c.length == 3).count()
                > self.policy.max_three_jump_compounds
        {
            self.add(
                FindingKind::Violation,
                Rule::Structure,
                None,
                None,
                "The program exceeds the selected profile three-jump compound limit.",
            );
        }
        if self.eulers > self.policy.max_eulers && self.policy.max_eulers > 0 {
            self.add(
                FindingKind::Violation,
                Rule::Euler,
                None,
                None,
                "The program exceeds the selected profile Euler limit.",
            );
        }
        let conflict_indices: Vec<_> = self
            .compounds
            .iter()
            .filter(|c| sequences > 0 && !c.sequence && c.euler && self.policy.max_sequences > 0)
            .map(|c| c.index)
            .collect();
        for index in conflict_indices {
            self.add(
                FindingKind::Violation,
                Rule::Euler,
                Some(index),
                None,
                "A combination performed with a sequence must not use Euler.",
            );
        }
        if self.repetition_unknown {
            self.add(
                FindingKind::Unknown,
                Rule::Repetition,
                None,
                None,
                "Incomplete jump or element details prevent a definitive repetition check.",
            );
        }
        if self.jump_count > self.policy.max_jumps {
            self.add(
                FindingKind::Violation,
                Rule::Count,
                None,
                None,
                if self.bronze_diagnostics() {
                    "Bronze permits at most four jump elements."
                } else {
                    "The program exceeds the selected profile jump element limit."
                },
            );
        } else if self.jump_count + self.unspecified > self.policy.max_jumps {
            self.add(
                FindingKind::Unknown,
                Rule::Count,
                None,
                None,
                "An unspecified element may exceed the jump element limit.",
            );
        }
        if self.policy.require_axel && !self.axel {
            self.add(
                if self.axel_unknown {
                    FindingKind::Unknown
                } else {
                    FindingKind::Violation
                },
                Rule::Axel,
                None,
                None,
                "The selected profile requires an Axel-type jump.",
            );
        }
    }
}

/// Checks selected Adult jump rules; an empty list does not establish compliance.
pub fn validate_program(program: &Program) -> ValidationResult {
    if program.ruleset != RulesetId::IsuAdult2026_27July16
        || program.season
            != (Season {
                start_year: 2026,
                end_year: 2027,
            })
    {
        return ValidationResult {
            coverage: Coverage::Partial,
            findings: vec![Finding {
                kind: FindingKind::Unsupported,
                rule_id: SCOPE_RULE,
                element_index: None,
                component_index: None,
                explanation: "Only the 2026/27 Adult ruleset is supported.",
                source: SourceReference {
                    document: "Skating Program Lab architecture".into(),
                    version: "issue-13".into(),
                    location: "Current state, validator scope".into(),
                },
            }],
        };
    }
    validate_elements(&program.elements, &crate::adult_profile(program.category))
}

pub(crate) fn validate_elements(
    elements: &[PlannedElement],
    profile: &crate::RuleProfile,
) -> ValidationResult {
    let mut validator = Validator::new(profile);
    for (index, element) in elements.iter().enumerate() {
        match element {
            PlannedElement::Unspecified => {
                validator.unspecified += 1;
                validator.repetition_unknown = true;
                validator.axel_unknown = true;
            }
            PlannedElement::Jump(jump) => {
                validator.jump_count += 1;
                match jump {
                    JumpElement::Solo(j) => validator.jump(j, index, None),
                    JumpElement::Combination(parts) => validator.compound(parts, index, false),
                    JumpElement::Sequence(parts) => validator.compound(parts, index, true),
                }
            }
            _ => {}
        }
    }
    for (index, element) in elements.iter().enumerate() {
        if crate::ProfileElementFamily::of(element)
            .is_some_and(|family| !profile.families.allowed.contains(&family))
        {
            validator.add(
                FindingKind::Violation,
                Rule::Eligibility,
                Some(index),
                None,
                "The selected profile prohibits this element family or variant.",
            );
        }
    }
    for required in &profile.families.required {
        if !elements
            .iter()
            .any(|e| crate::ProfileElementFamily::of(e) == Some(*required))
        {
            validator.add(
                if validator.unspecified > 0 {
                    FindingKind::Unknown
                } else {
                    FindingKind::Violation
                },
                Rule::Eligibility,
                None,
                None,
                "A required element family or variant is missing or unspecified.",
            );
        }
    }
    validator.program_checks();
    ValidationResult {
        coverage: Coverage::Partial,
        findings: validator.findings,
    }
}
