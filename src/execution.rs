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
    /// None means the recognized details are unknown.
    pub details: Option<RecognitionDetails>,
}

/// The canonical recognition consists of unmarked component codes, a level,
/// and calls. A full protocol code is derived from these fields, not stored twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionDetails {
    components: Vec<ComponentCode>,
    level: Option<Level>,
    calls: Option<Vec<RecordedCall>>,
}

/// A supported unmarked code. Marks and levels are stored separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentCode(String);

impl TryFrom<&str> for ComponentCode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let jump = value
            .strip_prefix(['1', '2', '3', '4', '5'])
            .is_some_and(|suffix| matches!(suffix, "T" | "S" | "Lo" | "F" | "Lz" | "A"));
        let element = matches!(
            value,
            "Eu" | "CSp"
                | "SSp"
                | "USp"
                | "LSp"
                | "CoSp"
                | "FCSp"
                | "FSSp"
                | "FUSp"
                | "FLSp"
                | "FCoSp"
                | "CCSp"
                | "CSSp"
                | "CUSp"
                | "CCoSp"
                | "FCCSp"
                | "FCSSp"
                | "FCUSp"
                | "FCCoSp"
                | "StSq"
                | "ChSq1"
                | "ChSp1"
        );
        if jump || element {
            Ok(Self(value.into()))
        } else {
            Err(value.into())
        }
    }
}

impl ComponentCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognitionError {
    EmptyComponents,
    InvalidTarget,
    ConflictingCalls,
}

impl RecognitionDetails {
    pub fn new(
        components: Vec<ComponentCode>,
        level: Option<Level>,
        calls: Option<Vec<RecordedCall>>,
    ) -> Result<Self, RecognitionError> {
        if components.is_empty() {
            return Err(RecognitionError::EmptyComponents);
        }
        if level.is_some()
            && (components.len() != 1
                || components[0].as_str().starts_with(char::is_numeric)
                || matches!(components[0].as_str(), "Eu" | "ChSq1" | "ChSp1"))
        {
            return Err(RecognitionError::InvalidTarget);
        }
        if let Some(ref calls) = calls {
            for (index, entry) in calls.iter().enumerate() {
                let structure = matches!(entry.call, PanelCall::Combination | PanelCall::Sequence);
                let valid_target = match entry.target {
                    CallTarget::Element => {
                        !structure
                            && (components.len() == 1
                                || !(entry.call.is_rotation() || entry.call.is_edge()))
                    }
                    CallTarget::Component(i) => !structure && i < components.len(),
                    CallTarget::BetweenComponents(i) => structure && i > 0 && i <= components.len(),
                };
                if !valid_target {
                    return Err(RecognitionError::InvalidTarget);
                }
                for previous in &calls[..index] {
                    let same_target = previous.target == entry.target
                        || (components.len() == 1
                            && matches!(
                                (previous.target, entry.target),
                                (CallTarget::Element, CallTarget::Component(0))
                                    | (CallTarget::Component(0), CallTarget::Element)
                            ));
                    if same_target
                        && (previous.call == entry.call
                            || (previous.call.is_rotation() && entry.call.is_rotation())
                            || (previous.call.is_edge() && entry.call.is_edge())
                            || (matches!(
                                previous.call,
                                PanelCall::Combination | PanelCall::Sequence
                            ) && structure))
                    {
                        return Err(RecognitionError::ConflictingCalls);
                    }
                }
            }
        }
        Ok(Self {
            components,
            level,
            calls,
        })
    }

    pub fn components(&self) -> &[ComponentCode] {
        &self.components
    }
    pub fn level(&self) -> Option<Level> {
        self.level
    }
    pub fn calls(&self) -> Option<&[RecordedCall]> {
        self.calls.as_deref()
    }

    /// Render the stored notation when calls are known. This does not validate legality.
    pub fn notation(&self) -> Option<String> {
        let calls = self.calls.as_deref()?;
        let mut result = String::new();
        for (index, component) in self.components.iter().enumerate() {
            if index > 0 {
                result.push('+');
            }
            for entry in calls
                .iter()
                .filter(|c| c.target == CallTarget::BetweenComponents(index))
            {
                result.push_str(entry.call.symbol().trim_start_matches('+'));
                result.push('+');
            }
            result.push_str(component.as_str());
            if self.components.len() == 1 {
                if let Some(level) = self.level {
                    result.push_str(level.symbol());
                }
            }
            for entry in calls
                .iter()
                .filter(|c| c.target == CallTarget::Component(index))
            {
                result.push_str(entry.call.symbol());
            }
        }
        for entry in calls.iter().filter(|c| {
            c.target == CallTarget::BetweenComponents(self.components.len())
                || c.target == CallTarget::Element
        }) {
            result.push_str(entry.call.symbol());
        }
        Some(result)
    }
}

/// Basic level is distinct from numbered levels and from an unknown level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Basic,
    One,
    Two,
    Three,
    Four,
}

impl TryFrom<u8> for Level {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(value),
        }
    }
}

impl Level {
    fn symbol(self) -> &'static str {
        match self {
            Self::Basic => "B",
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
        }
    }
}

/// A supplied protocol annotation, separate from a panel decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringAnnotation {
    /// x: second-half base-value credit; event applicability is not inferred.
    SecondHalfCredit,
}

/// Supplied scoring annotations for a recognized element, indexed in its list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringAnnotations {
    pub recognized_element: usize,
    pub annotations: Vec<ScoringAnnotation>,
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
}

impl PanelCall {
    fn is_rotation(self) -> bool {
        matches!(self, Self::Quarter | Self::UnderRotated | Self::Downgraded)
    }
    fn is_edge(self) -> bool {
        matches!(self, Self::UnclearEdge | Self::WrongEdge)
    }
    fn symbol(self) -> &'static str {
        match self {
            Self::Quarter => "q",
            Self::UnderRotated => "<",
            Self::Downgraded => "<<",
            Self::UnclearEdge => "!",
            Self::WrongEdge => "e",
            Self::ReducedSpinValue => "V",
            Self::NotCounted => "*",
            Self::Repetition => "+REP",
            Self::Combination => "+COMBO",
            Self::Sequence => "+SEQ",
        }
    }
}
