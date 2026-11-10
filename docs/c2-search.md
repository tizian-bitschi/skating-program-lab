# C2 proved program adoption (#157)

The browser starts optimization only through **Calculate best program**. It
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
