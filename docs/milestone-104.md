# Controlled optimizer acceptance (#104)

## Result and scope

The accepted #101 contextual objective, #102 incumbent retention and #103
pruning remain covered by independent finite-word tests. The new experiment
adds Adult immutable completion to the earlier diagnostic comparison and
retains all native observations, including failures. It measures the combined
production pruning against the post-#102 baseline. It does not establish a
separate speedup for #101 or #102, or an across-domain speedup over #68/#81.

The baseline is `cf1343fbf82829ca10969562a71b3b48cd0bbcf8`; the measured new
revision is `c91473ad7e7677356a75ccda5fa5e0a16fcec7dd`, based on integration
revision `ef77a504965de14b8b0a8374847680f04aa86174`. Both checkouts receive
the identical test-only diagnostics overlay. No production solver, API,
architecture, CI or scoring-table change is included in this acceptance PR.

## Reproduction

From a clean checkout of this PR's implementation commit, with the repository
toolchain and cached dependencies installed:

```powershell
./benchmarks/optimizer-acceptance/run.ps1 -ExpectedRevision HEAD -VerifyOnly
./benchmarks/optimizer-acceptance/test-runner.ps1
./benchmarks/optimizer-acceptance/run.ps1 -ExpectedRevision HEAD -Runs 5
python benchmarks/optimizer-acceptance/summarize.py target/acceptance-<reported-id> target/acceptance-summary
./benchmarks/senior-grand-prix/test-runner.ps1
```

The runner refuses a wrong revision, tracked/untracked dirt, unknown baseline,
or changed fixture/generation dependency. It creates detached checkouts under
an ignored target directory, copies only the two test-instrumentation files,
uses the historical pinned lock with `--offline --locked`, and builds separate
binaries in separate target directories. These checkouts deliberately contain
the recorded diagnostic overlay; the original source checkout stays clean.
Worktrees and logs are retained for inspection. The existing #81 runner and
its clean-checkout, revision and lock-restoration guards remain unchanged.

The [native archive](../benchmarks/optimizer-acceptance/observations-2026-11-08/native-observations.zip)
contains ten complete result logs, both exact request/domain manifests and
`host.txt`. The latter records commit IDs, input Git object IDs, overlay and
binary SHA-256 hashes, Windows build, CPU, toolchain and run conditions.
The manifests have identical SHA-256 hashes across the two revisions. They
pin all generated variants, recognition choices, profile/SOV identities,
exclusions, original entries, locks, positions, insertion gaps and limits.
Each result log includes applied source-bound rules, coverage/proof scope,
omissions, credited per-element BV, status and termination. The
[160-row CSV](../benchmarks/optimizer-acceptance/observations-2026-11-08/observations.csv)
also preserves rejection counts and reasons from the native diagnostic.

Configuration: Rust 1.99.1, Pumpkin 0.5.0 default seed 42, Windows x64
10.0.26200, Intel family 6 model 183, 32 logical processors, debug builds,
250 ms cooperative budget, 200 checked proposals per call. Adult result cap
is 100; Senior cap is 1000. Five pairs alternate before/after order. Compilation
is excluded. No observation is dropped as a warmup. Background load, power
and thermal state are uncontrolled confounders; this is one native host and
one build mode, not a release or WebAssembly performance claim.

Adult full requests reproduce the unchanged six-slot historical broad-domain
fixture. Its explicitly requested combination generation is a benchmark
abstraction, not an inference about an individual's compound ability. Its
exact finite output is recorded. Adult completion uses separately listed
SOV codes at many spin levels without feature proofs, with one immutable
original at zero-based position 2, five insertions and boundaries 0/1. Bronze
and Silver lock `1T`; Gold and Masters lock `1A`. No compound is inferred in
these new completion domains. Senior uses the unchanged four source-derived
eleven-slot large full/completion requests, including the historical explicit
advanced-solo addition exclusions. Those exclusions remain part of both
sides, not a claim that contextual assumed-recognition search needs them.

## Observations

Ranges below are all five native observations, rounded to 0.1 ms. A dash
means no verified candidate, not infeasibility. Times start at the shared
search's recorded start, include preflight, and end after successful final
proposal validation. They exclude candidate generation. Total public call
time is recorded separately; completion reconstruction and final assessment
can finish later. Result serialization is outside the measured call.

| Workload | Verified runs before / after | First verified ms before / after | Checked proposals before / after |
| --- | --- | --- | --- |
| Bronze full | 5/5 / 5/5 | 34.5–36.4 / 35.1–39.2 | 54 / 54 |
| Bronze completion | 5/5 / 5/5 | 1.5–1.8 / 1.5–3.4 | 74–85 / 80–86 |
| Silver full | 5/5 / 5/5 | 15.1–16.7 / 4.9–5.4 | 29 / 16–17 |
| Silver completion | 5/5 / 5/5 | 3.9–4.9 / 2.6–2.9 | 62–65 / 40 |
| Gold full | 0/5 / 5/5 | — / 13.4–14.2 | 93 / 11 |
| Gold completion | 5/5 / 5/5 | 9.3–11.8 / 2.7–3.1 | 15 / 42–49 |
| Masters full | 0/5 / 5/5 | — / 15.4–16.5 | 62 / 8 |
| Masters completion | 5/5 / 5/5 | 39.5–43.7 / 4.0–4.2 | 85–88 / 40–43 |
| France Women full | 0/5 / 0/5 | — / — | 3 / 11 |
| France Women completion | 5/5 / 5/5 | 58.4–125.9 / 58.3–126.0 | 12–14 / 12–15 |
| France Men full | 0/5 / 4/5 | — / 200.9–203.7 | 3 / 2–12 |
| France Men completion | 5/5 / 5/5 | 54.8–89.9 / 58.6–93.8 | 3–4 / 3 |
| Canada Women full | 0/5 / 0/5 | — / — | 3 / 2 |
| Canada Women completion | 5/5 / 5/5 | 141.0–149.6 / 115.9–133.9 | 13 / 13 |
| Canada Men full | 0/5 / 0/5 | — / — | 3 / 2 |
| Canada Men completion | 5/5 / 5/5 | 154.3–164.8 / 152.3–170.5 | 16 / 16 |

All 160 calls ended `Interrupted`, with `enumeration_complete=false` and zero
or one returned word. Baseline: 50 `FeasibleUnproven`, 30 `Incomplete`.
New: 64 `FeasibleUnproven`, 16 `Incomplete`. Public calls took 250.395–256.246
ms despite the 250 ms cooperative budget. No run proved the optimum or
exhaustive ties. Final BV and every raw diagnostic are in the CSV/archive;
first feasibility and objective quality are distinct measurements.

Bronze and several Senior cases show no improvement. Gold completion reaches
an incumbent sooner while checking more proposals. Senior full remains
unreliable: only four of twenty new runs succeeded, compared with fourteen
of twenty in the earlier #103 diagnostic slice. These are different run
conditions, including detailed output between calls and additional Adult
completion requests. Do not pool the two experiments or hide this negative
result. This comparison fixes the workload within each pair; it does not
control all host effects between experiments.

Generation and preflight durations, total call time, first verified time,
validator proposals, invalid-rule counts, final BV and ties are measured.
An improvement sequence, exact proof timestamp, memory and internal nodes
are not exposed by this instrumentation and remain `NA`. The CSV never
substitutes total call time for an unmeasured proof time. Accepted proposal
count includes non-improving words; rule-rejection counts can overlap.

## Correctness and retained acceptance

| Requirement | Executable evidence |
| --- | --- |
| All six profiles; full finite feasible sets and exact ordered ties | `adult_spin_role_optimizer_matches_complete_fixture_sets_and_bv`, `senior_spin_role_optimizer_matches_complete_fixture_sets_and_bv` in `tests/program_optimizer.rs`; literal fixture values, eligibility and exhaustive ordered words establish expectations independently of production scoring. |
| Contextual 80% maximum and explicit compound alternative | `assumed_senior_credited_optima_match_literal_ordered_oracle` compares raw and credited maxima, all ties, personal exclusions and an accepted double-3A case; `assumed_senior_completion_matches_eleven_slot_credited_oracle` independently enumerates anchored additions. |
| Per-element and aggregate manual/completion agreement | The latter completion test now reconstructs selections from original/inserted provenance and compares every value against literal June 2 SOV values and Rule 612's 80% formula, then against `assess_manual_bv`. Both insertion orders and explicit `3A+2T` are covered; production scorers do not calculate expected values. |
| Incumbent retention and truthful interruption | The contextual full fixture asserts `FeasibleUnproven`, incomplete enumeration and a retained oracle-optimal candidate at its proposal limit; completion interruption/anchor tests and all timed observations preserve proof distinctions. |
| Spin levels without feature proofs | Existing Gold/Senior assumed-recognition oracles and the new Adult completion requests explicitly select SOV levels without supplying feature evidence. This does not assert actual panel qualification. |
| Pruning and provenance | Six-profile exhaustive feasible-set tests plus Gold/Adult and Senior completion tests check immutable originals, absolute anchors, gaps and exclusion scopes. Source-bound validators remain authoritative. |
| Historical acceptance | `docs/milestone-63.md`, `docs/milestone-75.md`, `docs/milestone-89.md`, the 625-row June 2 SOV tests, all twelve `tests/official_replay.rs` exact TES fixtures, optional GOE previews and detailed evidence validators remain in the default/minimal suites. Historical #68/#81 CSVs and runner files are unchanged. |

## Engineering decision and review

Retain the two #103 production constraints. Do not adopt the unsuccessful
Senior choreographic-spin or compound-cap prototypes. Budget about 260 ms
of observed synchronous debug call time for these 250 ms requests on this
host, plus generation and scheduling overhead; this is guidance from this
sample, not an upper bound or service guarantee. Use bounded requests and
explicitly handle no incumbent on broad Senior full generation. Narrow
immutable completion is more reliable in this sample, but does not prove
optimality or exhaustive ties at this budget.

The next solver experiment targets time inside Pumpkin on the three Senior
full domains that still fail. Require identical-input repeated measurements
and independent feasible-set equivalence before adopting another constraint.
Do not expand personal ability domains or change UI, profiles, scoring
editions, GOE semantics or public APIs to improve a benchmark score. #56 and
#58 stay open. Parent #100 is not closed by this PR.

Validation and external review status are recorded in the PR. Required checks:
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo test --no-default-features`, both runner guard suites, and the native
paired experiment. Review is pending until the prescribed external reviewer
returns a final classification; no approval is implied by these measurements.
