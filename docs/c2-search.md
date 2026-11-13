# C2 proved program adoption (#157)

The browser starts optimization only through explicit calculation actions in
[Ability, My program or Suggestions](c2-panels.md). It
submits one immutable context, personal maxima and enabled settings, retained
occurrences, rotation preference and exact connection exclusions. Changing a
calculation constraint cancels/disposes the previous search and marks shown
results stale. Locale, replaceable entries, completeness and free whole-entry
order do not change the composition problem. A late creation response is released
rather than attached to a changed workspace.

`service/src/search.rs` owns the asynchronous HTTP adapter. A native owner clones
the immutable catalog on its own thread and constructs `PlanningSearchSession`
there. Only DTOs, immutable request metadata, atomic control and thread handles
cross the HTTP boundary. It proves the maximum, then automatically collects in
chunks of 25 until core exhaustion or cancellation. There is no solver deadline
or total result limit. The maximum is published only after proof; each adoptable
program receives a fresh complete shared assessment before publication. Indexed
required provenance maps to stable workspace IDs. Newly adopted occurrences get
new IDs and remain replaceable. Adoption uses the editor's ordinary replacement
and change event, so the same workspace autosave and assessment apply.
[Alternative adoption and undo](c2-adoption.md) adds replacement explanations
and one scoped undo action without changing search identity or freshness rules.

The service exposes creation, cursor updates, cancellation and disposal at
`/api/v1/searches`. Updates default to 25 results and transfer at most 100;
`limit=0` reads status without advancing a cursor. Replays preserve result IDs.
Invalid future cursors reject explicitly. Invalid input, unsupported coverage,
infeasibility and internal failure remain distinct. Responses retain request and
revision references and available required-entry/rule references without showing
Rust Debug messages.

A public `GET /api/v1/searches/identity` returns a random process namespace. The
page combines it with a random 256-bit suffix for its temporary creation retry
capability. Old namespaces reject after restart. Creation receipts prevent a
lost response from starting duplicate work; a conflicting retry rejects.
Search IDs and access tokens are random 256-bit values. Access tokens travel in
Authorization headers and remain in page memory. Disposed/expired receipts
become compact unavailable tombstones after the actual native thread joins;
they no longer retain requests or result histories.

Default live-owner capacity is two. Admission has no queue and returns 503 with
retry guidance. Cancellation and the fixed 15-minute valid-contact lease reach
busy owners through atomic flags. The monitor joins only finished threads and
releases capacity only after that join. Terminal native sessions drop promptly;
DTOs remain available under the lease/disposal contract. `SearchServiceOptions`
and a monotonic `SearchClock` let HTTP tests control capacity/time without
changing the production lease or introducing test HTTP endpoints.

## Verified on 2026-11-10

- The new HTTP seam tests passed red before implementation, then passed the
  native search path. The complete service suite passes 12 tests: existing
  repertoire/manual tests plus three new search scenarios.
- A literal Bronze fixture independently bounds the maximum at 10.30: retained
  CCSp1 + CLSp1 + ChSq1 contribute 8.30, and two Flip plus two Loop jumps add
  2.00. All five expected optimal whole-element partitions are delivered. A
  retained `1Lo+1F` and excluded inverse `1F+1Lo` leave exactly the independently
  expected whole plus solo Flip/Loop result, also 10.30.
- HTTP checks cover fresh compliant assessments, required provenance, bounded
  reads/status-only cursor/replay, creation retries/conflicts, access isolation,
  proof preservation after cancellation, disposal, busy Senior owner capacity,
  expiry without valid contact, recovery of capacity after termination and
  restart rejection of old creation identity. Invalid retained Off content is
  an indexed request failure; a valid impossible retained multiplicity is
  infeasible with diagnostics and no published maximum/program.
- Three real Chrome/browser scenarios pass on separate ports 3159/3160 without
  reusing existing servers. They cover keyboard calculation/adoption, required
  IDs and flags, added replaceable content, persisted adoption and reload
  without another search, semantic invalidation and delayed namespace/create
  replies. Real source-based fixtures produce assessed/adoptable programs for
  all six profiles and the independent registered test policy. Observed
  click-to-first-adoptable times in the final targeted run were 1.38–1.49 seconds
  on this machine; these are observations, not an SLA.
- Production TypeScript/Vite build and service Clippy with warnings denied pass.
  Existing library code and its unchanged full suites were not modified/rerun.

## Following slices

The adapter already collects automatically; #158 adds full progressive browsing,
cooperative pause/continue and command reconciliation around the same `Job`
control flags, retained native session and immutable cursor log. It must keep the
owner/session alive while paused and never recreate it on continue. There is no
current pause endpoint or visible pause promise. #159 supplies user-facing exact
exclusion/rotation controls around the existing shared request fields. #161 adds
complete reconnect/retry workflows around page-held creation/access identities,
cursors and shown data; no live identity belongs in durable workspace storage.

The full 16/14-alternative independent oracle, in-call pause/resume checkpoints,
lost control-response matrix, deterministic termination-barrier capacity tests,
combined network/device/layout matrix, screen-reader checks and measured resource
budgets remain their dependent/parent acceptance work. Viewport or Chrome tests
do not establish physical-device, native-language participant or deployment
acceptance.

Final integrated verification: all 20 current browser tests passed on ports
3159/3160 in 53.7 seconds after merging the current integration branch and the
workspace readiness changes. The final production build, Rust formatting check
and `git diff --check` passed. The exact five-composition literal HTTP oracle
was rerun successfully after adding its full-set assertion. The compiler emitted
only the environment's incremental-cache hard-link/copy warning; Clippy with
warnings denied passed.

## Progressive collection and controls (#158)

`POST /api/v1/searches/{id}/pause` and `/continue` accept the same page-held
Authorization capability and `{ commandId }` receipt as cancellation. Pause is
valid only during collection; continue is valid only after confirmed pause.
Invalid transitions return 409. Reusing a receipt for a different action also
returns 409; replaying its original action does not repeat work. Cancellation
remains harmless for an already terminal search and preserves its proof/data.

Requested controls remain distinct from acknowledgement: `controlRequested`
reports pause/cancel/continue while pending and clears when the native owner
acknowledges that state. A paused owner retains its actual borrowed core session
and solver. It sleeps on a condition variable rather than restarting or polling
the solver; independent cancellation/disposal/lease signals wake it. The timed
condition-variable wait also covers a concurrent shutdown notification.

The browser continues lease-renewing bounded cursor reads while paused. Every
confirmed result stays browsable and adoptable. Checkboxes select any confirmed
programs for a comparison of whole elements, localized names, retained/added
provenance and credited values. Stable result keys preserve reading position and
selection as deltas append. Pending collection controls use `aria-disabled` with
a guarded action so requesting pause/continue does not remove keyboard focus.
No total is claimed before actual core exhaustion. A pending command keeps its
original command ID through an interrupted-response retry.

### Verification on 2026-11-10

- The HTTP control test failed against the missing endpoints before their
  implementation, then passed. All 16 service HTTP tests pass, including seven
  search scenarios. Service Clippy with warnings denied passes.
- The independent Bronze oracle uses literal retained CCSp1/CLSp1/ChSq1 = 8.30,
  plus two Flips, two Loops and one Toe/Salchow = 2.40 in four jump entries.
  It asserts all 16 complete optimal identities at 10.70 through three-result
  cursor pages. Exact whole `1F+1Lo` exclusion removes exactly two, leaving all
  14 expected identities. Replay IDs, full edition/transition keys, required
  provenance and fresh ordinary HTTP assessments agree for every program.
- An HTTP fixture preserves two required ignored zero-value Waltz occurrences
  under separate IDs in every confirmed composition, with zero credited values.
- A real Silver HTTP search pauses during collection, holds its confirmed count,
  continues under the same search ID and delivers further results, then cancels
  while keeping the proved maximum and confirmed history. Invalid proof-stage
  and terminal pause/continue transitions and conflicting receipts reject.
- The real Chrome flow on dedicated ports 3163/3164 verifies pause/continue,
  comparison, adoption, English/German/Russian labels and keyboard focus. The
  first passing targeted flow took 26.7 seconds, including native proof and
  browser interaction. This is an observation, not a latency guarantee.
- TypeScript/Vite production build passes. Unchanged core suites were not rerun.

Exact finite-set equality is also checked after a confirmed native pause and
continue of the 16-composition session. The busy Silver fixture separately
checks continued deliveries before cancellation. Large-result rendering/performance,
combined layout and cross-engine/device acceptance remain #162/parent work.

During browser verification, native `disabled` removed focus from the pending
control; the guarded `aria-disabled` control corrected that observed failure.
An earlier combined run also hit capacity from sessions created by preceding
tests. Search tests now capture every successful creation capability and delete
their own sessions through the public HTTP seam in `afterEach`, including stale
responses and the registered-policy service. A deliberately lost real pause
response is retried with the original command ID before paused cancellation.

Final search browser verification passed all four scenarios in 54.2 seconds in
the original order with explicit HTTP cleanup. After preserving cancellation
priority over a pending pause/continue request, the focused real control flow
passed again in 27.5 seconds. Final build, formatting and diff checks pass.

## Live recovery (#161)

Connection recovery retains the page's creation receipt, access capability,
command receipt, delivered cursor and confirmed programs. A lost accepted create
or control response retries that exact request; it does not allocate another
calculation. Cursor reads are sequenced against controls and newer reads, so a
delayed pre-pause snapshot cannot overwrite confirmed pause or rewind delivery.
Control response snapshots are never used as delivery cursors. Result IDs remain
the service's IDs and replayed deltas are deduplicated.

Transport interruptions offer connection retry. Typed service, input, capacity
and unavailable-session failures retain separate messages; an unavailable
session stops polling and enables the explicit calculation action. Confirmed
programs and the proved maximum remain in page memory. Recovery never silently
creates a replacement after expiry, service restart, changed constraints or
reload. No automatic admission queue is added. A pagehide handler sends a
best-effort authenticated keepalive DELETE and forgets live page identity;
failed unload delivery remains covered by the independent 15-minute lease.
Ordinary disposal also uses DELETE directly, so failure of an earlier separate
cancel request cannot prevent a disposal attempt.

### Verification on 2026-11-10

- The new lost-create/unavailable-session browser regression failed before the
  change because unavailable sessions still offered connection retry; it then
  passed with explicit new calculation recovery.
- The real Chrome search suite passes six scenarios on dedicated ports
  3167/3168. Requests pass through the production HTTP routes; response loss and
  delay are injected after real service processing. Tests cover lost accepted
  creation/pause/continue/cancel responses with unchanged creation/command
  receipts, proof reconnect, paused disconnect with the same cursor, delayed
  pre-pause reads, result identity deduplication, preserved confirmed programs,
  semantic-edit rejection of stale replies and actual reload without restart.
  Captured capabilities are disposed through public HTTP after each test.
- All 21 service HTTP tests pass. The additional paused-owner scenario uses
  the existing injectable monotonic clock: valid contact at 899 and 1798 seconds
  retains pause and its confirmed count; no contact through 2699 seconds expires
  the session. Replaying the old create stays unavailable. A fresh explicit
  create is accepted after termination frees capacity. Four consecutive fresh
  create/dispose cycles at capacity one each reject reads of their disposed
  handles and admit subsequent work.
  Existing HTTP coverage retains busy proof expiry, restart rejection, access
  isolation, finite exhaustion and cancelled proof/data preservation.
- Production TypeScript/Vite build, service Clippy with warnings denied, Rust
  formatting and diff checks pass after integration of 9414fd1. The environment
  emitted incremental-cache hard-link warnings and copied files instead.

These checks exercise browser event/network behavior and actual native owners;
they do not establish physical-device or screen-reader acceptance. The combined
screen-reader runtime blocker remains recorded in the parent evidence. Large
result rendering and retained-memory measurements belong to #162. The unchanged
library baseline was not rerun for this controller change.
