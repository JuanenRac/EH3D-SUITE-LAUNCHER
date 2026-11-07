//! The list of recent projects: one path per line, the newest first, no path twice, at most `MAX_RECENT`.

use crate::limit;
use std::fs;
use std::path::Path;

pub const MAX_RECENT: usize = 10;

/// Reads the list; a missing or damaged file is an empty list, never an error (the list is a convenience).
pub fn load(path: &Path) -> Vec<String> {
    load_with_limit(path, limit::mib(limit::DEFAULT_MAX_FILE_MIB))
}

/// Reads the list like `load`; a file larger than `max_bytes` is an empty list too.
pub fn load_with_limit(path: &Path, max_bytes: u64) -> Vec<String> {
    load_checked(path, max_bytes).unwrap_or_default()
}

/// Reads the list; a file that is not there is an empty list, and one that is larger than `max_bytes`, that cannot be read or that is not text is the error with its code.
pub fn load_checked(path: &Path, max_bytes: u64) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = limit::read_text_limited(path, max_bytes)?;
    let mut seen: Vec<String> = Vec::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && l.len() < 4096 && !l.contains('\0'))
    {
        if !seen.iter().any(|s| s == line) {
            seen.push(line.to_string());
        }
    }
    seen.truncate(MAX_RECENT);
    Ok(seen)
}

/// Puts `entry` at the head of `list` (it is not listed twice) and cuts the list to `MAX_RECENT`.
pub fn remember(list: &mut Vec<String>, entry: &str) {
    list.retain(|e| e != entry);
    list.insert(0, entry.to_string());
    list.truncate(MAX_RECENT);
}

/// The full path of an existing file, the way the list keeps it: a file that does not exist cannot be remembered, and the same file is never listed twice under two spellings
/// (`./a.eh3dlist` and `a.eh3dlist`).
pub fn canonical(file: &str) -> Result<String, String> {
    let full = Path::new(file)
        .canonicalize()
        .map_err(|e| format!("{file}: {e}"))?;
    let text = full.to_string_lossy().into_owned();
    // Windows gives the long form `\\?\C:\...`
    Ok(text.strip_prefix(r"\\?\").unwrap_or(&text).to_string())
}

/// Puts the file at the head of the list kept in `path` and returns the list. The reading, the change and the writing happen under a lock, so two runs at the same time
/// do not lose each other's entry.
pub fn add(path: &Path, file: &str) -> Result<Vec<String>, String> {
    add_with_limit(path, file, limit::mib(limit::DEFAULT_MAX_FILE_MIB))
}

/// `add`, reading the list only when it is not larger than `max_bytes`.
pub fn add_with_limit(path: &Path, file: &str, max_bytes: u64) -> Result<Vec<String>, String> {
    let entry = canonical(file)?;
    crate::project::with_lock(path, || {
        let mut list = load_with_limit(path, max_bytes);
        remember(&mut list, &entry);
        save(path, &list)
            .map(|()| list)
            .map_err(|e| format!("{}: {e}", path.display()))
    })?
}

/// Writes the list.
pub fn save(path: &Path, list: &[String]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    crate::project::write_atomic(path, &(list.join("\n") + "\n"))
}
