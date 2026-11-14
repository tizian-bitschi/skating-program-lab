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
independently reproduced the same 115 → 69 totals at 12.40 BV in all three widths.

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

## Standards

The independent Standards review found no documented-rule violations or
actionable code smells, including KISS, DRY and SOLID concerns. It inspected the
diff from `16763b56187ba1d1600d2cd00699edd5515e79b5` against `AGENTS.md`,
`docs/architecture.md`, `docs/agents/issue-tracker.md` and `GLOSSARY.md`.

## Spec

The independent Spec reviewer found no missing requirements, scope creep or
incorrect behavior. Using `ui-ux-pro-max` and opened PNGs, it inspected refreshed
desktop card start/end, two-program comparison and reload; 390px card start/end
and comparison; 320px required-path card start/end, comparison, exclusions, undo
and Russian reload; additional ability and German reload views. Names, codes,
values and wrapped labels remained readable, with reachable actions, visible
focus and fixed-navigation clearance in those views.

Review totals: Standards 0 findings; Spec/UI 0 findings.

## Validation (2026-11-14)

- TypeScript/Vite production build passed (`npm run build --prefix web`).
- All 83 browser regressions passed in 9.7 minutes, including the seven new
  real-service continuation tests and both paths at all three widths
  (`npx playwright test --workers=1`, from `web/`).
- All 24 service integration tests passed with
  `cargo test --release --manifest-path service/Cargo.toml --offline --tests`.
  The seven-profile/four-workflow public HTTP matrix completed in 81.37s.
- The initial Debug service run passed 23 integration tests, then its unchanged
  matrix reached the test harness wait limit after 297.62s while collecting
  Senior Women alternatives after proof. No Rust/service implementation changed.
  The complete Release run above verifies the same service tests; this report
  does not claim a passing Debug matrix or a solver latency guarantee.

Browser checks used Chrome and isolated native services on ports 3173/3174,
with `CARGO_TARGET_DIR` pointing to the existing local build cache. The final
screenshots include resolved post-reload assessment, card start/end captures and
two different selected programs in comparison. Historical screenshot files
regenerated by the regression suite are kept at their original recorded version.
