# C2 repertoire application (#155)

The new application consists of `web/` (Vue 3, TypeScript, Vite) and `service/`
(Axum/Tokio, depending on the unchanged library). Build static assets with
`npm ci --prefix web` and `npm run build --prefix web`, then run
`cargo run --manifest-path service/Cargo.toml`. Open
`http://127.0.0.1:3000`; `C2_LISTEN=127.0.0.1:3155` selects another local
address. The service serves the built assets and `/api/v1` on the same origin.
For frontend development, Vite proxies `/api` to the local service.
No production deployment, HTTPS termination or provider setup is included.

## Public boundary

`GET /api/v1/profiles` advertises the six built-in immutable profile/table
contexts and original source references. `POST /api/v1/repertoire` takes
`requestId`, `revision`, `context: { profile, ruleEdition, scoringTable }` and
`personalLimits: { jumps, spins, steps, unlevelledElements }`. Jump/spin maps
use canonical form IDs and `{ enabled, maximum? }`; steps uses the same object.
Unlevelled choices use booleans. Off is independent of Basic (0), Single (1),
and an omitted maximum. All lower eligible levels are retained, including V
variants. Unsupported JSON is rejected with 400; unavailable context and invalid
limits receive typed 422 errors. No category/edition substitution occurs.

Full metadata contains twenty regular-spin forms, six jump kinds, steps,
choreography and planned Waltz. Metadata includes category availability even
when an individual form cannot be selected. Core selection decides eligibility
and personal maxima; planned Waltz is resolved through core manual assessment.
The `core_limits` adapter is shared with subsequent assessment/search work.
Exact element keys are service-owned and versioned. UI settings persist in page
memory across category switches; there is no durable workspace storage yet.

The navbar, repertoire controls, empty program and empty suggestion regions
follow the selected C2 workspace structure. German, English and Russian are
available; the browser language selects the initial locale with English fallback.
Canonical codes and values are unchanged by locale. Full spin labels distinguish
Camel/Change Camel and Layback/Change Layback. Compact desktop labels expose
full names on hover/focus and in accessible control names. No assessment,
optimization, save or recovery outcome is simulated.

## Verification on 2026-11-10

- Public Axum router tests: six profiles, twenty forms, exact editions,
  unknown context, Off/Basic/omitted maxima, malformed limits, correlated
  successful replies, and an independent registered policy excluding Toe loop.
- Chrome via Playwright against the real same-origin Rust service: category
  changes preserve explicit Basic, enabled toggles change the real repertoire,
  language switching, keyboard Space toggles, canonical element codes, and
  all six built-in profiles, desktop region fit, and horizontal reflow at 1280×720, 1440×900, 390×844 and 320×900 in all three languages.
- `npm run build --prefix web` passes TypeScript and Vite production build.
- `cargo test --manifest-path service/Cargo.toml --offline` and service
  `cargo fmt --check` / Clippy with warnings denied pass (3 HTTP tests).
- Initial failures exposed an occupied port, sandboxed loopback access,
  lowercase core profile labels, an unavailable Gold non-flying fixture, a focus tooltip intercepting a control, and an overbroad text assertion matching CSp
  inside FCSp. The tests now use a dedicated port and exact canonical-code rows.
- Reference design inspected against the selected C2 image; desktop controls
  were compacted after the first screenshot exceeded the viewport height.

These are Chromium desktop/emulated viewport checks, not physical-phone tests.
Native-language domain review, screen-reader walkthrough, touch hardware,
contrast measurement, three cold-start timing runs and registered-policy browser
presentation remain part of combined acceptance. No whole WCAG or performance
claim follows from these checks. The independent registered policy is currently
verified at HTTP, not through a separate browser fixture server. Manual entry,
search, alternatives, exclusions, storage and recovery belong to subsequent
#154 slices. No library code, CI or security configuration changed.

