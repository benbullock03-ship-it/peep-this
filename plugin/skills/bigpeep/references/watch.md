# Watching little peeps

Big Peep never stops a little peep. Quiet is asked about, not acted on.

## Timer

When a wave starts, run `sleep 300` in the background. Each time it fires, check every running little peep, then start it again while any is still running.

## Quiet

For each little peep keep its last sign of life, starting at spawn:
- a message from it (a reply or a notification);
- a change in `git status --porcelain -- <its paths>`, or in the modified times of those files;
- a commit touching its paths (little peeps do not commit, so this only catches a mistake).

Quiet for 10 minutes or more: send it one message with SendMessage, by its name:

```
Big Peep here. One line: where are you on <id>, and what are you waiting on? Keep working.
```

Then reset its quiet clock. It is asked again only after another 10 quiet minutes.

## Reporting the answer

- On reply, tell the owner one line: `<id> status (<m> min quiet): <answer>`.
- No reply by the next check: tell the owner `<id>: no answer to status ask`. Still do not stop it.
- Every ask goes in that ticket's notes in the report: `asked at <m>m: <answer, or "no answer">`.
