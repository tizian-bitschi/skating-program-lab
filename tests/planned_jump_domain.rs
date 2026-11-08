use skating_program_lab::*;
use std::collections::BTreeSet;

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
        let actual =
            selectable_jump_elements(&registry, &profile.id, profile.scoring_table).unwrap();
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
        if f[4] == "true" {
            assert!(actual.iter().any(|c| c.code == "1T+1T+1T"));
            assert!(!actual.iter().any(|c| c.code == "2T+2T+2T"));
            assert!(actual.iter().any(|c| c.code == "4Lz+Eu+3F+3T"));
        }
    }
}
