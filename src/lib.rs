//! Source-traceable identities for the initial Adult singles ruleset.
//!
//! These identities do not implement rules or establish event applicability.

use std::{fmt, str::FromStr};

mod execution;
pub use execution::*;

mod plan;
pub use plan::*;

/// A skating season, identified by its starting and ending years.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Season {
    pub start_year: u16,
    pub end_year: u16,
}

/// Adult singles Free Skating categories in the initial scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdultCategory {
    Bronze,
    Silver,
    Gold,
    Masters,
}

/// A document, its version, and a precise location within that version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReference {
    pub document: String,
    pub version: String,
    pub location: String,
}

/// An unrecognized version identifier, retained for the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVersion(pub String);

impl fmt::Display for UnknownVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown version: {}", self.0)
    }
}
impl std::error::Error for UnknownVersion {}

/// Supported ruleset identities. A new source version requires a new identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulesetId {
    IsuAdult2026_27July16,
}

impl RulesetId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IsuAdult2026_27July16 => "isu-adult-2026-27-2026-07-16",
        }
    }
}

impl FromStr for RulesetId {
    type Err = UnknownVersion;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "isu-adult-2026-27-2026-07-16" => Ok(Self::IsuAdult2026_27July16),
            _ => Err(UnknownVersion(value.to_owned())),
        }
    }
}

/// Supported Scale of Values identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringTableId {
    Isu2786June2_2026,
}

impl ScoringTableId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Isu2786June2_2026 => "isu-2786-2026-06-02",
        }
    }
}

impl FromStr for ScoringTableId {
    type Err = UnknownVersion;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "isu-2786-2026-06-02" => Ok(Self::Isu2786June2_2026),
            _ => Err(UnknownVersion(value.to_owned())),
        }
    }
}

mod validation;
pub use validation::*;
