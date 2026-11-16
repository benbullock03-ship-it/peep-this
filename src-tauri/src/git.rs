use std::path::{Path, PathBuf};

/// Branch checked out in the repo holding `cwd`, from `HEAD` alone; a detached head gives its short sha.
pub fn branch(cwd: &str) -> Option<String> {
    if cwd.is_empty() {
        return None;
    }
    let head = std::fs::read_to_string(git_dir(Path::new(cwd))?.join("HEAD")).ok()?;
    parse_head(&head)
}

/// `.git` is a directory, or a file (`gitdir: <path>`) in worktrees and submodules.
fn git_dir(cwd: &Path) -> Option<PathBuf> {
    let dot = cwd.ancestors().map(|d| d.join(".git")).find(|p| p.exists())?;
    if dot.is_dir() {
        return Some(dot);
    }
    let text = std::fs::read_to_string(&dot).ok()?;
    let target = Path::new(text.trim().strip_prefix("gitdir:")?.trim());
    Some(dot.parent()?.join(target))
}

pub fn parse_head(head: &str) -> Option<String> {
    let head = head.trim();
    if let Some(r) = head.strip_prefix("ref:") {
        let r = r.trim();
        let name = r.strip_prefix("refs/heads/").unwrap_or(r);
        return (!name.is_empty()).then(|| name.to_string());
    }
    (head.len() >= 7 && head.chars().all(|c| c.is_ascii_hexdigit())).then(|| head[..7].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!("peepthis-git-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn head_parsing() {
        assert_eq!(parse_head("ref: refs/heads/main\n").as_deref(), Some("main"));
        assert_eq!(parse_head("ref: refs/heads/feature/login").as_deref(), Some("feature/login"));
        assert_eq!(parse_head("8af3bd1c0ffee00000000000000000000000000a\n").as_deref(), Some("8af3bd1"));
        assert_eq!(parse_head(""), None);
        assert_eq!(parse_head("garbage!"), None);
    }

    #[test]
    fn finds_head_from_a_subdirectory_and_through_a_worktree_file() {
        let repo = temp_dir();
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::create_dir_all(repo.join("src/deep")).unwrap();
        assert_eq!(branch(repo.join("src/deep").to_str().unwrap()).as_deref(), Some("main"));

        let wt = temp_dir();
        std::fs::create_dir_all(repo.join(".git/worktrees/wt")).unwrap();
        std::fs::write(repo.join(".git/worktrees/wt/HEAD"), "ref: refs/heads/feature\n").unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", repo.join(".git/worktrees/wt").display())).unwrap();
        assert_eq!(branch(wt.to_str().unwrap()).as_deref(), Some("feature"));

        assert_eq!(branch(""), None);
        std::fs::remove_dir_all(repo).unwrap();
        std::fs::remove_dir_all(wt).unwrap();
    }
}
