---
id: AV-33
title: Opt-in model summary for the digest
status: todo
depends: [AV-32]
attended: true
---

# AV-33 — Opt-in model summary for the digest

**What to build:** Stub. A setting, off by default, that adds one model-written sentence per session to the away digest (spec §11.9).

- [ ] Headless `claude -p` on the Haiku model, fed the session's event facts and last assistant message; never the transcript wholesale
- [ ] The summary session must not appear as a row or trigger hooks into the widget
- [ ] Facts card renders first; sentences fill in when ready; failure or offline leaves the facts card as is
- [ ] Setting text states that it spends rate limit and sends session facts to the model

**Done when:** the owner turned it on, returned to a digest with sensible one-line summaries, and saw no phantom session row.

Decide: attended because it spends the owner's quota and changes the offline promise.
