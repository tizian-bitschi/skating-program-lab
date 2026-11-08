#![cfg(feature = "optimizer")]
use skating_program_lab::*;
use std::collections::BTreeSet;

type Identity = Vec<(String, bool)>;

struct Continue;
impl PlanningSearchControl for Continue {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        false
    }
}

#[derive(Clone)]
struct JumpAtom {
    identity: (String, bool),
    stock: [u8; 4],
    sequence: usize,
    compound: usize,
    three: usize,
    eulers: usize,
    euler_combination: usize,
}

// Adult technical requirements July 16 2026 pp.8–9: five jump entries,
// two compounds, one sequence, one three-jump compound, repetition <=2.
// The public grammar supplies permitted exact words, not composition feasibility.
fn jump_partitions(category: AdultCategory, stock_codes: [&str; 4]) -> BTreeSet<Identity> {
    let profile = adult_profile(category);
    let mut atoms = Vec::new();
    for assumption in selectable_jump_elements(
        &ProfileRegistry::adult(),
        &profile.id,
        profile.scoring_table,
        RotationPreference::default(),
    )
    .unwrap()
    {
        let mut stock = [0; 4];
        let mut permitted = true;
        for component in assumption
            .code
            .split('+')
            .filter(|s| *s != "SEQ" && *s != "Eu")
        {
            if let Some(i) = stock_codes.iter().position(|s| *s == component) {
                stock[i] += 1;
            } else {
                permitted = false;
            }
        }
        let length: u8 = stock.iter().sum();
        if permitted && stock.iter().all(|n| *n <= 2) {
            atoms.push(JumpAtom {
                identity: (
                    assumption.code.clone(),
                    assumption.direct_transitions_assumed,
                ),
                stock,
                sequence: usize::from(assumption.code.ends_with("+SEQ")),
                compound: usize::from(length > 1),
                three: usize::from(length == 3),
                eulers: assumption.code.split('+').filter(|s| *s == "Eu").count(),
                euler_combination: usize::from(
                    assumption.code.contains("Eu") && !assumption.code.ends_with("+SEQ"),
                ),
            });
        }
    }
    atoms.sort_by(|a, b| a.identity.cmp(&b.identity));
    fn walk(
        atoms: &[JumpAtom],
        start: usize,
        left: [u8; 4],
        budget: [usize; 5],
        chosen: &mut Identity,
        out: &mut BTreeSet<Identity>,
    ) {
        if left == [0; 4] {
            out.insert(chosen.clone());
            return;
        }
        if chosen.len() == 5 {
            return;
        }
        for i in start..atoms.len() {
            let a = &atoms[i];
            let cost = [
                a.compound,
                a.sequence,
                a.three,
                a.eulers,
                a.euler_combination,
            ];
            if (0..4).any(|k| a.stock[k] > left[k])
                || (0..5).any(|k| cost[k] > budget[k])
                || (a.sequence > 0 && budget[4] == 0)
                || (a.euler_combination > 0 && budget[1] == 0)
            {
                continue;
            }
            let remaining = std::array::from_fn(|k| left[k] - a.stock[k]);
            let remaining_budget = std::array::from_fn(|k| budget[k] - cost[k]);
            chosen.push(a.identity.clone());
            walk(atoms, i, remaining, remaining_budget, chosen, out);
            chosen.pop();
        }
    }
    let mut result = BTreeSet::new();
    walk(&atoms, 0, [2; 4], [2, 1, 1, 1, 1], &mut vec![], &mut result);
    result
}

// Independently transcribed role checks, without the production role matcher:
// Gold nonflying change-foot combination + flying spin;
// Masters nonflying combination + flying spin without foot change.
fn nonjump_partitions(
    category: AdultCategory,
    spin_total: u16,
    step_value: u16,
) -> BTreeSet<Identity> {
    let profile = adult_profile(category);
    let choices = selectable_elements(
        &ProfileRegistry::adult(),
        &profile.id,
        profile.scoring_table,
    )
    .unwrap();
    let spins: Vec<_> = choices
        .iter()
        .filter_map(|c| match c.kind {
            SelectableElementKind::RegularSpin(spin) => Some((c.row, spin)),
            _ => None,
        })
        .collect();
    let steps: Vec<_> = choices
        .iter()
        .filter(|c| {
            c.kind == SelectableElementKind::StepSequence && c.row.base_hundredths == step_value
        })
        .collect();
    let choreo: Vec<_> = choices
        .iter()
        .filter(|c| {
            c.kind == SelectableElementKind::ChoreographicSpin && c.row.base_hundredths == 350
        })
        .collect();
    assert!(!steps.is_empty() && !choreo.is_empty());
    let gold = category == AdultCategory::Gold;
    let mut result = BTreeSet::new();
    for (i, (a, sa)) in spins.iter().enumerate() {
        for (b, sb) in &spins[i + 1..] {
            if sa == sb || a.base_hundredths + b.base_hundredths != spin_total {
                continue;
            }
            let combination = |s: &SpinAbbreviation| {
                s.kind == RegularSpinKind::Combination
                    && s.entry == SpinEntry::NonFlying
                    && (!gold || s.change_foot)
            };
            let flying =
                |s: &SpinAbbreviation| s.entry == SpinEntry::Flying && (gold || !s.change_foot);
            if !((combination(sa) && flying(sb)) || (combination(sb) && flying(sa))) {
                continue;
            }
            for step in &steps {
                for ch in &choreo {
                    let mut identity = [a.notation, b.notation, step.row.notation, ch.row.notation]
                        .into_iter()
                        .map(|s| (s.to_string(), false))
                        .collect::<Identity>();
                    identity.sort();
                    result.insert(identity);
                }
            }
        }
    }
    result
}

fn compare(category: AdultCategory, codes: [&str; 4], jump_bv: u32, spin_bv: u16, step_bv: u16) {
    let jumps = jump_partitions(category, codes);
    let nonjumps = nonjump_partitions(category, spin_bv, step_bv);
    assert!(!jumps.is_empty() && !nonjumps.is_empty());
    let mut expected = BTreeSet::new();
    for jump in &jumps {
        for nonjump in &nonjumps {
            let mut identity = jump.clone();
            identity.extend(nonjump.clone());
            identity.sort();
            expected.insert(identity);
        }
    }
    let profile = adult_profile(category);
    let request = PlanningSearchRequest {
        personal_limits: PersonalLimits::default(),
        rotation_preference: RotationPreference::default(),
        profile: profile.id,
        scoring_table: profile.scoring_table,
        required: vec![],
        exclusions: vec![],
    };
    let PlanningSearchOutcome::ProvenMaximum(result) =
        search_maximum_bv(&ProfileRegistry::adult(), &request, &Continue).unwrap()
    else {
        panic!("maximum not proved");
    };
    assert_eq!(
        result.maximum_base_hundredths,
        jump_bv + u32::from(spin_bv + step_bv + 350)
    );
    assert_eq!(
        result.alternative_status,
        PlanningAlternativeStatus::LimitReached
    );
    let actual: BTreeSet<_> = result
        .alternatives
        .iter()
        .map(|p| {
            assert!(p
                .assessment
                .legality
                .validation
                .validation
                .findings
                .is_empty());
            let mut identity = p
                .elements
                .iter()
                .map(|a| (a.code.clone(), a.direct_transitions_assumed))
                .collect::<Identity>();
            identity.sort();
            identity
        })
        .collect();
    assert_eq!(
        actual.len(),
        result.alternatives.len(),
        "duplicate exact identities"
    );
    // Independently enumerate the whole optimum set while preserving the public
    // five-result contract. The historical uncapped experiment checked equality.
    assert_eq!(
        expected.len(),
        if category == AdultCategory::Gold {
            526
        } else {
            244
        }
    );
    assert_eq!(actual.len(), 5);
    assert!(
        actual.is_subset(&expected),
        "returned an unexpected exact optimum"
    );
}

#[test]
fn gold_five_optima_belong_to_the_independent_complete_partition_oracle() {
    // Communication2786 and existing source fixtures: 2*(2Lo+2S+2T+1A)=1080.
    compare(
        AdultCategory::Gold,
        ["2Lo", "2S", "2T", "1A"],
        1080,
        720,
        270,
    );
}

#[test]
fn masters_five_optima_belong_to_the_independent_complete_partition_oracle() {
    // Communication2786: 2*(2A+2Lz+2F+2Lo)=1780; this stock satisfies Axel.
    compare(
        AdultCategory::Masters,
        ["2A", "2Lz", "2F", "2Lo"],
        1780,
        800,
        410,
    );
}
