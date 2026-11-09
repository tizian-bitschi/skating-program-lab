# Selected prototype: C2 Mixer

The user selected **C2 Mixer** on 2026-11-09 and declared the prototype exploration complete. This file is the current decision record for [Shape one program workspace and an understandable element-repertoire filter](https://github.com/tizian-bitschi/skating-program-lab/issues/148), within the [browser UI map](https://github.com/tizian-bitschi/skating-program-lab/issues/145).

## Selected interaction

- One program-centered workspace with category/season and language in one navbar. No additional page-heading row or persistent prototype banner.
- Full desktop width, 16px outer padding, 68px mixer channels. All fourteen jump/spin/step faders and both choreographic controls are available together, without repertoire tabs.
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

## Verification of the selection cleanup

- JavaScript syntax parses; static checks found no alternate renderer, view-switcher markup, routing enum or `ui.variant` reference. Whitespace check passes.
- Connected Chrome shows one workspace, fourteen faders and zero comparison switchers.
- At 1280×720 the document scroll dimensions equal the viewport.
- Axel Off raises the retained-element conflict; restoring it clears the conflict and allows the illustrative search. Two example suggestions are delivered.
- The connection builder still opens; Escape closes it and restores focus to its trigger.
- [Selected prototype screenshot](prototype-ui-review/selected-c2/selected-c2.jpg).

## Boundaries and next decisions

This is an approved interaction direction, not production UI or complete product acceptance. Calculations and result delivery remain simulated. The catalog is illustrative; the production adapter must use the authoritative Rust repertoire and assessment. State is in memory only. No actual optimizer connection, storage, HTTP service, verified WASM execution, physical-phone performance measurement or participant usability test is supplied.

The earlier independent screenshot review concerned the exploration before selection. Its findings remain useful evidence, not proof that every later revision passed review. User selection does not establish accessibility conformance or comprehension across age/language groups.

The remaining map decisions cover anonymous storage/recovery, multilingual terminology, executable runtime feasibility, architecture/integration and measurable acceptance criteria. Complete those before synthesizing the implementation specification and build tickets. Do not promote the throwaway source directly to production.
