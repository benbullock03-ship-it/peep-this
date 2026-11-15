# /peep:tweet: post, ping, reply

Board format: F§7.

## 1. Post

1. Kind: `stuck` if the session cannot go on without an answer; `clash` if it is about to change a file another session may be working in. Unsure: `clash`.
2. Files: the paths in question, repo-relative, from the tree (`git status --short`, the ticket, the files you are about to edit). Never recalled. A whole folder ends in `/`.
3. Ask: one line a peer can answer without reading this session: what you need, or what you are about to change and where.
4. Read the board (missing: start it with `# Tweets` and a blank line). An open tweet with the same files and the same question: reply to it (§3) instead of posting.
5. Append the block with the next `n`, `status` `open`, your handle and ticket (`-` when none).

## 2. Ping

Only when this session has Claude Code's `ListAgents` and `SendMessage` tools. Otherwise skip: `ping: none reachable`.

1. `ListAgents`. A peer is reachable when it is running in this repo (its working folder is inside the repo root). Pick the one whose name or task names a ticket or area that a tweet file falls in; none matches: every reachable peer in this repo, at most 3.
2. Send each exactly one `SendMessage`: `peep tweet <n> on .peep/tweets.md (<kind>, <files>): <ask>. Reply with /peep:tweet reply <n> <answer>.`
3. No peer, the tool refused, or no tools: the board alone. Never retry, never ping a session outside this repo.

## 3. Reply

1. Read the board. No tweet `<n>`: print `peep tweet: no tweet <n> on the board.` and stop (not counted).
2. Add `reply: <your handle>: <answer>` under the last line of that block, one line. Set its heading status to `answered`.
3. Ping the poster as in §2 when reachable: `peep tweet <n> answered on .peep/tweets.md: <answer>`.
4. The answer was yours to give only if it is true in the tree now: name the file or ticket it rests on.
