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
