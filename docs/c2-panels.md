# Four planning areas (#165)

The workspace has Ability, My program, Suggestions and Exclusions. On screens
below 1100px, fixed bottom navigation selects one full panel. The document has
bottom padding and scroll clearance for the navigation and device safe area.
Navigation uses named buttons with an active indication and visible keyboard
focus. Each panel remembers its document reading position during the session.
On desktop, all four areas use normal page flow; My program and Suggestions
appear side by side. Program, assessment, comparison and result lists no longer
compete in small independently scrolling windows. The element-picker dialog
retains its own bounded list.

After ability configuration, **Calculate suggestions** and **Specify elements
first** have equal styling and placement. The first starts the existing real
search and opens Suggestions. The second opens My program; **Calculate around
required elements** submits the current required occurrences and opens
Suggestions. Suggestions also retains its existing explicit calculation and
search controls. Adopting a suggestion opens My program.

Manual additions, including built connections, are required by default. The
required checkbox releases or requires one occurrence. Duplicate whole elements
keep distinct IDs; both exact keys and their multiplicity reach the service.
Adopted calculated additions remain interchangeable. Saved workspace flags are
restored unchanged. Category/context, sliders, exact exclusions, language and
the expandable mixed-direction option remain available.

Panels stay mounted. Navigation does not alter calculation inputs, invalidate a
search, acquire a second owner or dispose an existing owner. Search results,
comparison selection, result pages, manual queries and program inputs survive
switches. Search credentials and results remain session-only; reload restores
the versioned workspace without starting optimization.

This replaces the earlier C2 fixed-console presentation and phone links/collapsed
mixer described in the historical acceptance evidence. It does not implement
later #164 slices for direct card exclusions, mutation undo, difference-first
comparison or removal after ability/exclusion edits. Existing conflict reporting
continues until those slices replace it. Domain, scoring and HTTP contracts are
unchanged.

## Acceptance checks

The primary seam is Playwright against both existing native local services.
`web/tests/panels.spec.ts` exercises the two entry paths at 1280×800, 390×844
and 320×844 with Adult Silver: single jumps; USp Basic, SSp1, CSp Basic and
CCoSp1; other spins off; ChSq enabled; W and ChSp off; mixed directions off.
It checks actual suggestions, two required Loop occurrences with separate IDs,
release/require, unchanged service submissions and identity-preserving adoption.
No historical total-result count is assumed.

The same tests check comparison selection, pagination and phone reading-position
restoration, navigation target sizes, bottom-focus clearance and no horizontal
overflow. Another real-service test holds a search-creation response while
switching all four panels, then confirms the original owner produces results
without an extra creation or disposal. Manual search text survives navigation.
Representative ability, program and suggestion screenshots accompany these
checks. Existing layout, repertoire, exact-exclusion, lifecycle/race, large-result
and workspace regressions cover translations, policies and storage recovery.

Run `npm run build --prefix web` followed by
`npx --prefix web playwright test --config web/playwright.config.ts --workers=1`.
Set `C2_TEST_PORT` and `C2_FIXTURE_PORT` to free local ports when needed.
This evidence does not establish screen-reader conformance or a sport-rule audit.

Screenshot evidence: [desktop program areas](evidence/165/required-1280.png),
[phone entry paths](evidence/165/ability-390.png),
[320px required occurrences and navigation](evidence/165/required-320.png).

Verification on 2026-11-13: TypeScript/Vite build passed; the full browser
regression passed 41/41. After correcting comparison-value wrapping, all four
panel tests passed again and their screenshots were refreshed. The initial
entry-path test failed on the previous UI because its discovery action was
absent. Windows sandbox restrictions required running build and browser checks
outside the sandbox.
