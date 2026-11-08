use super::*;
use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        branching::{
            branchers::independent_variable_value_brancher::IndependentVariableValueBrancher,
            value_selection::InDomainMax, variable_selection::InputOrder,
        },
        optimisation::{linear_sat_unsat::LinearSatUnsat, OptimisationDirection},
        predicate,
        results::{OptimisationResult, ProblemSolution, SatisfactionResult, SolutionReference},
        termination::TerminationCondition,
        variables::{DomainId, TransformableVariable},
    },
    Solver,
};
use std::ops::ControlFlow;
type CountBrancher = IndependentVariableValueBrancher<DomainId, InputOrder<DomainId>, InDomainMax>;

struct Cancellation<'a, C>(&'a C);
impl<C: PlanningSearchControl> TerminationCondition for Cancellation<'_, C> {
    fn should_stop(&mut self) -> bool {
        self.0.is_cancelled(PlanningSearchPhase::Solve)
    }
}

// Relax compound compatibility, retaining component repetition, listed-jump
// capacity, distinct abbreviations and spin roles. Every feasible composition
// belongs to this relaxation, so its best value is an admissible upper bound.
fn value_upper_bound(
    atoms: &[Atom],
    profile: &RuleProfile,
    policy: &ProgramPolicy,
    request: &PlanningSearchRequest,
) -> (i32, i32, i32) {
    let required: Vec<_> = request
        .required
        .iter()
        .filter_map(|r| atoms.iter().find(|a| a.assumption == r.element))
        .collect();
    let senior = crate::senior::selected(profile, ProfileRule::Repetition);
    let jump_bound: i32 = if senior {
        super::senior_bound::jump_upper_bound(atoms, profile, request)
    } else {
        let required_jumps: Vec<_> = required.iter().flat_map(|a| a.jumps.iter()).collect();
        let remaining =
            profile.jumps.max_jumps - required.iter().filter(|a| !a.jumps.is_empty()).count();
        let compounds = profile.jumps.max_compounds.min(
            required
                .iter()
                .filter(|a| a.combination() || a.sequence())
                .count()
                + remaining,
        );
        let required_compounds = required
            .iter()
            .filter(|a| a.combination() || a.sequence())
            .count();
        let threes = profile.jumps.max_three_jump_compounds.min(
            required.iter().filter(|a| a.jumps.len() == 3).count() + compounds - required_compounds,
        );
        let listed_slots =
            profile.jumps.max_jumps + compounds + threes * (profile.jumps.max_compound_length - 2);
        let solos: Vec<_> = atoms
            .iter()
            .filter(|a| matches!(a.element, PlannedElement::Jump(JumpElement::Solo(_))))
            .collect();
        // Same-direction compounds only add T/Lo, an Euler continuation, or
        // sequence Axels beyond their first jump. A sequence excludes a separate
        // Euler combination, and its internal Euler shares the listed-jump cap.
        let is_head =
            |jump: &ListedJump| !matches!(jump.kind, Some(JumpKind::ToeLoop | JumpKind::Loop));
        let mut heads_left = if request.rotation_preference == RotationPreference::SameDirection {
            let sequence_extra =
                profile.jumps.max_sequences * (profile.jumps.max_compound_length - 1);
            (profile.jumps.max_jumps + sequence_extra.max(profile.jumps.max_eulers))
                .saturating_sub(required_jumps.iter().filter(|j| is_head(j)).count())
        } else {
            usize::MAX
        };
        let mut components: Vec<_> = solos
            .iter()
            .flat_map(|a| {
                std::iter::repeat_n(
                    (a.value, is_head(&a.jumps[0])),
                    profile.jumps.max_repetitions
                        - required_jumps.iter().filter(|j| ***j == a.jumps[0]).count(),
                )
            })
            .collect();
        components.sort_unstable_by(|a, b| b.cmp(a));
        required
            .iter()
            .filter(|a| !a.jumps.is_empty())
            .map(|a| a.value)
            .sum::<i32>()
            + components
                .into_iter()
                .filter_map(|(value, head)| {
                    if head && heads_left == 0 {
                        return None;
                    }
                    if head {
                        heads_left -= 1;
                    }
                    Some(value)
                })
                .take(listed_slots - required_jumps.len())
                .sum::<i32>()
    };
    let mut spin_values: Vec<(SpinAbbreviation, i32)> = Vec::new();
    for atom in atoms {
        if let Some(spin) = atom.spin {
            if let Some((_, value)) = spin_values.iter_mut().find(|(s, _)| *s == spin) {
                *value = (*value).max(atom.value);
            } else {
                spin_values.push((spin, atom.value));
            }
        }
    }
    fn spin_bound(
        choices: &[(SpinAbbreviation, i32)],
        policy: &RegularSpinPolicy,
        selected: &mut Vec<SpinAbbreviation>,
        slots: usize,
    ) -> i32 {
        let mut best = if policy.required.iter().all(|g| {
            selected
                .iter()
                .any(|s| crate::spin_validation::matches_spin_role(g, *s))
        }) {
            0
        } else {
            i32::MIN / 2
        };
        if slots > 0 {
            for (i, (spin, value)) in choices.iter().enumerate() {
                selected.push(*spin);
                best = best.max(value + spin_bound(&choices[i + 1..], policy, selected, slots - 1));
                selected.pop();
            }
        }
        best
    }
    let mut required_spins: Vec<_> = required.iter().filter_map(|a| a.spin).collect();
    let required_choreographic_spins = required
        .iter()
        .filter(|a| a.family() == ProfileElementFamily::ChoreographicSpin)
        .count();
    // Choreographic spins consume the same overall spin allowance.
    let choreographic_slots =
        required_choreographic_spins.max(usize::from(policy.require_choreographic_spin));
    spin_values.retain(|(s, _)| !required_spins.contains(s));
    let spin_bound = required
        .iter()
        .filter(|a| a.spin.is_some())
        .map(|a| a.value)
        .sum::<i32>()
        + spin_bound(
            &spin_values,
            &policy.spins,
            &mut required_spins,
            policy.spins.max_spins.saturating_sub(
                required.iter().filter(|a| a.spin.is_some()).count() + choreographic_slots,
            ),
        )
        .max(0);
    let family_bound = |family, cap: usize| {
        let reserved: Vec<_> = required.iter().filter(|a| a.family() == family).collect();
        reserved.iter().map(|a| a.value).sum::<i32>()
            + atoms
                .iter()
                .filter(|a| a.family() == family)
                .map(|a| a.value)
                .max()
                .unwrap_or(0)
                * (cap - reserved.len()) as i32
    };
    let total = jump_bound
        + spin_bound
        + family_bound(ProfileElementFamily::StepSequence, policy.max_steps)
        + family_bound(
            ProfileElementFamily::ChoreographicSequence,
            policy.max_choreographic_sequences,
        )
        + family_bound(
            ProfileElementFamily::ChoreographicSpin,
            policy.max_choreographic_spins,
        );
    (jump_bound, spin_bound, total)
}

fn atom_upper(atom: &Atom, profile: &RuleProfile, required: &[&Atom]) -> usize {
    let senior = crate::senior::selected(profile, ProfileRule::Repetition);
    let mut upper = if atom.spin.is_some() {
        1
    } else if !atom.jumps.is_empty() {
        profile.jumps.max_jumps
    } else {
        1
    };
    if atom.combination() {
        upper = upper
            .min(profile.jumps.max_combinations)
            .min(profile.jumps.max_compounds);
    }
    if atom.sequence() {
        upper = upper
            .min(profile.jumps.max_sequences)
            .min(profile.jumps.max_compounds);
    }
    if let Some(limit) = profile.jumps.max_eulers.checked_div(atom.eulers) {
        upper = upper.min(limit);
    }
    for jump in &atom.jumps {
        upper = upper.min(
            if senior && jump.rotations == Some(1) {
                crate::senior::DEFINITION.max_kind_occurrences
            } else {
                profile.jumps.max_repetitions
            } / atom.jumps.iter().filter(|j| *j == jump).count(),
        );
    }
    if senior {
        for jump in &atom.jumps {
            let same_kind = atom.jumps.iter().filter(|j| j.kind == jump.kind).count();
            upper = upper.min(crate::senior::DEFINITION.max_kind_occurrences / same_kind);
        }
    }
    if !atom.jumps.is_empty() {
        let reserved = required.iter().filter(|a| !a.jumps.is_empty()).count();
        let own = required
            .iter()
            .filter(|a| a.assumption == atom.assumption)
            .count();
        upper = upper.min(profile.jumps.max_jumps - reserved + own);
    }
    for (select, cap) in [
        (
            Atom::combination as fn(&Atom) -> bool,
            profile.jumps.max_combinations,
        ),
        (
            Atom::sequence as fn(&Atom) -> bool,
            profile.jumps.max_sequences,
        ),
        (
            (|a: &Atom| a.combination() || a.sequence()) as fn(&Atom) -> bool,
            profile.jumps.max_compounds,
        ),
    ] {
        if select(atom) {
            let reserved = required.iter().filter(|a| select(a)).count();
            let own = required
                .iter()
                .filter(|a| a.assumption == atom.assumption)
                .count();
            upper = upper.min(cap - reserved + own);
        }
    }
    upper
}

pub(super) fn search(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    profile: &RuleProfile,
    policy: &ProgramPolicy,
    atoms: &[Atom],
    control: &impl PlanningSearchControl,
) -> Result<PlanningSearchOutcome, PlanningSearchError> {
    let senior = crate::senior::selected(profile, ProfileRule::Repetition);
    let required: Vec<_> = request
        .required
        .iter()
        .filter_map(|r| atoms.iter().find(|a| a.assumption == r.element))
        .collect();
    let mut solver = Solver::default();
    let (jump_bound, spin_bound, upper) = value_upper_bound(atoms, profile, policy, request);
    let max_elements = profile.jumps.max_jumps
        + policy.spins.max_spins
        + policy.max_steps
        + policy.max_choreographic_sequences;
    // Bound absolute activity before Pumpkin's i32 linear arithmetic, including
    // all independently bounded variables rather than only a feasible solution.
    let activity = atoms
        .iter()
        .map(|a| i64::from(a.value) * atom_upper(a, profile, &required) as i64)
        .sum::<i64>()
        + i64::from(upper)
        + if senior {
            atoms
                .iter()
                .filter(|a| {
                    matches!(a.element, PlannedElement::Jump(JumpElement::Solo(_)))
                        && a.jumps[0].rotations.is_some_and(|r| r >= 3)
                })
                .map(|a| {
                    i64::from(
                        a.value - i32::from(crate::scoring::repetition_hundredths(a.value as u16)),
                    )
                })
                .sum::<i64>()
        } else {
            0
        };
    if activity > i64::from(i32::MAX) {
        return Err(PlanningSearchError::UnsupportedCoverage {
            profile: request.profile.clone(),
            reason: "Complete count model exceeds supported exact integer arithmetic.",
        });
    }
    let mut counts = Vec::new();
    for atom in atoms {
        if control.is_cancelled(PlanningSearchPhase::Model) {
            return Ok(PlanningSearchOutcome::Cancelled);
        }
        let upper = atom_upper(atom, profile, &required);
        counts.push(solver.new_bounded_integer(0, upper as i32));
    }
    // Allocate variables before posting: Pumpkin forbids creating a variable
    // after a supported contradiction has made the root state inconsistent.
    let sequence_count = solver.new_bounded_integer(0, profile.jumps.max_sequences as i32);
    let euler_combination_count = solver.new_bounded_integer(0, profile.jumps.max_compounds as i32);
    let objective = solver.new_bounded_integer(0, upper);
    let jump_value = solver.new_bounded_integer(0, jump_bound);
    let spin_value = solver.new_bounded_integer(0, spin_bound);
    let advanced: Vec<_> = if senior {
        atoms
            .iter()
            .filter(|a| {
                a.jumps.len() == 1
                    && !a.combination()
                    && !a.sequence()
                    && a.jumps[0].rotations.is_some_and(|r| r >= 3)
            })
            .map(|a| {
                (
                    a.jumps[0].clone(),
                    a.value - i32::from(crate::scoring::repetition_hundredths(a.value as u16)),
                    solver.new_bounded_integer(0, 2),
                    solver.new_bounded_integer(0, 2),
                    solver.new_bounded_integer(0, 1),
                    solver.new_bounded_integer(0, 1),
                )
            })
            .collect()
    } else {
        vec![]
    };
    for (atom, &count) in atoms.iter().zip(&counts) {
        let minimum = request
            .required
            .iter()
            .filter(|r| r.element == atom.assumption)
            .count() as i32;
        let tag = solver.new_constraint_tag();
        solver.add_clause(vec![predicate!(count >= minimum)], tag);
    }
    let mut constrain = |coeff: &dyn Fn(&Atom) -> i32, low: i32, high: i32| {
        let terms: Vec<_> = atoms
            .iter()
            .zip(&counts)
            .filter_map(|(a, v)| {
                let n = coeff(a);
                (n != 0).then(|| v.scaled(n))
            })
            .collect();
        if terms.is_empty() {
            if low > 0 {
                let tag = solver.new_constraint_tag();
                solver.add_clause(vec![], tag);
            }
            return;
        }
        let tag = solver.new_constraint_tag();
        solver
            .add_constraint(pumpkin_solver::less_than_or_equals(
                terms.clone(),
                high,
                tag,
            ))
            .post();
        if low > 0 {
            let tag = solver.new_constraint_tag();
            solver
                .add_constraint(pumpkin_solver::greater_than_or_equals(terms, low, tag))
                .post();
        }
    };
    constrain(
        &|a| i32::from(!a.jumps.is_empty()),
        0,
        profile.jumps.max_jumps as i32,
    );
    constrain(
        &|a| i32::from(a.combination()),
        0,
        profile.jumps.max_combinations as i32,
    );
    constrain(
        &|a| i32::from(a.sequence()),
        0,
        profile.jumps.max_sequences as i32,
    );
    constrain(
        &|a| i32::from(a.combination() || a.sequence()),
        0,
        profile.jumps.max_compounds as i32,
    );
    constrain(
        &|a| i32::from(a.jumps.len() == 3),
        0,
        profile.jumps.max_three_jump_compounds as i32,
    );
    constrain(&|a| a.eulers as i32, 0, profile.jumps.max_eulers as i32);
    for permission in &profile.jumps.allowed {
        if senior {
            constrain(
                &|a| {
                    a.jumps
                        .iter()
                        .filter(|j| j.kind == Some(permission.kind))
                        .count() as i32
                },
                0,
                crate::senior::DEFINITION.max_kind_occurrences as i32,
            );
        }
        for rotations in permission.min_rotations..=permission.max_rotations.min(5) {
            constrain(
                &|a| {
                    a.jumps
                        .iter()
                        .filter(|j| {
                            j.kind == Some(permission.kind) && j.rotations == Some(rotations)
                        })
                        .count() as i32
                },
                0,
                if senior && rotations == 1 {
                    crate::senior::DEFINITION.max_kind_occurrences as i32
                } else {
                    profile.jumps.max_repetitions as i32
                },
            );
        }
    }
    if profile.jumps.require_axel {
        constrain(
            &|a| i32::from(a.jumps.iter().any(|j| j.kind == Some(JumpKind::Axel))),
            1,
            max_elements as i32,
        );
    }
    for (family, cap) in [
        (ProfileElementFamily::StepSequence, policy.max_steps),
        (
            ProfileElementFamily::ChoreographicSequence,
            policy.max_choreographic_sequences,
        ),
        (
            ProfileElementFamily::ChoreographicSpin,
            policy.max_choreographic_spins,
        ),
    ] {
        constrain(
            &|a| i32::from(a.family() == family),
            i32::from(
                policy.require_choreographic_spin
                    && family == ProfileElementFamily::ChoreographicSpin,
            ),
            cap as i32,
        );
    }
    constrain(
        &|a| i32::from(a.spin.is_some() || a.family() == ProfileElementFamily::ChoreographicSpin),
        0,
        policy.spins.max_spins as i32,
    );
    let mut abbreviations = Vec::new();
    for atom in atoms {
        if let Some(a) = atom.spin {
            if !abbreviations.contains(&a) {
                abbreviations.push(a);
            }
        }
    }
    for abbreviation in abbreviations {
        constrain(&|a| i32::from(a.spin == Some(abbreviation)), 0, 1);
    }
    for group in &policy.spins.required {
        constrain(
            &|a| {
                i32::from(
                    a.spin
                        .is_some_and(|s| crate::spin_validation::matches_spin_role(group, s)),
                )
            },
            1,
            policy.spins.max_spins as i32,
        );
    }
    for family in &profile.families.required {
        constrain(
            &|a| i32::from(a.family() == *family),
            1,
            max_elements as i32,
        );
    }
    for (aggregate, select) in [
        (
            sequence_count,
            (|a: &Atom| a.sequence()) as fn(&Atom) -> bool,
        ),
        (
            euler_combination_count,
            (|a: &Atom| a.combination() && a.eulers > 0) as fn(&Atom) -> bool,
        ),
    ] {
        let mut terms: Vec<_> = atoms
            .iter()
            .zip(&counts)
            .filter(|(a, _)| select(a))
            .map(|(_, v)| v.scaled(1))
            .collect();
        terms.push(aggregate.scaled(-1));
        let tag = solver.new_constraint_tag();
        solver
            .add_constraint(pumpkin_solver::equals(terms, 0, tag))
            .post();
    }
    let tag = solver.new_constraint_tag();
    solver.add_clause(
        vec![
            predicate!(sequence_count == 0),
            predicate!(euler_combination_count == 0),
        ],
        tag,
    );
    for (jump, _, total, solos, repeated, reduced) in &advanced {
        for (aggregate, solo_only) in [(*total, false), (*solos, true)] {
            let mut terms: Vec<_> = atoms
                .iter()
                .zip(&counts)
                .filter_map(|(a, v)| {
                    let n = if solo_only && (a.combination() || a.sequence()) {
                        0
                    } else {
                        a.jumps.iter().filter(|j| *j == jump).count() as i32
                    };
                    (n != 0).then(|| v.scaled(n))
                })
                .collect();
            terms.push(aggregate.scaled(-1));
            let tag = solver.new_constraint_tag();
            solver
                .add_constraint(pumpkin_solver::equals(terms, 0, tag))
                .post();
        }
        for (aggregate, flag) in [(*total, *repeated), (*solos, *reduced)] {
            let tag = solver.new_constraint_tag();
            solver.add_clause(vec![predicate!(aggregate <= 1), predicate!(flag == 1)], tag);
            let tag = solver.new_constraint_tag();
            solver.add_clause(vec![predicate!(aggregate >= 2), predicate!(flag == 0)], tag);
        }
    }
    for (aggregate, jumps) in [(jump_value, true), (spin_value, false)] {
        let mut terms: Vec<_> = atoms
            .iter()
            .zip(&counts)
            .filter(|(a, _)| {
                if jumps {
                    !a.jumps.is_empty()
                } else {
                    a.spin.is_some()
                }
            })
            .map(|(a, v)| v.scaled(a.value))
            .collect();
        if jumps {
            terms.extend(
                advanced
                    .iter()
                    .map(|(_, loss, _, _, _, reduced)| reduced.scaled(-loss)),
            );
        }
        terms.push(aggregate.scaled(-1));
        let tag = solver.new_constraint_tag();
        solver
            .add_constraint(pumpkin_solver::equals(terms, 0, tag))
            .post();
    }
    for (high_only, cap) in [
        (false, crate::senior::DEFINITION.max_repeated_advanced),
        (true, crate::senior::DEFINITION.max_repeated_quad_or_quint),
    ] {
        let terms: Vec<_> = advanced
            .iter()
            .filter(|(j, ..)| !high_only || j.rotations.is_some_and(|r| r >= 4))
            .map(|(_, _, _, _, flag, _)| flag.scaled(1))
            .collect();
        if !terms.is_empty() {
            let tag = solver.new_constraint_tag();
            solver
                .add_constraint(pumpkin_solver::less_than_or_equals(terms, cap as i32, tag))
                .post();
        }
    }
    // Exact credited section totals strengthen propagation across the full repertoire.
    let mut terms: Vec<_> = atoms
        .iter()
        .zip(&counts)
        .filter(|(a, _)| a.jumps.is_empty() && a.spin.is_none())
        .map(|(a, v)| v.scaled(a.value))
        .collect();
    terms.extend([
        jump_value.scaled(1),
        spin_value.scaled(1),
        objective.scaled(-1),
    ]);
    let tag = solver.new_constraint_tag();
    solver
        .add_constraint(pumpkin_solver::equals(terms, 0, tag))
        .post();
    if control.is_cancelled(PlanningSearchPhase::Solve) {
        return Ok(PlanningSearchOutcome::Cancelled);
    }
    let mut ordered: Vec<_> = atoms.iter().zip(&counts).collect();
    ordered.sort_by(|(a, _), (b, _)| {
        b.value
            .cmp(&a.value)
            .then(a.assumption.code.cmp(&b.assumption.code))
    });
    let mut variables = vec![objective, jump_value, spin_value];
    variables.extend(ordered.into_iter().map(|(_, v)| *v));
    variables.extend([sequence_count, euler_combination_count]);
    variables.extend(
        advanced
            .iter()
            .flat_map(|(_, _, total, solos, repeated, reduced)| {
                [*total, *solos, *repeated, *reduced]
            }),
    );
    let mut brancher = CountBrancher::new(InputOrder::new(&variables), InDomainMax);
    let mut incumbent_error = None;
    let mut incumbent_cancelled = false;
    let outcome = solver.optimise(
        &mut brancher,
        &mut Cancellation(control),
        &mut ResolutionResolver::default(),
        LinearSatUnsat::new(
            OptimisationDirection::Maximise,
            objective,
            |_: &Solver, solution: SolutionReference, _: &CountBrancher, _: &ResolutionResolver| {
                let selected: Vec<_> = counts
                    .iter()
                    .map(|&v| solution.get_integer_value(v))
                    .collect();
                match reconstruct(
                    registry,
                    request,
                    profile,
                    atoms,
                    &selected,
                    solution.get_integer_value(objective) as u32,
                    control,
                ) {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        incumbent_cancelled = true;
                        return ControlFlow::<()>::Break(());
                    }
                    Err(error) => {
                        incumbent_error = Some(error);
                        return ControlFlow::<()>::Break(());
                    }
                }
                if control.is_cancelled(PlanningSearchPhase::IncumbentFound) {
                    incumbent_cancelled = true;
                    ControlFlow::<()>::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        ),
    );
    if let Some(error) = incumbent_error {
        return Err(error);
    }
    if incumbent_cancelled {
        return Ok(PlanningSearchOutcome::Cancelled);
    }
    let solution=match outcome {
        OptimisationResult::Optimal(s)=>s,
        OptimisationResult::Unsatisfiable=>return Ok(PlanningSearchOutcome::Infeasible {diagnostics:vec![PlanningDiagnostic {required_input_indices:(0..request.required.len()).collect(),rule_id:None,message:"No permitted composition satisfies the required occurrences, exclusions and profile roles.".into()}]}),
        _=>return Ok(PlanningSearchOutcome::Cancelled),
    };
    let maximum = solution.get_integer_value(objective) as u32;
    let mut selected: Vec<_> = counts
        .iter()
        .map(|&v| solution.get_integer_value(v))
        .collect();
    let Some(first) = reconstruct(
        registry, request, profile, atoms, &selected, maximum, control,
    )?
    else {
        return Ok(PlanningSearchOutcome::Cancelled);
    };
    let mut alternatives = vec![first];
    let mut status = PlanningAlternativeStatus::Cancelled;
    let mut vectors = vec![selected.clone()];
    let mut expanded = 0;
    let mut blocked = 0;
    let minimum: Vec<_> = atoms
        .iter()
        .map(|a| {
            request
                .required
                .iter()
                .filter(|r| r.element == a.assumption)
                .count() as i32
        })
        .collect();
    if !control.is_cancelled(PlanningSearchPhase::MaximumProved) {
        let tag = solver.new_constraint_tag();
        solver.add_clause(vec![predicate!(objective == maximum as i32)], tag);
        'collect: loop {
            if control.is_cancelled(PlanningSearchPhase::Alternatives) {
                break;
            }
            if alternatives.len() == 5 {
                status = PlanningAlternativeStatus::LimitReached;
                break;
            }
            // Keep exact atoms: permitted compound order aliases preserve every
            // count coefficient and contextual solo credit, without another proof.
            while expanded < vectors.len() {
                let base = vectors[expanded].clone();
                expanded += 1;
                for (source, atom) in atoms.iter().enumerate() {
                    if base[source] <= minimum[source] || !(atom.combination() || atom.sequence()) {
                        continue;
                    }
                    for (target, alias) in atoms.iter().enumerate() {
                        if control.is_cancelled(PlanningSearchPhase::Alternatives) {
                            break 'collect;
                        }
                        if source == target || !compound_alias(atom, alias) {
                            continue;
                        }
                        let mut next = base.clone();
                        next[source] -= 1;
                        next[target] += 1;
                        if vectors.contains(&next) {
                            continue;
                        }
                        let Some(candidate) = reconstruct(
                            registry, request, profile, atoms, &next, maximum, control,
                        )?
                        else {
                            break 'collect;
                        };
                        alternatives.push(candidate);
                        vectors.push(next);
                        if alternatives.len() == 5 {
                            status = PlanningAlternativeStatus::LimitReached;
                            break 'collect;
                        }
                    }
                }
            }
            // Strict supermultisets cannot share the fixed optimum in these
            // positive-credit profiles. Retain full-vector blocking otherwise.
            while blocked < vectors.len() {
                let vector = &vectors[blocked];
                let positive = atoms.iter().all(|a| a.value > 0);
                let block: Vec<_> = counts
                    .iter()
                    .zip(vector)
                    .filter_map(|(&v, &n)| {
                        if positive {
                            (n > 0).then(|| predicate!(v <= n - 1))
                        } else {
                            Some(predicate!(v != n))
                        }
                    })
                    .collect();
                let tag = solver.new_constraint_tag();
                solver.add_clause(block, tag);
                blocked += 1;
                if control.is_cancelled(PlanningSearchPhase::Alternatives) {
                    break 'collect;
                }
            }
            let mut termination = CollectionCancellation(control);
            let mut resolver = ResolutionResolver::default();
            let next = match solver.satisfy(&mut brancher, &mut termination, &mut resolver) {
                SatisfactionResult::Satisfiable(s) => Some(
                    counts
                        .iter()
                        .map(|&v| s.solution().get_integer_value(v))
                        .collect::<Vec<_>>(),
                ),
                SatisfactionResult::Unsatisfiable(..) => {
                    status = PlanningAlternativeStatus::Exhausted;
                    None
                }
                SatisfactionResult::Unknown(..) => None,
            };
            let Some(next) = next else { break };
            selected = next;
            let Some(candidate) = reconstruct(
                registry, request, profile, atoms, &selected, maximum, control,
            )?
            else {
                break;
            };
            alternatives.push(candidate);
            vectors.push(selected.clone());
        }
    }
    Ok(PlanningSearchOutcome::ProvenMaximum(PlanningMaximum {
        rotation_preference: request.rotation_preference,
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        maximum_base_hundredths: maximum,
        alternatives,
        alternative_status: status,
    }))
}

fn compound_alias(a: &Atom, b: &Atom) -> bool {
    a.combination() == b.combination()
        && a.sequence() == b.sequence()
        && a.eulers == b.eulers
        && a.value == b.value
        && a.jumps.len() == b.jumps.len()
        && a.jumps.iter().all(|j| {
            a.jumps.iter().filter(|x| *x == j).count() == b.jumps.iter().filter(|x| *x == j).count()
        })
}

struct CollectionCancellation<'a, C>(&'a C);
impl<C: PlanningSearchControl> TerminationCondition for CollectionCancellation<'_, C> {
    fn should_stop(&mut self) -> bool {
        self.0.is_cancelled(PlanningSearchPhase::Alternatives)
    }
}

fn reconstruct(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    profile: &RuleProfile,
    atoms: &[Atom],
    selected: &[i32],
    maximum: u32,
    control: &impl PlanningSearchControl,
) -> Result<Option<PlanningProgram>, PlanningSearchError> {
    if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
        return Ok(None);
    }
    let mut elements = Vec::new();
    let mut provenance = Vec::new();
    for (atom, &n) in atoms.iter().zip(selected) {
        if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
            return Ok(None);
        }
        let required: Vec<_> = request
            .required
            .iter()
            .enumerate()
            .filter(|(_, r)| r.element == atom.assumption)
            .collect();
        let n = n as usize;
        if n < required.len() {
            return Err(PlanningSearchError::InconsistentAssessment);
        }
        for i in 0..n {
            if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
                return Ok(None);
            }
            elements.push(atom.assumption.clone());
            provenance.push(if let Some((input_index, r)) = required.get(i) {
                PlanningProvenance::Required {
                    id: r.id.clone(),
                    input_index: *input_index,
                }
            } else {
                PlanningProvenance::Added
            });
        }
    }
    let (assessment, program, _) = crate::manual_bv::assess_selections(
        registry,
        profile,
        &manual(request, elements.clone(), SubmissionState::Complete),
    )
    .map_err(PlanningSearchError::Profile)?;
    if assessment.total.exact_hundredths != Some(i64::from(maximum))
        || !assessment
            .legality
            .validation
            .validation
            .findings
            .is_empty()
    {
        return Err(PlanningSearchError::InconsistentAssessment);
    }
    if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
        return Ok(None);
    }
    Ok(Some(PlanningProgram {
        program,
        elements,
        provenance,
        assessment,
    }))
}
