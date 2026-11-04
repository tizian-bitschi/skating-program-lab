# Senior Grand Prix native benchmarks (#81)

## Reproduction and scope

Runtime and domain source: `ed1f4ae` (base `c14943a`). Checkout that commit
and run `pwsh -File benchmarks/senior-grand-prix/run.ps1`. It builds Debug and
Release offline, with the committed benchmark dependency snapshot, before timing
the native binaries at 250, 1000, 5000 and 30000 ms. Rust dependencies and the
MSVC/Rust toolchain must already be installed; offline does not mean vendored
dependencies. The runner restores an existing root lockfile after the run.
On another native platform, copy the benchmark lock to `Cargo.lock`, build with
`cargo build --offline --locked --example senior_grand_prix` (also `--release`),
then invoke `target/{debug,release}/examples/senior_grand_prix <milliseconds>`.

The unchanged production solver is the baseline. No pruning/model optimization
is adopted in this PR, so there is no before/after speedup claim. Anchored and
unanchored cases are different requests, not a controlled optimization comparison.
The four budgets use identical requests apart from their deadlines. Pumpkin
0.5.0 defaults to RNG seed **42** (`ConstraintSatisfactionSolver::default`);
the library does not override it. Result order can still change with termination
timing. Each build/budget/scenario has one observation, without statistical or
cross-host latency claims.

The run manifest is [senior-grand-prix-host.txt](senior-grand-prix-host.txt).
Host: Windows NT 10.0.26200.0, x86_64 MSVC; Intel64 Family 6 Model 183 Stepping 1,
32 logical processors; rustc 1.99.1 (`58cf8f9c3`, LLVM 23.1.1), cargo 1.99.1.
CPU marketing name, physical RAM, process peak RSS, internal search nodes,
exact time of first valid incumbent and exact time of optimum proof are
**unmeasured**, not zero. The public API exposes only final candidates and
validator proposal counts. For a returned candidate, first-valid time lies in
`[0, call_us]`; for `OptimalProven`, proof time also lies in that interval.
These are bounds, not measurements of internal events. `call_us` includes API
preflight, model construction, solving and result assessment; preparation outside
the API is separate. Compilation, CSV formatting and post-return anchor assertions
are excluded. Other host activity is uncontrolled. Cooperative limits can overshoot.

## Source programs and planning assumptions

Use the rank-1 programs from all four [#76 fixture sources](fixtures/grand-prix-2026-27.md):

| Source | Skater | Clean solo rows selected as medium/large anchors |
| --- | --- | --- |
| gpfra2026-W | Ami NAKAI | 1: 3A; 7: 3Lz (printed x) |
| gpfra2026-M | Adam SIAO HIM FA | 2: 4S; 3: 3A |
| gpcan2026-W | Mao SHIMADA | 1: 3A; 7: 3Lo (printed x) |
| gpcan2026-M | Mihhail SELEVKO | 1: 4S; 3: 4T |

All eleven actual recognized rows, their order, calls, printed BASE and other
cited values remain in `tests/fixtures/grand-prix-2026-27/elements.tsv`.
`programs.tsv` supplies the name/nation/start identity; `sources.tsv` supplies
the exact protocol URL, page edition and SHA-256. The benchmark selects source
+ rank 1, checks the eleven ordered row indices and carries source/page/row,
original notation and printed BASE into each derived anchor's provenance.
Run `cargo run --offline --locked --example senior_grand_prix -- --describe`
to inspect every selected original row, request, generated variant, omission
reason and conditional completion anchor. This dump is untimed.

**None of these four published programs fits the supported plan-evidence model
without supplied assumptions.** Protocol recognition has no revolution counts,
feature attempts/order, step qualification, choreography movement or transition
evidence. The replay report proves supplied numerical panel replay, not a
fully evidenced plan or proposed feature execution. Calls (`q`, `<`, `!`, stars,
REP), Euler units and longer actual sequences cannot silently become clean plans.

The bounded abstraction uses the source's five non-jump families, in source order
for anchors: flying Camel (France Women: flying Sit), change-foot combination,
StSq, ChSq and ChSp. Regular spins become **Basic** generated plans: eight
revolutions per generated position, explicit three-position change-foot
combination, no feature attempts. These are hypothetical movement facts, not
official panel evidence; printed levels 3/4 are not preserved in the plan.
StSq becomes Basic, two clean difficult turns, full ice, Spiral movement and
explicit qualification; ChSq assumes Spiral + SpreadEagle and visibility;
ChSp assumes three revolutions on one foot. All generated recognition is
explicitly hypothetical. No global or official optimum follows.

The selected clean solo jump kinds/rotations are retained for anchors. Their
printed second-half BASE is preserved in provenance, but the search objective
uses **unfactored catalog BV**. They are source-derived conditional anchors,
not unchanged copies of the full official programs. The omitted source compounds,
marked jumps, all unselected jump rows and level features stay in the source
fixture/dump. No video inference or TES objective is used.

## Finite domains

| Scale | Full optimization | Completion |
| --- | --- | --- |
| Small/exhaustive | Six slots; 2A/3A plus the five Basic roles (7 variants); six mandatory role groups | Five conditional role anchors; one addition from the same 7 variants; all six gaps |
| Medium/anchored | Ten slots; 1T/2T/2A plus the seven source-derived anchor variants; no ordering anchors | Seven immutable ordered anchors (two source solos + five abstracted roles); three additions from 1T/2T/2A; gaps 0, 3, 7 |
| Large/realistic attempt | Eleven slots; all six kinds, rotations 2–4; solos and supported two-jump direct combinations, five Basic role templates | Same generated domain plus seven immutable ordered anchors; four additions; gaps 0, 3, 7 |

Large generation yields 124 variants and 61 audited omissions for each source.
4Lz is an explicit personal exclusion, including compounds containing it.
4A remains a hypothetical eligible alternative; Axel sequences have no confirmed direct-step
tuples, so each attempted sequence is omitted with an unsupported-contract reason.
Euler, marked recognition, level-feature inference and three-jump combinations
are outside the declared domain. All these choices restrict the benchmark;
`CompleteDeclaredDomain` does not mean all Senior candidates are generated.

Large **completion additionally excludes each generated solo with >=3 rotations**
by variant index (eleven additions). This avoids the existing cross-contract boundary:
Senior manual assessment applies second clean solo 80% credit, but the finite
optimizer maximizes unadjusted catalog values. An otherwise valid repeated 3A
completion returns `InconsistentAssessment`, not a candidate. A dedicated test
preserves that explicit failure. Broad completion is therefore not a claim that
arbitrary recognized originals or high-solo additions are supported. Compounds
still pass the common Senior repetition/structure validator.

Proposal caps are 10000 for small and 200 for medium/large; tie cap is 1000,
ordered equivalence, equal-optimum enumeration enabled. No request ceiling changes.
Completion solver variant count includes separate original identities; full
medium has 10 solver variants despite generating only three additions.
The #79 integer proof still applies: eleven `u16` element values sum to at most
720885 hundredths; absolute equality activity is at most 1441770, inside `i32`.
This representational bound is not a feasible-program upper bound reported by
the benchmark, and it does not add contextual credit to the objective.

## Correctness evidence

`tests/senior_benchmarks.rs` uses an independent combinatorial enumerator:
one Axel plus each five roles exactly once produces **1440** feasible ordered
words. With literal SOV coefficients (2A 330, 3A 800, CCoSpB 200, FCSpB 190,
ChSq1 350, StSqB 160, ChSp1 350), the optimum is **2050** with **720** ordered
ties. Every independently feasible word passes shared validation; the full
optimizer returns the exact optimal word set. Excluding 3A independently yields
**1580**, again all 720 words. These assertions do not call the production scorer
to obtain expected BV. The small mandatory groups and six-slot bound exclude
any other feasible multiset. France Women uses FSSpB 200 instead of FCSpB 190:
its corresponding optimum is 2060.

Completion independently checks all six optimal insertion positions at 800 added
BV for all four sources, source identity, immutable entries/evidence/recognition,
absolute positions, restricted gaps, result/proposal/generation caps, an impossible
missing-role fixture, and zero-deadline eleven-slot `Incomplete`. Medium completion
has exact total/added values and 30 ties: two 2A plus one 2T add 790; three 2A
violate repetition. Original BV is 2650/3020/2540/3170 in source-table order.
Existing Adult, feature-order/reuse, Senior repetition/negative and protocol replay
regressions run unchanged in the full suite.

## Measurements and interpretation

Raw observations: [Debug 250](senior-grand-prix-debug-250.csv),
[1000](senior-grand-prix-debug-1000.csv), [5000](senior-grand-prix-debug-5000.csv),
[30000](senior-grand-prix-debug-30000.csv);
[Release 250](senior-grand-prix-release-250.csv),
[1000](senior-grand-prix-release-1000.csv), [5000](senior-grand-prix-release-5000.csv),
[30000](senior-grand-prix-release-30000.csv).

At the **5 s** budget, ranges across the four source cases are:

| Case | Debug API ms | Debug proposals | Release API ms | Release proposals | Result / complete ordered ties |
| --- | ---: | ---: | ---: | ---: | --- |
| Small full | 1140.577–1251.585 | 720 | 192.658–217.806 | 720 | OptimalProven / 720 |
| Small completion | 9.285–9.992 | 6 | 2.327–2.487 | 6 | OptimalProven / 6 |
| Medium full | 5003.420–5004.355 | 26–75 | 1222.058–2792.750 | 200 | Incomplete / none |
| Medium completion | 93.398–107.312 | 40 | 17.038–17.921 | 40 | OptimalProven / 30 |
| Large full | 5001.592–5003.296 | 2–6 | 5009.750–5012.826 | 6–10 | Incomplete / none |
| Large completion | 5002.400–5004.239 | 5–55 | 3418.535–4160.679 | 200 | Incomplete / none |

Small optimum BV is 2060 for France Women and 2050 for the other cases;
medium completion optimum BV is 3440/3810/3330/3960 in source-table order.
At 250 ms, all small full runs prove the objective, but Debug does not finish
the 720-tie enumeration. At 1 s some Debug tie lists also remain partial.
All small completion runs exhaust six ties. Medium completion exhausts thirty
ties in all measured runs. The broader full/large scenarios find no incumbent
in the measurements; their BV and feasible upper bound remain unavailable.

At 30 s Debug large full checks only 2–6 proposals and takes
30006.325–30012.947 ms; large completion checks 143–185 proposals in
30007.270–30010.915 ms, also without an incumbent. The 200-proposal limit already
stops Release medium full and large completion before the 5 s budget. Increasing
only a deadline cannot overcome that proposal limit. These measurements separate
slow internal search (few returned proposals) from many invalid validator
proposals; they do not assign unexposed node counts or internal timings.
At 30 s Release large full takes 29645.165–30079.936 ms for 58–200 checked
proposals; large completion takes 3340.393–7263.552 ms for 200 proposals.
All eight still return `Incomplete`, no candidates and no feasible objective
bound. Across all eight files there are **192 observations**; no broad full or
large completion attempt finds an incumbent. Native wall times vary even with
fixed request and seed; the CSV records each observed run without smoothing.

`NA` marks unmeasured/unavailable fields. `proven_upper_bound` is populated only
when `OptimalProven` proves the returned finite-domain objective. No optimistic
solver objective rejected by the validator becomes a bound on a legal program.
`ties` counts returned equal-best candidates, not all ties unless
`enumeration_complete=true`.

Four separate claims must survive in consumers:

1. Generation covers its declared finite input domain, after audited exclusions.
2. `OptimalProven` proves the objective only inside supplied variants/constraints.
3. `enumeration_complete` separately proves exhaustion of ordered optimal ties.
4. Regulatory/motion coverage remains conditional and partial in every scenario.

The [historical #68 baseline](milestone-68.md) used 19–64 variants, six slots,
200 validator proposals and 250 ms: 24 Debug/Release runs, **no incumbent**.
It is retained intact. Different Senior domains do not establish a speedup over
that Adult workload. Small mandatory roles and medium immutable anchors narrow
the search considerably. Large cases expose optimistic whole-program assignments
that are repeatedly rejected after solving; `proposals` counts those validator
checks, not internal Pumpkin nodes. A stopped no-incumbent run is `Incomplete`,
never `InfeasibleProven`.

No safe measured solver optimization is established here. Broad repetition/role
pruning needs a shared typed predicate and exhaustive equivalence evidence across
Adult, Senior and anchored identity semantics. Encoding a second rule definition
in Pumpkin clauses or claiming that a restricted domain improves the same request
does not provide that evidence. Keep this pinned baseline for a later controlled
change; compare identical inputs/lock/toolchain/host before adopting it.

Practical guidance: start with a small explicit domain and immutable evidenced
anchors, choose an explicit cooperative budget and accept `Incomplete`. Keep
generator omissions, conditional evidence and proof flags with the result.
Do not increase ceilings or infer an interactive success guarantee from a large
budget. An interactive latency target remains unset pending a controlled
optimization and repeated host measurements.

## Validation

`cargo fmt --check`, `cargo clippy --offline --all-targets -- -D warnings`,
`cargo test --offline` and `cargo test --offline --no-default-features` pass.
The four new benchmark tests include the explicit Senior completion mismatch.
The runner completed both builds and all 192 rows; a separate data check verifies
24 rows per file, proposal caps, proof-bound/status consistency and unavailable
internal metrics. Production source, public APIs and CI configuration are unchanged.
