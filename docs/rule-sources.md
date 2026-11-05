# Rule sources and open questions

The [assumed-recognition coverage contract](specs/assumed-recognition.md) explains which construction rules are evaluated under an exact selection. Individual values retain their SOV row, page and edition.

Checked 2026-11-01. Page numbers below refer to printed document pages. The source list supports the scoped requirements, not a complete rule database. Confirm current event-specific versions and later amendments before implementing each rule; preserve the source version and precise location with every rule and fixture.

| Source and date | Relevant locations and use | Official source |
| --- | --- | --- |
| International Adult Figure Skating Competitions Technical Announcement, updated 2026-07-16 | pp. 2, 4: scope and precedence; p. 6: time deductions; pp. 8–11: Masters, Gold, Silver and Bronze singles Free Skating. Applies to Vernon 2026 and Oberstdorf 2027 as stated on p. 2. Downloaded official PDF SHA-256: `1bda1ba7e5faee8d9faeced87f2fc4b5994dfead2cd5bdc82ad711e507264da1`. | [Adult announcement](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Fansite/Figure-Skating/2026-2027/Events/Adult-Skating/Documents/IAFSC-Technical-Announcement---2026-27--1--1786956273-6936.pdf); [Vernon organizer's July 16 copy](https://skatinginbcresults.com/web/events/announcements/2026-2027/IAFSC%20Technical%20Announcement%202026-27%20071626.pdf) |
| ISU Communication 2786, updated 2026-06-02, valid from 2026-07-01 | pp. 2, 8, 14: jumps, spin and choreographic values, GOE columns and combination reference-jump remarks. The updated date and table labels were verified visually; text extraction loses table rows. Replaces 2707. | [Updated Scale of Values](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2786-SinglesPairs-SOV-Update-260602-1780481099-6311.pdf) |
| ISU Communication 2788, updated 2026-05-19 | pp. 2–6: levels, calls and choreographic recognition; pp. 12, 14, 16: GOE guidelines and starting limits. Replaces 2701. | [Levels and GOE](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2788-SP-Levels-and-GOE-2026-Updated-MAY-19-1779351385-9626.pdf) |
| Technical Panel Handbook, Single Skating 2026/27, official index dated 2026-07-15 | pp. 2, 6, 25, 30–33: recognition and calls, including V, *, +REP, +COMBO and +SEQ. [Federation-hosted copy](https://s3.us-east-2.amazonaws.com/sidearm.nextgen.sites/usafs.sidearmsports.com/documents/2026/7/23/TP-HandbookSingles-26-27-15-July-2026.pdf). | [Handbook PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/TP-HandbookSingles-26-27-15-July-2026-1784641784-3364.pdf) |
| Sports Rules — Figure Skating, effective 2026-06-12 | Rule 353.1(b–h, n), pp. 9–10; Rule 502 pp. 50–51; Rule 612 pp. 70–72: GOE aggregation, second-half factor, duration, deductions and element definitions. Apply Adult provisions where specified. | [Sports Rules PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf) |
| Referees and Judges Handbook 2026/27, cover dated 2026-05-19, index dated 2026-05-26 | pp. 2–3, fall in choreographic sequence; pp. 2, 6, starting-GOE restrictions for any execution error and single q; p. 5, +REP has no intrinsic GOE reduction. A single q starts at most +3 and finishes at most +1; multiple errors start at most +2. | [Judges handbook PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/Handbook-for-Referees-and-Judges-2026-27-1779278300-9402.pdf) |
| Technical Committee Q&A 1, April 2026, index 2026-04-21 | p. 1, Choreospins Q1: three consecutive revolutions on the same foot, on one or two blades. | [Q&A 1 PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/QA1-SPTC-Choreoelements-1776773178-2054.pdf) |
| Technical Committee Q&A 2, May 2026, index 2026-05-19 | p. 1: step-sequence feature 5 depends on a qualifying feature 4 combination. Adult category restrictions still apply. | [Q&A 2 PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/QA2-SPTC-1779180084-8133.pdf) |
| Technical Committee Q&A 3, October 2026, index 2026-10-08 | p. 1 Q2–4: calls and GOE for starred jumps in Free Skating combinations/sequences; panel recognition of difficult camel variations. | [Q&A 3 PDF](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/uploads/isunewrevamp/document/original/f0e21d6e25df4e42395cb70a6c06b269b590597e823034cbd94476e1ea6dd502.pdf) |

The [official handbook index](https://isu.org/en/figure-skating-rules/figure-skating-rule/handbooks-single-and-pair-skating) lists handbooks and Q&As. The May 5 original 2786 is superseded for the examples here by the identified June 2 update. Check for subsequent revisions at implementation time.

## Grand Prix protocol observations

The [2026/27 Grand Prix fixture inventory](fixtures/grand-prix-2026-27.md)
records four official France/Canada Men/Women Free Skating Judges Scores reports,
retrieved 2026-11-04, with exact segment URLs, printed editions and SHA-256 digests.
Twelve selected real programs preserve recognized codes/calls, credited BASE,
GOE, individual judge grades, scores, PCS and separate deductions with field
locations. These are observed panel reports, not legality/optimality labels or
new SOV data. The existing June 2, 2026 Communication 2786 edition still applies.

## Applicability and precedence

The Adult announcement p. 4 applies the ISU Constitution, Sports Rules, pertinent Communications and the announcement. For a conflict with pertinent regulations or Communications it states that the announcement prevails, provided the Communication contains no special comments for Adult competitions. Apply that explicit qualification to the affected provision; do not claim a blanket precedence for all later material. Q&As clarify their stated subject and scope; check dates, event applicability and any later amendments. An unresolved conflict or scope question blocks a definitive affected finding, score or optimum, with the uncertainty and sources returned to the user.

A published [ISU protocol](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Fansite/Figure-Skating/2025-2026/Events/International-Competitions/Documents/Protocols/CrystalSkateO2025-Protocol-1764858603-091.pdf) identifies x as second-half credit and F as a fall. This illustrates result notation, not applicability to the selected Adult events.

## Euler and remaining questions

- Adult pp. 8–10 permit up to two combinations or one combination without Euler and one sequence. An Euler between listed jumps in a combination or sequence is not listed, is allowed once per program, and no foot change may follow before the next jump. Bronze explicitly forbids it in its combination (p. 11). The general handbook p. 33 gives the consequences of a separate Euler combination and sequence in either order: subsequent jumps are starred and the first jump receives +REP. Check Adult-specific application of those consequences, including Bronze limits, before scoring.
- Confirm whether general rules on jump-type repetition and element counting apply unchanged to Adult categories alongside the announcement's listed-jump limit. Check full technical consequences, not only plan violations.
- Sports Rule 353.1(h)(iii), p. 9, specifies a 1.1 factor on BV, not GOE, for the last three eligible Free Skating jump elements started in the second half of the required duration, rounded individually to two decimals, with interruption qualifications. The July 16 Adult announcement applies the Sports Rules and contains no explicit second-half exception; older or other competition announcements are not evidence for these events. The source-based default is to apply the general rule. No event-specific exception was found in the sources checked; check subsequent event notices and applicable Adult Communications before asserting an official result. Record the jump timings and any applicable exception.
- Confirm the interaction of Adult choreographic-sequence wording with listed movements, and panel fallback identification of a choreographic spin. Q&A 1 resolves the same-foot condition but does not settle every Adult-specific scenario.
- Verify all applicable calls, GOE rounding and duration consequences, including incomplete programs, with source-derived fixtures. User-supplied final GOE is not a substitute for a judges' panel result.
- Define and review the optimizer's finite candidate domain, feature evidence and equivalence for each run; a proof covers only that model. Obtain independent domain review before claiming scoring or full rule coverage.

## Direct two-jump transition subset

[ISU Sports Rules 2026, Rule 610, p. 65](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf) define a combination using the landing foot as the next take-off foot. The [U.S. Figure Skating judges' manual, p. 43](https://www.usfigureskating.org/system/files/Test%20Judge%20Manual%20for%20Singles%20Pairs.pdf), describes the usual back outside landing edge and the matching take-off edge for a loop or toe loop, with mirrored feet for clockwise skaters. This supports the bounded same-direction direct transition ending in a toe loop or loop; a plan does not establish the actual landing, transition or panel recognition. Other direct transitions remain unknown, not automatically illegal. The implementation does not infer Euler footwork or sequence steps from a jump name.

[Sports Rules Rule 353.1(h)(i), p. 9](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf) gives one GOE for the most difficult jump after summing combination base values. Communication 2786 p. 14 identifies difficulty by the applicable reduced base value. For example, a recognized clean `1Lz+1T` has BV 1.00 and GOE +2 of 0.12, giving 1.12 before timing credit. This calculation is not a panel decision and requires independent domain review.

## Registering a new official profile

Follow the [profile ADR](adr/0002-rule-profiles-proposal.md). Verify the exact
edition and event/federation applicability of each source before registering
policy. Bind rule identifiers to precise citations; record the qualified
precedence and explicitly scoped replacement for every event override. Supply
independent source-derived fixtures, review the supported mechanism versions
and retain unsupported or ambiguous semantics explicitly. Structural registry
validation does not verify authority or resolve a regulatory dispute.

Create a new profile version for policy or source changes and keep prior entries.
Select SOV editions separately; category eligibility does not alter catalog
values. Synthetic test provenance never establishes an official class. The
current Adult definitions reuse the editions and limitations documented above.

The profile source inventory must contain the exact selected SOV document edition
and every rule/policy citation. Scoring binding overrides are currently rejected
as unsupported rather than ignored; use an explicitly cited new scoring-policy
version. Jump-rule citation overrides remain scoped to the named event.
