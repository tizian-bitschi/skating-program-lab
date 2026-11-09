# Incremental planning measurements (#143)

Build and run on an idle native Windows host, after all tests and compiler jobs finish:

```powershell
cargo build --offline --release --example planning_sessions
$revision = git rev-parse HEAD
pwsh -NoProfile -File benchmarks/planning-sessions/run-session.ps1 -ExpectedRevision $revision -OutputDirectory target/session-measurements -Runs 3
```

The runner requires a clean checkout, records binary/lock hashes and toolchain,
and starts six serial fresh Release processes: uninterrupted and paused collection,
three times each. Every run checks the complete finite oracle, fresh manual BV,
construction compliance, personal restrictions and required IDs before reporting
success. Every process emits its full request, every timed batch and exact identity
set. The runner compares complete identity sets across both modes and all runs.
No warmup is discarded. An external 120-second watchdog records experimental
failure; it is no API time budget or infeasibility result.

The independent Bronze fixture uses 1T/1S (40 each), 1F/1Lo (50 each), all
ordered pairs, at most four entries, one pair and two occurrences per kind.
An independent bounded multiset enumeration derives jump maximum 240; required
CCSp1/CLSp1/ChSq1 add 830. It preserves distinct internal compound order.
Optional ignored Waltz, Lutz and Axel are excluded. Rotation preference is
AllowMixedDirections, with default personal limits and no step ceiling.

`new_us` times session construction. `proof_us` times prove through its return,
including initial representative assessment. `first_five_us` times the first batch
of five; `active_collection_us` sums every collection batch through proved
exhaustion and excludes the pause call and consumer idle time. Each `BATCH` line
retains its own duration, size and state. `pause_us` times a cooperative pause
requested at the next public checkpoint after five deliveries. `resumed_batch_us`
times the first resumed batch of three. The pause call must deliver nothing and
preserve confirmed results. No simulated user delay is inserted. Fresh independent
manual reassessment and oracle comparisons occur outside measured API durations.
Windows process peak resident memory is sampled after metrics while the child
waits for ACK; the handshake is outside measured durations.

Compare the first-five consumer amount with legacy capped delivery only after
checking the same request and identity semantics. Complete session exhaustion has
a different amount of work and is reported separately. The companion legacy
comparison runner uses unchanged request manifests on the archived parent
baseline and current Release example. Native observations establish no latency SLA,
UI, browser or WASM readiness. Host load and thermal conditions remain uncontrolled.

## Identical-request legacy regression comparison

Build `planning_readiness` in separate clean checkouts at baseline
`c295da41527bb2fe13353f4138865e56f4e9b143` and current
`913ba5b4938eefd3e1af2a44682b2facf47cb342` with
`cargo build --offline --release --example planning_readiness`. Reuse the same
retained generated `Cargo.lock` for both builds; the observation directory contains
both copies and their provenance/hash. Preserve the two binaries separately.
The recorded run used the following command from the #143 worktree:

```powershell
pwsh -NoProfile -File benchmarks/planning-sessions/run-legacy-comparison.ps1 -BaselineBinary ../issue-140-evidence/baseline-c295da4.exe -BaselineRevision c295da41527bb2fe13353f4138865e56f4e9b143 -CurrentBinary ../issue-140-evidence/current-913ba5b.exe -CurrentRevision 913ba5b4938eefd3e1af2a44682b2facf47cb342 -OutputDirectory ../issue-140-evidence/legacy-measurements -Runs 3 -WatchdogSeconds 120
python benchmarks/planning-sessions/summarize-legacy.py benchmarks/planning-sessions/observations-issue-140-legacy
```

The runner records 144 fresh serial processes (two revisions × six profiles ×
four scenarios × three runs), including failures, and compares complete request
manifests byte-for-byte across revisions and repetitions. The summary script
requires the exact matrix, passing exits, source-derived maxima and status/count
expectations, verifies every manifest hash and computes each metric's median of
three samples. Ratios divide current median by baseline median. Missing phases
for preflight infeasibility stay empty. The Bronze required case intentionally
changes from five nominal legacy alternatives to four distinct exhausted
compositions; collection timings there cover different delivered amounts.
Retained raw outputs and observations permit independent recomputation.
