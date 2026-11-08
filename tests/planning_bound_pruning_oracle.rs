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
struct JumpAtom<const N: usize> {
    identity: (String, bool),
    stock: [u8; N],
    sequence: usize,
    compound: usize,
    three: usize,
    eulers: usize,
    euler_combination: usize,
}

// Adult technical requirements July 16 2026 pp.8–9: five jump entries,
// two compounds, one sequence, one three-jump compound, repetition <=2.
// The public grammar supplies permitted exact words, not composition feasibility.
fn jump_partitions<const N: usize>(
    registry: &ProfileRegistry,
    profile: &RuleProfile,
    stock_codes: [&str; N],
    initial_stock: [u8; N],
    max_entries: usize,
) -> BTreeSet<Identity> {
    let mut atoms = Vec::new();
    for assumption in selectable_jump_elements(
        registry,
        &profile.id,
        profile.scoring_table,
        RotationPreference::default(),
    )
    .unwrap()
    {
        let mut stock = [0; N];
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
        if permitted && stock.iter().zip(initial_stock).all(|(n, cap)| *n <= cap) {
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
    fn walk<const N: usize>(
        atoms: &[JumpAtom<N>],
        start: usize,
        left: [u8; N],
        budget: [usize; 5],
        chosen: &mut Identity,
        out: &mut BTreeSet<Identity>,
        max_entries: usize,
    ) {
        if left == [0; N] {
            out.insert(chosen.clone());
            return;
        }
        if chosen.len() == max_entries {
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
            if (0..N).any(|k| a.stock[k] > left[k])
                || (0..5).any(|k| cost[k] > budget[k])
                || (a.sequence > 0 && budget[4] == 0)
                || (a.euler_combination > 0 && budget[1] == 0)
            {
                continue;
            }
            let remaining = std::array::from_fn(|k| left[k] - a.stock[k]);
            let remaining_budget = std::array::from_fn(|k| budget[k] - cost[k]);
            chosen.push(a.identity.clone());
            walk(
                atoms,
                i,
                remaining,
                remaining_budget,
                chosen,
                out,
                max_entries,
            );
            chosen.pop();
        }
    }
    let mut result = BTreeSet::new();
    walk(
        &atoms,
        0,
        initial_stock,
        [2, 1, 1, 1, 1],
        &mut vec![],
        &mut result,
        max_entries,
    );
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
    let jumps = jump_partitions(
        &ProfileRegistry::adult(),
        &adult_profile(category),
        codes,
        [2; 4],
        5,
    );
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

#[test]
fn both_senior_profiles_match_their_complete_credited_optimum_partition_oracle() {
    for category in [SeniorCategory::Women, SeniorCategory::Men] {
        let registry = ProfileRegistry::senior();
        let profile = senior_profile(category);
        // Source fixture pp.4–5: two 4A, 4Lz/4F/4Lo, four distinct 1400-point
        // quints. Nine listed components fit six entries only as four quint
        // solos and a three-/two-jump pair. Both 4A are in compounds, so no loss.
        // Five listed quint kinds tie at 1400; every choice of four survives.
        let mut jumps = BTreeSet::new();
        let quints = ["5T", "5S", "5Lo", "5F", "5Lz"];
        for omitted in 0..quints.len() {
            let codes: [&str; 8] = ["4A", "4Lz", "4F", "4Lo"]
                .into_iter()
                .chain(
                    quints
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != omitted)
                        .map(|(_, s)| *s),
                )
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            jumps.extend(jump_partitions(
                &registry,
                &profile,
                codes,
                [2, 1, 1, 1, 1, 1, 1, 1],
                6,
            ));
        }
        assert_eq!(jumps.len(), 50);
        let mut expected = BTreeSet::new();
        for jump in &jumps {
            // Source SOV: combination 420 + flying 380 + StSq4 410 + two
            // choreographic elements 350 each = 1910. Exact abbreviations tie.
            for combination in ["CCoSp4", "FCCoSp4"] {
                for flying in ["FCSp4", "FLSp4"] {
                    let mut identity = jump.clone();
                    identity.extend(
                        [combination, flying, "StSq4", "ChSq1", "ChSp1"]
                            .into_iter()
                            .map(|s| (s.to_string(), false)),
                    );
                    identity.sort();
                    expected.insert(identity);
                }
            }
        }
        assert_eq!(expected.len(), 200);
        let request = PlanningSearchRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile: profile.id,
            scoring_table: profile.scoring_table,
            required: vec![],
            exclusions: vec![],
        };
        let PlanningSearchOutcome::ProvenMaximum(result) =
            search_maximum_bv(&registry, &request, &Continue).unwrap()
        else {
            panic!("expected a proved maximum");
        };
        assert_eq!(result.maximum_base_hundredths, 13310);
        assert_eq!(result.alternatives.len(), 5);
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
                let mut identity: Identity = p
                    .elements
                    .iter()
                    .map(|a| (a.code.clone(), a.direct_transitions_assumed))
                    .collect();
                identity.sort();
                identity
            })
            .collect();
        assert_eq!(actual.len(), 5);
        assert!(actual.is_subset(&expected));
    }
}
