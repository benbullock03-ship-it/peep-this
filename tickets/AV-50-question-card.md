---
id: AV-50
title: Question card for AskUserQuestion
status: in-progress
depends: []
attended: true
---

# AV-50 — Question card for AskUserQuestion

**What to build:** When a session calls `AskUserQuestion`, show a read-only question card instead of the approval card. Today the request is classed as an unknown `shell` tool (`risk.rs`) and the card shows Allow / Deny over a raw JSON dump (`approvalCard.ts` args fallback). Decided in the grill on 2026-11-15; the "never answers questions" rule stands.

- [x] Verify first, against a live session: how the widget learns the question was answered once the hook is released (`PostToolUse` for `AskUserQuestion`, or the state leaving `waiting`), and that a released `PermissionRequest` for `AskUserQuestion` shows the terminal prompt as normal. Add the payloads as fixtures
- [x] Backend: `AskUserQuestion` is a question, not a risk tier. Release the hook at once so the terminal shows the question immediately; keep the card pending until the question is answered. No rule suggestion, no trail entry
- [x] Card head: tool name, a `question` chip in accent yellow in place of the tier chip, and `1 of N` when the call carries more than one question
- [x] Card body: the first question's full text, wrapped; its option labels as inert chips (muted border, no hover, default cursor, option description in the tooltip)
- [x] Card actions: one button, `Answer in terminal`, which focuses the session. No Allow, Deny, Deny with reason, reason box or countdown bar
- [x] Row bubble: first line of the first question, clipped to the bubble width, in the filled waiting colour
- [x] Keyboard inbox: question cards queue after real approvals; hints show `Enter` (terminal), `Tab` (next), `Esc` (back) only; `Y` / `N` / `R` do nothing on them
- [x] Toast: the existing waiting toast with the question text as its body, no buttons; click focuses
- [x] Collapsed strip: the same flyout as approvals
- [x] Tests: classifier and broker release path (`cargo test`), card rendering for one and several questions (vitest)

**Done when:** a live `AskUserQuestion` shows the question and its options readably in the widget, the terminal prompt appears with no delay, `Answer in terminal` focuses that session, the card goes when the question is answered, and the owner has confirmed it in `npm run tauri dev`.

## Decisions (grill 2026-11-15)

| Question | Ruling |
|---|---|
| Card role | Read-only question card; widget does not answer |
| Content | Question text plus option labels; descriptions in tooltips |
| Several questions | First question in full, `1 of N` in the head |
| Hold | Release at once; card stays until answered |
| Chips | Inert, dim outline |
| Bubble | Question text, clipped |
| Inbox | Included, `Enter` / `Tab` / `Esc` only |
| Trail | No entry |

Taken as defaults without a ruling, for the owner to overrule: toast body is the question text; collapsed strip uses the approval flyout; questions never suggest rules.

**Not taken:** answering inline, stacking every question, logging questions or answers to the trail.

## Verified 2026-11-15 (Claude Code 2.1.325)

Real interactive `claude` TUI through the AV-20 PTY driver and capture server (`--permission-mode default --setting-sources project --model haiku`), every hook answered 204 on arrival.

- **`PermissionRequest` fires for `AskUserQuestion`**, about 10ms after its `PreToolUse`, with the same `tool_input.questions` (each: `question`, `header`, `options[{label, description}]`, `multiSelect`). No `tool_use_id`, as for other tools.
- **Terminal prompt after an immediate 204: shown as normal.** The question dialog (tabs per question, options, "Type something", "Chat about this") was on screen at the first snapshot.
- **A `permission_prompt` `Notification` ("Claude needs your permission") still fires about 6s later.** The session stays `waiting`; the bubble reads the held question, not this text.
- **Answered:** `PostToolUse` for `AskUserQuestion` follows the submit (its `tool_input` gains `answers`), then `Stop`. `PostToolUse` moves the session out of `waiting`, which is what drops the card.
- **Esc ("User declined to answer questions"): no hook at all.** No `PostToolUse`, `PostToolUseFailure` or `Stop`; the next event was `SessionEnd`. The turn ends on screen.
- Fixtures: `permission_request_question.json`, `post_tool_use_question.json`.

## Decisions taken without a ruling

- **`question` is a fourth `Tier` value** (`risk.rs`, serialised `"question"`), not a separate request type. It reuses the pending list, so cards, child rows, inbox and badge needed no second channel. The broker refuses `decide` for it, skips rules and never calls the recorder.
- **The card goes when the session (or the asking subagent) leaves `waiting`**, session death, widget hidden or quit. It ignores the 60s window and the focused-terminal release: the ruling is "stays until answered".
- **Esc in the terminal leaves the card and the `waiting` row up until that session's next event** (the next prompt clears it). Releasing at once gives up the connection-close signal approvals use to notice a terminal "no". Holding the connection silently instead would catch Esc, but the hook's 75s timeout looks identical, so an unanswered question would drop to `idle` at 75s; that is the worse failure. Overrule if Esc matters more.
- **Hidden widget: no card**, as for approvals; the hook is still released at once.
- **Badge and hotkey count include questions** ("N pending approvals").
- **`Answer in terminal` uses the list's existing click delegation** (`.card-terminal` beside `.ps-peep` in `rows.ts`), so a failed focus flashes the row like a peep click.
- **Collapsed strip:** the flyout mounts no cards today (AV-42 left the seam), for approvals or questions. A question gets the same flyout row, with the question in its bubble; a click focuses the terminal.
- **"Card rendering" tests are on `questionOf`** (the card's view model, one and several questions). The repo has no DOM test environment and none was added.
- **Task text:** the `PermissionRequest` event for a question sets the session task to the question's first line instead of "Needs permission: AskUserQuestion".
- A demo question sits on `research-02` in `?demo`.

## Ruled by the owner 2026-11-15

Four of the decisions above, confirmed as built: Esc leaves the card until the session's next event (no silent hold, no dismiss button); the badge and hotkey count include questions; the flyout shows the question in its bubble only, no card; card tests stay on `questionOf`, no DOM environment.

## Owed before done

- Owner check in `npm run tauri dev` with a live `AskUserQuestion`: the card reads well, the terminal prompt is there with no delay, `Answer in terminal` and inbox `Enter` focus the session, the toast shows the question with no buttons, the card goes on answering.
- Built and proven so far: `risk.rs` (`ask_user_question_is_a_question_not_a_risk_tier`), `approve.rs` (`question_is_released_at_once_and_its_card_stays_until_answered`, `question_shows_no_card_while_the_widget_is_hidden`), `notify.rs` (`notify_question_toast_carries_the_question_and_no_buttons`), `server.rs` (`question_fixtures_map_to_the_question_text_and_a_plain_post_tool_use`), `approval.test.ts` (`questionOf`), `inbox.test.ts`, `say.test.ts`; card seen in `?demo` in light and dark.
