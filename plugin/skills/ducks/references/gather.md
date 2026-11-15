# Gather

Cheap reads only. Never read the archive whole, never run `build`, run `test` once.

## Commands

1. `git status --short` and `git branch --show-current`.
2. Frontmatter of every `<tickets>*.md` (F§2): `id`, `status`, `depends`, `reason`, `wake`. Read the first lines of each file only.
3. For each `doing` ticket: `git log -1 --format=%cr%x09%h --grep="<id>:"`.
4. The setup's `test` command, once, with a 5-minute limit. Keep the exit code and the first few failing test names. No `test`: "no test command".
5. `CLAUDE.md` at the root, if present.

## Loose ends

| Kind | Found when |
|---|---|
| Stale `doing` | status `doing`, no commit naming `<id>:` in the last 7 days, and no uncommitted change to its ticket file |
| `done`, waiting | status `done`: Claude's claim, waiting for the owner's close |
| Wake met | parked, `wake: ticket:<X>` and X is closed (see below) |
| Wake met | parked, `wake: date:<D>` and D is today or earlier |
| Wake not met | parked, condition not yet met: listed, no action |
| Wake free text | parked, `wake` is neither form: listed for the owner, never judged |
| Parked, broken | parked without `reason` or `wake` (F§2 says it must have both) |
| Failing tests | `test` exit code not 0 |
| Uncommitted | any line in `git status`. Mark "not mine" if the handoff lists it, "unrelated" if it is outside `<tickets>` and no `doing` ticket names its path |
| Rule that bites | a `CLAUDE.md` rule the next ticket's paths or kind would trip (commits, file edits, generated files, release). At most 3 |

## Is ticket X closed?

X is closed when its file in `<tickets>` says `status: closed`, or `<archive><X>-*.md` exists. Look in the archive for that one id only. X not found anywhere: treat as free text and say "X not found".

## Next ticket

The handoff's "Then:" ticket if it is still `open` or `doing`; else the lowest-numbered `open` ticket whose `depends` are all closed. Tickets the answers will wake count as `open`.
