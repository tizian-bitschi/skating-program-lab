use super::*;
use std::collections::BTreeMap;
type ComponentState = (usize, usize, usize, usize, bool);

// Relax whole-element packing, retaining Senior component stock and structural
// capacity. Quint repeats are necessarily solos; other optional solo losses are relaxed.
pub(super) fn jump_upper_bound(
    atoms: &[Atom],
    profile: &RuleProfile,
    request: &PlanningSearchRequest,
) -> i32 {
    upper_bound(atoms, profile, request, None)
}

// Requiring a component does not require a solo or reserve a whole element.
// Every completion containing it remains in this relaxed credited-value DP.
pub(super) fn component_upper_bound(
    atoms: &[Atom],
    profile: &RuleProfile,
    request: &PlanningSearchRequest,
    component: &ListedJump,
) -> i32 {
    upper_bound(atoms, profile, request, Some(component))
}

fn upper_bound(
    atoms: &[Atom],
    profile: &RuleProfile,
    request: &PlanningSearchRequest,
    component: Option<&ListedJump>,
) -> i32 {
    let required: Vec<_> = request
        .required
        .iter()
        .filter_map(|r| atoms.iter().find(|a| a.assumption == r.element))
        .filter(|a| !a.jumps.is_empty())
        .collect();
    let required_compounds = required
        .iter()
        .filter(|a| a.combination() || a.sequence())
        .count();
    let required_threes = required.iter().filter(|a| a.jumps.len() == 3).count();
    let required_nonquint_solos = required
        .iter()
        .filter(|a| {
            !a.combination() && !a.sequence() && a.jumps[0].rotations.is_some_and(|r| r < 5)
        })
        .count();
    // components, quints, repeated advanced types, repeated high types, Axel.
    let mut dp = BTreeMap::from([((0usize, 0usize, 0usize, 0usize, false), 0i32)]);
    for permission in &profile.jumps.allowed {
        let solos: Vec<_> = atoms
            .iter()
            .filter(|a| {
                !a.combination()
                    && !a.sequence()
                    && a.jumps.len() == 1
                    && a.jumps[0].kind == Some(permission.kind)
            })
            .collect();
        let mut options = Vec::new();
        fn vectors(
            solos: &[&Atom],
            required: &[&Atom],
            component: Option<&ListedJump>,
            i: usize,
            state: ComponentState,
            value: i32,
            options: &mut Vec<(ComponentState, i32)>,
        ) {
            if i == solos.len() {
                options.push((state, value));
                return;
            }
            let a = solos[i];
            let jump = &a.jumps[0];
            let r = jump.rotations.expect("listed solo");
            let minimum = required
                .iter()
                .flat_map(|a| &a.jumps)
                .filter(|j| *j == jump)
                .count()
                .max(usize::from(component == Some(jump)));
            let required_solos = required
                .iter()
                .filter(|a| !a.combination() && !a.sequence() && a.jumps[0] == *jump)
                .count();
            for n in minimum..=(if r == 1 {
                crate::senior::DEFINITION.max_kind_occurrences
            } else {
                crate::senior::MAX_IDENTICAL_DOUBLE_OR_ABOVE
            })
            .min(crate::senior::DEFINITION.max_kind_occurrences - state.0)
            {
                let repeated = n == 2 && r >= 3;
                let loss = if repeated && (r == 5 || required_solos == 2) {
                    a.value - i32::from(crate::scoring::repetition_hundredths(a.value as u16))
                } else {
                    0
                };
                vectors(
                    solos,
                    required,
                    component,
                    i + 1,
                    (
                        state.0 + n,
                        state.1 + if r == 5 { n } else { 0 },
                        state.2 + usize::from(repeated),
                        state.3 + usize::from(repeated && r >= 4),
                        state.4 || (n > 0 && jump.kind == Some(JumpKind::Axel)),
                    ),
                    value + n as i32 * a.value - loss,
                    options,
                );
            }
        }
        vectors(
            &solos,
            &required,
            component,
            0,
            (0, 0, 0, 0, false),
            0,
            &mut options,
        );
        let mut next = BTreeMap::new();
        for (&(c, q, r, h, a), &value) in &dp {
            for &((dc, dq, dr, dh, da), dv) in &options {
                let state = (c + dc, q + dq, r + dr, h + dh, a || da);
                if state.0
                    <= profile.jumps.max_jumps
                        + profile.jumps.max_compounds
                        + profile.jumps.max_three_jump_compounds
                    && state.1 <= profile.jumps.max_jumps
                    && state.2 <= crate::senior::DEFINITION.max_repeated_advanced
                    && state.3 <= crate::senior::DEFINITION.max_repeated_quad_or_quint
                {
                    next.entry(state)
                        .and_modify(|v: &mut i32| *v = (*v).max(value + dv))
                        .or_insert(value + dv);
                }
            }
        }
        dp = next;
    }
    let mut best = 0;
    for jumps in required.len()..=profile.jumps.max_jumps {
        for compounds in required_compounds
            ..=profile
                .jumps
                .max_compounds
                .min(required_compounds + jumps - required.len())
        {
            for threes in required_threes
                ..=profile
                    .jumps
                    .max_three_jump_compounds
                    .min(compounds)
                    .min(required_threes + compounds - required_compounds)
            {
                let Some(quints) = jumps.checked_sub(compounds + required_nonquint_solos) else {
                    continue;
                };
                for (&(c, q, _, _, axel), &value) in &dp {
                    if c <= jumps + compounds + threes
                        && q <= quints
                        && (!profile.jumps.require_axel || axel)
                    {
                        best = best.max(value);
                    }
                }
            }
        }
    }
    best
}
