//! Fixed requests shared verbatim by both sides of the #104 experiment.
use super::*;

#[path = "../../../tests/support/generation_defaults.rs"]
mod generation_defaults;

fn describe(label: &str, value: &impl std::fmt::Debug) {
    if let Some(path) = std::env::var_os("ACCEPTANCE_MANIFEST") {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(file, "INPUT {label}\n{value:#?}").unwrap();
    }
}

fn measure<T: std::fmt::Debug>(label: &str, run: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let (result, d) = capture(run);
    let elapsed = start.elapsed();
    println!("MEASURE {label} call_us={} {d:?}", elapsed.as_micros());
    println!("RESULT {label} {result:#?}");
    result
}

fn budget() -> Option<Duration> {
    Some(Duration::from_millis(250))
}

#[test]
#[ignore = "controlled wall-clock benchmark; run serially"]
fn controlled_acceptance() {
    let registry = ProfileRegistry::adult();
    for category in [
        AdultCategory::Bronze,
        AdultCategory::Silver,
        AdultCategory::Gold,
        AdultCategory::Masters,
    ] {
        let label = format!("Adult-{category:?}");
        let generation = workflows::bounds_domain(category);
        describe(&label, &generation);
        let start = Instant::now();
        let generated = generate_program_candidates(&registry, &generation).unwrap();
        println!("GENERATION {label} us={}", start.elapsed().as_micros());
        describe(&label, &generated);
        let mut q = workflows::optimization(&generation, generated.variants, 6);
        q.max_proposals = 200;
        q.max_solutions = 100;
        q.time_limit = budget();
        describe(&label, &q);
        measure(&format!("{label}-full"), || {
            optimize_program(&registry, &q).unwrap()
        });

        // Explicit SOV choices, including every spin level; no feature proofs.
        // No compound is inferred from an individual's solo capability.
        let codes: &[&str] = match category {
            AdultCategory::Bronze => &[
                "1T", "1Lo", "1S", "SSpB", "SSp1", "SSp2", "CSpB", "CSp1", "CSp2", "ChSq1",
            ],
            AdultCategory::Silver => &[
                "1T", "1Lo", "1S", "SSpB", "SSp1", "SSp2", "CCoSpB", "CCoSp1", "CCoSp2", "ChSq1",
            ],
            _ => &[
                "1A", "2T", "2S", "CCoSpB", "CCoSp1", "CCoSp2", "CCoSp3", "CCoSp4", "FSSpB",
                "FSSp1", "FSSp2", "FSSp3", "FSSp4", "StSq2", "ChSp1",
            ],
        };
        let choices: Vec<_> = codes.iter().map(|c| AssumedRecognition::new(*c)).collect();
        let generation = generation_defaults::request(adult_profile(category));
        describe(
            &format!("{label}-completion-generation"),
            &(&generation, &choices),
        );
        let start = Instant::now();
        let generated =
            generate_program_candidates_with_assumed_recognition(&registry, &generation, &choices)
                .unwrap();
        println!(
            "GENERATION {label}-completion us={}",
            start.elapsed().as_micros()
        );
        describe(&format!("{label}-completion-domain"), &generated);
        let first = generated.domain.variants[0].clone();
        let choices = generated.recognition;
        let completion = ProgramCompletionRequest {
            season: q.season,
            profile: q.profile.clone(),
            scoring_table: q.scoring_table,
            policy: q.policy.clone(),
            original: vec![CompletionEntry {
                id: "immutable-original".into(),
                locked: true,
                entry: ManualProgramEntry {
                    planned: first.element.clone(),
                    evidence: first.evidence.clone(),
                    intended_recognition: None,
                    scenario: None,
                },
            }],
            variants: generated.domain.variants.clone(),
            generated_domain: Some(generated.domain),
            exclusions: vec![],
            global_exclusions: vec![],
            insertion: CompletionInsertionPolicy {
                boundaries: vec![0, 1],
                absolute_positions: vec![("immutable-original".into(), 2)],
            },
            min_additions: 5,
            max_additions: 5,
            objective: q.objective,
            max_proposals: 200,
            max_solutions: 100,
            enumerate_equal_optima: true,
            time_limit: budget(),
        };
        describe(&format!("{label}-completion"), &completion);
        let result = measure(&format!("{label}-completion"), || {
            complete_program_with_assumed_recognition(
                &registry,
                &completion,
                &choices[..1],
                &choices,
            )
            .unwrap()
        });
        for candidate in &result.candidates {
            assert!(
                matches!(&candidate.provenance[2], CompletionProvenance::Original { id, original_index: 0, locked: true } if id == "immutable-original")
            );
            assert_eq!(candidate.program.elements[2], first.element);
        }
    }
    let registry = ProfileRegistry::senior();
    for source in senior::SOURCES {
        let generation = senior::generation(source, senior::Scale::Large);
        describe(source, &generation);
        let start = Instant::now();
        let generated = generate_program_candidates(&registry, &generation).unwrap();
        println!("GENERATION {source} us={}", start.elapsed().as_micros());
        describe(source, &generated);
        let mut full = senior::full(source, generated.variants.clone(), 11);
        full.time_limit = budget();
        describe(&format!("{source}-full"), &full);
        measure(&format!("{source}-full"), || {
            optimize_program(&registry, &full).unwrap()
        });
        let mut completion = senior::completion(source, senior::Scale::Large, generated);
        completion.time_limit = budget();
        describe(&format!("{source}-completion"), &completion);
        let result = measure(&format!("{source}-completion"), || {
            complete_program(&registry, &completion).unwrap()
        });
        for candidate in &result.candidates {
            senior::verify_originals(&completion, candidate);
        }
    }
}
