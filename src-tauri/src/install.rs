//! Integration install and uninstall (spec §7): hook entries and the statusline wrapper in `~/.claude/settings.json`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::ingest::hooks;
use crate::ingest::server::{self, Endpoint, DEFAULT_PORT, TOKEN_HEADER};
use crate::statusline_original;
use crate::store::{Integration, Store};

pub const WRAPPER_EXE: &str = "peepthis-statusline.exe";
const WRAPPER_MARK: &str = "peepthis-statusline";
const BACKUP_INFIX: &str = ".peepthis-backup-";
const TMP_SUFFIX: &str = ".peepthis-tmp";
const BOM: &[u8] = b"\xEF\xBB\xBF";

#[derive(Debug, Clone)]
pub struct Paths {
    pub settings: PathBuf,
    pub data_dir: PathBuf,
    /// The freshly built or bundled wrapper, next to the main exe.
    pub wrapper_src: PathBuf,
}

impl Paths {
    pub fn for_this_machine() -> Paths {
        let wrapper_src = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join(WRAPPER_EXE)))
            .unwrap_or_else(|| PathBuf::from(WRAPPER_EXE));
        Paths { settings: claude_settings_path(), data_dir: server::default_dir(), wrapper_src }
    }

    /// Copied here so dev rebuilds never lock it and the path survives the app moving.
    pub fn wrapper_dest(&self) -> PathBuf {
        self.data_dir.join("bin").join(WRAPPER_EXE)
    }
}

/// Claude Code honours `CLAUDE_CONFIG_DIR` in place of `~/.claude`.
fn claude_settings_path() -> PathBuf {
    let dir = match std::env::var_os("CLAUDE_CONFIG_DIR").filter(|d| !d.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from(std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).unwrap_or_default())
            .join(".claude"),
    };
    dir.join("settings.json")
}

/// Forward slashes because Claude Code runs the statusline through Git Bash.
pub fn status_line_command(exe: &Path) -> String {
    let s = exe.to_string_lossy().replace('\\', "/");
    if s.contains(' ') { format!("\"{s}\"") } else { s }
}

fn is_our_hook(h: &Value) -> bool {
    h.get("type").and_then(Value::as_str) == Some("http")
        && h.get("url")
            .and_then(Value::as_str)
            .is_some_and(|u| u.starts_with("http://127.0.0.1:") && u.contains("/hook/"))
}

fn is_our_status_line(v: &Value) -> bool {
    v.get("command").and_then(Value::as_str).is_some_and(|c| c.contains(WRAPPER_MARK))
}

/// Removes our hook entries, then any group, event array or `hooks` object our removal left empty.
fn strip_hooks(root: &mut Map<String, Value>, prune_root: bool) -> bool {
    let Some(Value::Object(events)) = root.get_mut("hooks") else { return false };
    let mut any = false;
    events.retain(|_, groups| {
        let Value::Array(list) = groups else { return true };
        let mut touched = false;
        list.retain_mut(|group| {
            let Some(Value::Array(hs)) = group.get_mut("hooks") else { return true };
            let before = hs.len();
            hs.retain(|h| !is_our_hook(h));
            if hs.len() == before {
                return true;
            }
            touched = true;
            !hs.is_empty()
        });
        any |= touched;
        !(touched && list.is_empty())
    });
    if prune_root && any && events.is_empty() {
        root.shift_remove("hooks");
    }
    any
}

/// Applies an install to `root`; returns the `statusLine` value to record as the original, or `None` to keep the record.
fn apply_install(root: &mut Map<String, Value>, ep: &Endpoint, command: &str) -> Result<Option<Value>, String> {
    strip_hooks(root, false);
    let events = root.entry("hooks").or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(events) = events else { return Err("settings.json: `hooks` is not an object".into()) };
    let ours = hooks::settings_hooks(ep.port, &ep.token);
    for (event, groups) in ours.as_object().into_iter().flatten() {
        let slot = events.entry(event.clone()).or_insert_with(|| Value::Array(Vec::new()));
        let Value::Array(list) = slot else { return Err(format!("settings.json: `hooks.{event}` is not an array")) };
        list.extend(groups.as_array().into_iter().flatten().cloned());
    }

    let current = root.get("statusLine").cloned();
    // Re-recording a statusLine that is already ours would make the wrapper its own original.
    let record = match &current {
        Some(v) if is_our_status_line(v) => None,
        Some(v) => Some(v.clone()),
        None => Some(Value::Null),
    };
    let mut sl = match current {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    };
    sl.insert("type".into(), "command".into());
    sl.insert("command".into(), command.into());
    root.insert("statusLine".into(), Value::Object(sl));
    Ok(record)
}

/// `record` is the parsed `statusline-original.json` (`None` when there is none). Returns whether `root` changed.
fn apply_uninstall(root: &mut Map<String, Value>, record: Option<&Value>) -> bool {
    let mut changed = strip_hooks(root, true);
    match root.get("statusLine") {
        Some(sl) if is_our_status_line(sl) => {
            match record.and_then(|r| r.get("statusLine")).filter(|v| !v.is_null()) {
                Some(original) => {
                    root.insert("statusLine".into(), original.clone());
                }
                None => {
                    root.shift_remove("statusLine");
                }
            }
            changed = true;
        }
        Some(_) => println!("peepthis: statusLine was changed since install; leaving it"),
        None => {}
    }
    changed
}

/// `Ok(None)` when the file does not exist. Refuses anything that is not a JSON object.
fn read_settings(path: &Path) -> Result<Option<(Vec<u8>, Map<String, Value>)>, String> {
    let raw = match fs::read(path) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };
    let text = raw.strip_prefix(BOM).unwrap_or(&raw);
    match serde_json::from_slice::<Value>(text) {
        Ok(Value::Object(m)) => Ok(Some((raw, m))),
        Ok(_) => Err(format!("{} is not a JSON object; left unchanged", path.display())),
        Err(e) => Err(format!("{} is not valid JSON ({e}); left unchanged", path.display())),
    }
}

fn to_pretty(root: Map<String, Value>) -> Result<String, String> {
    let mut text = serde_json::to_string_pretty(&Value::Object(root)).map_err(|e| e.to_string())?;
    text.push('\n');
    serde_json::from_str::<Value>(&text).map_err(|e| format!("refusing to write settings: output does not parse ({e})"))?;
    Ok(text)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = with_suffix(path, TMP_SUFFIX);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// `YYYYMMDD-HHMMSS`, UTC.
fn utc_stamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Never overwrites an earlier backup, which may be the only copy of the pre-install file.
fn backup(settings: &Path, raw: &[u8], now: SystemTime) -> io::Result<PathBuf> {
    let base = with_suffix(settings, &format!("{BACKUP_INFIX}{}", utc_stamp(now)));
    let mut path = base.clone();
    let mut n = 2;
    while path.exists() {
        path = with_suffix(&base, &format!("-{n}"));
        n += 1;
    }
    fs::write(&path, raw)?;
    Ok(path)
}

fn install_wrapper(src: &Path, dest: &Path) -> io::Result<()> {
    let bytes = fs::read(src)?;
    if fs::read(dest).is_ok_and(|d| d == bytes) {
        return Ok(());
    }
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = with_suffix(dest, ".new");
    fs::write(&tmp, &bytes)?;
    if fs::rename(&tmp, dest).is_err() {
        // A running exe cannot be overwritten, but it can be renamed out of the way.
        let old = with_suffix(dest, ".old");
        let _ = fs::remove_file(&old);
        fs::rename(dest, &old)?;
        fs::rename(&tmp, dest)?;
        let _ = fs::remove_file(&old);
    }
    Ok(())
}

pub fn install(paths: &Paths, ep: &Endpoint) -> Result<(), String> {
    let existing = read_settings(&paths.settings)?;
    if !paths.wrapper_src.is_file() {
        return Err(format!(
            "statusline wrapper not found at {} (dev: run `cargo build --bin peepthis-statusline` in src-tauri)",
            paths.wrapper_src.display()
        ));
    }
    let (raw, mut root) = match existing {
        Some((raw, root)) => (Some(raw), root),
        None => (None, Map::new()),
    };
    let record = apply_install(&mut root, ep, &status_line_command(&paths.wrapper_dest()))?;
    let text = to_pretty(root)?;

    if let Some(raw) = &raw {
        let b = backup(&paths.settings, raw, SystemTime::now()).map_err(|e| format!("backup failed, nothing changed: {e}"))?;
        println!("peepthis: backed up settings to {}", b.display());
    }
    install_wrapper(&paths.wrapper_src, &paths.wrapper_dest())
        .map_err(|e| format!("could not copy the statusline wrapper: {e}"))?;
    if let Some(record) = record {
        write_atomic(&statusline_original::path(&paths.data_dir), statusline_original::to_json(&record).as_bytes())
            .map_err(|e| format!("could not record the original statusLine: {e}"))?;
    }
    write_atomic(&paths.settings, text.as_bytes()).map_err(|e| format!("could not write settings: {e}"))?;
    println!("peepthis: integration installed");
    Ok(())
}

pub fn uninstall(paths: &Paths) -> Result<(), String> {
    let record_path = statusline_original::path(&paths.data_dir);
    let Some((raw, mut root)) = read_settings(&paths.settings)? else { return Ok(()) };
    let record = fs::read(&record_path).ok().and_then(|r| {
        serde_json::from_slice::<Value>(r.strip_prefix(BOM).unwrap_or(&r)).ok()
    });
    if apply_uninstall(&mut root, record.as_ref()) {
        let text = to_pretty(root)?;
        let b = backup(&paths.settings, &raw, SystemTime::now()).map_err(|e| format!("backup failed, nothing changed: {e}"))?;
        println!("peepthis: backed up settings to {}", b.display());
        write_atomic(&paths.settings, text.as_bytes()).map_err(|e| format!("could not write settings: {e}"))?;
    }
    let _ = fs::remove_file(&record_path);
    println!("peepthis: integration uninstalled");
    Ok(())
}

/// Run by the NSIS uninstaller: unhooks Claude Code, then drops the wrapper copy nothing points at any more.
pub fn remove_for_app_uninstall(paths: &Paths) -> Result<(), String> {
    uninstall(paths)?;
    let dest = paths.wrapper_dest();
    for p in [with_suffix(&dest, ".old"), with_suffix(&dest, ".new"), dest.clone()] {
        let _ = fs::remove_file(p);
    }
    if let Some(dir) = dest.parent() {
        let _ = fs::remove_dir(dir); // only if empty
    }
    Ok(())
}

pub fn integration_status(paths: &Paths, ep: &Endpoint) -> Integration {
    let Ok(Some((_, root))) = read_settings(&paths.settings) else { return Integration::Missing };
    let ours: Vec<(&str, &Value)> = root
        .get("hooks")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .flat_map(|(event, groups)| {
            groups
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|g| g.get("hooks").and_then(Value::as_array))
                .flatten()
                .filter(|h| is_our_hook(h))
                .map(move |h| (event.as_str(), h))
        })
        .collect();
    let status_line = root.get("statusLine").filter(|v| is_our_status_line(v));
    if ours.is_empty() && status_line.is_none() {
        return Integration::Missing;
    }
    let hooks_current = ours.len() == hooks::EVENTS.len()
        && hooks::EVENTS.iter().all(|event| {
            ours.iter().any(|(e, h)| {
                e == event
                    && h.get("url").and_then(Value::as_str) == Some(hooks::hook_url(ep.port, event).as_str())
                    && h.pointer(&format!("/headers/{TOKEN_HEADER}")).and_then(Value::as_str) == Some(ep.token.as_str())
            })
        });
    let dest = paths.wrapper_dest();
    let command_current = status_line
        .and_then(|sl| sl.get("command"))
        .and_then(Value::as_str)
        .is_some_and(|c| c == status_line_command(&dest));
    let exe_current = matches!((fs::read(&paths.wrapper_src), fs::read(&dest)), (Ok(a), Ok(b)) if a == b);
    if hooks_current && command_current && exe_current { Integration::Installed } else { Integration::Outdated }
}

/// Managed Tauri state for the install commands.
pub struct Integrator {
    paths: Paths,
    endpoint: Option<Endpoint>,
    busy: Mutex<()>,
}

impl Integrator {
    pub fn new(paths: Paths, endpoint: Option<Endpoint>) -> Integrator {
        Integrator { paths, endpoint, busy: Mutex::new(()) }
    }

    pub fn refresh(&self, store: &Store) {
        if let Some(ep) = &self.endpoint {
            store.set_integration(integration_status(&self.paths, ep));
        }
    }

    pub fn install(&self, store: &Store) -> Result<(), String> {
        let _guard = self.busy.lock().unwrap_or_else(|e| e.into_inner());
        let ep = self.endpoint.as_ref().ok_or("the event server is not running")?;
        // Hooks hold the port literally; one installed against a fallback port breaks on the next restart.
        if ep.port != DEFAULT_PORT {
            return Err(format!("port {DEFAULT_PORT} is taken by another program; free it and restart Peep This"));
        }
        let result = install(&self.paths, ep);
        self.refresh(store);
        result.inspect_err(|e| eprintln!("peepthis: install failed: {e}"))
    }

    pub fn uninstall(&self, store: &Store) -> Result<(), String> {
        let _guard = self.busy.lock().unwrap_or_else(|e| e.into_inner());
        let result = uninstall(&self.paths);
        self.refresh(store);
        result.inspect_err(|e| eprintln!("peepthis: uninstall failed: {e}"))
    }
}

#[tauri::command]
pub fn install_integration(it: tauri::State<'_, Integrator>, store: tauri::State<'_, Arc<Store>>) -> Result<(), String> {
    it.install(&store)
}

#[tauri::command]
pub fn uninstall_integration(it: tauri::State<'_, Integrator>, store: tauri::State<'_, Arc<Store>>) -> Result<(), String> {
    it.uninstall(&store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const ROUND_TRIP: [&str; 4] = ["empty.json", "statusline_only.json", "user_hooks.json", "hooks_and_padded_statusline.json"];

    fn ep() -> Endpoint {
        Endpoint { port: DEFAULT_PORT, token: TOKEN.into() }
    }

    /// `\r` stripped: autocrlf checkouts would otherwise break the byte-exact round trips.
    fn fixture(name: &str) -> Vec<u8> {
        let raw = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings").join(name)).unwrap();
        raw.into_iter().filter(|&b| b != b'\r').collect()
    }

    struct Env {
        root: PathBuf,
        paths: Paths,
    }

    impl Drop for Env {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    impl Env {
        fn new(settings: Option<&str>) -> Env {
            static N: AtomicU32 = AtomicU32::new(0);
            let root = std::env::temp_dir()
                .join(format!("peepthis-install-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
            let _ = fs::remove_dir_all(&root);
            let paths = Paths {
                settings: root.join("claude").join("settings.json"),
                data_dir: root.join("appdata"),
                wrapper_src: root.join("build").join(WRAPPER_EXE),
            };
            fs::create_dir_all(root.join("claude")).unwrap();
            fs::create_dir_all(root.join("build")).unwrap();
            fs::write(&paths.wrapper_src, b"wrapper v1").unwrap();
            if let Some(name) = settings {
                fs::write(&paths.settings, fixture(name)).unwrap();
            }
            Env { root, paths }
        }

        fn settings(&self) -> Vec<u8> {
            fs::read(&self.paths.settings).unwrap()
        }

        fn json(&self) -> Value {
            serde_json::from_slice(&self.settings()).unwrap()
        }

        fn record(&self) -> Option<Value> {
            fs::read(statusline_original::path(&self.paths.data_dir)).ok().map(|r| serde_json::from_slice(&r).unwrap())
        }

        fn backups(&self) -> Vec<PathBuf> {
            let mut v: Vec<PathBuf> = fs::read_dir(self.root.join("claude"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.to_string_lossy().contains(BACKUP_INFIX))
                .collect();
            v.sort();
            v
        }

        fn status(&self) -> Integration {
            integration_status(&self.paths, &ep())
        }
    }

    #[test]
    fn fixtures_are_in_written_format() {
        for name in ROUND_TRIP.iter().chain(&["stale_install.json"]) {
            let raw = fixture(name);
            let Value::Object(m) = serde_json::from_slice(&raw).unwrap() else { panic!() };
            assert_eq!(to_pretty(m).unwrap().as_bytes(), raw.as_slice(), "{name}");
        }
    }

    #[test]
    fn install_then_uninstall_restores_each_fixture() {
        for name in ROUND_TRIP {
            let env = Env::new(Some(name));
            let original = env.settings();
            assert_eq!(env.status(), Integration::Missing, "{name}");
            install(&env.paths, &ep()).unwrap();
            assert_eq!(env.status(), Integration::Installed, "{name}");
            let backups = env.backups();
            assert_eq!(backups.len(), 1, "{name}");
            assert_eq!(fs::read(&backups[0]).unwrap(), original, "{name}");

            uninstall(&env.paths).unwrap();
            assert_eq!(env.json(), serde_json::from_slice::<Value>(&original).unwrap(), "{name}");
            assert_eq!(env.settings(), original, "{name}: key order and format survive");
            assert_eq!(env.status(), Integration::Missing, "{name}");
            assert!(env.record().is_none(), "{name}");
            assert_eq!(env.backups().len(), 2, "{name}: uninstall backs up too");
            assert!(!with_suffix(&env.paths.settings, TMP_SUFFIX).exists());
        }
    }

    #[test]
    fn app_uninstall_restores_settings_and_removes_the_wrapper_copy() {
        let env = Env::new(Some(ROUND_TRIP[0]));
        let original = env.settings();
        install(&env.paths, &ep()).unwrap();
        assert!(env.paths.wrapper_dest().exists());

        remove_for_app_uninstall(&env.paths).unwrap();
        assert_eq!(env.settings(), original);
        assert!(!env.paths.wrapper_dest().exists());
        assert!(!env.paths.wrapper_dest().parent().unwrap().exists());
        assert!(env.paths.data_dir.exists(), "app settings outside bin/ stay");
    }

    #[test]
    fn reinstall_is_idempotent_and_keeps_the_real_original() {
        for name in ROUND_TRIP {
            let env = Env::new(Some(name));
            install(&env.paths, &ep()).unwrap();
            let (once, record) = (env.settings(), env.record());
            install(&env.paths, &ep()).unwrap();
            assert_eq!(env.settings(), once, "{name}");
            assert_eq!(env.record(), record, "{name}");
            assert!(!is_our_status_line(&record.unwrap()["statusLine"]), "{name}");
        }
    }

    #[test]
    fn install_keeps_user_hooks_and_statusline_extras() {
        let env = Env::new(Some("hooks_and_padded_statusline.json"));
        let before = env.json();
        install(&env.paths, &ep()).unwrap();
        let after = env.json();
        let pre = after["hooks"]["PreToolUse"].as_array().unwrap();
        let orig_pre = before["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(&pre[..orig_pre.len()], orig_pre.as_slice());
        assert_eq!(pre.len(), orig_pre.len() + 1);
        assert_eq!(after["hooks"]["SessionStart"], before["hooks"]["SessionStart"]);
        assert_eq!(after["statusLine"]["padding"], 2);
        assert_eq!(after["statusLine"]["command"], status_line_command(&env.paths.wrapper_dest()));
        assert_eq!(env.record().unwrap()["statusLine"], before["statusLine"]);
        assert_eq!(fs::read(env.paths.wrapper_dest()).unwrap(), b"wrapper v1");
        for event in hooks::EVENTS {
            let last = after["hooks"][event].as_array().unwrap().last().unwrap();
            assert_eq!(last["hooks"][0]["url"], hooks::hook_url(DEFAULT_PORT, event));
        }
    }

    #[test]
    fn missing_settings_file_is_created_and_uninstalls_to_empty() {
        let env = Env::new(None);
        install(&env.paths, &ep()).unwrap();
        assert!(env.backups().is_empty());
        assert_eq!(env.record(), Some(serde_json::json!({ "statusLine": null })));
        assert_eq!(env.status(), Integration::Installed);
        uninstall(&env.paths).unwrap();
        assert_eq!(env.json(), serde_json::json!({}));
    }

    #[test]
    fn install_over_stale_entries_replaces_them() {
        let env = Env::new(Some("stale_install.json"));
        let original = statusline_original::to_json(&serde_json::json!({ "type": "command", "command": "sh ~/.claude/old.sh" }));
        fs::create_dir_all(&env.paths.data_dir).unwrap();
        fs::write(statusline_original::path(&env.paths.data_dir), &original).unwrap();
        assert_eq!(env.status(), Integration::Outdated);

        install(&env.paths, &ep()).unwrap();
        assert_eq!(env.status(), Integration::Installed);
        let text = String::from_utf8(env.settings()).unwrap();
        assert!(!text.contains(":47000/") && !text.contains("stale-token"), "{text}");
        assert!(!text.contains("SessionStart/"), "entries for events no longer installed are dropped");
        assert_eq!(fs::read(statusline_original::path(&env.paths.data_dir)).unwrap(), original.as_bytes());

        uninstall(&env.paths).unwrap();
        let after = env.json();
        assert_eq!(after["statusLine"]["command"], "sh ~/.claude/old.sh");
        assert_eq!(after["hooks"]["Stop"][0]["hooks"][0]["command"], "echo done");
        assert!(after["hooks"].get("PreToolUse").is_none());
    }

    #[test]
    fn uninstall_keeps_a_statusline_the_user_changed_since() {
        let env = Env::new(Some("statusline_only.json"));
        install(&env.paths, &ep()).unwrap();
        let Value::Object(mut root) = env.json() else { panic!() };
        root.insert("statusLine".into(), serde_json::json!({ "type": "command", "command": "mine.sh" }));
        fs::write(&env.paths.settings, to_pretty(root).unwrap()).unwrap();

        uninstall(&env.paths).unwrap();
        let after = env.json();
        assert_eq!(after["statusLine"]["command"], "mine.sh");
        assert!(after.get("hooks").is_none());
        assert!(env.record().is_none());
    }

    #[test]
    fn invalid_settings_are_refused_untouched() {
        let env = Env::new(Some("invalid.json"));
        let before = env.settings();
        let err = install(&env.paths, &ep()).unwrap_err();
        assert!(err.contains("not valid JSON"), "{err}");
        assert!(uninstall(&env.paths).is_err());
        assert_eq!(env.settings(), before);
        assert!(env.backups().is_empty());
        assert!(!env.paths.data_dir.exists());
    }

    #[test]
    fn missing_wrapper_aborts_before_any_write() {
        let env = Env::new(Some("statusline_only.json"));
        let before = env.settings();
        fs::remove_file(&env.paths.wrapper_src).unwrap();
        assert!(install(&env.paths, &ep()).unwrap_err().contains("wrapper not found"));
        assert_eq!(env.settings(), before);
        assert!(env.backups().is_empty());
        assert!(!env.paths.data_dir.exists());
    }

    #[test]
    fn status_outdated_cases() {
        let env = Env::new(Some("user_hooks.json"));
        install(&env.paths, &ep()).unwrap();
        assert_eq!(env.status(), Integration::Installed);

        let other_token = Endpoint { port: DEFAULT_PORT, token: "f".repeat(64) };
        assert_eq!(integration_status(&env.paths, &other_token), Integration::Outdated);
        let other_port = Endpoint { port: 50000, token: TOKEN.into() };
        assert_eq!(integration_status(&env.paths, &other_port), Integration::Outdated);

        fs::write(&env.paths.wrapper_src, b"wrapper v2").unwrap();
        assert_eq!(env.status(), Integration::Outdated, "a rebuilt wrapper needs copying again");
        install(&env.paths, &ep()).unwrap();
        assert_eq!(env.status(), Integration::Installed);

        let moved = Paths { data_dir: env.root.join("elsewhere"), ..env.paths.clone() };
        assert_eq!(integration_status(&moved, &ep()), Integration::Outdated);
    }

    #[test]
    fn status_missing_without_settings_or_with_only_user_entries() {
        assert_eq!(Env::new(None).status(), Integration::Missing);
        assert_eq!(Env::new(Some("invalid.json")).status(), Integration::Missing);
        assert_eq!(Env::new(Some("hooks_and_padded_statusline.json")).status(), Integration::Missing);
    }

    #[test]
    fn backups_never_overwrite_each_other() {
        let env = Env::new(Some("empty.json"));
        let t = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let a = backup(&env.paths.settings, b"first", t).unwrap();
        let b = backup(&env.paths.settings, b"second", t).unwrap();
        assert!(a.to_string_lossy().ends_with("settings.json.peepthis-backup-20231114-221320"));
        assert_ne!(a, b);
        assert_eq!(fs::read(a).unwrap(), b"first");
        assert_eq!(fs::read(b).unwrap(), b"second");
    }

    #[test]
    fn utc_stamps() {
        assert_eq!(utc_stamp(UNIX_EPOCH), "19700101-000000");
        assert_eq!(utc_stamp(UNIX_EPOCH + Duration::from_secs(951_782_400)), "20000229-000000");
        assert_eq!(utc_stamp(UNIX_EPOCH + Duration::from_secs(1_735_689_599)), "20241231-235959");
    }

    #[test]
    fn statusline_command_uses_forward_slashes_and_quotes_spaces() {
        assert_eq!(status_line_command(Path::new(r"C:\Users\a\AppData\Roaming\PeepThis\bin\x.exe")), "C:/Users/a/AppData/Roaming/PeepThis/bin/x.exe");
        assert_eq!(status_line_command(Path::new(r"C:\Users\a b\x.exe")), "\"C:/Users/a b/x.exe\"");
    }

    #[test]
    fn only_loopback_hook_urls_count_as_ours() {
        assert!(is_our_hook(&serde_json::json!({ "type": "http", "url": "http://127.0.0.1:47821/hook/Stop" })));
        assert!(!is_our_hook(&serde_json::json!({ "type": "http", "url": "https://example.com/hook/Stop" })));
        assert!(!is_our_hook(&serde_json::json!({ "type": "command", "command": "curl http://127.0.0.1:1/hook/Stop" })));
    }
}
