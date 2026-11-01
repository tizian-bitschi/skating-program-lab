//! Observations and supplied panel decisions; neither is inferred from a plan.

use crate::{RulesetId, SourceReference};

/// Facts recorded for one observed movement. Indices are zero-based and refer
/// to the corresponding program's element list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedElement {
    pub ruleset: RulesetId,
    pub source: Option<SourceReference>,
    pub planned_element: Option<usize>,
    pub movement: Option<String>,
    /// Intended number of rotations in the observed attempt, not a rotation call.
    pub attempted_rotations: Option<u8>,
    /// None means the fall was not observed, not that no fall occurred.
    pub fall: Option<bool>,
}

/// A user-supplied decision. Indices refer to the corresponding plan and
/// observation lists; associations and supplied fields are not validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelRecognition {
    pub ruleset: RulesetId,
    pub source: Option<SourceReference>,
    pub planned_element: Option<usize>,
    pub observed_element: Option<usize>,
    pub outcome: RecognitionOutcome,
    pub code: Option<String>,
    pub level: Option<Level>,
    /// Ordered recognized components, including transitions when present.
    /// None means unknown; indices in calls refer to this list.
    pub components: Option<Vec<String>>,
    /// None means unknown; an empty list means explicitly no calls.
    pub calls: Option<Vec<RecordedCall>>,
}

/// Basic level is distinct from numbered levels and from an unknown level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Basic,
    Number(u8),
}

/// The target is explicit: a whole element or a zero-based component of it.
/// Component indices refer to the ordered recognized components, not the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCall {
    pub target: CallTarget,
    pub call: PanelCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTarget {
    Element,
    Component(usize),
    /// Before the component at this index; the list length denotes the end.
    BetweenComponents(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognitionOutcome {
    Recognized,
    Ignored,
    NoValue,
    Unknown,
}

/// Supplied technical marks and combination/sequence notation. No legality is inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelCall {
    /// q
    Quarter,
    /// <
    UnderRotated,
    /// <<
    Downgraded,
    /// !
    UnclearEdge,
    /// e
    WrongEdge,
    /// V: reduced value of a spin.
    ReducedSpinValue,
    /// *: the marked element or jump does not count.
    NotCounted,
    /// +REP: repetition treatment, including the 80% rule where applicable.
    Repetition,
    /// +COMBO: combination notation, including an interrupted combination.
    Combination,
    /// +SEQ: sequence notation, including an interrupted sequence.
    Sequence,
    /// x: supplied second-half base-value credit, not inferred from timing.
    SecondHalfCredit,
}
