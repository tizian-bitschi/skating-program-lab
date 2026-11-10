# C2 local workspace (#160)

The browser automatically stores one `skating-c2-workspace` localStorage record.
Version 1 explicitly allowlists the exact profile/rule/table context, ordered
program occurrence IDs, backend whole-element keys, canonical codes, retention
flags, complete/draft selection, personal limits (including Off and omitted
maxima), exact excluded connection keys, mixed-direction preference and locale.
It never serializes a search session, access/creation/retry capability, result
history or the entire reactive application. Adopted entries use the same parent
mirror and `ManualEditor.replaceEntries` seam as restoration.

Restoration first validates the complete schema and exact registered context,
then asks the real repertoire and ordinary assessment endpoints to validate the
stored settings and program identities. Original element contexts may differ
from the selected category: the service revalidates them without changing their
keys or dropping exclusions. No category fallback occurs for rejected snapshots.
The editor's next ordinary assessment includes restored exclusions and rotation
preference. Restoration never starts or attaches an optimization.

Saved locale takes priority over a supported browser locale, then English.
Language changes save the preference without changing program/request content.
Controls wait for initial restoration so early input cannot be replaced by a
late restore. Storage failures preserve the previous snapshot and stop automatic
writes. Retry repeats loading/validation, or retries saving current edits after a
write failure. The separately labeled replacement action explicitly starts an
empty workspace; unsuccessful writes leave the original record intact. Successful
keyboard recovery returns focus to category selection. Users can continue editing
in memory when storage is unavailable; failure feedback remains visible.

## Verification

2026-11-10, Windows, Chrome 156 (headless engine), built Vite production assets,
local native Debug service, dedicated ports 3161/3162, no server reuse:

- Nine workspace browser tests passed: exact ID/retention/locale/limits reload,
  automatic assessment without any search request, corrupt and unsupported
  schemas, unavailable exact context and invalid keys, explicit keyboard
  recovery/focus, blocked storage, simulated quota failure with atomic previous
  save preservation and retry, browser/saved locale priority and English fallback,
  serializer field allowlist, genuine service-issued connection identity across
  category changes, retained exact exclusions/rotation in ordinary requests,
  and German/English/Russian storage-error reflow at 1280/390/320 CSS pixels.
- Reload to matching localized assessment was 439ms in the final workspace run,
  unthrottled loopback. This is an observation, not the parent controlled-network
  performance gate.
- All sixteen then-current browser tests passed before the final restored
  restriction propagation/focus additions. The manual oracle loop now explicitly
  starts each independent fixture with an empty workspace rather than relying on
  reload to erase content. Initial failures exposed this fixture assumption and
  a real early-edit/late-restore race; the readiness guard fixes the race.
- `npm run build` (including TypeScript checking) passed. No Rust/service or
  library implementation changed; unchanged library/service suites were not
  repeated. Final browser/static check results are recorded below.

The genuine search-adoption flow lands with #157 and still needs combined parent
verification; this slice tests its existing shared replacement seam through
restored backend-issued content. Controlled network timing, real screen-reader
checks, other browser engines and the complete integrated child/adult heuristic
walkthrough remain parent acceptance work. Viewport emulation is not a physical
phone test or native-language/participant approval.
