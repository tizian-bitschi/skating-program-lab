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
        results::{OptimisationResult, ProblemSolution, SolutionReference},
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
    let threes = profile
        .jumps
        .max_three_jump_compounds
        .min(required.iter().filter(|a| a.jumps.len() == 3).count() + remaining);
    let listed_slots =
        profile.jumps.max_jumps + compounds + threes * (profile.jumps.max_compound_length - 2);
    let solos: Vec<_> = atoms
        .iter()
        .filter(|a| matches!(a.element, PlannedElement::Jump(JumpElement::Solo(_))))
        .collect();
    let mut components: Vec<_> = solos
        .iter()
        .flat_map(|a| {
            std::iter::repeat_n(
                a.value,
                profile.jumps.max_repetitions
                    - required_jumps.iter().filter(|j| ***j == a.jumps[0]).count(),
            )
        })
        .collect();
    components.sort_unstable_by(|a, b| b.cmp(a));
    let jump_bound: i32 = required
        .iter()
        .filter(|a| !a.jumps.is_empty())
        .map(|a| a.value)
        .sum::<i32>()
        + components
            .into_iter()
            .take(listed_slots - required_jumps.len())
            .sum::<i32>();
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
            policy.spins.max_spins - required.iter().filter(|a| a.spin.is_some()).count(),
        )
        .max(0);
    let family_bound = |family, cap: usize| {
        atoms
            .iter()
            .filter(|a| a.family() == family)
            .map(|a| a.value)
            .max()
            .unwrap_or(0)
            * cap as i32
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

pub(super) fn search(
    registry: &ProfileRegistry,
    request: &PlanningSearchRequest,
    profile: &RuleProfile,
    policy: &ProgramPolicy,
    atoms: &[Atom],
    control: &impl PlanningSearchControl,
) -> Result<PlanningSearchOutcome, PlanningSearchError> {
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
        .map(|a| i64::from(a.value) * max_elements as i64)
        .sum::<i64>()
        + i64::from(upper);
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
                profile.jumps.max_repetitions / atom.jumps.iter().filter(|j| *j == jump).count(),
            );
        }
        counts.push(solver.new_bounded_integer(0, upper as i32));
    }
    // Allocate variables before posting: Pumpkin forbids creating a variable
    // after a supported contradiction has made the root state inconsistent.
    let sequence_count = solver.new_bounded_integer(0, profile.jumps.max_sequences as i32);
    let euler_combination_count = solver.new_bounded_integer(0, profile.jumps.max_compounds as i32);
    let objective = solver.new_bounded_integer(0, upper);
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
        for rotations in permission.min_rotations..=permission.max_rotations {
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
                profile.jumps.max_repetitions as i32,
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
    constrain(
        &|a| if a.jumps.is_empty() { 0 } else { a.value },
        0,
        jump_bound,
    );
    constrain(
        &|a| if a.spin.is_some() { a.value } else { 0 },
        0,
        spin_bound,
    );
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
    // Exact catalog values; absolute activity was checked before construction.
    let mut terms: Vec<_> = atoms
        .iter()
        .zip(&counts)
        .map(|(a, v)| v.scaled(a.value))
        .collect();
    terms.push(objective.scaled(-1));
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
    let mut variables = vec![objective];
    variables.extend(ordered.into_iter().map(|(_, v)| *v));
    variables.extend([sequence_count, euler_combination_count]);
    let mut brancher = CountBrancher::new(InputOrder::new(&variables), InDomainMax);
    let outcome = solver.optimise(
        &mut brancher,
        &mut Cancellation(control),
        &mut ResolutionResolver::default(),
        LinearSatUnsat::new(
            OptimisationDirection::Maximise,
            objective,
            |_: &Solver, _: SolutionReference, _: &CountBrancher, _: &ResolutionResolver| {
                if control.is_cancelled(PlanningSearchPhase::IncumbentFound) {
                    ControlFlow::<()>::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        ),
    );
    let solution=match outcome {
        OptimisationResult::Optimal(s)=>s,
        OptimisationResult::Unsatisfiable=>return Ok(PlanningSearchOutcome::Infeasible {diagnostics:vec![PlanningDiagnostic {required_input_indices:(0..request.required.len()).collect(),rule_id:None,message:"No permitted composition satisfies the required occurrences, exclusions and profile roles.".into()}]}),
        _=>return Ok(PlanningSearchOutcome::Cancelled),
    };
    if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
        return Ok(PlanningSearchOutcome::Cancelled);
    }
    let mut elements = Vec::new();
    let mut provenance = Vec::new();
    for (atom, &count) in atoms.iter().zip(&counts) {
        if control.is_cancelled(PlanningSearchPhase::Reconstruction) {
            return Ok(PlanningSearchOutcome::Cancelled);
        }
        let required: Vec<_> = request
            .required
            .iter()
            .enumerate()
            .filter(|(_, r)| r.element == atom.assumption)
            .collect();
        let n = solution.get_integer_value(count) as usize;
        if n < required.len() {
            return Err(PlanningSearchError::InconsistentAssessment);
        }
        for i in 0..n {
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
    let maximum = solution.get_integer_value(objective) as u32;
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
    let cancelled = control.is_cancelled(PlanningSearchPhase::MaximumProved)
        || control.is_cancelled(PlanningSearchPhase::Alternatives);
    Ok(PlanningSearchOutcome::ProvenMaximum(PlanningMaximum {
        profile: request.profile.clone(),
        scoring_table: request.scoring_table,
        maximum_base_hundredths: maximum,
        alternatives: vec![PlanningProgram {
            program,
            elements,
            provenance,
            assessment,
        }],
        alternative_status: if cancelled {
            PlanningAlternativeStatus::Cancelled
        } else {
            PlanningAlternativeStatus::LimitReached
        },
    }))
}
