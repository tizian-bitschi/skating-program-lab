# Complete planned-program construction

`assess_manual_bv(...).planning_compliance()` returns `Ok(Compliant)` or
`Ok(Noncompliant)` for complete, understood plans. `PlanningComplianceError`
identifies draft submissions, localized malformed/missing input, unavailable
mechanisms and missing audited inventory. These errors are not compliance outcomes.
Known category-prohibited jumps, levels and compound shapes are noncompliant.
The original assessment layouts, BV errors, detailed coverage and draft repair
APIs remain unchanged. Violations retain rule ID, source edition and element or
component index in `legality.validation.validation.findings`; program-wide
counts and missing obligations have no single offending element.
Personal limits and rotation preferences remain separate from category compliance.

The positive `planning_rule_inventory()` contains eight mandatory construction
families. Assessment admits their supported profile mechanisms explicitly, even
when no submitted element exercises a family. The verdict requires every family
entry and rejects unknown/unsupported input findings. It never infers complete
coverage from an empty partial finding list. Search reconstruction, every equal
optimum, neutral ignored-Waltz augmentation and step-ceiling reassessment use this
same verdict; disagreement returns `InconsistentAssessment`.

## Source editions and audited inventory

Sources selected for 2026/27:

- [A: Adult technical announcement, July 16](https://skatinginbcresults.com/web/events/announcements/2026-2027/IAFSC%20Technical%20Announcement%202026-27%20071626.pdf), pp.4,8–11.
- [R: Sports Rules, effective June 12](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf), Rule610 p.65; Rule612 pp.70–72.
- [H: Singles Technical Panel Handbook, July 15](https://s3.us-east-2.amazonaws.com/sidearm.nextgen.sites/usafs.sidearmsports.com/documents/2026/7/23/TP-HandbookSingles-26-27-15-July-2026.pdf), pp.14–25,27–33.
- [C: Communication2788, May19 update](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2788-SP-Levels-and-GOE-2026-Updated-MAY-19-1779351385-9626.pdf), pp.2–6.
- [Q: April choreography clarification](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/QA1-SPTC-Choreoelements-1776773178-2054.pdf), p.1.

| Positive construction family | Evaluated content | Sources |
| --- | --- | --- |
| Input | Exact supported row/components; complete identities; coherent sequence-step assumptions | Input contract; H27–33 |
| Eligibility/counts/Axel | Jump repertoire, allowed/required element families, jump maximum, Axel obligation | A8–11; R70–72 |
| Compounds/transitions/Euler | Compound/combination/sequence budgets, listed length, length-three budget, Euler placement/count, same-foot intended connections, direct-step sequence assumption, no Euler combination beside sequence, solo-only quint | A8–11; R65,70–72; H27–33 |
| Repetition | Exact jump limits; all-singles across-rotation kind cap3; Senior repeated advanced types and quad/quint subset | A8–11; R70–71 |
| Spin roles/counts/abbreviations/levels | Total spin cap, distinct exact abbreviations (full/V share identity), required nonflying/flying/combination roles, foot-change permissions, planned regular-spin level cap | A8–11; R70–72; H14–25; C2–6 |
| Step count/level/movement | Step maximum and level ceiling; category movement/feature conditions under explicit exact-code recognition intentions | A8–11; R70–72; C2–3 |
| Choreography counts/obligations | ChSp and ChSq caps; mandatory ChSp; spin and movement intentions | A8–11; R70–72; C5–6; Q1 |
| Intended code recognition | Exact codes assert intended recognition prerequisites and selected level; no actual execution is inferred | H14–25; C2–6; Q1 |

| Profile | Jump policy | Spin roles and other family/level conditions |
| --- | --- | --- |
| Bronze | ≤4, singles except Axel; one two-jump combination; no sequence/Euler; W ignored | ≤2 distinct nonflying one-position spins, level≤1; ChSq≤1 |
| Silver | ≤4, singles; ≤2 compounds, ≤1 sequence, ≤1 length3; W counts as downgraded1A | ≤2 spins; mandatory nonflying one-position regular spin; second any nature including optional ChSp; regular level≤2; ChSq≤1 |
| Gold | ≤5, singles and2T/2S/2Lo; same compound/Euler rules as Silver | ≤3: nonflying change-foot combination, flying with optional foot change, mandatory ChSp; regular level≤3; StSq≤1, level≤2 with intended movement; excluded step features3–5 |
| Masters | ≤5, singles/doubles, Axel required; same compound/Euler rules | ≤3: nonflying combination, flying without foot change, mandatory ChSp; regular and step level≤4; StSq≤1 |
| Senior Women | ≤6, Axel required; quint solo-only; general compound/Euler rules | ≤3: combination, separate flying without foot/position change, ChSp; StSq≤1 and ChSq≤1; level≤4 |
| Senior Men | Same technical policy as Women | Same technical policy as Women |
| Registered synthetic | Explicit registered jump policy and named mechanisms | Explicit `register_with_program_policy` spin/step/choreography data and synthetic citations; no category-name inference |

Adult identical listed jumps are capped at2. R's General remarks apply kind cap3
to all singles Free Programs, including Adult; A does not replace that condition.
Senior identical doubles/advanced jumps≤2, only two repeated advanced types,
including at most one quad/quint type. A second identical advanced solo receives
80% BV; this credit adjustment does not make construction noncompliant.

## Exact-code recognition intentions

Selecting a regular spin code asserts intended minimum total/per-foot revolutions,
required basic positions, position holds and qualifying features for its level.
Selecting StSq asserts intended full-ice coverage, qualifying turns/features,
feature dependencies and category movement conditions, including Gold's movement
and excluded features. ChSq1 asserts the intended required distinct movements and
visibility. ChSp1 asserts consecutive same-foot qualifying revolutions and the
intended choreography. Silver's optional ChSp additionally intends four total
revolutions and a two-revolution basic-position hold; its fixed `1` is not a
regular-spin level ceiling. These are planning intentions, without performed
revolutions, actual edges, elapsed duration or panel decisions. Detailed APIs
continue to evaluate supplied evidence and report contradictions independently.

`+SEQ` requires an explicit qualifying direct-step intention. Ordinary `+Axel`
combinations can use permitted same-foot turns and do not imply a sequence.
Waltz counting/classification follows the [explicit Waltz contract](waltz-planning.md).
Masters/Senior require an explicit selected classification where applicable;
source silence supplies no inferred default.

## Independent bounds and capability

Silver's optional ChSp is supported by A p.10 ('second spin of any nature'), R p.72
and C pp.5–6. SOV2786 June2 gives the source upper bound4.90 jumps +2.90 CLSp2
+3.50 ChSp1 +3.50 ChSq1 =14.80. Required CSSp2 lowers it to14.70. The old
14.30 proof assumed an incomplete spin repertoire. Existing old manual plans
retain their original numerical values.

Count search admits policies above eleven slots when integer arithmetic remains
safe. It checks a source-row-based integer envelope and total solver activity;
ordered immutable completion retains its own separate slot bound. The synthetic
twelve-slot regression exercises real registry policy through manual assessment
and maximum search. Existing registered-policy tests cover selection and ordered
completion. No official provenance is assigned to synthetic rules.
