# Issue tracker: GitHub

Issues and specs live in GitHub Issues for
`tizian-bitschi/skating-program-lab`. Use the `gh` CLI.

## Repository selection

Run commands from this repository's worktree. Before creating the first
issue in a session, verify the target:

gh repo view --json nameWithOwner

The result must be `tizian-bitschi/skating-program-lab`. If it differs,
resolve the mismatch before writing. Never create project issues in the
repository that distributes these skills.

## Common operations

- Create: `gh issue create --title "..." --body-file <file>`
- Read: `gh issue view <number> --json number,title,body,labels,comments`
- List: `gh issue list --state open --json number,title,body,labels`
- Read comments: `gh issue view <number> --comments`
- Comment: `gh issue comment <number> --body-file <file>`
- Add a label: `gh issue edit <number> --add-label "..."`
- Remove a label: `gh issue edit <number> --remove-label "..."`
- Assign to the current user: `gh issue edit <number> --add-assignee @me`

For multiline bodies, write the exact text to a temporary file and pass
it with `--body-file`.

To close an issue, post the explanation first, then run
`gh issue close <number>`. If the issue is already closed, post the
comment separately.

## Parent and child issues

Use GitHub sub-issues when available.

With `gh` 2.94 or later:

- Create a child: `gh issue create --parent <parent> ...`
- Attach an existing child:
  `gh issue edit <parent> --add-sub-issue <child>`

With an older CLI, obtain the child's database ID:

gh api repos/tizian-bitschi/skating-program-lab/issues/<child> --jq .id

Then attach it:

gh api --method POST repos/tizian-bitschi/skating-program-lab/issues/<parent>/sub_issues -F sub_issue_id=<child-database-id>

If sub-issues are unavailable, add `Part of #<parent>` to the beginning
of the child body and link the child in the parent's task list.

## Blocking dependencies

Use GitHub's native issue dependencies when available.

With `gh` 2.94 or later:

- Create with blockers: `gh issue create --blocked-by <n>,<n> ...`
- Add a blocker:
  `gh issue edit <child> --add-blocked-by <blocker>`

With an older CLI, obtain the blocker's database ID:

gh api repos/tizian-bitschi/skating-program-lab/issues/<blocker> --jq .id

Then add the dependency:

gh api --method POST repos/tizian-bitschi/skating-program-lab/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-database-id>

Use numeric database IDs for these API calls, not issue numbers or
node IDs.

Count open blockers:

gh api repos/tizian-bitschi/skating-program-lab/issues/<number> --jq .issue_dependencies_summary.blocked_by

If native dependencies are unavailable, add
`Blocked by: #<n>, #<n>` to the beginning of the child body.
An issue is unblocked when every blocker is closed.

## Pull requests as a triage surface

PRs as a request surface: no.

## Skill terminology

- "Publish to the issue tracker" means create a GitHub issue.
- "Fetch the relevant ticket" means read the GitHub issue and its comments.
- For triage roles, use `docs/agents/triage-labels.md`.

## Wayfinding operations

The map is one issue labelled `wayfinder:map`. Its body contains Notes,
Decisions-so-far and Fog.

- Create the map with `gh issue create --label wayfinder:map ...`.
- Link child tickets as sub-issues. If unavailable, use the parent task
  list and `Part of #<parent>` convention above.
- Label children by type: `wayfinder:research`, `wayfinder:prototype`,
  `wayfinder:grilling` or `wayfinder:task`.
- Represent blockers using the dependency rules above.
- To find the next ticket, inspect the map's open children in map order.
  Skip tickets with an open blocker or an assignee.
- Claim a ticket with `gh issue edit <number> --add-assignee @me`
  before other writes for that ticket.
- To resolve a ticket, comment with the answer, close it, and append a
  concise result with a link to the map's Decisions-so-far section.
