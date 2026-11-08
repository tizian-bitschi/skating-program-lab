#![cfg(feature = "optimizer")]
use skating_program_lab::*;

fn request(category: AdultCategory) -> PlanningSearchRequest {
    PlanningSearchRequest {
        profile: adult_profile(category).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        required: vec![],
        exclusions: vec![],
    }
}

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[test]
fn empty_bronze_request_proves_the_source_derived_maximum() {
    let request = request(AdultCategory::Bronze);
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::adult(), &request, &Continue).unwrap()
    else {
        panic!("expected a proved maximum")
    };
    // Four jump slots and one two-jump combination permit five components.
    // Repetition <=2: 2*1Lz(.60) + 3*(1F/1Lo)(.50) = 2.70.
    // Two distinct nonflying one-position spin abbreviations: CCSp1 and CLSp1,
    // each2.40 (Communication2786 pp7-8); ChSq1=3.50 (p14).
    assert_eq!(result.maximum_base_hundredths, 1100);
    assert_eq!(result.alternatives.len(), 1);
    let candidate = &result.alternatives[0];
    assert_eq!(candidate.assessment.total.exact_hundredths, Some(1100));
    assert!(candidate
        .assessment
        .legality
        .validation
        .validation
        .findings
        .is_empty());
    assert!(candidate
        .provenance
        .iter()
        .all(|p| *p == PlanningProvenance::Added));
}

#[test]
fn empty_silver_request_proves_the_source_derived_maximum() {
    let PlanningSearchOutcome::ProvenMaximum(result) = search_maximum_bv(
        &ProfileRegistry::adult(),
        &request(AdultCategory::Silver),
        &Continue,
    )
    .unwrap() else {
        panic!("expected a proved maximum")
    };
    // Seven listed jumps: 2*1A(110)+2*1Lz(60)+2*1F(50)+1Lo(50)=490.
    // CCoSp2=300 and CLSp2=290, ChSq1=350; Eu is zero value.
    assert_eq!(result.maximum_base_hundredths, 1430);
}

fn require(request: &mut PlanningSearchRequest, id: &str, code: &str) {
    request.required.push(RequiredPlanningElement {
        id: id.into(),
        element: AssumedRecognition::new(code),
    });
}
fn proved(request: &PlanningSearchRequest) -> PlanningMaximum {
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::adult(), request, &Continue).unwrap()
    else {
        panic!("expected maximum")
    };
    result
}

#[test]
fn silver_preserves_exact_free_placement_and_duplicate_occurrences() {
    let mut r = request(AdultCategory::Silver);
    require(&mut r, "spin", "CSSp2");
    require(&mut r, "compound", "1F+1Lo");
    let result = proved(&r);
    assert_eq!(result.maximum_base_hundredths, 1420);
    for (input_index, required) in r.required.iter().enumerate() {
        let c = &result.alternatives[0];
        let index = c
            .provenance
            .iter()
            .position(|p| {
                *p == PlanningProvenance::Required {
                    id: required.id.clone(),
                    input_index,
                }
            })
            .unwrap();
        assert_eq!(c.elements[index], required.element);
    }
    r.required.reverse();
    let reversed = proved(&r);
    assert_eq!(
        result.alternatives[0].elements,
        reversed.alternatives[0].elements
    );
    let mut r = request(AdultCategory::Bronze);
    require(&mut r, "first", "1Lz");
    require(&mut r, "second", "1Lz");
    let c = &proved(&r).alternatives[0];
    assert_eq!(c.elements.iter().filter(|e| e.code == "1Lz").count(), 2);
    assert_eq!(
        c.provenance
            .iter()
            .filter(|p| matches!(p, PlanningProvenance::Required { .. }))
            .count(),
        2
    );
}

#[test]
fn a_required_solo_is_not_satisfied_by_a_compound_component() {
    let mut r = request(AdultCategory::Bronze);
    require(&mut r, "solo", "1Lz");
    require(&mut r, "unit", "1Lz+1Lo");
    let result = proved(&r);
    assert_eq!(result.maximum_base_hundredths, 1100);
    let c = &result.alternatives[0];
    assert!(c.elements.iter().any(|e| e.code == "1Lz"));
    assert!(c.elements.iter().any(|e| e.code == "1Lz+1Lo"));
}

#[test]
fn semantic_exclusions_cover_compounds_rotations_and_named_spin_forms() {
    let mut r = request(AdultCategory::Silver);
    r.exclusions = vec![
        PlanningExclusion::JumpKind(JumpKind::Axel),
        PlanningExclusion::SpinKind(RegularSpinKind::Layback),
    ];
    let result = proved(&r);
    // Seven permitted components: Lz,Lz,F,F,Lo,Lo,S =360. Best spins300+280.
    assert_eq!(result.maximum_base_hundredths, 1290);
    let c = &result.alternatives[0];
    assert!(c
        .elements
        .iter()
        .all(|e| !e.code.contains('A') && !e.code.contains("LSp")));
    r.exclusions = vec![PlanningExclusion::ExactJump(ListedJump {
        kind: Some(JumpKind::Axel),
        rotations: Some(1),
    })];
    assert_eq!(proved(&r).maximum_base_hundredths, 1300);
}

#[test]
fn required_conflicts_and_malformed_requests_keep_input_indices() {
    let mut r = request(AdultCategory::Silver);
    require(&mut r, "unit", "1F+1Lo");
    r.exclusions = vec![PlanningExclusion::JumpKind(JumpKind::Flip)];
    let Err(PlanningSearchError::InvalidRequest { diagnostics, .. }) =
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue)
    else {
        panic!("expected input conflict")
    };
    assert_eq!(diagnostics[0].required_input_indices, vec![0]);
    r.exclusions.clear();
    r.required[0].element = AssumedRecognition::new("2T");
    let Err(PlanningSearchError::InvalidRequest { diagnostics, .. }) =
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue)
    else {
        panic!("expected category prohibition")
    };
    assert_eq!(diagnostics[0].required_input_indices, vec![0]);
    assert!(diagnostics[0].rule_id.is_some());
    r.required.clear();
    r.exclusions.push(PlanningExclusion::ExactJump(ListedJump {
        kind: None,
        rotations: Some(1),
    }));
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue),
        Err(PlanningSearchError::InvalidRequest { .. })
    ));
    r.exclusions.clear();
    require(&mut r, "same", "1T");
    require(&mut r, "same", "1T");
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue),
        Err(PlanningSearchError::InvalidRequest { .. })
    ));
}

#[test]
fn established_required_repetition_is_infeasible_with_rule_diagnostics() {
    let mut r = request(AdultCategory::Bronze);
    for id in ["one", "two", "three"] {
        require(&mut r, id, "1T");
    }
    let PlanningSearchOutcome::Infeasible { diagnostics } =
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue).unwrap()
    else {
        panic!("expected infeasible")
    };
    assert!(diagnostics
        .iter()
        .any(|d| d.rule_id.is_some() && d.required_input_indices.contains(&2)));
}

#[test]
fn exclusions_can_make_the_required_silver_spin_role_impossible() {
    let mut r = request(AdultCategory::Silver);
    r.exclusions = [
        RegularSpinKind::Camel,
        RegularSpinKind::Sit,
        RegularSpinKind::Upright,
        RegularSpinKind::Layback,
    ]
    .into_iter()
    .map(PlanningExclusion::SpinKind)
    .collect();
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue).unwrap(),
        PlanningSearchOutcome::Infeasible { .. }
    ));
}

struct CancelAt(PlanningSearchPhase);
impl PlanningSearchControl for CancelAt {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        phase == self.0
    }
}
#[test]
fn cancellation_before_proof_exposes_no_program_and_after_proof_retains_it() {
    for phase in [
        PlanningSearchPhase::Domain,
        PlanningSearchPhase::Model,
        PlanningSearchPhase::Solve,
        PlanningSearchPhase::IncumbentFound,
        PlanningSearchPhase::Reconstruction,
    ] {
        assert_eq!(
            search_maximum_bv(
                &ProfileRegistry::adult(),
                &request(AdultCategory::Bronze),
                &CancelAt(phase)
            )
            .unwrap(),
            PlanningSearchOutcome::Cancelled,
            "{phase:?}"
        );
    }
    for phase in [
        PlanningSearchPhase::MaximumProved,
        PlanningSearchPhase::Alternatives,
    ] {
        let PlanningSearchOutcome::ProvenMaximum(result) = search_maximum_bv(
            &ProfileRegistry::adult(),
            &request(AdultCategory::Bronze),
            &CancelAt(phase),
        )
        .unwrap() else {
            panic!("maximum must survive")
        };
        assert_eq!(result.maximum_base_hundredths, 1100);
        assert_eq!(
            result.alternative_status,
            PlanningAlternativeStatus::Cancelled
        );
    }
}

#[test]
fn unsupported_profiles_and_unknown_versions_are_not_infeasible() {
    for category in [AdultCategory::Gold, AdultCategory::Masters] {
        assert!(matches!(
            search_maximum_bv(&ProfileRegistry::adult(), &request(category), &Continue),
            Err(PlanningSearchError::UnsupportedCoverage { .. })
        ));
    }
    let mut r = request(AdultCategory::Bronze);
    r.profile.version = "unknown".into();
    assert!(matches!(
        search_maximum_bv(&ProfileRegistry::adult(), &r, &Continue),
        Err(PlanningSearchError::Profile(_))
    ));
}

#[test]
fn complete_public_repertoire_matches_an_independent_small_exhaustive_oracle() {
    // Source-defined Bronze restriction: only 1T(.40) and 1S(.40), <=4
    // elements, <=1 two-listed-jump compound, each listed jump <=2.
    // Exhaust all ordered words; this oracle has no production generator,
    // validator, scorer or Pumpkin dependency.
    let atoms = [
        (1, 0, false),
        (0, 1, false),
        (2, 0, true),
        (1, 1, true),
        (1, 1, true),
        (0, 2, true),
    ];
    fn exhaustive(atoms: &[(u8, u8, bool)], slots: u8, t: u8, s: u8, compounds: u8) -> u32 {
        let mut best = u32::from(t + s) * 40;
        if slots == 4 {
            return best;
        }
        for &(dt, ds, compound) in atoms {
            let nc = compounds + u8::from(compound);
            if t + dt <= 2 && s + ds <= 2 && nc <= 1 {
                best = best.max(exhaustive(atoms, slots + 1, t + dt, s + ds, nc));
            }
        }
        best
    }
    let mut r = request(AdultCategory::Bronze);
    for (id, code) in [
        ("camel", "CCSp1"),
        ("layback", "CLSp1"),
        ("choreo", "ChSq1"),
    ] {
        require(&mut r, id, code);
    }
    r.exclusions = [
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
        JumpKind::Axel,
    ]
    .into_iter()
    .map(PlanningExclusion::JumpKind)
    .collect();
    let result = proved(&r);
    assert_eq!(
        result.maximum_base_hundredths,
        830 + exhaustive(&atoms, 0, 0, 0, 0)
    );
    let c = &result.alternatives[0];
    let reassessed = assess_manual_bv(
        &ProfileRegistry::adult(),
        &ManualBvRequest {
            profile: r.profile,
            scoring_table: r.scoring_table,
            submission: SubmissionState::Complete,
            elements: c.elements.clone(),
        },
    )
    .unwrap();
    assert_eq!(c.assessment, reassessed);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
}

#[test]
fn intended_v_required_value_is_reserved_without_adding_its_full_abbreviation() {
    let mut r = request(AdultCategory::Silver);
    require(&mut r, "reduced", "CCoSp2V");
    require(&mut r, "one-position", "CSSp2");
    let result = proved(&r);
    assert_eq!(result.maximum_base_hundredths, 1345);
    assert!(result.alternatives[0]
        .elements
        .iter()
        .any(|e| e.code == "CCoSp2V"));
    assert!(!result.alternatives[0]
        .elements
        .iter()
        .any(|e| e.code == "CCoSp2"));
}

#[test]
fn profile_slots_are_maxima_and_excluded_jumps_do_not_have_to_be_filled() {
    let mut r = request(AdultCategory::Bronze);
    r.exclusions = [
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Loop,
        JumpKind::Flip,
        JumpKind::Lutz,
        JumpKind::Axel,
    ]
    .into_iter()
    .map(PlanningExclusion::JumpKind)
    .collect();
    let result = proved(&r);
    assert_eq!(result.maximum_base_hundredths, 830);
    assert_eq!(result.alternatives[0].elements.len(), 3);
}

#[test]
fn required_sequences_and_euler_units_remain_whole_and_exact() {
    for (code, direct) in [("1F+1A+1Lo+SEQ", true), ("1F+Eu+1Lo", false)] {
        let mut r = request(AdultCategory::Silver);
        require(&mut r, "unit", code);
        r.required[0].element.direct_transitions_assumed = direct;
        let result = proved(&r);
        let c = &result.alternatives[0];
        let index = c
            .provenance
            .iter()
            .position(|p| matches!(p, PlanningProvenance::Required { .. }))
            .unwrap();
        assert_eq!(c.elements[index], r.required[0].element);
        assert!(c
            .assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty());
    }
}
