#[cfg(feature = "optimizer")]
mod measured {
    use skating_program_lab::*;
    use std::cell::Cell;
    use std::io::{self, Write};
    use std::time::{Duration, Instant};

    struct Control {
        start: Instant,
        first: [Cell<Option<Duration>>; 7],
        cancel: String,
        cancel_phase: Option<PlanningSearchPhase>,
        triggered: Cell<Option<Duration>>,
        proved: Cell<bool>,
    }
    fn index(phase: PlanningSearchPhase) -> usize {
        match phase {
            PlanningSearchPhase::Domain => 0,
            PlanningSearchPhase::Model => 1,
            PlanningSearchPhase::Solve => 2,
            PlanningSearchPhase::IncumbentFound => 3,
            PlanningSearchPhase::Reconstruction => 4,
            PlanningSearchPhase::MaximumProved => 5,
            PlanningSearchPhase::Alternatives => 6,
        }
    }
    impl PlanningSearchControl for Control {
        fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
            let now = self.start.elapsed();
            let first = &self.first[index(phase)];
            if first.get().is_none() {
                first.set(Some(now));
            }
            if phase == PlanningSearchPhase::MaximumProved {
                self.proved.set(true);
            }
            let cancel = self.cancel_phase == Some(phase)
                || (self.cancel == "PostproofReconstruction"
                    && self.proved.get()
                    && phase == PlanningSearchPhase::Reconstruction);
            if cancel && self.triggered.get().is_none() {
                self.triggered.set(Some(now));
            }
            cancel
        }
    }
    fn micros(value: Option<Duration>) -> String {
        value.map(|v| v.as_micros().to_string()).unwrap_or_default()
    }
    pub fn run() {
        let args: Vec<_> = std::env::args().collect();
        let name = args.get(1).expect("profile");
        let scenario = args.get(2).expect("scenario");
        let (registry, profile, empty_max) = match name.as_str() {
            "Bronze" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Bronze),
                1100,
            ),
            "Silver" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Silver),
                1480,
            ),
            "Gold" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Gold),
                2420,
            ),
            "Masters" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Masters),
                3340,
            ),
            "Women" => (
                ProfileRegistry::senior(),
                senior_profile(SeniorCategory::Women),
                13310,
            ),
            "Men" => (
                ProfileRegistry::senior(),
                senior_profile(SeniorCategory::Men),
                13310,
            ),
            _ => panic!("unknown profile"),
        };
        let mut request = PlanningSearchRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: vec![],
            exclusions: vec![],
        };
        let required: &[&str] = match scenario.as_str() {
            "empty" | "excluded" => &[],
            "required" if name == "Silver" => &["1F+1Lo", "CSSp2"],
            "required" if name == "Bronze" => &["1F"],
            "required" => &["1A"],
            "impossible" => &["1F", "1F", "1F", "1F"],
            _ => panic!("unknown scenario"),
        };
        for (i, code) in required.iter().enumerate() {
            request.required.push(RequiredPlanningElement {
                id: format!("required-{i}"),
                element: AssumedRecognition::new(*code),
            });
        }
        if scenario == "excluded" {
            request
                .exclusions
                .push(PlanningExclusion::JumpKind(JumpKind::ToeLoop));
        }
        // Debug is a complete lossless field manifest for these fixed request types.
        println!("REQUEST\t{request:?}");
        if args.get(3).is_some_and(|s| s == "manifest") {
            return;
        }
        let cancel = args.get(3).cloned().unwrap_or_default();
        let cancel_phase = [
            PlanningSearchPhase::Domain,
            PlanningSearchPhase::Model,
            PlanningSearchPhase::Solve,
            PlanningSearchPhase::IncumbentFound,
            PlanningSearchPhase::Reconstruction,
            PlanningSearchPhase::MaximumProved,
            PlanningSearchPhase::Alternatives,
        ]
        .into_iter()
        .find(|p| format!("{p:?}") == cancel);
        assert!(cancel.is_empty() || cancel_phase.is_some() || cancel == "PostproofReconstruction");
        let control = Control {
            start: Instant::now(),
            first: std::array::from_fn(|_| Cell::new(None)),
            cancel,
            cancel_phase,
            triggered: Cell::new(None),
            proved: Cell::new(false),
        };
        let result = search_maximum_bv(&registry, &request, &control);
        let total = control.start.elapsed();
        let (outcome, maximum, alternatives, status, passed) = match &result {
            Ok(PlanningSearchOutcome::ProvenMaximum(m)) => (
                "ProvenMaximum",
                m.maximum_base_hundredths.to_string(),
                m.alternatives.len().to_string(),
                format!("{:?}", m.alternative_status),
                scenario != "impossible"
                    && !m.alternatives.is_empty()
                    && m.alternatives.len() <= 5
                    && (scenario != "empty" || m.maximum_base_hundredths == empty_max)
                    && m.alternatives.iter().all(|p| {
                        p.assessment.total.exact_hundredths
                            == Some(i64::from(m.maximum_base_hundredths))
                            && p.assessment
                                .legality
                                .validation
                                .validation
                                .findings
                                .is_empty()
                            && p.assessment.planning_compliance()
                                == Ok(PlanningCompliance::Compliant)
                    })
                    && (control.cancel.is_empty()
                        || (control.proved.get()
                            && m.alternative_status == PlanningAlternativeStatus::Cancelled)),
            ),
            Ok(PlanningSearchOutcome::Infeasible { .. }) => (
                "Infeasible",
                String::new(),
                "0".into(),
                String::new(),
                scenario == "impossible" && control.cancel.is_empty(),
            ),
            Ok(PlanningSearchOutcome::Cancelled) => (
                "Cancelled",
                String::new(),
                "0".into(),
                String::new(),
                control.triggered.get().is_some() && !control.proved.get(),
            ),
            Err(error) => {
                eprintln!("{error:?}");
                ("Error", String::new(), "0".into(), String::new(), false)
            }
        };
        let first = |i: usize| control.first[i].get();
        let domain = first(1);
        let model = first(2).zip(first(1)).map(|(s, m)| s.saturating_sub(m));
        let proof = first(5);
        let alternatives_time = proof.map(|p| total.saturating_sub(p));
        let response = control.triggered.get().map(|t| total.saturating_sub(t));
        println!("METRIC\t{name},{scenario},{},{outcome},{maximum},{alternatives},{status},{},{},{},{},{},{},{},{passed}",
            control.cancel, micros(domain), micros(model), micros(first(3)), micros(proof), micros(alternatives_time), total.as_micros(), micros(response));
        io::stdout().flush().unwrap();
        // Keep the process alive so the parent reads the OS lifetime peak before exit.
        let mut ack = String::new();
        io::stdin().read_line(&mut ack).unwrap();
        assert!(passed, "readiness check failed: {result:?}");
    }
}
#[cfg(feature = "optimizer")]
fn main() {
    measured::run();
}
#[cfg(not(feature = "optimizer"))]
fn main() {
    eprintln!("planning_readiness requires optimizer");
}
