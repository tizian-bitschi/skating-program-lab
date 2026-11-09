# Accepted browser/service planning contract

**Status: accepted for planning/specification on 2026-11-09. The user explicitly confirmed Q1–Q4 with “vier mal ja”. No production implementation or deployment is included.**

Decision ticket: [Choose the browser runtime and integration contract](https://github.com/tizian-bitschi/skating-program-lab/issues/151). This contract preserves the [confirmed local-workspace/recovery behavior](https://github.com/tizian-bitschi/skating-program-lab/issues/149#issuecomment-6750385948) and uses the [bounded native-service feasibility evidence](../../experiments/service-proof/README.md). It does not promote the experiment's handwritten HTTP transport or convenience defaults.

## 1. Accepted application boundary

- One C2 browser application: Vue 3, TypeScript and Vite, delivered as static assets.
- One native Rust HTTP service: Axum and Tokio around the existing library. No skating/scoring rules are duplicated in TypeScript.
- UI and `/api/v1/` share one HTTPS origin in deployment. No hosting provider is selected here.
- HTTP/JSON with polling initially; no WebSocket or event-stream dependency is needed for this contract.
- First deployment uses one service instance and a bounded set of native search owners. The solver and its borrowed registry stay on their owner thread. HTTP state contains handles/control flags and DTOs only; CPU-heavy stateless work also stays off asynchronous request executors.
- Technical compatibility is documented in the [primary-source fact check](service-contract-technical-notes.md). The framework basis is selected; package versions and measured performance are not established by this document.

The user accepted the architecture and contract direction below. Production code starts only after the planning/specification steps.

## 2. One shared planning context

Every repertoire request, assessment and search identifies the selected profile, immutable rule/policy edition and scoring-table edition. The service returns these references; the client does not guess versions from a category label. Unknown/unavailable editions reject explicitly, without falling back to a different category or season.

A planning request contains:

| Field | Meaning |
| --- | --- |
| `requestId`, `revision` | Browser-generated correlation and semantic input revision; echoed in responses. |
| `context` | Profile/rule/scoring editions selected from the service catalog. |
| `requiredElements` | Stable workspace entry ID plus one exact whole-element key for each retained occurrence. Duplicate occurrences have different entry IDs. Input order does not fix placement. |
| `personalLimits.jumps` | Per-kind `enabled` plus optional maximum listed class 1–5. Single Axel means 1A, not one physical turn. |
| `personalLimits.spins` | Per exact spin form `enabled` plus optional maximum level. Basic is encoded as 0; levels 1–4 follow. Flying/foot-change forms are separate; V variants share the same form's maximum. |
| `personalLimits.steps` | Enabled and optional maximum level, using the same Basic/1–4 representation. |
| `personalLimits.unlevelledElements` | Enabled/disabled choices such as choreographic content; no invented level. |
| `rotationPreference` | Same-direction connections by default; mixed directions only by explicit choice. |
| `excludedConnections` | Exact whole combination/sequence keys selected through the builder. No new lower-spin-level holes are exposed by this product field. |

An omitted maximum means no additional personal ceiling. Disabled is separate from Basic/Single. Lower eligible values remain included. Personal restrictions intersect category eligibility and cannot expand it. An excluded whole connection leaves its component jumps available elsewhere.

Element keys are stable within their versioned planning context and supplied by the backend. They identify the complete intended element/assumption bundle, not a translated label. This preserves compounds, sequence intentions and category-specific choices such as Waltz classification. The client must not assemble authoritative identities by concatenating display text.

The saved program contains all current entries plus their retained/replaceable state. Only retained occurrences become optimization requirements. Applying an alternative preserves required entry identities; newly adopted entries receive stable workspace identities. Reordering freely placed whole entries or changing language/layout does not create a new optimization problem.

## 3. Catalog and connection builder

- Profiles advertise the four workflows only when their mechanisms are supported. Registered/future categories are not presumed supported merely because they have a label.
- The repertoire exposes all modeled planning families/forms with category availability and eligible levels, including all twenty regular-spin forms. Category-unavailable forms can therefore be visible and disabled, rather than apparently missing.
- Labels use stable translation keys/canonical codes; data and rule sources remain versioned. The service supplies exact published/credited values where applicable; the browser does not calculate BV from slider settings.
- The builder requests the next choices for a prefix and combination/sequence mode. Responses include allowed next intentions, whether the whole can finish, and its exact element key. The API does not require a transfer of every compound to the browser.
- Existing excluded connections remain available for restoration and are clearly flagged. Excluding a two-jump whole does not silently exclude every longer connection sharing its prefix. Other repertoire restrictions still apply.
- Selection includes permitted zero-value and ignored intentions according to the core model. The client deduplicates retransmission by result ID, not by inventing its own composition identity.

## 4. Service operations

Paths and DTO meaning are accepted planning boundaries, not endpoints implemented by this document.

| Operation | Purpose |
| --- | --- |
| `GET /api/v1/profiles` | Supported categories, editions and capabilities. |
| `POST /api/v1/repertoire` | Full known individual-form metadata and current availability under the context/repertoire. |
| `POST /api/v1/connections/options` | Small prefix-based choice response for the connection builder. |
| `POST /api/v1/program-assessments` | Assess currently described program entries, independently of optimization. |
| `POST /api/v1/searches` | Admit one immutable planning request and start proving its maximum asynchronously. |
| `GET /api/v1/searches/{id}/updates?after={cursor}&limit={n}` | Read status and new confirmed results; never start computation as a GET side effect. |
| `POST /api/v1/searches/{id}/pause` | Request cooperative pause during alternative collection. |
| `POST /api/v1/searches/{id}/continue` | Continue the same paused session and its immutable request. |
| `POST /api/v1/searches/{id}/cancel` | Request terminal cooperative cancellation. |
| `DELETE /api/v1/searches/{id}` | Release/dispose a search; cancellation must still reach a busy owner. |

Search creation returns 202 with opaque search identity, an access token and a status reference. This means accepted, not proved or successful. Search/control tokens are unpredictable, instance/restart-safe capabilities held in page memory, sent in authorization headers rather than query strings, and never included in shareable links. A client cannot read/control another search by guessing a counter.

Creation has an unpredictable page-held `creationKey` so a lost response can be recovered by the original initiating page without starting duplicate work or exposing access tokens to another caller. It is a temporary retry capability, not a stored workspace field or public lookup key. Mutating controls carry a `commandId`. Retrying the same logical command does not create another search, collect another batch or repeat cancellation work. One collection operation is active per search; out-of-order/invalid transitions have explicit conflict responses. Exact retry receipt mechanics are implementation details to verify at the HTTP seam.

Initial implementation defaults for measurement: native publication chunks around 25; result reads default to 25 and accept at most 100 per call; active status polling about once per second. A status-only poll can omit result delivery without advancing the cursor. These are tunable delivery parameters, not a total alternative limit, a solver deadline or a performance SLA.

## 5. Proof, collection and visible control

| Service status | Meaning |
| --- | --- |
| `running`, phase `proving` | No maximum is published yet; cancellation is available. |
| `running`, phase `collecting` | Maximum has been proved; confirmed alternatives can be requested/delivered. |
| `paused` | The same collection session is retained; confirmed results stay available. |
| `completed` | Alternative enumeration is explicitly exhausted by the core. |
| `infeasible` | No compliant composition exists for this valid request, as established by the core. |
| `cancelled` | Cancellation was acknowledged; continuation requires a new search. |
| `failed` | Typed request/processing/internal failure; never a substitute for infeasibility. |
| `expired` / unavailable-session error | The session cannot be resumed; not evidence of enumeration completion. |

Command acceptance and completed control are distinct. Show pause/cancel requested until the native operation acknowledges it. The service can signal a busy owner independently of its work queue. Pause is offered during collection, not as an unsupported promise to checkpoint maximum proof. A valid maximum already established before cancellation remains valid; no unproved candidate is labeled optimal.

Status includes proof certainty, maximum BV in integer hundredths when proved, confirmed-result count, activity/control state and explicit collection completion. Total alternative count is unknown until exhausted. A full batch is not exhaustion.

Results have stable per-search IDs, exact whole-element content, required-occurrence provenance, per-element/total credited BV and shared construction assessment. They are immutable confirmed deliveries. Cursor retries return the same identities and the browser deduplicates only repeated IDs. Already delivered data stays visible while collection continues or pauses.

The service automatically collects in internal chunks after maximum proof, until pause, cancellation or exhaustion (confirmed Q3). The browser polls for confirmed deltas; reads do not start more computation. A temporary disconnection does not itself cancel the search: its lease still governs availability. Payload pagination bounds transfer size, not computation or memory. There is no protocol-level total count cap.

Delivery pagination does not make core memory usage constant. Live-owner capacity counts workers until they actually exit, including after cancellation/disposal. Resource failure is reported honestly with confirmed data retained where possible; it must never silently truncate results or claim `completed`. Numeric capacity/memory budgets belong to measured acceptance/deployment planning.

## 6. Manual BV and category assessment

The request carries the same context/limits and all current ordered program entries. Preserve stable entry references when mapping the core's indexed findings. A fully described, supported plan receives a binary compliant/noncompliant construction verdict, credited BV where calculable, and findings with rule/source references. Program-wide obligations need not have a single offending entry.

Unknown/missing planning inputs or unsupported mechanisms are input/capability errors, not a fabricated compliance result. Draft handling never labels an incomplete assessment compliant. Personal conflicts are separate from category violations and do not change catalog values. Duration, actual execution, GOE and PCS remain outside this planning verdict/objective.

Assessment refreshes automatically after a committed edit or restored workspace, with a short debounce where useful (confirmed Q2). Drag preview does not send an assessment on every pointer movement. Optimization still starts only on the explicit calculation action. This ordinary assessment remains distinct from starting an optimization.

## 7. Error contract

Errors contain a stable code/category, translation key/parameters, request/revision references, and applicable entry/field/rule/source references. Do not expose Rust Debug strings as the user's explanation. A late reply for an old revision/search cannot overwrite current UI state.

| HTTP/result | Treatment |
| --- | --- |
| 400 | Malformed transport/JSON. |
| 422 | Immediately detected invalid/unsupported context or planning input, with localized field references. |
| Accepted job later finds invalid input | Typed `request_invalid` failure, not `infeasible`; preserve the responsible entry/constraint references. |
| 200 assessment + noncompliant verdict | A real rule violation in an understood plan, not an HTTP parsing failure. |
| 409 | Invalid search transition, conflicting command identity or already outstanding work. |
| 404/410 + `session_unavailable` | Unknown, inaccessible, lost or expired search; do not attach to a new job reusing an old identity. |
| 503 + retry guidance | Temporary global capacity exhaustion; no admission queue in the first contract. Tell the user the search has not started. |
| 429 | A separately configured client request-rate limit, if one is needed. |
| 5xx / typed internal failure | Unexpected failure or core/reassessment mismatch; not noncompliance or infeasibility. |

The status choices follow the [HTTP fact check](service-contract-technical-notes.md#http-semantics). Production validation is the approved DTO/schema plus domain checks, not the experiment's convenience defaults. No automatic downgrade to another category or rule edition is allowed.

## 8. Confirmed browser data/recovery behavior

One small, versioned workspace snapshot is stored automatically in localStorage: context references, program entries/retention flags, repertoire limits, excluded connections and locale. The last adopted alternative is saved as program content. Neither the running search/token nor its potentially huge result history is durably stored. Storage parsing/version/quota/access failures must be handled without falsely claiming a successful save or silently destroying the original snapshot.

Reload restores the workspace and does not reattach/restart the old optimization. The user can start a fresh one. The restored program's ordinary assessment refreshes automatically; optimization still requires the explicit calculation action.

Changed calculation constraints cancel the old search, mark already shown results out of date and require an explicit new optimization. Language/layout and whole-entry reordering alone do not invalidate a freely placed composition search.

With the page still open, a short connection loss retains the in-memory search identity, token, last cursor and shown results. Reconnect to that same session if available, replay only idempotent commands as needed, and never duplicate results. Previously paused work stays paused. If the session is gone, keep already shown data and offer a new search; do not silently recompute.

## 9. Confirmed session lease

Expire after **15 minutes without valid browser contact**, measured by the service. Active polling and a lightweight heartbeat while paused renew the lease. This is not a 15-minute maximum computation time. A suspended phone/browser might stop making contact; if the lease expires, the approved unavailable-session recovery applies.

Lease monitoring must reach busy owners independently of their command loop, and live capacity is released only when the owner has actually stopped. A lost response or failed close/unload request cannot leave permanent orphan work. Terminal solver resources should be released promptly; confirmed delivery data remains available under the lease/disposal contract until the client has reconciled it. After receiving an exhausted result set completely, the client may release the server session while keeping the displayed results in page memory.

The experiment's four-second test expiry and 120-second default are not proposed product policy.

## 10. Acceptance record and remaining scope

The user explicitly confirmed all four proposals on 2026-11-09:

1. One same-origin Vue/TypeScript/Vite browser application and Axum/Tokio Rust service, with versioned HTTP/JSON, polling and the shared domain boundary.
2. Automatic ordinary BV/rule assessment after committed edits and restore; optimization only on explicit action.
3. Automatic service-side collection and incremental browser delivery of all equal optima, without a total cap, until pause/cancel/exhaustion.
4. A 15-minute lease without valid browser contact, renewed while contact continues, including during pause.

This records an accepted planning decision. It does not implement or deploy the product, certify performance or select package versions. Final language content and measurable device/performance acceptance remain their separate map topics before synthesis of the implementation specification. Do not reopen the accepted C2 layout, storage location or four policies without new evidence or user direction.
