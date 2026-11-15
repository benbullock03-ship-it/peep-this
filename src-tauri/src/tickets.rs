//! A repo's tickets folder, read for the ticket a Peep skill call links (spec §13.2).

use std::path::{Path, PathBuf};

use crate::state::TicketLink;

/// The first `PREFIX-N` argument, uppercased, with its title when the ticket file is found under `cwd`.
pub fn link(args: &str, cwd: Option<&str>) -> Option<TicketLink> {
    let id = args.split_whitespace().find(|t| is_ticket_id(t))?.to_ascii_uppercase();
    let title = cwd.filter(|c| !c.is_empty()).and_then(|c| title(Path::new(c), &id));
    Some(TicketLink { id, title })
}

fn is_ticket_id(t: &str) -> bool {
    let Some((prefix, n)) = t.split_once('-') else { return false };
    prefix.starts_with(|c: char| c.is_ascii_alphabetic())
        && prefix.chars().all(|c| c.is_ascii_alphanumeric())
        && !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
}

/// The repo setup's `tickets:` line (`plugin/references/formats.md` §1), else `tickets/`.
pub fn tickets_dir(repo: &Path) -> PathBuf {
    let setup = std::fs::read_to_string(repo.join(".peep").join("setup.md")).unwrap_or_default();
    let folder = setup
        .lines()
        .find_map(|l| l.strip_prefix("tickets:"))
        .map(|v| v.trim().trim_matches(['"', '\'']))
        .filter(|v| !v.is_empty());
    repo.join(folder.unwrap_or("tickets"))
}

fn title(repo: &Path, id: &str) -> Option<String> {
    let prefix = format!("{id}-");
    let file = std::fs::read_dir(tickets_dir(repo)).ok()?.flatten().map(|e| e.path()).find(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_uppercase();
        name.starts_with(&prefix) && name.ends_with(".MD")
    })?;
    let text = std::fs::read_to_string(file).ok()?;
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    lines
        .take_while(|l| l.trim() != "---")
        .find_map(|l| l.strip_prefix("title:"))
        .map(|t| t.trim().trim_matches(['"', '\'']).to_string())
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("peepthis-tickets-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: PathBuf, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn id_from_args_and_title_from_tickets_folder() {
        let dir = repo("default");
        write(dir.join("tickets/AV-12-thing.md"), "---\nid: AV-12\ntitle: \"Make the thing\"\n---\n\ntitle: not this\n");
        let cwd = dir.to_str();
        assert_eq!(link("close av-12", cwd), Some(TicketLink { id: "AV-12".into(), title: Some("Make the thing".into()) }));
        assert_eq!(link("AV-99", cwd), Some(TicketLink { id: "AV-99".into(), title: None }));
        assert_eq!(link("AV-1", cwd).unwrap().title, None, "AV-1 must not match AV-12");
        assert_eq!(link("34 extra-arg pre-x", cwd), None);
        assert_eq!(link("", cwd), None);
    }

    #[test]
    fn repo_setup_names_the_tickets_folder() {
        let dir = repo("setup");
        write(dir.join(".peep/setup.md"), "---\nprefix: W\ntickets: work/issues/\n---\n");
        write(dir.join("work/issues/W-3-x.md"), "---\ntitle: From setup\n---\n");
        write(dir.join("tickets/W-3-y.md"), "---\ntitle: Wrong folder\n---\n");
        assert_eq!(link("W-3", dir.to_str()).unwrap().title.as_deref(), Some("From setup"));
    }
}
