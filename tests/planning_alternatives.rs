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
