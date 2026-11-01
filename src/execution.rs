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
    pub level: Option<u8>,
    /// None means unknown; an empty list means explicitly no calls.
    pub calls: Option<Vec<PanelCall>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognitionOutcome {
    Recognized,
    Ignored,
    NoValue,
    Unknown,
}

/// Exact supplied rotation and edge marks, without combination validation.
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
}
