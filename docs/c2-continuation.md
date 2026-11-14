# Planning workspace continuation (#170)

The four-area workspace restores its existing version 1 inputs after reload:
exact category/rule/table context, ability, ordered program occurrences and IDs,
required status, draft/completeness, exact exclusions, rotation preference and
language. The existing validator and non-destructive storage recovery remain
authoritative; no new persistence format or history mechanism is introduced.

Suggestions explains that panel switches retain results/comparison and reload
requires explicit calculation. Its calculation button uses the restored inputs.
Search credentials, sessions, comparisons and calculated result history are
never saved. Undo is available during the current page session only.

## Integrated browser evidence

`web/tests/continuation.spec.ts` uses the real native service, production Vite
assets and the existing public browser seam. Both ability-first discovery and
required-occurrence-first planning run at 1280×800, 390×844 and 320×844:

- Configure the audited Adult Silver fixture: six single-jump ceilings; USp
  Basic, SSp1, CSp Basic, CCoSp1; other spins off; ChSq on; ChSp/W off; same
  rotation directions only. The required-first path starts with required ChSq1.
- Calculate, select cards, compare full connections and switch panels without
  losing comparison. Add a required pair, exclude it from comparison, verify
  removal and restore its exact identity with undo.
- Collect the exact pair `1Lz+1Lo` and sequence `1Lz+1A+1A+SEQ` exclusions
  without automatic search, then explicitly refresh and inspect every result page.
- Adopt, undo adoption, adopt again, disable Loop and remove whole affected
  occurrences, then undo. Each checkpoint opens a separate page to verify the
  coherent saved inputs without discarding the original page's undo action.
- Reload the original page, verify the exact saved workspace, empty results and
  comparison, no search requests, and the localized explanation beside a reachable
  keyboard-focused calculate action in English, German and Russian.

The existing adoption suite covers duplicate required occurrences and later-edit
undo protection; exclusion and ability suites cover duplicate whole removals,
negative controls and preservation of later unrelated edits. Existing workspace
tests cover context compatibility, completeness, rotation, language, corrupt and
unsupported snapshots, unavailable storage and atomic quota recovery. Existing
search regressions retain all six profiles, the independently registered policy,
proof/partial/infeasible/failure distinctions and stale-response lifecycle checks.

The service counts are observations rather than new fixed regression oracles.
On 2026-11-14 discovery reproduced 115 compositions at 12.40 BV and 69 at 12.40
after the two exact exclusions. Required ChSq1 uses the same fixture; the test
logs its observed totals independently.

Screenshots in `docs/evidence/170/` retain ability sliders, complete cards,
expanded comparison, exclusion management, undo and reload in all three widths.
Representative views: [desktop comparison](evidence/170/discovery-comparison-1280.png),
[phone full card](evidence/170/required-card-390.png),
[phone undo](evidence/170/required-undo-390.png),
[narrow Russian reload](evidence/170/required-reload-320-ru.png).

Viewport emulation does not establish physical phone behavior, screen-reader
conformance or a sport-rule audit. The dated fixed-console, retained-conflict and
collapsed-mixer observations in [C2 acceptance](c2-acceptance.md) are historical;
the parent #164 decisions and current linked workflow documents supersede them.
