use skating_program_lab::*;

#[test]
fn exhaustive_reference_finds_all_timed_ties() {
    let ReferenceResult::Optimal {
        base_hundredths,
        candidates,
    } = enumerate_bronze_reference(false).unwrap()
    else {
        panic!("expected optimum")
    };
    // 1Lz = 0.60 (2786 p. 2); at most two repetitions (Adult p. 11).
    assert_eq!(base_hundredths, 120);
    assert_eq!(
        candidates.iter().map(|c| c.slots).collect::<Vec<_>>(),
        vec![
            [true, true, false],
            [true, false, true],
            [false, true, true]
        ]
    );
    for candidate in candidates {
        assert_eq!(candidate.coverage, Coverage::Partial);
        assert_eq!(candidate.base_hundredths, 120);
        assert_eq!(candidate.program.elements.len(), 2);
        assert!(validate_program(&candidate.program).findings.is_empty());
    }
}

#[test]
fn excluding_the_only_variant_is_infeasible() {
    assert_eq!(
        enumerate_bronze_reference(true),
        Ok(ReferenceResult::Infeasible)
    );
}
