//! Moods (spec §13.5): at most one per session, derived from the session, its neighbours and its repo. Never changes state.

use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::away::{read_settings, save_setting, settings_path};
use crate::state::{Session, State};
use crate::store::Store;
use crate::tickets::Repo;

pub const SWEAT_FROM_PCT: f64 = 70.0;
/// The context nudge setting (AV-70): 50 to 95 in steps of 5.
pub const NUDGE_PCT_DEFAULT: f64 = 80.0;
const NUDGE_KEY: &str = "nudgePct";
const MOODS_KEY: &str = "moods";
pub const WORRY_MS: u64 = 5 * 60 * 1000;
pub const CHEER_MS: u64 = 8 * 1000;
pub const GREET_MS: u64 = 5 * 1000;
/// A session must predate the newcomer by this much to greet it, so sessions found in one startup scan don't greet each other.
pub const GREET_GAP_MS: u64 = 2 * 1000;
pub const NAP_AFTER_MS: u64 = 10 * 60 * 1000;
/// Two sessions' edits of one file this close together clash (ADR 0002).
pub const CLASH_WINDOW_MS: u64 = 5 * 60 * 1000;
/// Startled lasts this long after the second edit.
pub const CLASH_HOLD_MS: u64 = 30 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Startled,
    Cheer,
    Greet,
    Tweet,
    Worry,
    Yawn,
    Sweat,
    Nap,
}

/// The one mood `s` shows now; the order of the checks is the priority.
/// `limit_pct` is the higher account rate limit; from 80% every session sweats (AV-81).
pub fn derive(s: &Session, sessions: &[&Session], repos: &[Repo], nudge_pct: f64, limit_pct: f64, now: u64) -> Option<Mood> {
    if !s.alive {
        return None;
    }
    let own = || repos.iter().filter(|r| r.sessions.contains(&s.id));
    let cheer = own().filter_map(|r| r.last_closed.as_ref()).any(|c| {
        now.saturating_sub(c.at) < CHEER_MS && s.ticket.as_ref().is_none_or(|t| t.id != c.id)
    });
    let greet = sessions.iter().any(|o| {
        o.id != s.id
            && o.alive
            && o.born_at >= s.born_at + GREET_GAP_MS
            && now.saturating_sub(o.born_at) < GREET_MS
    });
    let worry = s.worried_at.is_some_and(|t| now.saturating_sub(t) < WORRY_MS) || own().any(|r| !r.stale.is_empty());
    let pct = s.context_pct.unwrap_or(0.0);
    let nap = matches!(s.state, State::Idle | State::Success) && now.saturating_sub(s.state_since) >= NAP_AFTER_MS;
    [
        (clash(s, sessions, now).is_some(), Mood::Startled),
        (cheer, Mood::Cheer),
        (greet, Mood::Greet),
        (own().any(|r| tweeting(s, r)), Mood::Tweet),
        (limit_pct >= crate::limit::WARN_PCT, Mood::Sweat),
        (worry, Mood::Worry),
        (pct >= nudge_pct, Mood::Yawn),
        (pct >= SWEAT_FROM_PCT, Mood::Sweat),
        (nap, Mood::Nap),
    ]
    .into_iter()
    .find_map(|(on, m)| on.then_some(m))
}

/// An open tweet `s` posted (its ticket is the `from`, or it edited the board) or got (it edited a file the tweet names).
fn tweeting(s: &Session, repo: &Repo) -> bool {
    let norm = |p: &str| p.replace('\\', "/").to_ascii_lowercase();
    let root = norm(&repo.root);
    let under = |abs: &str, rel: &str| {
        let want = format!("{}/{}", root.trim_end_matches('/'), norm(rel));
        let abs = norm(abs);
        if want.ends_with('/') { abs.starts_with(&want) } else { abs == want }
    };
    repo.tweets.iter().any(|t| {
        s.ticket.as_ref().is_some_and(|k| k.id.eq_ignore_ascii_case(&t.from))
            || s.edits.iter().any(|(p, _)| under(p, ".peep/tweets.md") || t.files.iter().any(|f| under(p, f)))
    })
}

/// Windows paths: one file may arrive with either slash or another letter case.
pub fn same_path(a: &str, b: &str) -> bool {
    let sep = |c: char| c == '/' || c == '\\';
    a.len() == b.len() && a.chars().zip(b.chars()).all(|(x, y)| x.eq_ignore_ascii_case(&y) || (sep(x) && sep(y)))
}

/// The file `s` most recently clashed on, while the hold lasts. Named by the least spelling any session used, so every party groups alike.
pub fn clash(s: &Session, sessions: &[&Session], now: u64) -> Option<String> {
    if !s.alive {
        return None;
    }
    let others = || sessions.iter().filter(|o| o.id != s.id && o.alive);
    let (_, path) = s
        .edits
        .iter()
        .filter_map(|(p, at)| {
            let when = others()
                .flat_map(|o| o.edits.iter())
                .filter(|(q, bt)| same_path(p, q) && at.abs_diff(*bt) <= CLASH_WINDOW_MS)
                .map(|(_, bt)| (*at).max(*bt))
                .max()?;
            (now.saturating_sub(when) < CLASH_HOLD_MS).then_some((when, p))
        })
        .max_by_key(|(when, _)| *when)?;
    let name = others().flat_map(|o| o.edits.iter()).map(|(q, _)| q).filter(|q| same_path(path, q)).chain([path]).min()?;
    Some(name.clone())
}

/// AV-133: with the moods setting off only clash shows; clash outranks every other mood, so gating the winner is enough.
pub fn gate(m: Option<Mood>, moods_on: bool) -> Option<Mood> {
    m.filter(|m| moods_on || *m == Mood::Startled)
}

fn saved_moods(path: &Path) -> bool {
    read_settings(path).get(MOODS_KEY).and_then(Value::as_bool).unwrap_or(false)
}

pub fn load_moods(store: &Store, now: u64) {
    store.set_moods_on(saved_moods(&settings_path()), now);
}

#[tauri::command]
pub fn moods_on(store: tauri::State<'_, Arc<Store>>) -> bool {
    store.moods_on()
}

#[tauri::command]
pub fn set_moods_on(on: bool, store: tauri::State<'_, Arc<Store>>) -> Result<bool, String> {
    save_setting(&settings_path(), MOODS_KEY, Value::from(on)).map_err(|e| e.to_string())?;
    store.set_moods_on(on, crate::store::now_ms());
    Ok(on)
}

pub fn valid_nudge(pct: u64) -> bool {
    (50..=95).contains(&pct) && pct % 5 == 0
}

fn saved_nudge(path: &Path) -> f64 {
    read_settings(path)
        .get(NUDGE_KEY)
        .and_then(Value::as_u64)
        .filter(|p| valid_nudge(*p))
        .map_or(NUDGE_PCT_DEFAULT, |p| p as f64)
}

pub fn load_nudge(store: &Store, now: u64) {
    store.set_nudge_pct(saved_nudge(&settings_path()), now);
}

#[tauri::command]
pub fn nudge_pct(store: tauri::State<'_, Arc<Store>>) -> u64 {
    store.nudge_pct() as u64
}

#[tauri::command]
pub fn set_nudge_pct(pct: u64, store: tauri::State<'_, Arc<Store>>) -> Result<u64, String> {
    if !valid_nudge(pct) {
        return Err(format!("{pct}% is not a nudge choice (50 to 95 in steps of 5)"));
    }
    save_setting(&settings_path(), NUDGE_KEY, Value::from(pct)).map_err(|e| e.to_string())?;
    store.set_nudge_pct(pct as f64, crate::store::now_ms());
    Ok(pct)
}

/// Whether `command` runs any step of the repo setup's `test` line (`cd` steps don't count).
pub fn is_test_run(command: &str, setup_test: &str) -> bool {
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    let command = norm(command);
    setup_test
        .split(['&', '|', ';'])
        .map(norm)
        .filter(|step| !step.is_empty() && step != "cd" && !step.starts_with("cd "))
        .any(|step| command.contains(&step))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{apply, Event, TicketLink};
    use crate::tickets::Closed;

    const T0: u64 = 1_000_000_000;
    const MIN: u64 = 60_000;

    fn session(id: &str, born: u64) -> Session {
        Session::new(id, 1, id, "F:/p", "interactive", "cli", born)
    }

    fn mood(s: &Session, now: u64) -> Option<Mood> {
        derive(s, &[s], &[], NUDGE_PCT_DEFAULT, 0.0, now)
    }

    fn with_pct(pct: f64) -> Session {
        apply(session("a", T0), Event::Statusline { model: None, context_pct: Some(pct), tokens: None, cache_ttl_ms: None }, T0)
    }

    fn test_run(s: Session, passed: bool, now: u64) -> Session {
        apply(s, Event::TestRun { passed, event: Box::new(Event::PostToolUse) }, now)
    }

    #[test]
    fn tired_sweats_from_70_then_yawns_at_the_nudge() {
        assert_eq!(mood(&with_pct(69.9), T0), None);
        assert_eq!(mood(&with_pct(70.0), T0), Some(Mood::Sweat));
        assert_eq!(mood(&with_pct(79.0), T0), Some(Mood::Sweat));
        assert_eq!(mood(&with_pct(80.0), T0), Some(Mood::Yawn));
        assert_eq!(mood(&with_pct(96.0), T0), Some(Mood::Yawn));
        assert_eq!(derive(&with_pct(55.0), &[], &[], 50.0, 0.0, T0), Some(Mood::Yawn), "a nudge under 70 yawns without sweat");
    }

    #[test]
    fn worry_after_a_failed_test_run_until_it_passes_or_expires() {
        let s = test_run(session("a", T0), false, T0 + 1);
        assert_eq!(s.state, State::Thinking, "the wrapped event still applies");
        assert_eq!(mood(&s, T0 + 2), Some(Mood::Worry));
        assert_eq!(mood(&s, T0 + 1 + WORRY_MS), None);
        assert_eq!(mood(&test_run(s, true, T0 + 3), T0 + 4), None);
    }

    #[test]
    fn cheer_for_the_other_sessions_in_the_repo_when_a_ticket_closes() {
        let a = session("a", T0);
        let mut closer = session("b", T0);
        closer.ticket = Some(TicketLink { id: "AV-7".into(), title: None });
        let elsewhere = session("c", T0);
        let repo = Repo {
            root: "F:/p".into(),
            sessions: vec!["a".into(), "b".into()],
            tickets: vec![],
            last_closed: Some(Closed { id: "AV-7".into(), at: T0 + 10 }),
            ..Default::default()
        };
        let repos = [repo];
        let all = [&a, &closer, &elsewhere];
        assert_eq!(derive(&a, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 11), Some(Mood::Cheer));
        assert_eq!(derive(&closer, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 11), None, "the closer doesn't cheer itself");
        assert_eq!(derive(&elsewhere, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 11), None);
        assert_eq!(derive(&a, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 10 + CHEER_MS), None);
    }

    #[test]
    fn a_stale_doing_ticket_worries_every_session_in_its_repo() {
        let a = session("a", T0);
        let elsewhere = session("c", T0);
        let repo = Repo { root: "F:/p".into(), sessions: vec!["a".into()], tickets: vec![], stale: vec!["AV-7".into()], ..Default::default() };
        let repos = [repo];
        let all = [&a, &elsewhere];
        assert_eq!(derive(&a, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + NAP_AFTER_MS), Some(Mood::Worry), "worry beats nap");
        assert_eq!(derive(&elsewhere, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0), None);
    }

    #[test]
    fn tweet_while_an_open_tweet_names_the_session_and_clash_wins() {
        use crate::tickets::Tweet;
        let mut poster = session("a", T0);
        poster.ticket = Some(TicketLink { id: "AV-7".into(), title: None });
        let mut addressee = session("b", T0);
        addressee.edits.push((r"F:\P\src\Rows.ts".into(), T0));
        let mut replier = session("c", T0);
        replier.edits.push(("F:/p/.peep/tweets.md".into(), T0));
        let bystander = session("d", T0);
        let tweets = vec![Tweet { from: "AV-7".into(), files: vec!["src/".into()] }];
        let repo = Repo { root: "F:/p".into(), sessions: vec!["a".into(), "b".into(), "c".into(), "d".into()], tweets, ..Default::default() };
        let repos = [repo];
        let all = [&poster, &addressee, &replier, &bystander];
        let at = |s: &Session| derive(s, &all, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 1);
        assert_eq!(at(&poster), Some(Mood::Tweet));
        assert_eq!(at(&addressee), Some(Mood::Tweet), "a file the tweet names, any slash or case");
        assert_eq!(at(&replier), Some(Mood::Tweet));
        assert_eq!(at(&bystander), None);
        assert_eq!(derive(&poster, &all, &[], NUDGE_PCT_DEFAULT, 0.0, T0 + 1), None, "answered tweets leave the board's open list");
        let mut clasher = session("e", T0);
        clasher.edits.push((r"F:\P\src\Rows.ts".into(), T0));
        let both = [&addressee, &clasher];
        assert_eq!(derive(&addressee, &both, &repos, NUDGE_PCT_DEFAULT, 0.0, T0 + 1), Some(Mood::Startled), "clash takes the slot");
        assert_eq!(derive(&addressee, &all, &repos, NUDGE_PCT_DEFAULT, 90.0, T0 + 1), Some(Mood::Tweet), "tweet outranks sweat");
    }

    #[test]
    fn open_sessions_greet_a_newcomer_briefly() {
        let old = session("a", T0);
        let new = session("b", T0 + MIN);
        let all = [&old, &new];
        let at = |s: &Session, now| derive(s, &all, &[], NUDGE_PCT_DEFAULT, 0.0, now);
        assert_eq!(at(&old, T0 + MIN + 1), Some(Mood::Greet));
        assert_eq!(at(&new, T0 + MIN + 1), None, "the newcomer doesn't greet itself");
        assert_eq!(at(&old, T0 + MIN + GREET_MS), None);
        let twin = session("c", T0 + 1);
        assert_eq!(derive(&old, &[&old, &twin], &[], NUDGE_PCT_DEFAULT, 0.0, T0 + 2), None, "same startup scan");
    }

    #[test]
    fn nap_when_idle_past_the_threshold_and_cleared_by_a_prompt() {
        let s = session("a", T0);
        assert_eq!(mood(&s, T0 + NAP_AFTER_MS - 1), None);
        assert_eq!(mood(&s, T0 + NAP_AFTER_MS), Some(Mood::Nap));
        let done = apply(s.clone(), Event::Stop, T0 + 1);
        assert_eq!(mood(&done, T0 + 1 + NAP_AFTER_MS), Some(Mood::Nap));
        let woke = apply(s, Event::UserPromptSubmit, T0 + NAP_AFTER_MS + 1);
        assert_eq!(mood(&woke, T0 + NAP_AFTER_MS + 2), None);
    }

    #[test]
    fn priority_is_startled_cheer_greet_worry_yawn_sweat_nap() {
        let tired = test_run(with_pct(90.0), false, T0);
        assert_eq!(mood(&tired, T0 + 1), Some(Mood::Worry), "worry beats yawn");
        let new = session("b", T0 + MIN);
        assert_eq!(derive(&tired, &[&tired, &new], &[], NUDGE_PCT_DEFAULT, 0.0, T0 + MIN), Some(Mood::Greet));
        assert_eq!(mood(&with_pct(75.0), T0 + NAP_AFTER_MS), Some(Mood::Sweat), "sweat beats nap");
        let a = edit(tired, "F:/p/a.ts", T0 + MIN + 1);
        let b = edit(new, "F:/p/a.ts", T0 + MIN + 2);
        assert_eq!(derive(&a, &[&a, &b], &[], NUDGE_PCT_DEFAULT, 0.0, T0 + MIN + 3), Some(Mood::Startled), "startled beats greet");
    }

    #[test]
    fn a_rate_limit_at_80_sweats_over_worry_yawn_and_nap_but_not_brief_moods() {
        let at = |s: &Session, all: &[&Session], limit: f64, now| derive(s, all, &[], NUDGE_PCT_DEFAULT, limit, now);
        let s = session("a", T0);
        assert_eq!(at(&s, &[&s], 79.9, T0), None);
        assert_eq!(at(&s, &[&s], 80.0, T0), Some(Mood::Sweat));
        let tired = test_run(with_pct(90.0), false, T0);
        assert_eq!(at(&tired, &[&tired], 85.0, T0 + 1), Some(Mood::Sweat), "beats worry and yawn");
        assert_eq!(at(&s, &[&s], 85.0, T0 + NAP_AFTER_MS), Some(Mood::Sweat), "beats nap");
        let new = session("b", T0 + MIN);
        assert_eq!(at(&s, &[&s, &new], 85.0, T0 + MIN), Some(Mood::Greet), "greet is brief and goes first");
    }

    fn edit(s: Session, path: &str, now: u64) -> Session {
        apply(s, Event::Edited { path: path.into(), event: Box::new(Event::PostToolUse) }, now)
    }

    fn clashes(all: &[&Session], now: u64) -> Vec<Option<String>> {
        all.iter().map(|s| clash(s, all, now)).collect()
    }

    #[test]
    fn two_sessions_editing_one_file_within_5_min_clash_for_the_hold() {
        let a = edit(session("a", T0), "F:\\p\\src\\shell.css", T0);
        let b = edit(session("b", T0), "f:/p/src/shell.css", T0 + 4 * MIN);
        let both = Some("F:\\p\\src\\shell.css".to_string());
        assert_eq!(clashes(&[&a, &b], T0 + 4 * MIN), vec![both.clone(), both.clone()], "one file, either spelling");
        assert_eq!(derive(&a, &[&a, &b], &[], NUDGE_PCT_DEFAULT, 0.0, T0 + 4 * MIN), Some(Mood::Startled));
        assert_eq!(clashes(&[&a, &b], T0 + 4 * MIN + CLASH_HOLD_MS - 1), vec![both.clone(), both]);
        assert_eq!(clashes(&[&a, &b], T0 + 4 * MIN + CLASH_HOLD_MS), vec![None, None], "the hold expired");
        let elsewhere = edit(session("c", T0), "F:/p/src/other.css", T0 + 4 * MIN);
        assert_eq!(clash(&a, &[&a, &elsewhere], T0 + 4 * MIN), None);
    }

    #[test]
    fn edits_further_apart_than_the_window_do_not_clash() {
        let a = edit(session("a", T0), "F:/p/a.ts", T0);
        let b = edit(session("b", T0), "F:/p/a.ts", T0 + CLASH_WINDOW_MS + 1);
        assert_eq!(clashes(&[&a, &b], T0 + CLASH_WINDOW_MS + 2), vec![None, None]);
    }

    #[test]
    fn one_session_editing_a_file_twice_does_not_clash() {
        let a = edit(edit(session("a", T0), "F:/p/a.ts", T0), "F:/p/a.ts", T0 + MIN);
        assert_eq!(a.edits.len(), 1, "a repeat edit replaces the earlier one");
        let idle = session("b", T0);
        assert_eq!(clashes(&[&a, &idle], T0 + MIN), vec![None, None]);
        assert_eq!(mood(&a, T0 + MIN), None);
    }

    #[test]
    fn three_sessions_on_one_file_are_all_startled_and_ended_ones_drop_out() {
        let a = edit(session("a", T0), "F:/p/a.ts", T0);
        let b = edit(session("b", T0), "F:/p/a.ts", T0 + MIN);
        let c = edit(session("c", T0), "F:/p/a.ts", T0 + 2 * MIN);
        let file = Some("F:/p/a.ts".to_string());
        assert_eq!(clashes(&[&a, &b, &c], T0 + 2 * MIN), vec![file.clone(), file.clone(), file.clone()]);
        let gone = apply(c.clone(), Event::Ended, T0 + 2 * MIN);
        assert_eq!(clashes(&[&a, &b, &gone], T0 + 2 * MIN + CLASH_HOLD_MS - 1), vec![None, None, None], "a and b's own clash ended at T0+1min");
        assert_eq!(clashes(&[&a, &c], T0 + 2 * MIN), vec![file.clone(), file]);
    }

    #[test]
    fn attention_states_keep_their_mood_for_the_frontend_to_hide() {
        let s = apply(with_pct(85.0), Event::Notification { message: "q".into(), idle: false }, T0 + 1);
        assert_eq!(s.state, State::Waiting);
        assert_eq!(mood(&s, T0 + 2), Some(Mood::Yawn));
    }

    #[test]
    fn ended_sessions_have_no_mood() {
        let s = apply(with_pct(90.0), Event::Ended, T0 + 1);
        assert_eq!(mood(&s, T0 + 2), None);
    }

    #[test]
    fn nudge_setting_is_50_to_95_in_fives_and_round_trips() {
        assert!(valid_nudge(50) && valid_nudge(80) && valid_nudge(95));
        assert!(!valid_nudge(45) && !valid_nudge(82) && !valid_nudge(100));
        let dir = std::env::temp_dir().join(format!("peepthis-nudge-{}", std::process::id()));
        let path = dir.join("settings.json");
        assert_eq!(saved_nudge(&path), NUDGE_PCT_DEFAULT);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{"awayAfterMin":5,"nudgePct":83}"#).unwrap();
        assert_eq!(saved_nudge(&path), NUDGE_PCT_DEFAULT, "an odd saved value falls back");
        save_setting(&path, NUDGE_KEY, Value::from(60)).unwrap();
        assert_eq!(saved_nudge(&path), 60.0);
        assert_eq!(read_settings(&path)["awayAfterMin"], Value::from(5));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate_passes_only_clash_when_moods_are_off() {
        let all = [Mood::Startled, Mood::Cheer, Mood::Greet, Mood::Tweet, Mood::Worry, Mood::Yawn, Mood::Sweat, Mood::Nap];
        for m in all {
            assert_eq!(gate(Some(m), true), Some(m));
            assert_eq!(gate(Some(m), false), (m == Mood::Startled).then_some(m), "{m:?}");
        }
        assert_eq!(gate(None, false), None);
    }

    #[test]
    fn saved_moods_off_by_default_and_round_trips() {
        let dir = std::env::temp_dir().join(format!("peepthis-moods-{}", std::process::id()));
        let path = dir.join("settings.json");
        assert!(!saved_moods(&path));
        save_setting(&path, MOODS_KEY, Value::from(true)).unwrap();
        assert!(saved_moods(&path));
        save_setting(&path, MOODS_KEY, Value::from(false)).unwrap();
        assert!(!saved_moods(&path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_matches_any_non_cd_step_of_the_setup_line() {
        let setup = "npm test && cd src-tauri && cargo test";
        assert!(is_test_run("npm test", setup));
        assert!(is_test_run("cd src-tauri &&  cargo test mood", setup));
        assert!(is_test_run("npm  test -- --run", setup));
        assert!(!is_test_run("cd src-tauri", setup));
        assert!(!is_test_run("npm run build", setup));
        assert!(!is_test_run("ls", ""));
    }
}
