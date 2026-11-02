use pumpkin_solver::{
    conflict_resolvers::resolvers::ResolutionResolver,
    core::{
        optimisation::{linear_sat_unsat::LinearSatUnsat, OptimisationDirection},
        results::SolutionReference,
        results::{solution_iterator::IteratedSolution, OptimisationResult, ProblemSolution},
        termination::Indefinite,
        variables::TransformableVariable,
        DefaultBrancher,
    },
    Solver,
};
use skating_program_lab::{
    enumerate_bronze_reference, JumpElement, JumpKind, ListedJump, PlannedElement, ReferenceOutcome,
};
use std::ops::ControlFlow;
use z3::{
    ast::{Bool, Int},
    Optimize, SatResult, Solver as Z3Solver,
};

// This prototype models only clean 1Lz. Solution identity uses the full ordered sequence.
const SLOT_COUNT: usize = 3;
const LZ_BASE_HUNDREDTHS: i32 = 60;

fn sequence(slots: [bool; SLOT_COUNT]) -> Vec<PlannedElement> {
    slots
        .into_iter()
        .filter(|occupied| *occupied)
        .map(|_| {
            PlannedElement::Jump(JumpElement::Solo(ListedJump {
                kind: Some(JumpKind::Lutz),
                rotations: Some(1),
            }))
        })
        .collect()
}

fn pumpkin(exclude: bool) -> Option<(i32, Vec<Vec<PlannedElement>>)> {
    let mut s = Solver::default();
    let x = std::array::from_fn::<_, SLOT_COUNT, _>(|_| s.new_bounded_integer(0, 1));
    let obj = s.new_bounded_integer(0, 3);
    let tag = s.new_constraint_tag();
    let mut terms = x.iter().map(|v| v.scaled(1)).collect::<Vec<_>>();
    terms.push(obj.scaled(-1));
    s.add_constraint(pumpkin_solver::equals(terms, 0, tag))
        .post();
    let tag = s.new_constraint_tag();
    s.add_constraint(pumpkin_solver::greater_than_or_equals(x.to_vec(), 1, tag))
        .post();
    let tag = s.new_constraint_tag();
    s.add_constraint(pumpkin_solver::less_than_or_equals(
        x.to_vec(),
        if exclude { 0 } else { 2 },
        tag,
    ))
    .post();
    let mut brancher = s.default_brancher();
    let mut resolver = ResolutionResolver::default();
    let callback = |_: &Solver,
                    _: SolutionReference,
                    _: &DefaultBrancher,
                    _: &ResolutionResolver|
     -> ControlFlow<()> { ControlFlow::Continue(()) };
    let result = s.optimise(
        &mut brancher,
        &mut Indefinite,
        &mut resolver,
        LinearSatUnsat::new(OptimisationDirection::Maximise, obj, callback),
    );
    let optimum = match result {
        OptimisationResult::Optimal(solution) => solution.get_integer_value(obj),
        OptimisationResult::Unsatisfiable => return None,
        other => panic!("unproven optimisation: {other:?}"),
    };
    let tag = s.new_constraint_tag();
    s.add_constraint(pumpkin_solver::equals(vec![obj], optimum, tag))
        .post();
    let mut brancher = s.default_brancher();
    let mut resolver = ResolutionResolver::default();
    let mut termination = Indefinite;
    let mut iter = s.get_solution_iterator(&mut brancher, &mut termination, &mut resolver);
    let mut solutions = Vec::new();
    loop {
        match iter.next_solution() {
            IteratedSolution::Solution(solution, ..) => {
                let program = sequence(x.map(|v| solution.get_integer_value(v) == 1));
                if !solutions.contains(&program) {
                    solutions.push(program);
                }
            }
            IteratedSolution::Finished => break,
            _ => panic!("enumeration incomplete"),
        }
    }
    Some((LZ_BASE_HUNDREDTHS * optimum, solutions))
}

fn z3(exclude: bool) -> Option<(i32, Vec<Vec<PlannedElement>>)> {
    let x = [
        Int::new_const("x0"),
        Int::new_const("x1"),
        Int::new_const("x2"),
    ];
    let sum = Int::add(&x.iter().collect::<Vec<_>>());
    let constraints = x
        .iter()
        .flat_map(|v| [v.ge(0), v.le(1)])
        .chain([sum.ge(1), sum.le(if exclude { 0 } else { 2 })])
        .collect::<Vec<_>>();
    let opt = Optimize::new();
    for c in &constraints {
        opt.assert(c);
    }
    opt.maximize(&sum);
    match opt.check(&[]) {
        SatResult::Unsat => return None,
        SatResult::Unknown => panic!("unproven optimisation"),
        SatResult::Sat => (),
    }
    let optimum = opt
        .get_model()
        .unwrap()
        .eval(&sum, true)
        .unwrap()
        .as_i64()
        .unwrap();
    let proof = Z3Solver::new();
    for c in &constraints {
        proof.assert(c);
    }
    proof.assert(sum.gt(optimum));
    assert_eq!(proof.check(), SatResult::Unsat, "optimality unproven");
    let solver = Z3Solver::new();
    for c in &constraints {
        solver.assert(c);
    }
    solver.assert(sum.eq(optimum));
    let mut solutions = Vec::new();
    loop {
        match solver.check() {
            SatResult::Unsat => break,
            SatResult::Unknown => panic!("enumeration incomplete"),
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                let values = std::array::from_fn::<_, SLOT_COUNT, _>(|i| {
                    model.eval(&x[i], true).unwrap().as_i64().unwrap() == 1
                });
                let differences = x
                    .iter()
                    .zip(values)
                    .map(|(v, b)| v.eq(i64::from(b)).not())
                    .collect::<Vec<Bool>>();
                solver.assert(Bool::or(&differences.iter().collect::<Vec<_>>()));
                let program = sequence(values);
                if !solutions.contains(&program) {
                    solutions.push(program);
                }
            }
        }
    }
    Some((LZ_BASE_HUNDREDTHS * optimum as i32, solutions))
}

fn compare_with_reference(result: &Option<(i32, Vec<Vec<PlannedElement>>)>, exclude: bool) {
    let reference = enumerate_bronze_reference(exclude).unwrap();
    assert!(reference.domain.clean_recognition_assumed);
    assert_eq!(reference.domain.exclude_lutz, exclude);
    match (result, reference.outcome) {
        (
            Some((score, slots)),
            ReferenceOutcome::Optimal {
                base_hundredths,
                candidates,
            },
        ) => {
            let expected = candidates
                .iter()
                .map(|c| c.program.elements.clone())
                .collect::<Vec<_>>();
            assert_eq!((*score, slots), (i32::from(base_hundredths), &expected));
        }
        (None, ReferenceOutcome::Infeasible) => (),
        _ => panic!("solver disagrees with reference"),
    }
}

fn main() {
    for (name, solve) in [
        (
            "Pumpkin",
            pumpkin as fn(bool) -> Option<(i32, Vec<Vec<PlannedElement>>)>,
        ),
        ("Z3", z3),
    ] {
        for exclude in [false, true] {
            let result = solve(exclude);
            compare_with_reference(&result, exclude);
            match result {
                Some((score, slots)) => {
                    println!("{name}: optimal_proven {score}; enumeration_complete {slots:?}")
                }
                None => println!("{name}: infeasible_proven"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solvers_match_exhaustive_reference() {
        for solve in [pumpkin, z3] {
            for exclude in [false, true] {
                compare_with_reference(&solve(exclude), exclude);
            }
        }
    }

    #[test]
    fn solvers_match_independent_fixtures() {
        for solve in [pumpkin, z3] {
            let (score, programs) = solve(false).unwrap();
            assert_eq!(score, 120);
            assert_eq!(programs.len(), 1);
            assert_eq!(programs[0], sequence([true, true, false]));
            assert_eq!(solve(true), None);
        }
    }
}
