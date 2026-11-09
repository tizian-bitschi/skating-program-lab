# Unified program workspace prototype

Throwaway decision asset for [Shape one program workspace and an understandable element-repertoire filter](https://github.com/tizian-bitschi/skating-program-lab/issues/148), part of the [browser UI map](https://github.com/tizian-bitschi/skating-program-lab/issues/145). Not production UI, not an architecture choice, not an optimizer or complete category evaluator.

## Question

Which arrangement makes required whole elements, personal repertoire restrictions and progressive alternatives understandable in one workspace on desktop and phone?

The repository has no existing browser page, routing convention or UI component system to host this exploration. The standalone prototype therefore lives next to the domain source in `src/ui-workspace.prototype.html`. No production module imports it.

## Run

From the repository root:

```sh
python -m http.server 8766 --bind 127.0.0.1 --directory src
```

Open `http://127.0.0.1:8766/ui-workspace.prototype.html?variant=A`. No install, external resources, network requests or build step. The file can also be opened directly in a browser.

## Variants

- A: persistent repertoire sidebar alongside the plan; mobile opens repertoire in a dialog.
- B: program-first editor with a compact restriction summary and an on-demand repertoire dialog.
- C: repertoire-first wide family matrix above the program and suggestions.

A shared bottom switcher changes `?variant=A|B|C`; left/right keys switch when not editing or operating a control. All three share in-memory request state. Language can switch among German, English and Russian without clearing the plan. Display names/translations are prototype copy awaiting native-domain review.

## Interaction scope

Add/remove duplicate occurrences; preserve exact whole-element codes with Keep/Replaceable; move elements; undo request edits; edit jump-kind/rotation, exact spin/level and step maxima; exclude exact elements; opt into mixed directions; search the sample catalog independently of the repertoire; show localized personal conflicts; exercise illustrative proof/collection/pause/resume/exhaustion, mark two suggestions for comparison and apply a sample. Request edits mark previous sample results stale. Category metadata supports illustration only and does not replace the Rust rule engine.

The catalog deliberately contains representative examples, not the complete repertoire. No true optimality, numerical BV or complete compliance claim is made. Simulated sample results can coincide or omit real-world rules; they exist only to review interactions. Only the real Rust adapter may supply those claims in an eventual product. Session persistence and actual search recovery are separate undecided tickets; prototype state is memory-only and resets on reload.

## Review tasks

1. Begin with an empty program. Set a jump maximum, exclude Axel and start the example flow.
2. Add `1Lz+1T` as an intact required element. Explain the difference between excluding that compound and excluding all Lutz components.
3. Keep `CSSp2`, set its personal maximum to Basic, find the conflict and recover without losing other inputs.
4. Pause/resume the example alternatives, compare two, apply one and explain kept versus replaceable content.
5. Change restrictions after results exist and recognize stale results. Distinguish maximum proof from exhaustive collection.
6. Switch layouts and languages; repeat on a narrow display and with keyboard controls.

## Pending human decision

No variant is selected. The issue remains open until live user review. This prototype must not be promoted directly to production.

## Observed verification

Checked in connected desktop Chrome on 2026-11-09, using responsive viewport overrides at 390 and 320 CSS pixels as well as the normal desktop view. No physical phone or user-participant test was performed.

- JavaScript syntax parses; staged whitespace check passes.
- A jump-kind exclusion leaves a kept Axel visible with a localized conflict and disables search. The conflict and request survive A-to-B layout and German-to-Russian language changes.
- The mobile repertoire dialog exposes exact spin forms. Capping CSSp at Basic flags the kept CSSp2; removing the restriction clears the conflict. Escape closes the dialog and restores focus to its opener.
- Catalog text/code search finds the exact `1Lz+1T` compound. Adding it preserves the whole code as a kept occurrence; catalog search does not change repertoire restrictions.
- The illustrative search shows unproved then collecting states, unknown total count, and retained examples on pause. These observations are interaction checks, not real solver evidence.
- Keyboard left/right switches variants outside editing controls. Exact exclusions remain independent from jump-kind exclusions.
- Narrow A and Russian C were checked using DOM geometry; page scroll width did not exceed the visible layout width after fixing a wrapping results header. The mobile A filter dialog has no duplicate IDs.
- Mobile row actions and long filter labels were inspected visually and adjusted for wrapping. This is not a full accessibility audit or language-quality approval.

The design-system query for a sports product returned a marketing hero pattern unsuitable for this workspace; it was not applied. Targeted form guidance informed visible labels and submission feedback. The visual direction uses the documented Frost Bright palette and restrained system controls, not the off-topic generated recommendation.
