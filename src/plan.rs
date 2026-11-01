//! Intended program structure. These types do not validate or recognize elements.

use crate::{AdultCategory, RulesetId, Season};

/// An ordered Free Skating plan. The season and ruleset are selected independently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub season: Season,
    pub category: AdultCategory,
    pub ruleset: RulesetId,
    pub elements: Vec<PlannedElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedElement {
    Jump(JumpElement),
    RegularSpin(RegularSpin),
    ChoreographicSpin,
    StepSequence,
    ChoreographicSequence,
    /// An element whose type has not yet been selected.
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpElement {
    Solo(ListedJump),
    Combination(Vec<JumpPart>),
    Sequence(Vec<JumpPart>),
}

/// Transitions are separate from listed jumps, including when their details are unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpPart {
    Jump(ListedJump),
    Euler,
    OtherTransition(String),
    UnspecifiedTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedJump {
    pub kind: Option<JumpKind>,
    pub rotations: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpKind {
    ToeLoop,
    Salchow,
    Loop,
    Flip,
    Lutz,
    Axel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegularSpin {
    /// None means the entry has not been specified.
    pub entry: Option<SpinEntry>,
    /// None means the ordered steps have not been specified; an empty list is explicit.
    pub steps: Option<Vec<SpinStep>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinEntry {
    Flying,
    NonFlying,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpinStep {
    Position {
        position: Option<SpinPosition>,
        /// Whole intended revolutions; None preserves an unspecified count.
        revolutions: Option<u16>,
        /// None means feature attempts for this position are unspecified.
        feature_attempts: Option<Vec<String>>,
    },
    ChangeFoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinPosition {
    Upright,
    Sit,
    Camel,
}
