use skating_program_lab::*;
use std::collections::BTreeSet;

// Construct standard connections from source connector productions, rather than
// filtering the production enumerator's all-word domain. TP Handbook pp.28–29.
fn same_direction_words(
    alphabet: &[&str],
    max: usize,
    euler: bool,
    unlimited: bool,
) -> BTreeSet<String> {
    fn walk(
        words: &mut BTreeSet<String>,
        parts: Vec<&str>,
        alphabet: &[&str],
        max: usize,
        euler: bool,
        unlimited: bool,
        step: bool,
    ) {
        let listed: Vec<_> = parts.iter().copied().filter(|p| *p != "Eu").collect();
        if listed.iter().any(|j| {
            !(unlimited && j.starts_with('1')) && listed.iter().filter(|k| *k == j).count() > 2
        }) {
            return;
        }
        if listed.len() >= 2 {
            words.insert(format!(
                "{}{}",
                parts.join("+"),
                if step { "+SEQ" } else { "" }
            ));
        }
        if listed.len() == max {
            return;
        }
        for &jump in alphabet.iter().filter(|j| !j.starts_with('5')) {
            let mut append = |connector: &[&str], next_step| {
                let mut next = parts.clone();
                next.extend(connector);
                walk(words, next, alphabet, max, euler, unlimited, next_step);
            };
            if jump.ends_with('T') || jump.ends_with("Lo") {
                append(&[jump], step);
            }
            if euler && jump.ends_with('A') {
                append(&[jump], true);
            }
            if euler
                && !parts.contains(&"Eu")
                && (jump.ends_with('S') || jump.ends_with('F') || jump.ends_with('A'))
            {
                append(&["Eu", jump], step);
            }
        }
    }
    let mut words: BTreeSet<_> = alphabet.iter().map(|j| j.to_string()).collect();
    for &first in alphabet.iter().filter(|j| !j.starts_with('5')) {
        walk(
            &mut words,
            vec![first],
            alphabet,
            max,
            euler,
            unlimited,
            false,
        );
    }
    words
}

// Independent depth-first source grammar, with no production generator/validator helpers.
fn expand(
    words: &mut BTreeSet<String>,
    parts: Vec<&str>,
    alphabet: &[&str],
    max: usize,
    euler: bool,
    unlimited_single: bool,
) {
    let jumps: Vec<_> = parts.iter().filter(|&&p| p != "Eu").collect();
    if jumps.len() >= 2 && parts.last() != Some(&"Eu") {
        let repetitions_fit = jumps.iter().all(|&&jump| {
            (unlimited_single && jump.starts_with('1'))
                || jumps.iter().filter(|&&&p| p == jump).count() <= 2
        });
        if repetitions_fit {
            let code = parts.join("+");
            words.insert(code.clone());
            if euler
                && parts
                    .windows(2)
                    .any(|p| p[0] != "Eu" && p[1].ends_with('A'))
            {
                words.insert(format!("{code}+SEQ"));
            }
        }
    }
    if jumps.len() == max {
        return;
    }
    for &jump in alphabet {
        if jump.starts_with('5') {
            continue;
        }
        let mut next = parts.clone();
        next.push(jump);
        expand(words, next, alphabet, max, euler, unlimited_single);
    }
    if euler && !parts.is_empty() && !parts.contains(&"Eu") {
        let mut next = parts;
        next.push("Eu");
        expand(words, next, alphabet, max, euler, unlimited_single);
    }
}

#[test]
fn public_jump_listing_matches_independent_six_profile_grammar() {
    for line in include_str!("fixtures/planned-jump-grammar.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let f: Vec<_> = line.split('|').collect();
        let (registry, profile) = match f[0] {
            "Bronze" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Bronze),
            ),
            "Silver" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Silver),
            ),
            "Gold" => (ProfileRegistry::adult(), adult_profile(AdultCategory::Gold)),
            "Masters" => (
                ProfileRegistry::adult(),
                adult_profile(AdultCategory::Masters),
            ),
            "Women" => (
                ProfileRegistry::senior(),
                senior_profile(SeniorCategory::Women),
            ),
            "Men" => (
                ProfileRegistry::senior(),
                senior_profile(SeniorCategory::Men),
            ),
            _ => unreachable!(),
        };
        let alphabet: Vec<_> = f[1].split_whitespace().collect();
        let mut expected: BTreeSet<_> = alphabet.iter().map(|s| s.to_string()).collect();
        expand(
            &mut expected,
            vec![],
            &alphabet,
            f[2].parse().unwrap(),
            f[3] == "true",
            f[4] == "true",
        );
        assert_eq!(expected.len(), f[5].parse::<usize>().unwrap());
        let actual = selectable_jump_elements(
            &registry,
            &profile.id,
            profile.scoring_table,
            RotationPreference::AllowMixedDirections,
        )
        .unwrap()
        .into_iter()
        .filter(|c| !c.code.contains('W'))
        .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected.len());
        assert_eq!(
            actual
                .iter()
                .map(|c| c.code.clone())
                .collect::<BTreeSet<_>>(),
            expected,
            "{}",
            f[0]
        );
        assert!(actual
            .iter()
            .all(|c| c.direct_transitions_assumed == c.code.ends_with("+SEQ")));
        let standard = selectable_jump_elements(
            &registry,
            &profile.id,
            profile.scoring_table,
            RotationPreference::default(),
        )
        .unwrap();
        let standard_codes: BTreeSet<_> = standard
            .into_iter()
            .filter(|c| !c.code.contains('W'))
            .map(|c| c.code)
            .collect();
        assert_eq!(
            standard_codes,
            same_direction_words(
                &alphabet,
                f[2].parse().unwrap(),
                f[3] == "true",
                f[4] == "true"
            ),
            "same-direction {}",
            f[0]
        );
        if f[4] == "true" {
            assert!(actual.iter().any(|c| c.code == "1T+1T+1T"));
            assert!(!actual.iter().any(|c| c.code == "2T+2T+2T"));
            assert!(actual.iter().any(|c| c.code == "4Lz+Eu+3F+3T"));
        }
    }
}
