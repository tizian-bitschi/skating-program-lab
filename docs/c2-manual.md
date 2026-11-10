# C2 manual program editor (#156)

The existing same-origin Vue application now adds, removes and reorders real
whole program occurrences. Each occurrence has an independent stable ID and a
retained flag. The flag means required content for a later calculation, not a
fixed position. This slice does not start optimization or save a workspace.

The element search matches canonical codes and translated form names without
changing repertoire constraints. The connection builder receives prefix choices
from Rust, offers a valid two-jump finish independently of further continuation,
and preserves direct sequence-step assumptions in the backend-issued whole key.
Program entries retain their codes, IDs and flags across category changes.

## Integration boundary

`POST /api/v1/connections/options` takes the repertoire envelope (`requestId`,
`revision`, exact `context`, `personalLimits`) plus `prefix: string[]`, optional
`excludedConnections: string[]`, and optional `allowMixedDirections: boolean`.
`next` contains permitted next code parts; `nextParts` provides translated-name
metadata. `completions` provides complete keys, codes, parts, direct-step flags
and exact exclusion flags. An excluded pair remains visible and does not remove
longer completions with the same prefix. The shared builder also powers the
[exact exclusion and restoration controls](c2-exclusions.md).

`POST /api/v1/program-assessments` takes the same envelope plus ordered
`entries: [{ id, elementKey }]`, `submission: "draft" | "complete"`, optional
`excludedConnections`, and optional `allowMixedDirections`. The response echoes
request/revision/context and supplies `construction`, exact integer credited
values/outcome kinds, optional exact total, category findings and separate
personal conflicts. Findings preserve stable occurrence IDs, component indices,
rule and original source references. Personal maxima identify their setting and
maximum. Typed input failures use 422; malformed JSON/schema uses a typed 400;
worker failure is 500. None becomes a fabricated noncompliance verdict.

A key contains its original profile, rule edition, scoring table, exact code and
direct-step boolean. Exported `element_key` creates it;
`decode_element_key` requires the exact context; `resolve_workspace_element`
validates its original registered context before unchanged content is reassessed
against the newly selected category. Do not construct keys in TypeScript.

`web/src/api.ts` exports `ProgramEntry { id, elementKey, code, retained }`.
`ManualEditor` emits `change(entries, complete)` with cloned entries. Its exposed
`replaceEntries(next, fullyDescribed?)` replaces content without replacing IDs or
flags and triggers ordinary assessment. Search adoption and workspace restoration
must use this same method and one parent mirror, not another program store.
Omitting `fullyDescribed` preserves the current completeness selection.

Assessment runs 120ms after committed content/context/limit edits. Replies must
match both request ID and ordered revision; a late real reply cannot overwrite a
newer assessment. Search/name filtering and language changes do not send another
assessment. The initial empty program is a draft; marking a supported program
fully described requests the shared binary construction verdict. All credit,
including Senior repetition adjustment, comes from Rust rather than catalog sums.

## Verification record

On 2026-11-10, Windows, Chrome 156.0.8078.80, local native Debug service:

- Public HTTP: 3 repertoire tests and 6 manual tests passed. The independent
  literal compliant bases from `tests/manual_bv.rs` cover all six profiles plus
  the registered synthetic test policy. Duplicated spins become noncompliant;
  personal Off preserves category verdict/BV. Other cases cover original-context
  revalidation, duplicate occurrence mapping, Senior 4T credit 9.50/7.60 after
  reorder, pair/third completion, exact exclusions, sequence assumptions,
  malformed transport and correlated errors.
- Browser: all 8 tests passed, including actual compliant/noncompliant/recovery
  edits in all seven policies, duplicate IDs/retention/reorder/removal, a delayed
  real HTTP assessment, connection completion, translated names, keyboard add
  and reorder with restored visible focus, and de/en/ru reflow at 1280, 390 and
  320 CSS pixels without horizontal document overflow.
- Latest unthrottled complete-edit-to-verdict observations: Bronze 314ms,
  Silver 819ms, Gold 825ms, Masters 324ms, Women 342ms, Men 319ms,
  registered policy 318ms. These include the debounce and browser assertion;
  they are local observations, not the controlled-network acceptance gate.
  Earlier pre-final observations were 206–241ms. No outlier was discarded.
- `npm run build`, service `cargo fmt --check`, service
  `cargo clippy --all-targets -- -D warnings`, and `git diff --check` passed.
  The unchanged full library suite was not repeated.

The keyboard fixture initially requested a plain sit spin in Gold; that category
admits the flying form used by the corrected fixture. The genuine permitted
`FSSp1` flow passed. Browser execution requires local Chrome/loopback access;
sandbox-only launches did not start the service successfully and were stopped.

The combined throttled timing gate, real screen-reader check and full integrated
heuristic walkthrough remain parent-level work. These automated viewport checks
are browser proxies, not physical-phone or recruited-participant evidence. The
selected compact C2 layout remains; the large-results/layout ticket owns its
final combined viewport fit.

The [local workspace](c2-workspace.md) now automatically saves/restores the editor,
category, limits, exact exclusions, rotation preference and language. Search
sessions and result history stay outside durable storage.


The [proved program search and adoption](c2-search.md) now uses this editor
replacement/change seam. Required IDs and retention flags survive adoption;
newly added occurrences remain replaceable and autosave as ordinary program content.
