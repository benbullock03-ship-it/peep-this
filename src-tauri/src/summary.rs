//! Opt-in model sentence per digest entry (spec §11.9, AV-33): one headless `claude -p` on Haiku per card.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use serde_json::{json, Value};

use crate::away::{self, Away, Digest};
use crate::store::Store;

pub const MAX_ENTRIES: usize = 12;
const TIMEOUT: Duration = Duration::from_secs(45);

/// Entries are keyed `s1`..`sN` so session ids never reach the model; returns the prompt and the key map.
pub fn prompt(d: &Digest) -> (String, HashMap<String, String>) {
    let mut keys = HashMap::new();
    let facts: Vec<Value> = d
        .entries
        .iter()
        .take(MAX_ENTRIES)
        .enumerate()
        .map(|(i, e)| {
            let key = format!("s{}", i + 1);
            keys.insert(key.clone(), e.session_id.clone());
            let files: Vec<&str> = e.files.iter().map(|f| f.rsplit(['/', '\\']).next().unwrap_or(f)).collect();
            json!({
                "key": key,
                "session": e.name,
                "group": e.group,
                "minutes": e.took_ms / 60_000,
                "files": files,
                "filesChanged": e.file_count,
                "commands": e.commands,
                "approvalsSentToTerminal": e.released,
                "lastLine": e.last_line,
            })
        })
        .collect();
    let text = format!(
        "A developer is back at their desk. For each coding session below, write one plain sentence of at most 20 words \
saying what it did or what it is waiting on. Use only these facts; do not guess at anything else. \
Reply with only a JSON object mapping each key to its sentence.\n\n{}",
        Value::Array(facts)
    );
    (text, keys)
}

/// Sentences by session id from the model's reply; anything not a known key with a string value is dropped.
pub fn parse(out: &str, keys: &HashMap<String, String>) -> HashMap<String, String> {
    let (Some(a), Some(b)) = (out.find('{'), out.rfind('}')) else { return HashMap::new() };
    let Some(Value::Object(m)) = out.get(a..=b).and_then(|s| serde_json::from_str(s).ok()) else { return HashMap::new() };
    m.iter()
        .filter_map(|(k, v)| {
            let id = keys.get(k)?;
            let line = away::clip(v.as_str()?, away::LINE_MAX_CHARS);
            (!line.is_empty()).then(|| (id.clone(), line))
        })
        .collect()
}

fn new_session_id() -> std::io::Result<String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| std::io::Error::other(e.to_string()))?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    Ok(format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32]))
}

/// No tools, no transcript, no user settings (so no user hooks), no MCP servers.
fn run(prompt: &str, cwd: &Path, session_id: &str) -> Result<String, String> {
    let mut cmd = Command::new("claude");
    cmd.args(["-p", "--model", "haiku", "--tools", "", "--no-session-persistence", "--setting-sources", "project"])
        .args(["--strict-mcp-config", "--session-id", session_id])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().map_err(|e| format!("claude did not start: {e}"))?;
    let stdin_ok = child.stdin.take().map(|mut i| i.write_all(prompt.as_bytes()));
    let Some(mut stdout) = child.stdout.take() else { return Err("no stdout".into()) };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = tx.send(stdout.read_to_string(&mut s).map(|_| s));
    });
    let out = match rx.recv_timeout(TIMEOUT) {
        Ok(r) => r.map_err(|e| e.to_string()),
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("no reply in {}s", TIMEOUT.as_secs()));
        }
    };
    let status = child.wait().map_err(|e| e.to_string())?;
    if let Some(Err(e)) = stdin_ok {
        return Err(format!("could not send the prompt: {e}"));
    }
    if !status.success() {
        return Err(format!("claude exited with {status}"));
    }
    out
}

/// In the background; failure or offline leaves the facts card as it is.
pub fn start(away: Arc<Away>, store: Arc<Store>, d: Digest) {
    let spawned = std::thread::Builder::new().name("digest-summary".into()).spawn(move || {
        let Ok(id) = new_session_id() else { return };
        store.ignore(&id);
        let cwd = crate::ingest::server::default_dir().join("summary");
        if let Err(e) = std::fs::create_dir_all(&cwd) {
            eprintln!("peepthis: summary dir failed: {e}");
            return;
        }
        let (text, keys) = prompt(&d);
        match run(&text, &cwd, &id) {
            Ok(out) => {
                let sentences = parse(&out, &keys);
                println!("peepthis: digest summary {} of {} entries", sentences.len(), keys.len());
                away.fill(d.since, d.until, &sentences);
            }
            Err(e) => eprintln!("peepthis: digest summary failed: {e}"),
        }
    });
    if let Err(e) = spawned {
        eprintln!("peepthis: summary thread failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::away::{Activity, Note};
    use crate::state::{Session, State};

    const T0: u64 = 10_000_000;
    const MIN: u64 = 60_000;

    fn card(n: usize) -> Digest {
        let mut a = Activity::default();
        let mut sessions = Vec::new();
        for i in 0..n {
            let id = format!("uuid-{i:02}");
            let at = T0 + i as u64;
            a.note(&id, "", Note::Prompt, at);
            a.note(&id, "", Note::File("src/deep/app.ts".into()), at);
            a.note(&id, "", Note::File("C:\\p\\lib.rs".into()), at);
            a.note(&id, "", Note::Command, at);
            a.note(&id, "", Note::Released, at);
            a.note(&id, "", Note::Stop { line: Some("Tests pass.".into()) }, at + 12 * MIN);
            let mut s = Session::new(&id, 1, &format!("proj-{i}"), "C:/p", "interactive", "cli", T0 - MIN);
            s.state = State::Success;
            sessions.push(s);
        }
        away::digest(&a, &sessions, T0, T0 + 30 * MIN).unwrap()
    }

    #[test]
    fn prompt_keys_entries_and_sends_base_names_and_facts_only() {
        let d = card(15);
        let (text, keys) = prompt(&d);
        assert_eq!(keys.len(), MAX_ENTRIES);
        assert_eq!(keys["s1"], d.entries[0].session_id);
        assert!(!text.contains("uuid-"), "session ids stay local");
        assert!(text.contains(r#""files":["app.ts","lib.rs"]"#));
        assert!(text.contains(r#""minutes":12"#) && text.contains("Tests pass.") && text.contains(r#""group":"finished""#));
        assert!(!text.contains("deep/"));
    }

    #[test]
    fn parse_keeps_known_keys_and_one_clipped_line() {
        let keys = HashMap::from([("s1".to_string(), "id-1".to_string()), ("s2".to_string(), "id-2".to_string())]);
        let long = "y".repeat(300);
        let out = format!("```json\n{{\"s1\": \"Fixed the parser.\\nSecond line\", \"s2\": \"{long}\", \"s9\": \"stray\", \"s3\": 4}}\n```");
        let got = parse(&out, &keys);
        assert_eq!(got.len(), 2);
        assert_eq!(got["id-1"], "Fixed the parser.");
        assert_eq!(got["id-2"].chars().count(), away::LINE_MAX_CHARS);
        assert!(parse("sorry, offline", &keys).is_empty());
        assert!(parse("{not json}", &keys).is_empty());
        assert!(parse(r#"{"s1": "   "}"#, &keys).is_empty());
    }

    #[test]
    fn session_ids_are_v4_uuids() {
        let id = new_session_id().unwrap();
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(parts[2].starts_with('4'));
        assert!(matches!(parts[3].chars().next(), Some('8' | '9' | 'a' | 'b')));
        assert_ne!(id, new_session_id().unwrap());
    }
}
