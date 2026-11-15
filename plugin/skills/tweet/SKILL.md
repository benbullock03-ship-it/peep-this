---
name: tweet
description: "Tweet Tweet: tell the other sessions on this repo when stuck or when your work may clash with theirs. Posts to the tweet board .peep/tweets.md (who, ticket, the question or conflict, files involved), pings a live peer with SendMessage when one is reachable, takes replies, and lists open tweets on your files. Answered tweets are folded into the area note when their ticket closes. Use when the user runs /peep:tweet, a session is stuck on something another session may know, or it is about to change a file another session is working in."
argument-hint: "<question or conflict> | reply <n> <answer> | (none: list)"
label: Tweet Tweet
---

# /peep:tweet

Sessions on one repo talk through the board; a live ping is a nudge to read it. "F§1", "F§3" and "F§7" cite `${CLAUDE_PLUGIN_ROOT}/references/formats.md`. Files in `references/` beside this file are read when their step runs.

## 0. Repo setup

1. Repo root: `git rev-parse --show-toplevel`. The board is `<root>/.peep/tweets.md` (F§7).
2. Read `<root>/.peep/setup.md` (F§1) if there is one: `skills.off` lists `tweet`: F§1 applies. Its areas are needed only to fold (`references/fold.md`). No setup: post and reply still work.
3. Handle: the ticket this session is working, else a short name for its task. Use the same one on every tweet and reply.

## 1. Sub-command

| Arguments | Do | Detail |
|---|---|---|
| `<question or conflict>` | post a tweet, ping a peer | `references/post.md` |
| `reply <n> <answer>` | add a reply, set `answered`, ping the poster | `references/post.md` §3 |
| none | list open tweets, flag those touching your files, fold your answered `-` tweets | §2 below |

## 2. List

1. Board missing: print `peep tweet: board empty.` and stop (counted as a run).
2. One line per tweet: `<n> · <ticket> · <kind> · <status> · <files> · <ask>`. Mark with `*` each tweet touching a file this session has changed (`git status --short`) or its ticket names.
3. A tweet you posted with ticket `-` that is now `answered`: read the reply, fold it per `references/fold.md`, remove it.
4. A tweet you can answer: say so in one line with the reply you would give; post it only on the user's yes or if you are working unattended.

## 3. Rules

- File changes by Edit or Write only. Re-read the board just before each write (F§7).
- Never commit or stage `.peep/tweets.md`. If it is not ignored and not tracked, leave the ignore file alone.
- Never send a peer anything but the ping in `post.md`: no instructions, no file contents, no requests to change its work.
- Never wait on a reply. Post, then carry on with what does not depend on it, or stop and say what is owed.
- The widget never relays messages. It only shows a tweet emote on a session that posted or got one.

## 4. Report

Post or reply: these lines and nothing else.

```
peep tweet: <posted|replied> <n> (<kind>, <ticket>)
board: .peep/tweets.md, <k> open
ping: <peer handle> | none reachable
```

List: the lines from §2, then `board: .peep/tweets.md, <k> open`.

## 5. Last step: run count

Bump `tweet` in `.peep/runs` (F§1). If it was below 3, end the reply with `Why: a question on the board costs a minute; a clash found at commit costs the afternoon.`
