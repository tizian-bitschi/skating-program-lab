use skating_program_lab::*;

const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
const MANIFEST: &str = include_str!("fixtures/isu-2786-2026-06-02-manifest.tsv");
const VALUES: &str = include_str!("fixtures/isu-2786-2026-06-02-values.txt");
const SOURCE_VALUES: &str = include_str!("fixtures/isu-2786-2026-06-02-source-values.tsv");

#[test]
fn every_manifest_row_has_exact_catalog_metadata() {
    let mut checked = 0;
    for line in MANIFEST.lines().skip(1) {
        let cells: Vec<_> = line.split('\t').collect();
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
    assert_eq!(checked, 625);
}

#[test]
fn independent_numerical_fixtures_cover_every_family_and_all_goe_columns() {
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
    assert_eq!(checked, 42);
    assert_eq!(
        families,
        [
            "Jump",
            "Spin",
            "StepSequence",
            "ChoreographicSequence",
            "ChoreographicSpin",
            "Lift",
            "TwistLift",
            "ThrowJump",
            "DeathSpiral",
            "PivotFigure",
            "PairSpin",
            "PairSpinCombination",
            "ChoreographicPairSpin",
            "ChoreographicPairLift",
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
    for code in [
        "1LiB", "1TwB<<", "1TTh<", "BiDs2b", "PiF1", "PSp4", "PCoSp2V", "ChPSp1", "ChPLi1",
    ] {
        let row = catalog_row(TABLE, code).unwrap();
        assert_eq!(row.discipline, CatalogDiscipline::Pair);
        // A known Pair row does not grant a recognition/scoring API for that behavior.
        assert!(ComponentCode::try_from(code).is_err());
    }
    let lift = catalog_row(TABLE, "ChPLi1").unwrap();
    assert_eq!((lift.base_hundredths, lift.goe_hundredths[10]), (450, 250));
    assert_eq!(catalog_row(TABLE, "1TwB<<").unwrap().base_hundredths, 0);
}

#[test]
fn all_published_numeric_cells_match_the_independent_source_extraction() {
    check_source_values(SOURCE_VALUES);
}

fn check_source_values(fixture: &str) {
    assert!(fixture
        .lines()
        .any(|line| line == "# document=ISU Communication 2786"));
    assert!(fixture.lines().any(|line| line == "# version=2026-06-02"));
    assert!(fixture
        .lines()
        .any(|line| line == format!("# sha256={ISU_2786_JUNE_2_2026_SHA256}")));
    let mut seen = std::collections::BTreeSet::new();
    for line in fixture
        .lines()
        .filter(|line| !line.starts_with('#'))
        .skip(1)
    {
        let cells: Vec<_> = line.split('\t').collect();
        assert_eq!(cells.len(), 15, "all eleven printed numeric cells: {line}");
        assert_eq!(cells[0], TABLE.as_str());
        assert!(seen.insert(cells[3]), "duplicate {}", cells[3]);
        let row = catalog_row(TABLE, cells[3]).unwrap();
        assert_eq!(row.source.page.to_string(), cells[1]);
        assert_eq!(row.source.row.to_string(), cells[2]);
        let printed: Vec<i16> = cells[4..]
            .iter()
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(
            i32::from(row.base_hundredths),
            i32::from(printed[5]),
            "{} BASE",
            row.notation
        );
        assert_eq!(
            row.goe_hundredths[..5],
            printed[..5],
            "{} negative GOE",
            row.notation
        );
        assert_eq!(row.goe_hundredths[5], 0);
        assert_eq!(
            row.goe_hundredths[6..],
            printed[6..],
            "{} positive GOE",
            row.notation
        );
    }
    let expected: std::collections::BTreeSet<_> = MANIFEST
        .lines()
        .skip(1)
        .map(|line| line.split('\t').nth(6).unwrap())
        .collect();
    assert_eq!(seen.len(), 625);
    assert_eq!(seen, expected);
}

#[test]
fn numerical_audit_rejects_missing_columns_wrong_values_and_editions() {
    for (before, after) in [
        ("\t1T\t-20\t-16\t", "\t1T\t-19\t-16\t"),
        ("\t1T\t-20\t-16\t", "\t1T\t-20\t"),
        ("# version=2026-06-02", "# version=2026-05-08"),
        ("isu-2786-2026-06-02\t2\t2", "isu-2786-2026-05-08\t2\t2"),
        ("\t2\t3\t1S\t", "\t2\t3\t1T\t"),
        ("# sha256=980c", "# sha256=0000"),
    ] {
        assert!(SOURCE_VALUES.contains(before));
        let mutated = SOURCE_VALUES.replacen(before, after, 1);
        assert!(
            std::panic::catch_unwind(|| check_source_values(&mutated)).is_err(),
            "{after}"
        );
    }
}
