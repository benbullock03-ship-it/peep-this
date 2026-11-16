# Desktop webviews

For a desktop app whose UI is a web page (Tauri, WebView2, Electron). The page is checked over the Chrome DevTools Protocol (CDP); the Playwright MCP cannot attach to a webview that is already running.

## Launch with a debug port

- WebView2 (Tauri on Windows): set `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>` in the environment of the dev command. Electron: pass `--remote-debugging-port=<port>`.
- `http://127.0.0.1:<port>/json` lists the pages; pick by `url`, not by order. Each has a `webSocketDebuggerUrl`.
- A single-instance app hands off to a copy that is already running and exits at once (exit code 0, no window). Find it (`tasklist`, `Get-Process`), say so, stop it, and restart it in housekeeping if it was the owner's.
- A dev server already on the app's port may be another session's. If it serves this repo, reuse it (skip the app's own before-dev command) rather than kill it.

## DOM facts and actions

- One small script per check: open the page's websocket, send `Runtime.evaluate` with `awaitPromise: true, returnByValue: true`, print the value. Node 22+ has `fetch` and `WebSocket` built in; keep the script in the scratchpad, not the repo.
- The asserts in `dom-asserts.md` run unchanged through it.
- Drive state with the app's own commands from the page (Tauri: `window.__TAURI_INTERNALS__.invoke('<command>', {...})`), never by synthetic clicks on native chrome. Test the refusal path before the real one (an unknown id must error and change nothing).
- Element shots: `Page.captureScreenshot` with a `clip` from `getBoundingClientRect()`, saved as PNG and Read like any other shot.

## Real inputs

- Make the input real where it is cheap: a throwaway process, a scratch git repo, a fixture file. Spawn it from a scratch folder, never the owner's work.
- A child inherits the agent's environment. Clear variables that change its behaviour before launching (for Claude Code, every `CLAUDE*` variable: an inherited `CLAUDE_CODE_CHILD_SESSION` writes no session file).
- Prefer headless modes (`claude -p`) to interactive windows: their output is captured and they show no prompts.

## Native chrome

Window frame, tray, toasts and focus are outside the page.
- Look with an OS screenshot (PowerShell `CopyFromScreen`, macOS `screencapture`) and Read the PNG.
- Never send keystrokes to a window you cannot see: prompts have defaults (a trust dialog's default is "No, exit"). Use a command, a headless mode, or put it under "Owed to the owner".
- The app's own log is evidence: quote the line that shows the state change.

## Housekeeping

Stop every process the run started (app, children, throwaway sessions). Restart any copy of the app the run stopped. Remove scratch repos only if the run created them.
