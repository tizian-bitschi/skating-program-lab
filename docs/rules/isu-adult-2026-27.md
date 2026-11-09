# ISU Adult singles 2026/27

The [source editions](../rule-sources.md) and [implementation status](../implementation-status.md) define applicability and current support.


The following is a compact scope, not a substitute for [the announcement](../rule-sources.md) pp. 8–11. Limits concern the stated element type, not every movement in a program.

| Category | Jump elements and combinations | Spins and sequence | Fall deduction |
| --- | --- | --- | --- |
| Bronze | At most 4; single jumps excluding Axel; at most one two-listed-jump combination, no Euler. | At most 2 different-abbreviation spins, both in one position, no flying entry; spin level cap 1; at most one choreographic sequence. | 0.5 per fall outside elements; none separately for a fall in an element. |
| Silver | At most 4; singles including 1A; no doubles or triples. | At most 2 different-abbreviation spins, one a non-flying one-position spin; level cap 2; at most one choreographic sequence. | As Bronze. |
| Gold | At most 5; singles and 2T, 2S, 2Lo; no 2F, 2Lz, 2A or triples. | At most 3 different-abbreviation spins: a non-flying combination with change of foot, a flying spin and a choreographic spin; regular spin cap 3, step cap 2; at most one step sequence. | 1.0 per fall. |
| Masters | At most 5; singles and doubles; an Axel type is required; no triples or quadruples. | At most 3 different-abbreviation spins: a non-flying combination, a flying spin without change of foot and a choreographic spin; level cap 4; at most one step sequence. | 1.0 per fall. |

The required durations are Bronze 1:40, Silver 2:00, Gold 2:50 and Masters 3:00, each with a ±10 second tolerance (Adult pp. 8–11). A time outside the permitted range incurs 1.0 per started five seconds, subject to the other timing and incomplete-program rules (Adult p. 6; Sports Rules 353 and 502).

All four categories limit a listed jump to two occurrences. Silver, Gold and Masters permit up to two combinations or one combination and one sequence; one can contain up to three listed jumps and the other up to two. The second and/or third jump of a sequence is an Axel type with the specified direct step. The announcement allows an Euler between listed jumps once per program and does not count it as a listed jump. When a combination and a sequence are performed, the combination must not use an Euler. After Euler no change of foot is allowed before the next jump. Bronze explicitly forbids Euler in its combination (Adult pp. 8–11). The general handbook gives consequences for separate Euler combinations and sequences; check Adult applicability before using those consequences. Validation must count jumps across the entire program, including components of combinations, and distinguish combinations from sequences. It does not count movements within a choreographic sequence as independent elements without applying the applicable recognition rules.

The Adult announcement ignores a Waltz jump in Bronze; in Silver and Gold it is called as a downgraded single Axel (`1A<<`), which has zero BV in the Scale of Values. An ignored movement is distinct from a recognized zero-value element (Adult pp. 9–11). The explicit planning classifications and their counting/repetition consequences are documented in the [Waltz planning contract](../specs/waltz-planning.md). Preparation-only movements are not inferred from a technical code.

For regular spins the Adult minimum total revolutions without/with change of foot are Bronze 3/6, Silver 4/6, Gold 4/8 and Masters 5/8; Gold and Masters state these for the combination and flying spins, not the choreographic spin. No basic position with two revolutions yields no value. A spin with change of foot requires three revolutions on each foot; otherwise it is marked `V` (Adult pp. 8–11). Other recognition and minimum-revolution consequences depend on the precise case and rules; insufficient structured evidence yields unknown or the supplied panel outcome, not an invented call.

Regular spin recognition and level depend on positions, revolutions, change of foot and eligible features. Some features count only once across the ordered program: Communication 2788 p. 2 distinguishes first attempt for features 2–9 and 11–15 from first successful execution for feature 10. Choreographic spins have separate confirmation and fixed-base-value/GOE treatment; they do not receive an ordinary spin level. Q&A 1 requires the three consecutive revolutions on the same foot, on one or two blades. Choreographic and step sequences have different recognition, level and scoring rules. In Gold, a step sequence requires at least one skating movement such as a spiral, spread eagle or Ina Bauer; without it no level is given, and features 3, 4 and 5 are not awarded (Adult p. 9). The target MVP accepts recognized outcomes and supplied feature evidence, checks supported dependencies and reports unknown when evidence or a rule is missing. The current implementation stores feature attempts as descriptions; it does not recognize them or check their dependencies. Automatic derivation of all levels from detailed motions is later work.


## Acceptance examples

These are target examples. Existing tests cover some cases only in part; see [implementation status](../implementation-status.md). Other findings may coexist. See [rule sources](../rule-sources.md) for source editions. Independent human domain review is required before claiming complete correctness.

- **AC-01 — Bronze boundaries:** 1A produces a prohibited-jump finding; four otherwise eligible jump elements do not trigger the count limit, five do. A two-listed-jump combination with Euler triggers the Euler prohibition (Adult p. 11). Two occurrences of 1Lz do not trigger repetition; a third anywhere in solo or combination does. Two jump combinations exceed the Bronze combination limit.

- **AC-02 — Other categories:** Silver 2T is prohibited; Gold 2Lo is permitted by the jump-type rule but 2F and 2Lz are prohibited; Masters with no Axel type reports the missing requirement. In Silver, Gold or Masters a third combination violates the two-combination limit. These findings do not alone establish program validity (Adult pp. 8–10).

- **AC-03 — Features and recognition:** Given two ordered regular spins attempting the same once-per-program feature, only its first attempt can earn that feature; feature 10 instead counts at first successful execution (2788 p. 2, subject to category caps). A recognized choreographic spin has fixed base value and no regular spin level; a recognized CSSp1 uses its level-specific value. Three consecutive revolutions split by a change of foot do not satisfy Q&A 1's same-foot requirement. Panel fallback identification remains a separate supplied decision.

- **AC-08 — Gold step sequence:** With otherwise sufficient evidence, absence of the required skating movement yields no level. Features 3, 4 and 5 contribute no level credit; basic-level requirements are assessed independently (Adult p. 9).

- **AC-09 — Waltz jump and spins:** In Bronze, a Waltz jump is ignored, not automatically reported as a prohibited planned 1A. In Silver and Gold, an executed Waltz jump is called `1A<<` with BV 0.00. For an otherwise recognized Bronze change-foot spin with two revolutions on one foot and at least three on the other, the stated per-foot condition yields `V`; a spin with no basic position held for two revolutions has no value. These are distinct recognition outcomes and do not infer an entire level (Adult pp. 9–11, 2786 p. 3).
