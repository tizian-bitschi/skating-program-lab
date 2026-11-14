# Ability sliders and whole-entry removal (#168)

Ability uses direct native range controls with full localized form names,
secondary canonical IDs, 16px labels/readouts and category-supplied stops.
Off is distinct from Basic and levels 1–4; jumps show listed rotation classes
(Single through Quintuple), including Single Axel rather than physical turns.
All twenty spin controls, steps, choreography and Waltz remain present.
Unavailable forms are disabled and labelled. Desktop groups use a responsive
grid; phones use full-width rows so long translated names remain readable.
No dialog or extra activation is needed to adjust a limit. Arrow keys and
Home/End work; pointer previews stay local until release/change.

## Assessment and undo

A committed reduction invalidates the existing calculation immediately through
the existing search lifecycle. The ordinary revision/request-correlated manual
assessment identifies unavailable occurrences using `personalConflicts`, their
`entryId`, `setting` and `maximum`/`disabled` metadata. No client connection
grammar, category rules, scoring changes or HTTP schema changes are introduced.
After the current real assessment returns, affected whole occurrences are
removed, including required duplicates and compounds with an unavailable part.
Compounds are never rewritten. Until assessment succeeds, ordinary assessment
failure/retry remains available and no unverified removal is inferred.

Feedback names the changed ability and removed whole codes, including repeated
occurrences. The latest ability change offers a scoped undo: restore only that
setting and its removed occurrences using surviving neighbours as order anchors.
IDs, required flags and completeness are preserved. Later additions, removals,
requirement edits and completeness edits survive. Further ability edits replace
the undo action; category, exclusion and adoption changes expire it. Rapid
changes are assessed together without allowing late replies to alter newer state.
An occurrence still unavailable under another changed limit is not restored.

This supersedes the earlier UI behavior of leaving newly unavailable entries
in My program and requiring personal-conflict resolution. Assessment metadata
and separate category findings remain authoritative. Category switching keeps
its existing preservation behavior. Suggestions remain visibly outdated and
non-adoptable until an explicit calculation; slider previews, language and
panel navigation do not launch searches. Undo is session-local, not persisted.

## Browser evidence

`web/tests/ability.spec.ts` exercises the real native service: required and
interchangeable duplicate compounds, disabling a component, jump/spin ceiling
reductions, unaffected identities, completeness, scoped undo with later edits,
rapid changes and held real assessment/search responses. The called-Waltz
regression in `search.spec.ts` verifies that its service-reported Axel dependency
is removed and restored without a client-side exception.

Screenshots cover 1280×800, 390×844 and 320×844 in English, German and Russian:

| Width | English | German | Russian |
| --- | --- | --- | --- |
| 1280 | [Controls](evidence/168/ability-1280-en.png) | [Controls](evidence/168/ability-1280-de.png) | [Controls](evidence/168/ability-1280-ru.png) |
| 390 | [Controls](evidence/168/ability-390-en.png) | [Controls](evidence/168/ability-390-de.png) | [Controls](evidence/168/ability-390-ru.png) |
| 320 | [Controls](evidence/168/ability-320-en.png) | [Controls](evidence/168/ability-320-de.png) | [Controls](evidence/168/ability-320-ru.png) |

The [previous compact presentation](evidence/168/before-1280.png) used smaller
names and abbreviated readouts. Current [jump classes](evidence/168/jumps-1280-ru.png)
remain explicit. Named removal and undo are shown at [1280](evidence/168/removal-1280.png),
[390](evidence/168/removal-390.png) and [320](evidence/168/removal-320.png) pixels.

The browser checks full names, 44px range targets, visible keyboard focus,
bottom-navigation clearance and no horizontal page overflow. These are Chrome
viewport checks, not physical touch hardware or screen-reader certification.

Verification on 2026-11-14:

- `npm run build --prefix web`: TypeScript and production build passed.
- `C2_TEST_PORT=3161 npm test --prefix web -- --workers=1`: all 71 tests passed
  against the real service, including the ten new ability scenarios. The first
  full run passed 70 tests; the locale test used its existing fallback port 3161
  while the service used 3157. The explicit shared port resolved that setup error.
- `cargo test --manifest-path service/Cargo.toml --release --offline -- --test-threads=1`:
  all 24 tests passed, including seven profiles and four workflows.
  An initial parallel Debug run exited natively with `0xc000001d`; the complete
  search group then passed serially. The slow Debug workflow matrix was stopped
  after the complete Release suite supplied the native workflow evidence.
- Independent Standards review: zero findings. Independent Spec/UI review with
  UI/UX Pro Max: zero findings, including a second inspection of final screenshots.
- `git diff --check` passed. The unchanged core and solver checks run in PR CI.
