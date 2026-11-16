# Peep This

An always-on-top desktop companion for Claude Code. It lists the Claude Code sessions running on your machine, each with a peep (a small pixel marshmallow) that shows what the session is doing. It tells you when a session needs you, shows context-window and rate-limit usage, and lets you answer permission prompts from the widget. It ships with the `peep` skill pack, a set of Claude Code skills that teach a working rhythm.

## Where it stands

- **Offline.** The widget makes no network requests. It listens on `127.0.0.1` only, for events from your own Claude Code sessions, and its fonts are bundled.
- **No telemetry.** Nothing is collected, counted or sent anywhere. What it keeps (settings, the decision trail) stays in `%APPDATA%\PeepThis` on your machine.
- **Windows-first.** Built and tested on Windows. macOS and Linux are not supported yet.
- **Never acts without you.** It never types into a session, answers questions, sends replies or starts sessions. Permission decisions come only from a click in the widget.

## What the integration install changes

Nothing is changed until you choose **Install integration** from the tray menu. Then it:

1. Copies `~/.claude/settings.json` to `~/.claude/settings.json.peepthis-backup-<timestamp>` before touching it. An earlier backup is never overwritten.
2. Adds hook entries for Claude Code events (prompt submitted, tool use, permission requests, notifications, stop, subagents, session end). Your own hooks are kept. Each hook runs a small relay, `peepthis-statusline.exe`, that forwards the event to the widget on `127.0.0.1` and always exits quietly, even when the widget is closed.
3. Wraps your existing `statusLine` command. Your original command is recorded in `%APPDATA%\PeepThis\statusline-original.json`. The wrapper runs it and prints its output unchanged, then passes the context and rate-limit figures to the widget.
4. Adds the `peep` plugin through Claude Code's own `claude plugin` commands.

`settings.json` is written through a temp file and only replaced once the result parses as JSON. The hooks and statusline run under Git Bash, as Claude Code's do on Windows.

## How to undo it

- Choose **Uninstall integration** from the tray menu. It removes only Peep This's hook entries, restores your original `statusLine` command and removes the `peep` plugin. Your other hooks, skills and plugins are left alone.
- Uninstalling the app from Windows settings does the same, then removes the relay copy.
- If anything looks wrong, every backup is still next to `settings.json` as `settings.json.peepthis-backup-<timestamp>`. Copy one back over `settings.json`.

Turn off **Start at login** in the tray menu to stop it launching with Windows.

## Support

Peep This is free. If it helps you, you can [sponsor it](https://github.com/sponsors/OWNER-FILL-ME). The same link sits under **About** in the tray menu. There is no nag and nothing in the panel.

## Building from source

```
npm install
npm run tauri dev     # run the widget with hot reload
npm run tauri build   # release build
```

The first Rust build takes about five minutes.
