---
id: AV-112
title: Git branch and session title on the row and toast
status: open
depends: []
---

# AV-112 — Git branch and session title on the row and toast

**What to build:** Tell several sessions in one repo apart.

- [ ] Session title: the `/rename` name or ai-title when Claude Code records one (verify where: sessions file or transcript), else today's name
- [ ] Git branch of the session's cwd, read cheaply (`.git/HEAD`), refreshed on state change
- [ ] Row shows the title; branch as a tag on the open row; toast title `<title> · <branch>`
- [ ] Parser tests with fixtures; Shoot both themes

**Done when:** two sessions on different branches of one repo show distinct titles and branches on rows and toasts; the owner confirms.
