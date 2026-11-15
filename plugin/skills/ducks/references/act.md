# Act on the yes

Only the **Do** lines of the confirmed read-back, in this order. Edit or Write only. Stage by name, never `git add -A` or `.`. Never push.

1. **Wake** `<id>`: set `status: open`, delete the `reason` and `wake` lines, keep every other key. Commit the ticket file as `<id>: wake (<condition met>)`.
2. **Status change** (`<id> -> open`): set the status; delete `reason`/`wake` if present. Commit as `<id>: <old> -> <new>`.
3. **Close** `<id>`: follow `${CLAUDE_PLUGIN_ROOT}/skills/ticket/references/close.md` step 2, one ticket at a time. Step 1 there is already met: the read-back named it and the owner said yes.
4. After all of them, run the setup's `index` command once if it has one, and commit the index file by name as `ducks: index` if it changed.

A step fails (commit refused, file moved): stop there, leave the rest undone, and say which under `loose:` in the output.

A ticket parked on one closed in this run (`wake: ticket:<closed id>`) is not woken unconfirmed: list it under `loose:` as `ready to wake next run: <id>`.
