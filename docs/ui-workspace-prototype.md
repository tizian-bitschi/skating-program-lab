# Selected prototype: C2 Mixer

The user selected **C2 Mixer** on 2026-11-09 and declared the prototype exploration complete. This file is the current decision record for [Shape one program workspace and an understandable element-repertoire filter](https://github.com/tizian-bitschi/skating-program-lab/issues/148), within the [browser UI map](https://github.com/tizian-bitschi/skating-program-lab/issues/145).

## Selected interaction

- One program-centered workspace with category/season and language in one navbar. No additional page-heading row or persistent prototype banner.
- Full desktop width and 16px outer padding. All twenty-seven jump/spin/step faders, including twenty regular-spin forms, and both choreographic controls are available together, without repertoire tabs. Compact spin channels occupy two rows.
- Program and suggestions occupy bounded lower panels. The main desktop view fits at 1280×720 and 1440×900; long lists can scroll inside their own panels. Narrow/mobile displays reflow and can require document scrolling.
- Faders exclude a kind/form at Off and set downward-inclusive personal maxima otherwise. Level 3 includes Basic and levels 1/2/3. An edited explicit maximum survives category changes.
- Exact whole-element occurrences can be retained while placement stays free. Duplicate occurrences retain their identities.
- A clickable **Kombis & Sequenzen** builder excludes exact whole connections and allows them again. Its component jumps remain available elsewhere. There is no separate new lower-spin-level exception control or code-entry exclusion field in C2.
- Full spin names are available through accessible labels, titles and the focus inspector. German, English and Russian remain available. Translations still need domain-language review.

A/B/C renderers, alternate-view routing, keyboard view cycling and the comparison switcher have been removed from the active source. Legacy filter-view code and review scenario/debug controls were also removed. Old versions and their feedback remain in Git history and archival captures; they are not runnable alternatives in the current file.

## Run

From this prototype checkout:

```sh
python -m http.server 8766 --bind 127.0.0.1 --directory src
```

Open `http://127.0.0.1:8766/ui-workspace.prototype.html?lang=de`. The file defaults directly to C2; no variant parameter is required. There is no build step or external dependency.

## Verification of the selection cleanup (before spin-catalog completion)

- JavaScript syntax parses; static checks found no alternate renderer, view-switcher markup, routing enum or `ui.variant` reference. Whitespace check passes.
- Connected Chrome shows one workspace, fourteen faders and zero comparison switchers.
- At 1280×720 the document scroll dimensions equal the viewport.
- Axel Off raises the retained-element conflict; restoring it clears the conflict and allows the illustrative search. Two example suggestions are delivered.
- The connection builder still opens; Escape closes it and restores focus to its trigger.
- [Selected prototype screenshot](prototype-ui-review/selected-c2/selected-c2.jpg).

## Boundaries and next decisions

This is an approved interaction direction, not production UI or complete product acceptance. Calculations and result delivery remain simulated. The catalog is illustrative; the production adapter must use the authoritative Rust repertoire and assessment. State is in memory only. No actual optimizer connection, storage, HTTP service, verified WASM execution, physical-phone performance measurement or participant usability test is supplied.

The earlier independent screenshot review concerned the exploration before selection. Its findings remain useful evidence, not proof that every later revision passed review. User selection does not establish accessibility conformance or comprehension across age/language groups.

Do not promote the throwaway source directly to production. The following user direction guides the remaining integration work.

## User direction for implementation

- Confirmed: save data in local browser storage, without an account. The prototype still holds state only in memory; this records the implementation requirement, not a completed persistence feature.
- Preferred direction: use a service for the Rust backend, with future features in mind. The user's wording was tentative; a concrete service/API/session contract still needs an executable integration proof.
- Keep the agreed German/English/Russian and usability/performance scope. The user accepted this for now and asked to avoid overcomplication. Do not restart broad UX exploration or add a more elaborate process; preserve truthful verification of implemented behavior.

## Spin-catalog completion

Root cause: the initial hand-curated prototype list contained only seven regular-spin abbreviations. The existing Rust catalog contains twenty. The prototype also incorrectly labeled CLSp as change-foot Camel: **CSp = Camel, CCSp = change-foot Camel, LSp = Layback, CLSp = change-foot Layback**.

The prototype now derives all twenty forms from five spin families, flying/nonflying entry and with/without foot change. These populate both the mixer and element picker. Combination-spin V choices share the same exact-form maximum; unsupported execution-only V representations are not invented as planning choices.

The [availability snapshot](prototype-ui-review/spins/backend-selection.json) was produced by calling the public Rust `selectable_elements` API for all six built-in profiles, with optimizer features disabled. It controls category-unavailable forms without duplicating the category policy in UI conditionals. This is still a prototype snapshot: the real service must provide authoritative live selection data.

Verification: all 20 UI abbreviations match the catalog's clean regular-spin base rows; Camel/Change Camel/Layback/Change Layback names and all six selection snapshots passed independent checks. The element picker returned CSp2, CCSp2, FCSp2 and FCCSp2; adding CSp2 preserved its exact code in the program. A CSp level-1 maximum did not change CCSp's level-2 maximum. Gold disables ordinary Camel/change-foot Camel while allowing flying Camel and CCoSp, matching the API snapshot. At 1280×720 there are twenty spin faders, no document/bank overflow, and both original program rows fit. The slider rails were centered within their narrow controls.

[Complete-spin mixer screenshot](prototype-ui-review/spins/all-spin-forms.jpg). No backend service, persistence, physical-phone or participant test was implemented by this correction. Translated names remain prototype copy pending language review.
