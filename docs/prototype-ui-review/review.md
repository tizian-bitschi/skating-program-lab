# Independent screenshot UX review

Review date: 9 November 2026. Reviewer role: critical reviewer, not builder. This is a design recommendation for the next prototype, not a final human decision.

## Verdict

The shared interaction model still needs work before choosing a layout. The screens look orderly, but essential meanings depend on sentences: what a checked jump allows, whose limit applies, and whether keeping an element also fixes its position. Removing those sentences alone would make the product harder to use. Change the controls first, then remove text made redundant by those controls.

B provides a clear program-first desktop structure, but its phone summary wastes more space than A's compact disclosure. A is useful for people actively balancing restrictions and a program on desktop; C gives restrictions priority at the cost of hiding construction. The next trial should combine B's desktop hierarchy with A's compact mobile restriction entry. None has demonstrated self-explanatory operation. No usability study or comprehension test was performed.

## Priority findings from visible screenshots

### 1. The phone prioritizes setup and controls over the actual task

**Evidence:** `a-mobile.jpg` spends roughly the first 350 image pixels on branding, prototype status, title, introduction, category and repertoire before the program begins. The first element then uses a name/code area, a full-width Keep button and a separate move/delete row. Only part of the second element fits. `c-mobile.jpg` reaches only the third jump restriction; no program or calculate action is visible. In the fair initial capture `b-mobile-top.jpg`, B's repertoire summary wraps into four lines and pushes the program down to roughly 430 image pixels. B is less compact than A on the phone despite its program-first intent. `b-mobile.jpg` is a lower-page capture and was excluded from first-screen comparisons.

**Cause:** Desktop sections are stacked without rethinking their mobile priority. Every row exposes secondary operations, and repeated headings introduce the same workspace again.

**Concrete redesign:** Show a compact category control, the program and one visible primary action first. Put personal restrictions behind a summary button that names active restrictions. Use compact element rows with a visible preservation control; reveal order/delete operations when editing a row. Keep an accessible alternative to gestures. Do not solve density by making names or targets smaller. For an empty program, show both “Element hinzufügen” and “Optimales Programm suchen” immediately.

### 2. Personal limits and category limits look contradictory

**Evidence:** `a-desktop.jpg`, `c-desktop.jpg` and `a-mobile-filter.jpg` place “Ohne Limit” beside each jump while repeating “Kategorie erlaubt höchstens: Einfach” below it. The visible dropdown has no persistent visible label explaining which limit it edits. The checkboxes show a jump name but no explicit positive/negative action.

**Cause:** “Repertoire” combines ability, permission and rule limits. A reader has to resolve the two limits and infer checkbox semantics from the introductory paragraph.

**Concrete redesign:** Name the section “Persönliche Einschränkungen”. Give the select a visible group/column label “Mein Maximum”; use “Keine zusätzliche Grenze” for the unrestricted value. Show the category cap once in a distinct read-only rules summary, with exceptions next to affected rows. Label the binary choice as an explicit “Zulassen” state, or use a clearly labeled excluded-family list with “Sprungart ausschließen”. Do not remove accessible input labels when shortening visible copy. Reject or clearly mark choices that cannot affect this category rather than implying they override rules.

### 3. Keep is a state without enough local meaning

**Evidence:** `a-desktop.jpg`, `b-desktop.jpg` and `a-mobile.jpg` show a lock plus “Behalten”. Up/down controls appear beside or below the same element. The sentence beside calculation has to explain that preserved elements remain unchanged while their positions stay free.

**Cause:** A lock commonly suggests fixed position or an uneditable item, while the product needs to preserve an exact whole element and allow free placement. The button label alone does not expose both facts.

**Concrete redesign:** Try a visible row option “Bei Berechnung übernehmen” with a selected state and a separate position control. If the shorter “Behalten” survives, expose an explicit complementary state “Darf ersetzt werden” and a small section label “Für die Berechnung”. Preserve the whole combination or spin form as one item. Do not imply that preserving a combination merely preserves its component jumps. Test this with an exact combination and a changed position before deciding the copy works.

### 4. Conflict identification is local; recovery is not

**Evidence:** `a-mobile-conflict.jpg` correctly highlights “Einfacher Axel” and states “Durch dein Repertoire ausgeschlossen.” The visible actions remain Keep, arrows and delete. The restriction summary reports only a count, so the conflicting restriction is not named there.

**Cause:** The interface identifies the affected row but leaves users to find the inverse operation elsewhere. Deleting the element is the most visible escape even when the user wants to retain it.

**Concrete redesign:** Show two named actions in the conflict row: “Axel wieder zulassen” and “Ersetzen erlauben”. If a specific exact element was excluded, name that element rather than the whole family. Link category conflicts to the category/rule and never offer a personal override of a category rule. Keep the user's element while they choose. A real rule checker must report the actual violated requirement; this prototype does not yet demonstrate that.

### 5. The filter panel is a long form before it is an editing tool

**Evidence:** `a-mobile-filter.jpg` shows six repeated jump rows and an explanatory paragraph. The lower controls and completion action are outside the visible area; two vertical scrollbars are visible. `c-desktop.jpg` spends most of the screen on a completely unrestricted repertoire, including an advanced mixed-direction choice and reset.

**Cause:** The default state makes users scan everything they have not restricted. The three scopes—whole element, family and component maximum—do not appear as equally discoverable choices.

**Concrete redesign:** Start with active restrictions and “Einschränkung hinzufügen”. Offer scope choices “Ganzes Element”, “Sprungart / Pirouettenform” and “Maximale Umdrehung / Level”. Use element selection for whole elements instead of requiring remembered codes. State component scope in the selected value, e.g. “Kein Lutz, auch in Kombinationen”. Keep advanced direction options collapsed until requested. Use one scrolling surface inside a full-height mobile panel, with visible close/done controls. Retain tabs if they help selecting element types; do not add a second navigation system.

### 6. Prototype controls interfere with product review

**Evidence:** The dark “Entwurfsvergleich” overlay covers the lower program/repertoire in all phone captures and overlays results on desktop. The prototype disclaimer and rule-check disclaimer repeat development status.

**Cause:** Evaluation controls occupy the same space as the product task.

**Concrete redesign:** Put design switching outside the simulated viewport or in a collapsible review toolbar. Keep one persistent, accurate prototype disclaimer during evaluation. Remove redundant development explanations from the eventual product only when the real feature exists. Do not replace the current honest caveat with “Regeln erfüllt”.

### 7. Results expose solver language but hide the actual basis for choosing

**Evidence:** `b-mobile-results.jpg` shows “Maximum bewiesen · Alternativen werden gesammelt” and a truthful found count with an unknown total. This is a useful distinction between proved maximum and unfinished enumeration. `b-mobile-results-lower.jpg` shows code-only compositions, “Vorschlag für den freien Rest”, Apply and comparison controls. No base-value number or per-element credited values is visible. Both cards contain the retained 1A and CSSp2 although the footer calls them suggestions for the free rest. “Weitere Alternativen laden” coexists with a collection-in-progress state and a Pause control.

**Cause:** The status explains internal proof work, but the list does not make equal optimum value or preserved-versus-added elements inspectable. “Laden” suggests fetching display items, while “Pausieren” suggests an active background search. The screenshot does not establish which operation is running.

**Concrete redesign:** Use “Höchster Basiswert bestätigt” plus the value in a real computed result, and “2 gleichwertige Programme gefunden · Suche nach weiteren läuft”. Mark retained whole elements and new elements distinctly with text, and name the card a complete program if it is one. Offer element names as the default with codes available compactly. If enumeration runs automatically, use Pause/Resume and a separate “Weitere anzeigen” only for already found hidden items. If it runs in batches, use “Weitere Programme suchen” with an idle state between batches. Show completion only after exhaustion; never imply a known total earlier. Give Stop a clear consequence, e.g. “Suche beenden”; retain already found results. Preserve the conspicuous sample warning in this prototype. Do not invent a calculated BV in a mockup.

### 8. The four required uses are not all demonstrated by these screenshots

**Evidence:** Program construction, restriction editing, a personal conflict and a simulated result collection are visible. A full manual program's actual compliance result, exact whole-element selection/exclusion, combined jump/spin/step limits and catalogue search are not captured. The rule-check card explicitly says the real category check is not connected.

**Concrete redesign / verification:** Keep one workspace, with a rule-status summary next to the program and errors at affected elements when real checking exists. Test a complete manual program separately from optimization. For catalogue search, label the input “Element suchen” and keep it inside element selection; restriction editing must use “Ausschließen” and preserve active scope. Searching the catalogue must not silently change restrictions. This last risk is a design requirement, not an observed failure: source copy says search does not change repertoire, but no catalogue screenshot verifies the interaction. The next capture should demonstrate this separation instead of adding a paragraph asserting it.

## Exact text audit

Quoted text below is from screenshots unless explicitly marked source-only. Replacements are proposals, not validated translations.

| Current quote | Action | Replacement / context | Rationale |
|---|---|---|---|
| “Mein Programm” + “Dein Programm” | Remove duplicate | Keep one “Mein Programm” heading | Two ownership headings introduce the same object. |
| “Lege fest, was du laufen möchtest.” | Remove after controls improve | None | Does not identify an action unavailable from controls. |
| “Mein Repertoire” | Rename | “Persönliche Einschränkungen” | More accurate when no inventory of mastered skills is being recorded. |
| “Was darf in deinen Vorschlägen vorkommen?” | Remove after explicit controls | None | Presently compensates for uncertain checkbox meaning. |
| “Keine persönlichen Einschränkungen” | Keep | As collapsed summary | Useful state, not explanatory padding. |
| “Persönliche Einschränkungen: 1” | Rename | E.g. “Axel ausgeschlossen” | The affected choice matters more than an isolated count. Use a count only when accompanied by a useful summary. |
| “Repertoire anpassen” | Rename | “Einschränkungen bearbeiten” | Aligns trigger and destination. |
| “Ohne Limit” | Rename + add visible group label | “Keine zusätzliche Grenze”, under “Mein Maximum” | Separates personal settings from category rules. |
| “Kategorie erlaubt höchstens: Einfach” repeated | Move | Read-only category summary: “Sprünge: höchstens einfach” | Reduces repetition without hiding rule scope. Keep row exceptions local. |
| “Erlaubte Sprungarten gelten auch innerhalb von Kombinationen. Niedrigere Umdrehungen bleiben möglich.” | Replace through interaction, then remove | Scope in selected restriction; maximum select naturally denotes lower values | It carries essential semantics today. Deleting it without redesign is harmful. |
| Jump checkbox beside “Toeloop”, etc. | Keep label, clarify state | Visible “Zulassen” group label or explicit exclusion control | Names identify objects but not the action or polarity. |
| “Behalten” | Rename/test | “Bei Berechnung übernehmen” | Makes preservation conditional on calculation; still test exact-whole-element comprehension. |
| “Austauschbar” (source-only) | Rename/test | “Darf ersetzt werden” | Makes permission more explicit; not verified in initial screenshots. |
| “Füge Elemente hinzu. „Behalten“ sichert sie für die Berechnung.” | Remove after preservation control improves | None | Add button and explicit row control should carry this meaning. |
| “Maximaler Basiswert. Behaltene Elemente bleiben unverändert; ihre Position ist frei.” | Split/move | Search objective as “Höchster Basiswert”; placement represented separately | Objective is essential; preservation and position belong to controls. |
| “Rest berechnen” | Rename/test | “Optimales Programm suchen” | Current text implies a remainder even when the whole program can be replaced or an empty program can be optimized. |
| “Programmvorschläge” | Keep | Results heading | Identifies a distinct object. |
| “Hier erscheinen deine Vorschläge.” | Remove | Empty-state action/status only | Merely describes the empty area. |
| “Alle gleich optimalen Zusammenstellungen werden nach und nach gesammelt.” | Move to actual running state | E.g. “Maximum bestätigt · 4 gleichwertige Programme gefunden · Suche läuft” | Needed truth belongs with live result state, count and completion status. |
| “Einzelne Elemente ausschließen” | Rename/test | “Ganzes Element ausschließen” | Distinguishes whole-element exclusion from family/component restrictions. |
| “Auch Verbindungen mit unterschiedlicher Drehrichtung zulassen” | Move | Advanced restriction option | Relevant setting but excessive default priority. Keep a precise visible label when opened. |
| “Persönliche Einschränkungen zurücksetzen” | Keep/move | Within restriction editing, secondary action | Clear consequence and useful recovery. |
| “Durch dein Repertoire ausgeschlossen.” | Rename + actions | “Axel ist ausgeschlossen.” + named recovery actions | Names cause and exposes repair. |
| “Bedienprototyp · Beispielauswahl · keine echte Berechnung” | Keep in prototype | One persistent prototype status | Essential truth during evaluation; not eventual product copy. |
| “Regelprüfung im Entwurf” + development paragraph | Consolidate in prototype; replace only after real integration | “Regelprüfung noch nicht angebunden” in prototype status | Current location consumes program space and reads like a product feature heading. Never imply real verification now. |
| “Beispielablauf” | Keep in prototype | Close to simulated results | Prevents mistaking sample results for calculated programs. |
| “Maximum bewiesen · Alternativen werden gesammelt” | Rename | “Höchster Basiswert bestätigt · Weitere Programme werden gesucht” | Connects proof to the skating objective; use only after actual proof, and keep sample context now. |
| “Gefundene Alternativen: 2 · Gesamtzahl noch unbekannt” | Keep/shorten | “2 gleichwertige Programme gefunden · Suche läuft” | Count and incompleteness are essential truth; avoid suggesting that two is the total. |
| “Vorschlag für den freien Rest” | Remove/replace | “Vollständiges Programm” when the card includes retained elements | The visible cards contain both retained elements and proposed additions. |
| “Weitere Alternativen laden” | Rename according to actual behavior | “Weitere anzeigen” for known hidden results, or “Weitere Programme suchen” for new computation | Prevents confusing pagination with enumeration. |
| “Pausieren” | Keep conditionally | Pause only while enumeration is active; “Fortsetzen” when paused | A meaningful running state must match the control. |
| “Beenden” | Rename | “Suche beenden” | Names what stops and separates search from leaving the workspace. |
| “Zum Vergleich vormerken” | Rename/test | “Vergleichen” with selected state | Shorter visible action; preserve accessible state and make the resulting comparison reachable. |
| “Name oder Code suchen” (source-only) | Keep context, label visibly | “Element suchen” label; name/code placeholder optional | A catalogue query filters catalogue entries; exclusion is a separate named action. |
| “Entwurfsvergleich” / A/B/C names / preservation hint | Move outside product viewport | Review toolbar | Reviewer navigation, not skating task UI. |
| “Element hinzufügen”, category label, language label, element names/codes, errors | Keep | Contextually visible and accessibly named | Essential labels and truthful errors are not removable decoration. |

## A/B/C comparison

| Variant | Desktop evidence | Phone evidence | Best fit and cost |
|---|---|---|---|
| A | `a-desktop.jpg`: restrictions and program visible side by side; program and calculation fit in view. Narrow restriction column makes labels wrap and repeats caps. | `a-mobile.jpg`: restrictions collapse, making the program reachable. Still only one full element fits before the second. Filter panel is a long form. | Strong for repeated restriction/program iteration on desktop. Simultaneous controls impose more scanning on first use. |
| B | `b-desktop.jpg`: program receives most space; restrictions have a concise summary and explicit edit action. Results start near the bottom. Large top/row spacing still delays action. | `b-mobile-top.jpg`: unrestricted-state summary wraps into four lines; program starts lower than A. `b-mobile-results*.jpg` shows reachable results after scrolling but not first-screen priority. | Useful desktop baseline. On phone, replace the large summary with A's compact disclosure. Restriction editing requires opening a panel. |
| C | `c-desktop.jpg`: broad three-column restriction editor is easy to scan across; program begins below it. | `c-mobile.jpg`: repetitive restrictions fill the viewport; program construction is absent. | Good dedicated restriction-editing view. Weak default workspace for manual construction and immediate optimization. |

Do not infer a layout winner from typography, attractive spacing, or a single favorable scroll position. A and B can share the same mobile structure. C's broad restriction layout can be reused inside editing instead of being a separate default.

## Recommended next prototype change

Make one revision with B's desktop program-first layout and A's compact mobile restriction disclosure, keeping A available as a desktop comparison. Change only the shared interaction layer: compact program rows, one explicit preservation state, named active restrictions, and two inline conflict-recovery actions. Put the reviewer switch outside the viewport. Then compare the same initial viewport and the same conflict across A and B. This is a small next step, not approval to rebuild the entire product or create a design system.

Use four observable tasks: start with an empty program and request the maximum; preserve one exact combination while allowing it to move; exclude a component across combinations without excluding only one whole element; manually assemble a program and identify its rule status. Record first action, wrong turns, hidden required controls and successful recovery. Do not pre-explain Keep or repertoire. Test separately with adult and age-12+ participants; these screenshots provide no evidence that either group understood them.

## Method, references and limitations

Read the complete `ui-ux-pro-max/SKILL.md`, full `references/quick-reference.md`, and `references/pro-rules.md` (native-specific units were not applied as web conformance criteria). No new design system was generated. Primary applicable sections: accessibility semantics, content priority/mobile layout, field labels, progressive disclosure, error clarity/recovery, and navigation/state preservation.

Targeted searches used:

- `"mobile content priority disclosure" --domain ux`: top result Mobile Keyboards was off-topic for layout; Mobile First was relevant. Retried with `"progressive disclosure" --domain ux`; Heading Line Balance was off-topic. Content-priority/progressive-disclosure advice therefore uses the skill's explicit quick-reference guidance, not a claimed verified search match.
- `"error clarity recovery" --domain ux`: Error Recovery and Error Messages matched the conflict use case.
- `"input labels field grouping" --domain ux`: Input Labels and Error Placement matched the restriction controls.
- `"overwhelm options" --domain ux`, retry `"complex options" --domain ux`: no database match. Related recommendations use the built-in progressive-disclosure guidance.

All 11 supplied JPEGs were opened with `view_image`: `a-desktop.jpg`, `b-desktop.jpg`, `c-desktop.jpg`, `a-mobile.jpg`, `b-mobile-top.jpg`, `c-mobile.jpg`, `a-mobile-filter.jpg`, `a-mobile-conflict.jpg`, `b-mobile.jpg`, `b-mobile-results.jpg`, and `b-mobile-results-lower.jpg`. No browser interaction or UI source changes were made by this reviewer. Initial assessment used screenshots first. A narrow subsequent read of `workspace-prototype/src/ui-workspace.prototype.html` found copy for “Austauschbar”, result phases, catalogue search and recovery; source-only observations are explicitly distinguished above. This does not prove actual runtime behavior. Layout dimensions are approximate image observations, not CSS measurements. Keyboard operation, screen-reader names, focus order, contrast ratios, actual hit areas, performance, solver correctness and category rules were not audited. No WCAG compliance claim is made. English/Russian layouts and long localized strings remain unverified. Paused/exhausted/outdated result states, an open catalogue and an actual Replaceable row were not captured, so their interaction quality remains unverified.


## Capture index (added by coordinator)

These are original browser captures, without image editing. Use `b-mobile-top.jpg` for the initial B screen; `b-mobile.jpg` is a scrolled supplementary view. Mobile means a desktop-browser viewport override, not a physical-phone test.

- [a-desktop.jpg](screenshots/a-desktop.jpg)
- [a-mobile-conflict.jpg](screenshots/a-mobile-conflict.jpg)
- [a-mobile-filter.jpg](screenshots/a-mobile-filter.jpg)
- [a-mobile.jpg](screenshots/a-mobile.jpg)
- [b-desktop.jpg](screenshots/b-desktop.jpg)
- [b-mobile-results-lower.jpg](screenshots/b-mobile-results-lower.jpg)
- [b-mobile-results.jpg](screenshots/b-mobile-results.jpg)
- [b-mobile-top.jpg](screenshots/b-mobile-top.jpg)
- [b-mobile.jpg](screenshots/b-mobile.jpg)
- [c-desktop.jpg](screenshots/c-desktop.jpg)
- [c-mobile.jpg](screenshots/c-mobile.jpg)
