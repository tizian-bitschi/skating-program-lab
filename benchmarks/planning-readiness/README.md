# Complete theoretical planning measurements

Build and run from a clean checkout on Windows with PowerShell 7:

```powershell
cargo build --offline --release --example planning_readiness
$revision = git rev-parse HEAD
pwsh -NoProfile -File benchmarks/planning-readiness/run.ps1 -ExpectedRevision $revision -OutputDirectory target/planning-readiness -Runs 3 -WatchdogSeconds 120
python benchmarks/planning-readiness/summarize.py target/planning-readiness
```

Run without other compiler, test or benchmark jobs. The runner starts a fresh
process for each of 24 profile/scenario pairs and each cancellation phase, three
times in profile order. It retains every failure and continues the matrix. The
external watchdog terminates an experiment after 120 seconds; this is recorded
as `WatchdogFailure`, never infeasibility. It does not impose an API budget or
alter the complete repertoire. No warmup is excluded.

Requests use the built-in profile and June 2, 2026 table. Required cases use one
1F solo in Bronze, exact 1F+1Lo and CSSp2 in Silver, and one 1A solo elsewhere.
Excluded cases remove ToeLoop at all rotations and compound positions.
Impossible cases require four distinct caller IDs with exact 1F solos: Adult
identical repetition allows at most two and Senior jump-kind totals at most three.
Every repetition compares its entire request manifest byte for byte with the
first repetition. The manifest is complete one-line Rust Debug serialization,
UTF-8 without BOM and terminated by LF, with a SHA-256 in every CSV row.

`domain_us` ends at the first Model poll and includes request resolution and
preflight. `model_us` ends at the first Solve poll. `first_valid_us` is elapsed
time to the first IncumbentFound poll, after shared reconstruction and manual
assessment. `proof_us` ends at MaximumProved, including initial representative
validation. `alternatives_us` measures from that poll to API return; `total_us`
measures the whole API call. Missing phases are blank, including direct
preflight infeasibility. These are observable lifecycle boundaries, not isolated
solver CPU counters. Instrumentation itself is included in measured API time.

Cancellation cases use empty requests and trigger on the first poll of Domain,
Model, Solve, IncumbentFound, Reconstruction, MaximumProved or Alternatives.
PostproofReconstruction waits until MaximumProved, then cancels reconstruction
of an additional alternative. `cancel_response_us` measures trigger to return.
Before proof the example requires Cancelled with no result; after proof it
requires the known source-derived maximum and Cancelled alternative status.
The example checks the empty maxima against independent literals 1100, 1480,
2420, 3340, 13310 and 13310, and confirms every returned alternative's validation
and credited total. Other correctness evidence lives in the public integration
tests and source fixtures; assessment consistency alone is not an optimum oracle.

After API return the child flushes its metric line and waits for stdin ACK.
The runner reads Windows `Process.PeakWorkingSet64` while the child remains
alive, then sends ACK. This is lifetime process peak resident memory, including
runtime and domain/model allocation. The handshake is outside `total_us`.
Raw stdout/stderr, compiler/runtime versions, source revision, binary and lockfile
hashes and host hardware remain alongside the CSV. Host background load and
thermal state are uncontrolled. These native measurements establish no browser,
WASM, UI or latency guarantee.

`summarize.py` checks the exact 216-row matrix, every manifest hash, independent
literal expected values, proof/alternative status and cancellation outcomes before
writing the summary. Its worked value bounds are recorded in the [readiness report](../../docs/milestone-117.md).

For the pre-#132 baseline at `caa4127`, pass `baseline-caa4127` as the second
summary argument. The default `issue-132` expectation edition retains the new
Silver 14.80/14.70 maxima. The baseline keeps 14.30/14.20 and its four exhausted
Bronze required-1F alternatives. `bronze_required_flip_has_four_positive_partitions_and_unbounded_neutral_waltzes`
independently enumerates those four positive partitions; current ignored Waltz
augmentation has unbounded optimal multiplicity and returns five with LimitReached.
This versioning changes no baseline rows and retains all matrix/hash/status checks.
