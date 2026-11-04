# Senior Women and Men Free Skating, 2026/27

Issue: [#77](https://github.com/tizian-bitschi/skating-program-lab/issues/77).
Public contract approved on 2026-11-04 in the requesting chat; see the
[API proposal](../specs/issue-77-senior-profile-proposal.md).

## Reviewed sources

The [Sports Rules, effective 12 June 2026](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf)
provide the baseline. Page references below are printed page numbers.

| Constraint shared by Women/Men | Printed location |
| --- | --- |
| Six jump elements maximum; Axel required; eleven total slots maximum by summing family caps | Rule 612 p. 70 |
| Two combinations, or one Euler-free combination and one sequence; lengths at most three and two; one Euler between listed jumps; compounds exclude quints | Rule 612 p. 71 |
| Jump kind across rotations: three; identical doubles: two; repeated triple/quad/quint types: two, including at most one quad/quint type; identical advanced jump: twice | Rule 612 p. 71 |
| Second identical advanced solo receives 80% only when both occurrences are solos | Rule 612 p. 71 |
| Three spins maximum with separate combination, flying and choreographic roles; distinct abbreviations; flying minimum six revolutions, combination ten | Rule 612 pp. 70–71 |
| One StSq and one ChSq; full-ice StSq; ChSp three consecutive revolutions; ChSq two distinct movements | Rule 612 pp. 70–72 |
| Duration 240 seconds, tolerance ten; time consequences independent of TES | Rule 502 pp. 50–51; Rule 353.1.n pp. 11–12 |

The [Technical Panel Handbook, 15 July 2026](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/TP-HandbookSingles-26-27-15-July-2026-1784641784-3364.pdf)
pp. 14–15 permits flying entry for the combination, but requires a separate
flying spin without foot or position changes. Its pp. 16–25 explain position,
feature and recognition limits; pp. 27, 32–33 confirm repetition credit and
ordered extra-jump consequences. The identical edition was also checked using
the [USFS-hosted copy](https://s3.us-east-2.amazonaws.com/sidearm.nextgen.sites/usafs.sidearmsports.com/documents/2026/7/23/TP-HandbookSingles-26-27-15-July-2026.pdf).
[Communication 2788, May 19 update](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2788-SP-Levels-and-GOE-2026-Updated-MAY-19-1779351385-9626.pdf)
pp. 2–5 supplies feature rules. Existing shared checks retain unsupported
variation categories and explicit qualification assumptions; levels B–4 are
targets, not automatically awarded levels.

[Q&A 1, April 2026, p. 1](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/QA1-SPTC-Choreoelements-1776773178-2054.pdf)
requires the qualifying ChSp revolutions on the same foot.
[Q&A 2, May 2026, p. 1](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/QA2-SPTC-1779180084-8133.pdf)
clarifies dependent step features.
[Q&A 3, October 2026, p. 1 Q2–4](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/uploads/isunewrevamp/document/original/f0e21d6e25df4e42395cb70a6c06b269b590597e823034cbd94476e1ea6dd502.pdf)
clarifies starred compound tails and camel-variation recognition. It does not
authorize inferring missing panel calls from a plan.

## Profile coverage matrix

Both identities use version `2026-06-12` and the June 2 Communication 2786 table.
`ProfileRegistry::senior()` registers both; `adult()` keeps its four profiles.
The profile is selected explicitly in every profile-aware request.

| Area | Women Free / Men Free coverage |
| --- | --- |
| Identity | `isu-senior-2026-27-women-free` / `isu-senior-2026-27-men-free`; recognition `IsuSenior2026_27June12` |
| Jump plan | Shared `SeniorJumpPlanV1`; count, compound, kind/double/advanced repetitions, Axel, Euler placement and bounded transitions |
| Spin plan | Separate combination and single-position flying roles, counts including ChSp, abbreviation distinction, ten/six revolutions, existing position/per-foot checks |
| Steps/choreography | Shared evidence checks, full-ice coverage, feature dependencies, ChSp same-foot evidence and ChSq movements |
| Draft | Missing roles remain indexed repairable absences only when jointly insertable within all caps; no exact six-jump or eleven-element obligation |
| Numeric context | `SeniorScoringV1` derives second-solo 80% from complete clean supplied recognitions; manual intended and scenario stages remain separate; supplied panels are retained |
| Generation | Explicit finite clean solos through quads and supported direct compounds, spin and movement templates; no automatic Euler generation |
| Mixed/locked optimization | Shared validators, exact Senior identity, existing unfactored SOV objective; this objective excludes contextual REP and timing credit and is not a TES optimum |
| Unsupported | Panel inference, variation categories, Euler foot transitions/consequences, marked advanced numerical scenarios and progressive fall deductions; uncertainty remains visible |

Senior fall policy is `None`, never an Adult deduction default. Supported ordinary
time scenarios use the Senior duration and separate time-deduction source;
missing timing and interruption/incomplete-program consequences retain errors.
The clean context helper rejects incomplete or marked recognition instead of
reporting an exact unadjusted TES. Element-only lookup has no ordered context;
callers supply `+REP` or use profile/manual program scoring for the clean case.

Sports Rules and applicable Communications/clarifications apply in their stated
scope. Adult announcement precedence does not transfer to Senior. An event
replacement needs an explicit cited binding and justification; no latest-date
or category-name fallback is used. The inherited program policy remains
explicit for synthetic profiles. Named Senior mechanisms select the shared
typed Senior constraints even when a synthetic profile has a different name.

`tests/senior_profiles.rs` contains independent boundary and integration checks.
The full default and minimal-feature suites retain the Adult/custom regressions.
