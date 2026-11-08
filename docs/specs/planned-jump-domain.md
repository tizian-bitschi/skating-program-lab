# Planned jump domain

`selectable_jump_elements(registry, profile, table, rotation_preference)` returns uncapped exact
`AssumedRecognition` choices for atomic solo jumps, combinations and sequences.
`RotationPreference::default()` selects same-direction compounds. Explicit
`AllowMixedDirections` selects the broader theoretical connection grammar below.
The preference is separate from profile eligibility and panel recognition.
The [rotation contract](rotation-preference-proposal.md) defines the transition
table, manual conflicts and propagation across search and completion.
It uses the same profile and SOV selection as `assess_manual_bv`. The internal
enumerator polls cancellation during catalog and compound expansion; cancellation
returns no partial repertoire. The legacy bounded evidence generator retains its
explicit finite-domain contract and limits.

## Connection intentions

Sports Rules effective June 12, 2026, Rule 610, printed p. 65 defines a
combination by landing-foot/take-off-foot equality. It does not prescribe a
list of following jump kinds or a common rotational direction. Rule 612,
p. 71 permits the same or another single, double, triple or quadruple in
compounds. Therefore `+` explicitly intends a qualifying same-foot connection;
the planning grammar does not infer an executed edge, direction or foot from
the jump name. Restricting continuations to the customary same-direction T/Lo
subset omits theoretical intentions permitted by that definition. The detailed
evidence APIs retain their verified transition subset and uncertainty.

The July 15, 2026 Technical Panel Handbook, printed p. 28 explicitly classifies
listed jump + Euler + same-foot three turn + Axel as a combination. An Axel
inside a combination therefore does not automatically make it a sequence.
`+SEQ` additionally intends the direct step into an Axel in the second and/or
third listed position (Rule 610 p. 65; Adult July 16, 2026 pp. 8–10).
At least one such Axel immediately follows a listed jump, rather than Euler:
Euler's no-change-of-foot exit cannot supply that direct step. Thus
`2F+Eu+1A` is a combination; `2F+Eu+1A+SEQ` has no qualifying step;
`2F+Eu+2S+1A+SEQ` and `2F+1A+Eu+2S+SEQ` do.

Euler is a non-listed, zero-value component, permitted in either internal gap
once per program. It does not consume a listed-jump place. Bronze prohibits it.
The ordinary program validator still enforces once-per-program, compound count,
three-listed-jump count and Euler-combination/sequence incompatibility. Multiple
atomic alternatives in the repertoire do not imply their joint compatibility.

## Source fixture

The independent `tests/fixtures/planned-jump-grammar.tsv` transcribes the six
profile alphabets, listed lengths, Euler and repetition policies. The test
expands that grammar independently and compares the entire public code set.
With explicit `AllowMixedDirections`, expected sizes are 30 Bronze, 849 Silver, 2,652 Gold, 6,570 Masters and 51,518
for each Senior profile. Senior single jumps have no two-occurrence restriction;
double and advanced jumps do. Senior published quint solos remain present while
quints inside compounds remain prohibited (Handbook pp. 27–29; Rule 612 p. 71).

These are permitted code/connection intentions. They do not establish a skater's
physical ability, performed footwork, successful recognition or official judging.
Exact source URLs and editions remain in [rule sources](../rule-sources.md).
