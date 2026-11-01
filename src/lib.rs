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

impl SourceReference {
    /// Resolve known source editions to their official ISU documents; unknown editions have no URL.
    pub fn official_url(&self) -> Option<&'static str> {
        match (self.document.as_str(), self.version.as_str()) {
            ("ISU Communication 2786", "2026-06-02") => Some("https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2786-SinglesPairs-SOV-Update-260602-1780481099-6311.pdf"),
            ("ISU Communication 2788", "2026-05-19") => Some("https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2788-SP-Levels-and-GOE-2026-Updated-MAY-19-1779351385-9626.pdf"),
            ("International Adult Figure Skating Competitions Technical Announcement", "2026-07-16") => Some("https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Fansite/Figure-Skating/2026-2027/Events/Adult-Skating/Documents/IAFSC-Technical-Announcement---2026-27--1--1786956273-6936.pdf"),
            ("ISU Sports Rules \u{2014} Figure Skating", "2026-06-12") => Some("https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf"),
            _ => None,
        }
    }
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
    /// Synthetic identity used only to exercise version mismatch handling.
    #[cfg(test)]
    TestOther,
}

impl RulesetId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IsuAdult2026_27July16 => "isu-adult-2026-27-2026-07-16",
            #[cfg(test)]
            Self::TestOther => "test-other",
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

mod scoring;
pub use scoring::*;
