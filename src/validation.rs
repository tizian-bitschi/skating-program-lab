//! Partial, source-backed validation of intended programs.
use crate::ProfileRule as Rule;
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
pub(crate) const SEQUENCE_DIRECT_STEP_RULE: &str = "program-sequence-direct-step";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    Violation,
    Unknown,
    Unsupported,
}
/// One check actually evaluated, including successful checks with no finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRule {
    pub rule_id: &'static str,
    pub element_index: Option<usize>,
    pub component_index: Option<usize>,
    pub source: SourceReference,
}

pub(crate) fn record_rule(
    trace: &mut Vec<AppliedRule>,
    rule_id: &'static str,
    element_index: Option<usize>,
    component_index: Option<usize>,
    source: &SourceReference,
) {
    let application = AppliedRule {
        rule_id,
        element_index,
        component_index,
        source: source.clone(),
    };
    if !trace.contains(&application) {
        trace.push(application);
    }
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
    scoring_source: &SourceReference,
) -> Vec<crate::RuleBinding> {
    Rule::ALL
        .into_iter()
        .map(|rule| crate::RuleBinding {
            rule,
            id: rule.id(category),
            mechanism: if rule == Rule::Scoring {
                crate::RuleMechanism::AdultScoringV1
            } else {
                crate::RuleMechanism::JumpPlanV1
            },
            source: if rule == Rule::Scoring {
                scoring_source.clone()
            } else {
                rule.source(category_source)
            },
        })
        .collect()
}
impl Rule {
    fn id(self, category: AdultCategory) -> &'static str {
        match (self, category) {
            (Self::Scoring, _) => "isu-adult-2026-27-scoring",
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
    applied_rules: Vec<AppliedRule>,
    assumed_direct_sequences: &'a [usize],
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
    fn new(profile: &'a crate::RuleProfile, assumed_direct_sequences: &'a [usize]) -> Self {
        Self {
            profile,
            applied_rules: vec![],
            assumed_direct_sequences,
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
    fn apply(&mut self, rule: Rule, index: Option<usize>, component: Option<usize>) {
        let binding = self.profile.binding(rule);
        record_rule(
            &mut self.applied_rules,
            binding.id,
            index,
            component,
            &binding.source,
        );
    }
    fn add(
        &mut self,
        kind: FindingKind,
        rule: Rule,
        element_index: Option<usize>,
        component_index: Option<usize>,
        explanation: &'static str,
    ) {
        self.apply(rule, element_index, component_index);
        self.findings.push(Finding {
            kind,
            rule_id: self.profile.binding(rule).id,
            element_index,
            component_index,
            explanation,
            source: self.profile.binding(rule).source.clone(),
        });
    }
    fn jump(&mut self, jump: &ListedJump, index: usize, component: Option<usize>) {
        self.apply(Rule::Input, Some(index), component);
        self.apply(Rule::Eligibility, Some(index), component);
        self.apply(Rule::Repetition, Some(index), component);
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
            if (!crate::senior::selected(self.profile, Rule::Repetition) || rotations >= 2)
                && self
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
        self.apply(Rule::Transition, Some(index), None);
        self.apply(Rule::Euler, Some(index), None);
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
        self.apply(Rule::Structure, Some(index), None);
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
            record_rule(
                &mut self.applied_rules,
                SEQUENCE_DIRECT_STEP_RULE,
                Some(index),
                None,
                &SourceReference {
                    document: "ISU Communication 2788".into(),
                    version: "2026-05-19".into(),
                    location: "p. 3, jump sequences".into(),
                },
            );
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
            } else if !self.assumed_direct_sequences.contains(&index) {
                self.add(FindingKind::Unknown, Rule::Structure, Some(index), None, "The plan does not establish the direct step into the Axel or panel recognition.");
            }
        }
        for (i, part) in parts.iter().enumerate() {
            if let JumpPart::Jump(jump) = part {
                if crate::senior::selected(self.profile, Rule::Structure)
                    && jump.rotations.is_some_and(|n| n >= 5)
                {
                    self.add(FindingKind::Violation, Rule::Structure, Some(index), Some(i),
                        "Senior combinations and sequences permit only single through quadruple jumps.");
                }
                self.jump(jump, index, Some(i));
            }
        }
    }
    fn program_checks(&mut self) {
        if crate::senior::selected(self.profile, Rule::Repetition) {
            let unique: Vec<_> = self
                .occurrences
                .iter()
                .enumerate()
                .filter(|(i, jump)| !self.occurrences[..*i].contains(jump))
                .map(|(_, jump)| *jump)
                .collect();
            let repeated: Vec<_> = unique
                .iter()
                .filter(|jump| {
                    jump.1 >= 3
                        && self
                            .occurrences
                            .iter()
                            .filter(|other| *other == *jump)
                            .count()
                            >= 2
                })
                .collect();
            if repeated.len() > crate::senior::DEFINITION.max_repeated_advanced
                || repeated.iter().filter(|jump| jump.1 >= 4).count()
                    > crate::senior::DEFINITION.max_repeated_quad_or_quint
            {
                self.add(FindingKind::Violation, Rule::Repetition, None, None,
                    "Only two triple/quad/quint types may repeat, including at most one quad or quint type.");
            }
            for kind in [
                JumpKind::ToeLoop,
                JumpKind::Salchow,
                JumpKind::Loop,
                JumpKind::Flip,
                JumpKind::Lutz,
                JumpKind::Axel,
            ] {
                if self
                    .occurrences
                    .iter()
                    .filter(|jump| jump.0 == kind)
                    .count()
                    > crate::senior::DEFINITION.max_kind_occurrences
                {
                    self.add(
                        FindingKind::Violation,
                        Rule::Repetition,
                        None,
                        None,
                        "A jump kind across all rotation counts may occur at most three times.",
                    );
                }
            }
        }
        for rule in [
            Rule::Count,
            Rule::CompoundCount,
            Rule::Repetition,
            Rule::Euler,
        ] {
            self.apply(rule, None, None);
        }
        if self.policy.require_axel {
            self.apply(Rule::Axel, None, None);
        }
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
        if self.policy.max_compound_length >= 3 {
            self.apply(Rule::Structure, None, None);
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
    validate_elements_traced(elements, profile, &[]).0
}

pub(crate) fn validate_elements_traced(
    elements: &[PlannedElement],
    profile: &crate::RuleProfile,
    assumed_direct_sequences: &[usize],
) -> (ValidationResult, Vec<AppliedRule>) {
    let mut validator = Validator::new(profile, assumed_direct_sequences);
    if crate::senior::selected(profile, Rule::Count) {
        validator.apply(Rule::Count, None, None);
        if elements.len() > crate::senior::DEFINITION.max_elements {
            validator.add(
                FindingKind::Violation,
                Rule::Count,
                None,
                None,
                "Senior Free Skating permits at most eleven element slots.",
            );
        }
    }
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
        validator.apply(Rule::Eligibility, Some(index), None);
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
        validator.apply(Rule::Eligibility, None, None);
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
    (
        ValidationResult {
            coverage: Coverage::Partial,
            findings: validator.findings,
        },
        validator.applied_rules,
    )
}
