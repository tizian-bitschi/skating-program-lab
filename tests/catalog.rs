use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const MANIFEST: &str = include_str!("fixtures/isu-2786-2026-06-02-manifest.tsv");
const VALUES: &str = include_str!("fixtures/isu-2786-2026-06-02-values.txt");

#[test]
fn every_shared_manifest_row_has_exact_catalog_metadata() {
    let mut checked = 0;
    for line in MANIFEST.lines().skip(1) {
        let cells: Vec<_> = line.split('\t').collect();
        if cells[4] != "SingleAndPair" {
            continue;
        }
        let row = catalog_row(TABLE, cells[6]).unwrap_or_else(|| panic!("missing {}", cells[6]));
        assert_eq!(row.notation, cells[6]);
        assert_eq!(row.source.table, TABLE);
        assert_eq!(row.source.page.to_string(), cells[1]);
        assert_eq!(row.source.row.to_string(), cells[2]);
        assert_eq!(row.source.section, cells[3]);
        assert_eq!(format!("{:?}", row.discipline), cells[4]);
        assert_eq!(format!("{:?}", row.family), cells[5]);
        let level = match row.level {
            None => "-",
            Some(Level::Basic) => "B",
            Some(Level::One) => "1",
            Some(Level::Two) => "2",
            Some(Level::Three) => "3",
            Some(Level::Four) => "4",
        };
        assert_eq!(level, cells[7]);
        let notes: Vec<_> = row
            .notes
            .iter()
            .map(|note| match note {
                CatalogNote::PercentageGoe => "1",
                CatalogNote::ChoreographicGoe => "2",
                CatalogNote::ReducedBaseGoe => "3",
                CatalogNote::CombinationReference => "4",
                CatalogNote::Repetition => "5",
                CatalogNote::QuintJumps => "6",
                CatalogNote::Legend => "legend",
            })
            .collect();
        assert_eq!(notes.join(","), cells[9]);
        assert_eq!(row.goe_hundredths[5], 0);
        let source = row.source.reference(row.notation);
        assert_eq!(source.version, "2026-06-02");
        assert!(source.official_url().unwrap().contains("Update-260602"));
        checked += 1;
    }
    assert_eq!(checked, 402);
}

#[test]
fn independent_numerical_fixtures_cover_every_shared_family_and_all_goe_columns() {
    let mut families = std::collections::BTreeSet::new();
    let mut checked = 0;
    for line in VALUES.lines().filter(|line| !line.starts_with('#')) {
        let cells: Vec<_> = line.split('|').collect();
        let row = catalog_row(TABLE, cells[1]).unwrap();
        assert_eq!(row.source.page.to_string(), cells[0]);
        assert_eq!(row.base_hundredths, cells[2].parse::<u16>().unwrap());
        let expected: Vec<i16> = cells[3].split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(expected.len(), 11);
        assert_eq!(row.goe_hundredths.as_slice(), expected, "{}", row.notation);
        families.insert(format!("{:?}", row.family));
        checked += 1;
    }
    assert_eq!(checked, 23);
    assert_eq!(
        families,
        [
            "Jump",
            "Spin",
            "StepSequence",
            "ChoreographicSequence",
            "ChoreographicSpin"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
}

#[test]
fn exact_lookup_rejects_absent_codes_and_editions_without_fallback() {
    for code in [
        "", "CSSp", "CSSp5", "3Fb!", "1Tqq", "1T ", "1T+1T", "5A", "unknown",
    ] {
        assert!(catalog_row(TABLE, code).is_none(), "{code}");
    }
    for edition in ["isu-2786-2026-05-08", "isu-2786-2026-06-03", "unknown"] {
        assert_eq!(edition.parse::<ScoringTableId>().unwrap_err().0, edition);
    }
    assert_ne!(
        catalog_row(TABLE, "CSSp1").unwrap().base_hundredths,
        catalog_row(TABLE, "CSSp1V").unwrap().base_hundredths
    );
    assert!(std::ptr::eq(
        catalog_row(TABLE, "3A").unwrap(),
        catalog_row(TABLE, "3A").unwrap()
    ));
}

#[test]
fn known_catalog_rows_do_not_expand_recognition_or_scoring_support() {
    for (code, level, notation) in [
        ("3A", None, "3A"),
        ("CSSp", Some(Level::Two), "CSSp2"),
        ("StSq", Some(Level::One), "StSq1"),
    ] {
        assert!(catalog_row(TABLE, notation).is_some());
        let recognition = PanelRecognition {
            ruleset: RulesetId::IsuAdult2026_27July16,
            source: None,
            planned_element: None,
            observed_element: None,
            outcome: RecognitionOutcome::Recognized,
            details: Some(
                RecognitionDetails::new(
                    vec![ComponentCode::try_from(code).unwrap()],
                    level,
                    Some(vec![]),
                )
                .unwrap(),
            ),
        };
        assert_eq!(
            lookup_base_value(TABLE, &recognition, &[]),
            Err(BaseValueError::Unsupported)
        );
    }
    // Pair sections are deliberately absent until C, not fabricated from similar codes.
    assert!(catalog_row(TABLE, "PiF1").is_none());
    assert!(ComponentCode::try_from("PiF1").is_err());
}
