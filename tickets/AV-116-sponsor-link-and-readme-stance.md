---
id: AV-116
title: Sponsor link and README stance
status: doing
depends: []
attended: true
---

# AV-116 — Sponsor link and README stance

**What to build:** A quiet way to support the project, and the trust statement competitors lack.

- [ ] Tray About: one sponsor link (GitHub Sponsors or Ko-fi, owner picks the account); no nag, no badge in the panel
- [x] README: offline, no telemetry, Windows-first, what the integration install changes and how to undo it; the sponsor link
- [x] Draft the awesome-claude-code submission text in this ticket; submitting is an outward step and needs the owner's yes at the time

**Done when:** the About link opens the sponsor page, the README states the stance, and the submission draft is in this ticket.

## awesome-claude-code submission draft

Not submitted. Submitting needs the owner's yes at the time, a public repo link, a licence and the author fields filled.

- **Display name:** Peep This
- **Category:** Tooling (sub-category: usage monitors, or general if the list has no fit)
- **Primary link:** `<public repo URL, owner to fill>`
- **Author name / link:** `<owner to fill>`
- **License:** `<owner to pick; the repo has no LICENSE yet>`
- **Description:** An always-on-top Windows widget that lists every Claude Code session on the machine, each as a pixel peep showing its state (thinking, working, waiting, stuck). It alerts when a session needs input, shows context and 5-hour/7-day rate-limit usage, and answers permission prompts inline with risk tiers. Ships with the `peep` skill pack. Offline, no telemetry; the integration install backs up `settings.json` and uninstalls cleanly.
- **Validate claims:** Install, choose Install integration from the tray, open two Claude Code sessions, and ask one to run a shell command: its row turns to waiting and the approval card shows the full command.
- **Specific task:** Run two sessions at once and approve a tool call from the widget without switching to the terminal.

## Decisions taken without a ruling

- Sponsor platform: GitHub Sponsors (0% fees, listed first in `reports/2026-11-16-reflective-pass.md`); no account is recorded in the repo, so `SPONSOR_URL` in `src-tauri/src/tray.rs` is a marked placeholder, mirrored in the README Support link. The owner must fill both.
- The sponsor link sits in a tray **About** submenu (new, above Quit) holding a disabled version line and "Sponsor Peep This".
- URLs open with `ShellExecuteW` from the already-enabled `windows-sys` `Win32_UI_Shell` feature rather than adding `tauri-plugin-opener`; no capability change.
- README says the widget makes no network requests: the only sockets in `src-tauri/src` are the `127.0.0.1` server and relay. Revisit if the opt-in digest (`claude -p`) ships.
- Submission category Tooling; author, repo link and licence left as placeholders.

## Owed before done

- Owner fills the sponsor account in `SPONSOR_URL` (`src-tauri/src/tray.rs`) and the README Support link.
- Owner check in `npm run tauri dev`: tray menu, About, Sponsor Peep This opens the sponsor page in the browser.
