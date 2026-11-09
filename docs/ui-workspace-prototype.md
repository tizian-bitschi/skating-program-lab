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
- C2: a user-requested subvariant of C with vertical, discrete mixer faders. Open `?variant=C2` or use the existing comparison switcher; A/B/C remain available.

A shared comparison switcher changes `?variant=A|B|C|C2`; it sits at the bottom in A/B/C and in the header of desktop C2. Left/right keys switch when not editing or operating a control. Variants share in-memory request state. Language can switch among German, English and Russian without clearing the plan. Display names/translations are prototype copy awaiting native-domain review.

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


## Independent screenshot review

At the user's request, an independent subagent reviewed eleven desktop/mobile screenshots using ui-ux-pro-max. See the [review and text audit](prototype-ui-review/review.md). The report recommends improving the shared interaction model before choosing the final layout. Its proposed desktop B/mobile A combination is not a user-approved decision. No UI source changes were made during this review.

## Mixer subvariant C2

The user proposed a mixer/DJ-console interaction. The prototype tests one fader per jump kind or exact spin form, not a fader per occurrence in the program. This is an explicit working interpretation awaiting feedback.

- Jumps: Off, then the supported listed classes up to the selected category ceiling. Readouts show exact codes such as 1A or 2T, avoiding confusion between listed class and physical turns.
- Spins and steps: Off, Basic, then levels 1–4 within the category ceiling. Choreographic choices stay binary; they do not acquire invented levels.
- Moving to Off excludes the kind/form. Changing a nonzero fader records that exact personal maximum, including when it matches the category ceiling. Thus an explicit Single limit survives a later switch to a category permitting doubles. Absent personal limits initially display the effective category ceiling until edited; Reset restores unrestricted personal settings. Existing exact exclusions remain available under More restrictions.
- Native vertical range inputs support dragging, track activation and keyboard control. Live readouts update during movement; the request/history commits on change, so dragging does not repeatedly recreate the input.
- The mixer shares the existing request, conflict reporting and stale-result behavior. Changing C2 to C retains the same settings. Mobile uses three channels per row without horizontal scrolling.

Observed checks: ArrowDown excluded a kept Axel and produced a conflict; ArrowUp under Masters restored Axel with a maximum of Single; switching to C showed the same maximum. Home/ArrowUp on CSSp selected Basic and flagged the kept CSSp2. A step fader was excluded, and selecting Bronze disabled unavailable step content. At 320 and 390 CSS-pixel viewports, page scroll width equaled client width. JavaScript parsing and whitespace checks passed. No physical-phone or participant test was run; no real solver integration was added. The earlier independent review covers A/B/C, not C2.

[Desktop screenshot](prototype-ui-review/mixer/c2-desktop.jpg) · [Mobile screenshot](prototype-ui-review/mixer/c2-mobile.jpg)

## Compact desktop C2 revision

The user rejected the centered layout, wide strips and repertoire tabs. C2 now uses the full desktop viewport with 16px workspace side padding, 68px channel strips, all fourteen jump/spin/step faders and both choreographic switches visible at once. Program editing and suggestions share the lower half. Extra restrictions open in a separate dialog; the main repertoire has no family tabs. The review switcher moves into the header instead of covering the program. A/B/C retain their existing layouts.

The dense desktop mode is viewport-constrained. Main controls stay visible; program and result lists can scroll internally when content exceeds their panels. This is not a promise to show arbitrarily many alternatives simultaneously. Below the desktop breakpoint the grouped, tab-free controls reflow and the document may scroll; the fixed-viewport target in this revision is desktop.

Exact spin abbreviations label compact channels. Full names remain accessible labels and titles, and appear in the shared inspector when a fader receives focus. No repeated explanatory paragraphs are needed in the mixer area. This trade-off still needs human comprehension review.

Observed connected-Chrome checks with the two-element Silver sample and two simulated alternatives:

- 1440×900: document scroll dimensions exactly 1440×900; 14 faders; 68px strip width; no repertoire bank overflow; calculation action within the viewport.
- 1280×720: document scroll dimensions exactly 1280×720; banks fit; both result rows fit without result-list overflow after condensing rows; calculation and results end at y=708.
- Extra restrictions dialog opens, closes with Escape and restores focus. Fader focus exposes the full spin name. JavaScript syntax and whitespace checks pass.

These are prototype browser/layout checks, not physical-device, participant or backend correctness tests. Calculation remains simulated. Earlier screenshots are preserved as historical versions.

[Compact 1440×900 screenshot](prototype-ui-review/mixer/c2-compact-desktop.jpg) · [Compact 1280×720 screenshot](prototype-ui-review/mixer/c2-compact-1280.jpg)

## Interactive connection exclusions

The user clarified that personal level maxima are downward-inclusive: selecting level 3 includes Basic and levels 1/2/3. Excluding an individual lower spin level while allowing a higher one is not a required product interaction. C2 therefore exposes additional exact exclusions only for whole jump combinations/sequences. The older A/B/C experiments and the library's public capabilities are not removed.

The C2 header action is now **Kombis & Sequenzen**. It opens a touch/click builder: choose Combination or Sequence, select the first jump, then select a continuation from the illustrative choices. Selected jumps form a visible whole connection. A third jump is explicitly optional when an exact two-jump choice already exists. Clicking a selected block edits from that position. Exclude acts on the complete current connection; excluded entries can be allowed again directly. Single jumps, their maxima and spin/step settings are not changed by that action. Mixed-direction preference remains available separately within the panel.

No code typing or individual spin-level exclusion is offered in this C2 panel. Any exact non-connection exclusion carried over from another experiment remains visible in a legacy disclosure with removal only, so switching layouts never silently relaxes existing requests.

The prototype uses a curated sample repertoire, not a newly implemented domain rule engine. Continuations are selected from that finite sample pool, filtered using the existing prototype category/repertoire checks. Already excluded exact connections remain in the choice pool, allowing restoration and distinct longer connections. In a product adapter, the authoritative permitted domain must come from Rust. Missing sample paths do not prove rule infeasibility. Calculation is still simulated.

Observed Chrome checks: Lutz → Toeloop excluded only `1Lz+1T` while all jump-kind permissions remained true; Axel → Axel in sequence mode additionally excluded `1A+1A+SEQ`; allowing the Lutz connection again preserved the sequence exclusion. The optional third-jump path remained visible after excluding the exact two-jump whole. A 390px viewport showed no horizontal dialog overflow and no text-entry fields. Keyboard focus moves to the next choices or the whole-connection action. Syntax and whitespace checks pass; no physical-phone, participant or full-rule coverage claim is made.

[Builder desktop screenshot](prototype-ui-review/connections/builder-desktop.jpg) · [Builder mobile screenshot](prototype-ui-review/connections/builder-mobile.jpg)
