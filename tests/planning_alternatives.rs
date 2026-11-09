#![cfg(feature = "optimizer")]
use skating_program_lab::*;

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

fn restricted(allowed: &[JumpKind]) -> PlanningSearchRequest {
    PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::AllowMixedDirections,
        profile: adult_profile(AdultCategory::Bronze).id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        required: ["CCSp1", "CLSp1", "ChSq1"]
            .into_iter()
            .map(|code| RequiredPlanningElement {
                id: code.into(),
                element: AssumedRecognition::new(code),
            })
            .collect(),
        exclusions: [
            JumpKind::Waltz,
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Loop,
            JumpKind::Flip,
            JumpKind::Lutz,
            JumpKind::Axel,
        ]
        .into_iter()
        .filter(|kind| !allowed.contains(kind))
        .map(PlanningExclusion::JumpKind)
        .collect(),
    }
}
fn proved(
    request: &PlanningSearchRequest,
    control: &impl PlanningSearchControl,
) -> PlanningMaximum {
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::adult(), request, control).unwrap()
    else {
        panic!("expected proved maximum")
    };
    result
}
fn jump_codes(result: &PlanningMaximum) -> std::collections::BTreeSet<Vec<String>> {
    result
        .alternatives
        .iter()
        .map(|p| {
            p.elements
                .iter()
                .filter(|e| e.code.starts_with('1'))
                .map(|e| e.code.clone())
                .collect()
        })
        .collect()
}

#[test]
fn exactly_five_atomic_partitions_are_retained_without_requiring_four_jump_entries() {
    // Communication 2786: 1F=1Lo=.50, CCSp1=CLSp1=2.40, ChSq1=3.50.
    // Bronze stock is two of each listed jump, at most four entries and one pair.
    // Exhausting that stock permits these five whole-element partitions.
    let result = proved(&restricted(&[JumpKind::Flip, JumpKind::Loop]), &Continue);
    assert_eq!(result.maximum_base_hundredths, 1030);
    let expected = [
        vec!["1F", "1F", "1Lo", "1Lo"],
        vec!["1F+1F", "1Lo", "1Lo"],
        vec!["1F", "1F+1Lo", "1Lo"],
        vec!["1F", "1Lo", "1Lo+1F"],
        vec!["1F", "1F", "1Lo+1Lo"],
    ]
    .map(|codes| codes.into_iter().map(String::from).collect())
    .into_iter()
    .collect();
    assert_eq!(jump_codes(&result), expected);
    assert_eq!(result.alternatives.len(), 5);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
}

#[test]
fn bronze_required_flip_has_four_distinct_compositions_even_with_optional_waltzes() {
    // Announcement p11 and SOV: jump bound270 requires twoLz plus threeF/Lo.
    // A required solo F leaves precisely these four sorted whole-element multisets.
    let mut request = restricted(&[JumpKind::Lutz, JumpKind::Flip, JumpKind::Loop]);
    request.rotation_preference = RotationPreference::default();
    request.required.push(RequiredPlanningElement {
        id: "flip".into(),
        element: AssumedRecognition::new("1F"),
    });
    let result = proved(&request, &Continue);
    let expected = [
        vec!["1F", "1F", "1Lz", "1Lz+1Lo"],
        vec!["1F", "1F+1Lo", "1Lz", "1Lz"],
        vec!["1F", "1Lo", "1Lz", "1Lz+1Lo"],
        vec!["1F", "1Lo+1Lo", "1Lz", "1Lz"],
    ]
    .map(|v| v.into_iter().map(String::from).collect())
    .into_iter()
    .collect();
    assert_eq!(result.maximum_base_hundredths, 1100);
    assert_eq!(jump_codes(&result), expected);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    request
        .exclusions
        .retain(|e| *e != PlanningExclusion::JumpKind(JumpKind::Waltz));
    let neutral = proved(&request, &Continue);
    assert_eq!(neutral.maximum_base_hundredths, 1100);
    for candidate in &neutral.alternatives {
        assert_required_and_assessed(&ProfileRegistry::adult(), &request, candidate, 1100);
    }
    assert_eq!(neutral.alternatives.len(), 4);
    assert_eq!(jump_codes(&neutral), jump_codes(&result));
    assert!(neutral
        .alternatives
        .iter()
        .all(|p| p.elements.iter().all(|e| e.code != "W")));
    assert_eq!(
        neutral.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
}

#[test]
fn two_partitions_exhaust_and_duplicate_required_solos_leave_one() {
    let mut request = restricted(&[JumpKind::ToeLoop]);
    let result = proved(&request, &Continue);
    assert_eq!(result.maximum_base_hundredths, 910);
    assert_eq!(
        jump_codes(&result),
        [vec!["1T".into(), "1T".into()], vec!["1T+1T".into()]]
            .into_iter()
            .collect()
    );
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    for id in ["first", "second"] {
        request.required.push(RequiredPlanningElement {
            id: id.into(),
            element: AssumedRecognition::new("1T"),
        });
    }
    let result = proved(&request, &Continue);
    assert_eq!(result.maximum_base_hundredths, 910);
    assert_eq!(result.alternatives.len(), 1);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    let candidate = &result.alternatives[0];
    for (input_index, required) in request.required.iter().enumerate() {
        let index = candidate
            .provenance
            .iter()
            .position(|p| {
                *p == PlanningProvenance::Required {
                    id: required.id.clone(),
                    input_index,
                }
            })
            .unwrap();
        assert_eq!(candidate.elements[index], required.element);
    }
    request.required.reverse();
    let reversed = proved(&request, &Continue);
    assert_eq!(candidate.elements, reversed.alternatives[0].elements);
    assert_eq!(result.alternative_status, reversed.alternative_status);
}

#[test]
fn more_than_five_maxima_match_an_independent_bounded_atomic_oracle() {
    // Literal source values, independent of production generation/scoring.
    let codes = ["1T", "1S", "1F", "1Lo"];
    let values = [40, 40, 50, 50];
    let fixture = include_str!("fixtures/isu-2786-2026-06-02-source-values.tsv");
    for (code, value) in
        codes
            .into_iter()
            .zip(values)
            .chain([("CCSp1", 240), ("CLSp1", 240), ("ChSq1", 350)])
    {
        let row = fixture
            .lines()
            .map(|line| line.split('\t').collect::<Vec<_>>())
            .find(|row| row.get(3) == Some(&code))
            .unwrap();
        assert_eq!(row[9].parse::<u32>().unwrap(), value);
    }
    let mut atoms = Vec::new();
    for i in 0..4 {
        let mut stock = [0; 4];
        stock[i] = 1;
        atoms.push((codes[i].to_string(), stock, false, values[i]));
        for j in 0..4 {
            let mut stock = stock;
            stock[j] += 1;
            atoms.push((
                format!("{}+{}", codes[i], codes[j]),
                stock,
                true,
                values[i] + values[j],
            ));
        }
    }
    fn enumerate(
        atoms: &[(String, [u8; 4], bool, u32)],
        start: usize,
        state: ([u8; 4], bool, u32),
        chosen: &mut Vec<String>,
        best: &mut u32,
        maxima: &mut std::collections::BTreeSet<Vec<String>>,
    ) {
        let (stock, compound, value) = state;
        if value > *best {
            *best = value;
            maxima.clear();
        }
        if value == *best {
            let mut key = chosen.clone();
            key.sort();
            maxima.insert(key);
        }
        if chosen.len() == 4 {
            return;
        }
        for (i, (code, extra, pair, bv)) in atoms.iter().enumerate().skip(start) {
            if compound && *pair {
                continue;
            }
            let next = std::array::from_fn(|j| stock[j] + extra[j]);
            if next.iter().any(|&n| n > 2) {
                continue;
            }
            chosen.push(code.clone());
            enumerate(
                atoms,
                i,
                (next, compound || *pair, value + bv),
                chosen,
                best,
                maxima,
            );
            chosen.pop();
        }
    }
    let mut best = 0;
    let mut maxima = std::collections::BTreeSet::new();
    enumerate(
        &atoms,
        0,
        ([0; 4], false, 0),
        &mut vec![],
        &mut best,
        &mut maxima,
    );
    assert_eq!(best, 240);
    assert!(maxima.len() > 5);
    let request = restricted(&[
        JumpKind::ToeLoop,
        JumpKind::Salchow,
        JumpKind::Flip,
        JumpKind::Loop,
    ]);
    let registry = ProfileRegistry::adult();
    let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
    assert_eq!(
        session.prove(&Continue).unwrap(),
        PlanningSessionState::Collecting
    );
    let mut all = Vec::new();
    while session.state() == PlanningSessionState::Collecting {
        let batch = session
            .collect(std::num::NonZeroUsize::new(3).unwrap(), &CollectionContinue)
            .unwrap();
        assert_eq!(batch.state, session.state());
        assert!(batch.alternatives.len() <= 3);
        all.extend(batch.alternatives);
    }
    assert_eq!(session.state(), PlanningSessionState::Exhausted);
    assert_eq!(all.len(), maxima.len());
    assert_eq!(session.confirmed(), all);
    assert_eq!(program_jump_codes(&all), maxima);
    for candidate in &all {
        assert_required_and_assessed(&registry, &request, candidate, 1070);
    }
    let result = proved(&request, &Continue);
    assert_eq!(result.maximum_base_hundredths, 830 + best);
    assert_eq!(result.alternatives.len(), 5);
    assert_eq!(jump_codes(&result).len(), 5);
    assert!(jump_codes(&result).is_subset(&maxima));
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
    assert_eq!(result, proved(&request, &Continue));
    let mut reversed = request.clone();
    reversed.required.reverse();
    reversed.exclusions.reverse();
    let reversed = proved(&reversed, &Continue);
    assert_eq!(
        result
            .alternatives
            .iter()
            .map(|p| &p.elements)
            .collect::<Vec<_>>(),
        reversed
            .alternatives
            .iter()
            .map(|p| &p.elements)
            .collect::<Vec<_>>()
    );
    for candidate in result.alternatives {
        assert_eq!(candidate.assessment.total.exact_hundredths, Some(1070));
        assert!(candidate
            .assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty());
        let assessment = assess_manual_bv(
            &ProfileRegistry::adult(),
            &ManualBvRequest {
                personal_limits: PersonalLimits::default(),
                rotation_preference: RotationPreference::AllowMixedDirections,
                profile: request.profile.clone(),
                scoring_table: request.scoring_table,
                submission: SubmissionState::Complete,
                elements: candidate.elements,
            },
        )
        .unwrap();
        assert_eq!(candidate.assessment, assessment);
    }
}

struct CancelCollectionReconstruction {
    proved: std::cell::Cell<bool>,
}
impl PlanningSearchControl for CancelCollectionReconstruction {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if phase == PlanningSearchPhase::MaximumProved {
            self.proved.set(true);
        }
        self.proved.get() && phase == PlanningSearchPhase::Reconstruction
    }
}

#[test]
fn cancellation_during_alternative_reconstruction_retains_only_confirmed_programs() {
    let request = restricted(&[JumpKind::Flip, JumpKind::Loop]);
    let result = proved(
        &request,
        &CancelCollectionReconstruction {
            proved: std::cell::Cell::new(false),
        },
    );
    assert_eq!(result.maximum_base_hundredths, 1030);
    assert_eq!(result.alternatives.len(), 1);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Cancelled
    );
    assert_eq!(
        result.alternatives[0].assessment.total.exact_hundredths,
        Some(1030)
    );
}

struct CancelSecondCollectionReconstruction {
    last: std::cell::Cell<PlanningSearchPhase>,
    collections: std::cell::Cell<usize>,
}
impl PlanningSearchControl for CancelSecondCollectionReconstruction {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if phase == PlanningSearchPhase::Reconstruction
            && self.last.get() == PlanningSearchPhase::Alternatives
        {
            self.collections.set(self.collections.get() + 1);
        }
        self.last.set(phase);
        self.collections.get() == 2
    }
}

#[test]
fn cancellation_after_collecting_one_more_maximum_preserves_both() {
    let result = proved(
        &restricted(&[JumpKind::Flip, JumpKind::Loop]),
        &CancelSecondCollectionReconstruction {
            last: std::cell::Cell::new(PlanningSearchPhase::Domain),
            collections: std::cell::Cell::new(0),
        },
    );
    assert_eq!(result.maximum_base_hundredths, 1030);
    assert_eq!(result.alternatives.len(), 2);
    assert_eq!(jump_codes(&result).len(), 2);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Cancelled
    );
}
#[test]
fn cancellation_with_ignored_waltzes_preserves_all_collected_maxima_and_required_ids() {
    let mut request = restricted(&[JumpKind::Flip, JumpKind::Loop, JumpKind::Waltz]);
    for id in ["first-waltz", "second-waltz"] {
        request.required.push(RequiredPlanningElement {
            id: id.into(),
            element: AssumedRecognition::new("W"),
        });
    }
    let result = proved(
        &request,
        &CancelSecondCollectionReconstruction {
            last: std::cell::Cell::new(PlanningSearchPhase::Domain),
            collections: std::cell::Cell::new(0),
        },
    );
    assert_eq!(result.maximum_base_hundredths, 1030);
    assert_eq!(result.alternatives.len(), 2);
    assert_eq!(jump_codes(&result).len(), 2);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Cancelled
    );
    for candidate in result.alternatives {
        assert_eq!(
            candidate.elements.iter().filter(|e| e.code == "W").count(),
            2
        );
        for (input_index, required) in request.required.iter().enumerate() {
            let index = candidate
                .provenance
                .iter()
                .position(|p| {
                    *p == PlanningProvenance::Required {
                        id: required.id.clone(),
                        input_index,
                    }
                })
                .unwrap();
            assert_eq!(candidate.elements[index], required.element);
        }
        let assessment = assess_manual_bv(
            &ProfileRegistry::adult(),
            &ManualBvRequest {
                personal_limits: request.personal_limits.clone(),
                rotation_preference: request.rotation_preference,
                profile: request.profile.clone(),
                scoring_table: request.scoring_table,
                submission: SubmissionState::Complete,
                elements: candidate.elements,
            },
        )
        .unwrap();
        assert_eq!(candidate.assessment, assessment);
        assert_eq!(
            assessment.planning_compliance(),
            Ok(PlanningCompliance::Compliant)
        );
    }
}

#[test]
fn registered_ignored_mechanism_exhausts_required_waltz_compositions() {
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "synthetic-ignored-movement-identity".into();
    profile.category = "Synthetic identity fixture".into();
    profile.provenance = ProfileProvenance::Synthetic;
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    profile.sources.extend([
        policy.spins.source.clone(),
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    let mut registry = ProfileRegistry::adult();
    registry
        .register_with_program_policy(profile.clone(), policy)
        .unwrap();
    let mut request = restricted(&[JumpKind::ToeLoop, JumpKind::Waltz]);
    request.profile = profile.id;
    for id in ["first-waltz", "second-waltz"] {
        request.required.push(RequiredPlanningElement {
            id: id.into(),
            element: AssumedRecognition::new("W"),
        });
    }
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&registry, &request, &Continue).unwrap()
    else {
        panic!("maximum expected")
    };
    assert_eq!(result.maximum_base_hundredths, 910);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    assert_eq!(
        jump_codes(&result),
        [vec!["1T".into(), "1T".into()], vec!["1T+1T".into()]]
            .into_iter()
            .collect()
    );
    for candidate in &result.alternatives {
        assert_eq!(
            candidate.elements.iter().filter(|e| e.code == "W").count(),
            2
        );
        assert_required_and_assessed(&registry, &request, candidate, 910);
    }
    let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
    session.prove(&Continue).unwrap();
    while session.state() == PlanningSessionState::Collecting {
        session
            .collect(std::num::NonZeroUsize::new(1).unwrap(), &CollectionContinue)
            .unwrap();
    }
    assert_eq!(session.state(), PlanningSessionState::Exhausted);
    assert_eq!(session.confirmed().len(), 2);
    assert_eq!(program_jump_codes(session.confirmed()), jump_codes(&result));
    for candidate in session.confirmed() {
        assert_eq!(
            candidate.elements.iter().filter(|e| e.code == "W").count(),
            2
        );
        assert_required_and_assessed(&registry, &request, candidate, 910);
    }
    request
        .exclusions
        .push(PlanningExclusion::ExactElement("W".into()));
    assert!(matches!(
        search_maximum_bv(&registry, &request, &Continue),
        Err(PlanningSearchError::InvalidRequest { .. })
    ));
}

fn assert_required_and_assessed(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    candidate: &PlanningProgram,
    value: i64,
) {
    for (input_index, required) in request.required.iter().enumerate() {
        let index = candidate
            .provenance
            .iter()
            .position(|p| {
                *p == PlanningProvenance::Required {
                    id: required.id.clone(),
                    input_index,
                }
            })
            .unwrap();
        assert_eq!(candidate.elements[index], required.element);
    }
    let assessment = assess_manual_bv(
        registry,
        &ManualBvRequest {
            personal_limits: request.personal_limits.clone(),
            rotation_preference: request.rotation_preference,
            profile: request.profile.clone(),
            scoring_table: request.scoring_table,
            submission: SubmissionState::Complete,
            elements: candidate.elements.clone(),
        },
    )
    .unwrap();
    assert_eq!(assessment.total.exact_hundredths, Some(value));
    assert_eq!(
        assessment.planning_compliance(),
        Ok(PlanningCompliance::Compliant)
    );
    assert_eq!(candidate.assessment, assessment);
}

#[test]
fn counted_zero_value_waltzes_remain_distinct_optional_elements() {
    let registry = ProfileRegistry::adult();
    let profile = adult_profile(AdultCategory::Silver);
    let mut request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: ScoringTableId::Isu2786June2_2026,
        required: ["CSSp2", "ChSp1", "ChSq1"]
            .map(|code| RequiredPlanningElement {
                id: code.into(),
                element: AssumedRecognition::new(code),
            })
            .to_vec(),
        exclusions: vec![],
    };
    request.exclusions = selectable_jump_elements(
        &registry,
        &request.profile,
        request.scoring_table,
        request.rotation_preference,
    )
    .unwrap()
    .into_iter()
    .filter(|e| e.code != "W")
    .map(|e| PlanningExclusion::ExactElement(e.code))
    .collect();
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&registry, &request, &Continue).unwrap()
    else {
        panic!("maximum expected")
    };
    // Silver W is a counted downgraded Axel: zero BV, at most two occurrences.
    // CSSp2=280, ChSp1=350, ChSq1=350 (Communication 2786).
    assert_eq!(result.maximum_base_hundredths, 980);
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::Exhausted
    );
    assert_eq!(
        result
            .alternatives
            .iter()
            .map(|p| p.elements.iter().filter(|e| e.code == "W").count())
            .collect::<std::collections::BTreeSet<_>>(),
        [0, 1, 2].into_iter().collect()
    );
    assert_eq!(result.alternatives.len(), 3);
    for candidate in &result.alternatives {
        assert_required_and_assessed(&registry, &request, candidate, 980);
    }
}

#[test]
fn session_filled_batch_requires_a_separate_exhaustion_proof() {
    let registry = ProfileRegistry::adult();
    let mut session = PlanningSearchSession::new(
        &registry,
        restricted(&[JumpKind::Flip, JumpKind::Loop]),
        None,
    );
    assert_eq!(session.state(), PlanningSessionState::Unproved);
    assert_eq!(session.maximum_base_hundredths(), None);
    assert!(session.confirmed().is_empty());
    assert_eq!(
        session.prove(&Continue).unwrap(),
        PlanningSessionState::Collecting
    );
    assert_eq!(session.maximum_base_hundredths(), Some(1030));
    assert!(session.confirmed().is_empty());
    struct Collect;
    impl PlanningCollectionControl for Collect {
        fn action(&self, _: PlanningSearchPhase) -> PlanningCollectionAction {
            PlanningCollectionAction::Continue
        }
    }
    let batch = session
        .collect(std::num::NonZeroUsize::new(5).unwrap(), &Collect)
        .unwrap();
    assert_eq!(batch.alternatives.len(), 5);
    assert_eq!(batch.state, PlanningSessionState::Collecting);
    assert_eq!(session.confirmed(), batch.alternatives);
    let final_batch = session
        .collect(std::num::NonZeroUsize::new(5).unwrap(), &Collect)
        .unwrap();
    assert!(final_batch.alternatives.is_empty());
    assert_eq!(final_batch.state, PlanningSessionState::Exhausted);
}

struct CollectionContinue;
impl PlanningCollectionControl for CollectionContinue {
    fn action(&self, _: PlanningSearchPhase) -> PlanningCollectionAction {
        PlanningCollectionAction::Continue
    }
}
fn program_jump_codes(programs: &[PlanningProgram]) -> std::collections::BTreeSet<Vec<String>> {
    programs
        .iter()
        .map(|p| {
            p.elements
                .iter()
                .filter(|e| e.code.starts_with('1'))
                .map(|e| e.code.clone())
                .collect()
        })
        .collect()
}

struct CollectionActionAt(PlanningSearchPhase, PlanningCollectionAction);
impl PlanningCollectionControl for CollectionActionAt {
    fn action(&self, phase: PlanningSearchPhase) -> PlanningCollectionAction {
        if phase == self.0 {
            self.1
        } else {
            PlanningCollectionAction::Continue
        }
    }
}
#[test]
fn session_pauses_empty_and_interrupted_reconstruction_then_resumes_without_duplicates() {
    let registry = ProfileRegistry::adult();
    let request = restricted(&[JumpKind::Flip, JumpKind::Loop]);
    let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
    session.prove(&Continue).unwrap();
    let size = std::num::NonZeroUsize::new(2).unwrap();
    let pause = session
        .collect(
            size,
            &CollectionActionAt(
                PlanningSearchPhase::Alternatives,
                PlanningCollectionAction::Pause,
            ),
        )
        .unwrap();
    assert_eq!(pause.state, PlanningSessionState::Paused);
    assert!(pause.alternatives.is_empty());
    assert!(session.confirmed().is_empty());
    let first = session.collect(size, &CollectionContinue).unwrap();
    assert_eq!(first.alternatives.len(), 2);
    let pause = session
        .collect(
            size,
            &CollectionActionAt(
                PlanningSearchPhase::Reconstruction,
                PlanningCollectionAction::Pause,
            ),
        )
        .unwrap();
    assert_eq!(pause.state, PlanningSessionState::Paused);
    assert!(pause.alternatives.is_empty());
    assert_eq!(session.confirmed(), first.alternatives);
    assert_eq!(session.maximum_base_hundredths(), Some(1030));
    let mut all = first.alternatives;
    while matches!(
        session.state(),
        PlanningSessionState::Paused | PlanningSessionState::Collecting
    ) {
        all.extend(
            session
                .collect(size, &CollectionContinue)
                .unwrap()
                .alternatives,
        );
    }
    assert_eq!(all.len(), 5);
    assert_eq!(
        program_jump_codes(&all),
        jump_codes(&proved(&request, &Continue))
    );
    assert_eq!(session.confirmed(), all);
    for candidate in &all {
        assert_required_and_assessed(&registry, &request, candidate, 1030);
    }
}

#[test]
fn session_cancellation_is_terminal_and_retains_only_proved_deliveries() {
    struct CancelAt(PlanningSearchPhase);
    impl PlanningSearchControl for CancelAt {
        fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
            phase == self.0
        }
    }
    let registry = ProfileRegistry::adult();
    let request = restricted(&[JumpKind::ToeLoop]);
    let size = std::num::NonZeroUsize::new(1).unwrap();
    for phase in [
        PlanningSearchPhase::Domain,
        PlanningSearchPhase::Model,
        PlanningSearchPhase::Solve,
        PlanningSearchPhase::Reconstruction,
        PlanningSearchPhase::IncumbentFound,
        PlanningSearchPhase::MaximumProved,
    ] {
        let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
        assert_eq!(
            session.prove(&CancelAt(phase)).unwrap(),
            PlanningSessionState::Cancelled
        );
        assert_eq!(
            session.maximum_base_hundredths(),
            (phase == PlanningSearchPhase::MaximumProved).then_some(910)
        );
        assert!(session.confirmed().is_empty());
        assert_eq!(
            session.prove(&Continue),
            Err(PlanningSessionError::InvalidState {
                state: PlanningSessionState::Cancelled
            })
        );
        assert_eq!(
            session.collect(size, &CollectionContinue),
            Err(PlanningSessionError::InvalidState {
                state: PlanningSessionState::Cancelled
            })
        );
    }
    for phase in [
        PlanningSearchPhase::Alternatives,
        PlanningSearchPhase::Reconstruction,
    ] {
        let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
        session.prove(&Continue).unwrap();
        let first = session.collect(size, &CollectionContinue).unwrap();
        let cancelled = session
            .collect(
                size,
                &CollectionActionAt(phase, PlanningCollectionAction::Cancel),
            )
            .unwrap();
        assert_eq!(cancelled.state, PlanningSessionState::Cancelled);
        assert!(cancelled.alternatives.is_empty());
        assert_eq!(session.confirmed(), first.alternatives);
        assert_eq!(session.maximum_base_hundredths(), Some(910));
    }
}

#[test]
fn session_distinguishes_invalid_requests_infeasibility_and_single_solution() {
    let registry = ProfileRegistry::adult();
    let mut request = restricted(&[JumpKind::ToeLoop]);
    for id in ["first", "second"] {
        request.required.push(RequiredPlanningElement {
            id: id.into(),
            element: AssumedRecognition::new("1T"),
        });
    }
    let size = std::num::NonZeroUsize::new(1).unwrap();
    let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
    assert_eq!(
        session.collect(size, &CollectionContinue),
        Err(PlanningSessionError::InvalidState {
            state: PlanningSessionState::Unproved
        })
    );
    session.prove(&Continue).unwrap();
    assert_eq!(
        session.prove(&Continue),
        Err(PlanningSessionError::InvalidState {
            state: PlanningSessionState::Collecting
        })
    );
    let batch = session.collect(size, &CollectionContinue).unwrap();
    assert_eq!(batch.state, PlanningSessionState::Collecting);
    assert_eq!(batch.alternatives.len(), 1);
    assert_required_and_assessed(&registry, &request, &batch.alternatives[0], 910);
    assert_eq!(
        session.collect(size, &CollectionContinue).unwrap().state,
        PlanningSessionState::Exhausted
    );
    assert_eq!(
        session.collect(size, &CollectionContinue),
        Err(PlanningSessionError::InvalidState {
            state: PlanningSessionState::Exhausted
        })
    );
    request.required.push(RequiredPlanningElement {
        id: "third".into(),
        element: AssumedRecognition::new("1T"),
    });
    let mut impossible = PlanningSearchSession::new(&registry, request.clone(), None);
    assert_eq!(
        impossible.prove(&Continue).unwrap(),
        PlanningSessionState::Infeasible
    );
    assert!(impossible.maximum_base_hundredths().is_none());
    assert!(!impossible.diagnostics().is_empty());
    assert!(impossible.confirmed().is_empty());
    request.required[0].id = request.required[1].id.clone();
    let mut invalid = PlanningSearchSession::new(&registry, request, None);
    assert!(matches!(
        invalid.prove(&Continue),
        Err(PlanningSessionError::Search(
            PlanningSearchError::InvalidRequest { .. }
        ))
    ));
    assert_eq!(invalid.state(), PlanningSessionState::Failed);
    assert!(invalid.diagnostics().is_empty());
    assert!(invalid.maximum_base_hundredths().is_none());
}

#[test]
fn session_required_compound_is_indivisible_and_exact_exclusion_keeps_internal_order_distinct() {
    let registry = ProfileRegistry::adult();
    let mut request = restricted(&[JumpKind::Flip, JumpKind::Loop]);
    request.required.push(RequiredPlanningElement {
        id: "connection".into(),
        element: AssumedRecognition::new("1Lo+1F"),
    });
    request
        .exclusions
        .push(PlanningExclusion::ExactElement("1F+1Lo".into()));
    let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
    session.prove(&Continue).unwrap();
    let batch = session
        .collect(
            std::num::NonZeroUsize::new(10).unwrap(),
            &CollectionContinue,
        )
        .unwrap();
    assert_eq!(batch.state, PlanningSessionState::Exhausted);
    assert_eq!(batch.alternatives.len(), 1);
    let candidate = &batch.alternatives[0];
    assert_eq!(
        program_jump_codes(&batch.alternatives),
        [vec!["1F".into(), "1Lo".into(), "1Lo+1F".into()]]
            .into_iter()
            .collect()
    );
    assert_required_and_assessed(&registry, &request, candidate, 1030);
}

#[test]
fn session_partial_batches_preserve_committed_deliveries_on_pause_and_cancel() {
    struct AfterReconstruction {
        reconstructed: std::cell::Cell<bool>,
        action: PlanningCollectionAction,
    }
    impl PlanningCollectionControl for AfterReconstruction {
        fn action(&self, phase: PlanningSearchPhase) -> PlanningCollectionAction {
            if phase == PlanningSearchPhase::Reconstruction {
                self.reconstructed.set(true);
            }
            if phase == PlanningSearchPhase::Alternatives && self.reconstructed.get() {
                self.action
            } else {
                PlanningCollectionAction::Continue
            }
        }
    }
    let registry = ProfileRegistry::adult();
    let request = restricted(&[JumpKind::Flip, JumpKind::Loop]);
    for (action, state) in [
        (
            PlanningCollectionAction::Pause,
            PlanningSessionState::Paused,
        ),
        (
            PlanningCollectionAction::Cancel,
            PlanningSessionState::Cancelled,
        ),
    ] {
        let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
        session.prove(&Continue).unwrap();
        let batch = session
            .collect(
                std::num::NonZeroUsize::new(5).unwrap(),
                &AfterReconstruction {
                    reconstructed: std::cell::Cell::new(false),
                    action,
                },
            )
            .unwrap();
        assert_eq!(batch.state, state);
        assert_eq!(batch.alternatives.len(), 1);
        assert_eq!(session.confirmed(), batch.alternatives);
        assert_required_and_assessed(&registry, &request, &batch.alternatives[0], 1030);
    }
}
