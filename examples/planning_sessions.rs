//! Native public-session timing example; see benchmarks/planning-sessions/README.md.
#[cfg(feature = "optimizer")]
mod measured {
    use skating_program_lab::*;
    use std::{
        collections::BTreeSet,
        io::{self, Write},
        num::NonZeroUsize,
        time::Instant,
    };
    struct Continue;
    impl PlanningSearchControl for Continue {
        fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
            false
        }
    }
    impl PlanningCollectionControl for Continue {
        fn action(&self, _: PlanningSearchPhase) -> PlanningCollectionAction {
            PlanningCollectionAction::Continue
        }
    }
    struct Pause;
    impl PlanningCollectionControl for Pause {
        fn action(&self, _: PlanningSearchPhase) -> PlanningCollectionAction {
            PlanningCollectionAction::Pause
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

    fn oracle() -> BTreeSet<Vec<String>> {
        let codes = ["1T", "1S", "1F", "1Lo"];
        let values = [40, 40, 50, 50];
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

        maxima
    }
    fn identity(program: &PlanningProgram) -> Vec<String> {
        let mut codes: Vec<_> = program
            .elements
            .iter()
            .filter(|e| e.code.starts_with('1'))
            .map(|e| e.code.clone())
            .collect();
        codes.sort();
        codes
    }
    pub fn run() {
        let paused = match std::env::args().nth(1).as_deref() {
            Some("uninterrupted") => false,
            Some("paused") => true,
            _ => panic!("use uninterrupted or paused"),
        };
        let registry = ProfileRegistry::adult();
        let request = restricted(&[
            JumpKind::ToeLoop,
            JumpKind::Salchow,
            JumpKind::Flip,
            JumpKind::Loop,
        ]);
        let expected = oracle();
        println!(
            "REQUEST\t{:?}; maximum_step_level=None; first_batch=5; later_batch=3; paused={paused}",
            request
        );
        let start = Instant::now();
        let mut session = PlanningSearchSession::new(&registry, request.clone(), None);
        let new_us = start.elapsed().as_micros();
        let start = Instant::now();
        assert_eq!(
            session.prove(&Continue).unwrap(),
            PlanningSessionState::Collecting
        );
        let proof_us = start.elapsed().as_micros();
        assert_eq!(session.maximum_base_hundredths(), Some(1070));
        let mut active_collection_us = 0;
        let mut first_five_us = 0;
        let mut pause_us = 0;
        let mut resumed_us = 0;
        let mut batches = 0;
        while session.state() == PlanningSessionState::Collecting
            || session.state() == PlanningSessionState::Paused
        {
            let start = Instant::now();
            let batch = session
                .collect(
                    NonZeroUsize::new(if batches == 0 { 5 } else { 3 }).unwrap(),
                    &Continue,
                )
                .unwrap();
            let us = start.elapsed().as_micros();
            active_collection_us += us;
            if batches == 0 {
                first_five_us = us;
            }
            if paused && batches == 1 {
                resumed_us = us;
            }
            println!(
                "BATCH\t{batches},{us},{},{:?}",
                batch.alternatives.len(),
                batch.state
            );
            batches += 1;
            if paused && batches == 1 {
                let retained = session.confirmed().to_vec();
                let start = Instant::now();
                let pause = session
                    .collect(NonZeroUsize::new(3).unwrap(), &Pause)
                    .unwrap();
                pause_us = start.elapsed().as_micros();
                assert_eq!(pause.state, PlanningSessionState::Paused);
                assert!(pause.alternatives.is_empty());
                assert_eq!(session.confirmed(), retained);
            }
        }
        assert_eq!(session.state(), PlanningSessionState::Exhausted);
        let actual: BTreeSet<_> = session.confirmed().iter().map(identity).collect();
        assert_eq!(session.confirmed().len(), actual.len());
        assert_eq!(actual, expected);
        for candidate in session.confirmed() {
            let manual = assess_manual_bv_with_step_limit(
                &registry,
                &ManualBvRequest {
                    profile: request.profile.clone(),
                    scoring_table: request.scoring_table,
                    personal_limits: request.personal_limits.clone(),
                    rotation_preference: request.rotation_preference,
                    submission: SubmissionState::Complete,
                    elements: candidate.elements.clone(),
                },
                None,
            )
            .unwrap();
            assert_eq!(manual.total.exact_hundredths, Some(1070));
            assert_eq!(
                manual.planning_compliance(),
                Ok(PlanningCompliance::Compliant)
            );
            assert!(manual.personal_conflicts.is_empty());
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
        }
        println!("IDENTITIES\t{actual:?}");
        println!("METRIC\t{new_us},{proof_us},{first_five_us},{active_collection_us},{pause_us},{resumed_us},{},{batches},Exhausted,true", actual.len());
        io::stdout().flush().unwrap();
        let mut ack = String::new();
        io::stdin().read_line(&mut ack).unwrap();
    }
}
#[cfg(feature = "optimizer")]
fn main() {
    measured::run();
}
#[cfg(not(feature = "optimizer"))]
fn main() {
    eprintln!("requires optimizer feature");
}
