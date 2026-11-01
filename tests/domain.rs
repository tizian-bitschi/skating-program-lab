use skating_program_lab::{AdultCategory, RulesetId, ScoringTableId, Season, SourceReference};
use std::str::FromStr;

#[test]
fn supported_example_retains_all_fields() {
    // Adult announcement p. 11 and Communication 2786 p. 2.
    let example = (
        Season {
            start_year: 2026,
            end_year: 2027,
        },
        AdultCategory::Bronze,
        RulesetId::from_str("isu-adult-2026-27-2026-07-16").unwrap(),
        ScoringTableId::from_str("isu-2786-2026-06-02").unwrap(),
        SourceReference {
            document: "International Adult Figure Skating Competitions Technical Announcement"
                .into(),
            version: "2026-07-16".into(),
            location: "p. 11, Bronze Free Skating, jumps".into(),
        },
    );
    let (season, category, ruleset, table, source) = example;
    assert_eq!((season.start_year, season.end_year), (2026, 2027));
    assert!(matches!(category, AdultCategory::Bronze));
    assert_eq!(ruleset.as_str(), "isu-adult-2026-27-2026-07-16");
    assert_eq!(table.as_str(), "isu-2786-2026-06-02");
    assert_eq!(
        source.document,
        "International Adult Figure Skating Competitions Technical Announcement"
    );
    assert_eq!(source.version, "2026-07-16");
    assert_eq!(source.location, "p. 11, Bronze Free Skating, jumps");
}

#[test]
fn unknown_versions_are_rejected_without_fallback() {
    let ruleset = "isu-adult-2026-27-2026-07-17";
    let table = "isu-2786-2026-05-05";
    assert_eq!(RulesetId::from_str(ruleset).unwrap_err().0, ruleset);
    assert_eq!(ScoringTableId::from_str(table).unwrap_err().0, table);
    assert!(RulesetId::from_str("").is_err());
    assert!(ScoringTableId::from_str("").is_err());
}
