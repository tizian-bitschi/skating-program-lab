# Domain docs

This repository uses a single-context layout:

- `GLOSSARY.md` at the repository root defines domain vocabulary.
- `docs/adr/` contains architecture decision records.

## Before exploring

Read the root `GLOSSARY.md` and ADRs relevant to the task.

If `GLOSSARY-MAP.md` exists, follow its links and read the glossaries
relevant to the task.

If a glossary or ADR directory does not exist, proceed silently.
Do not suggest creating it solely because it is missing. The
`domain-modeling` skill creates domain documents when terms or decisions
are resolved.

## Use the glossary's vocabulary

Use glossary terms consistently in issue titles, specifications,
hypotheses, refactor proposals and test names. Respect explicitly
discouraged synonyms.

If a required concept is absent, check whether it matches the project's
existing language. Record a substantive vocabulary gap for
`domain-modeling`.

## Surface ADR conflicts

If a proposal contradicts an existing ADR, identify the ADR, explain the
conflict and state why the decision needs reconsideration. Do not
silently override an existing decision.
