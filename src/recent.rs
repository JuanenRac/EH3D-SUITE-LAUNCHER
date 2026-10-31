//! The list of recent projects: one path per line, the newest first, no path twice, at most `MAX_RECENT`.

use std::fs;
use std::path::Path;

pub const MAX_RECENT: usize = 10;

/// Reads the list; a missing or damaged file is an empty list, never an error (the list is a convenience).
pub fn load(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
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
    seen
}

/// Puts `entry` at the head of `list` (it is not listed twice) and cuts the list to `MAX_RECENT`.
pub fn remember(list: &mut Vec<String>, entry: &str) {
    list.retain(|e| e != entry);
    list.insert(0, entry.to_string());
    list.truncate(MAX_RECENT);
}

/// Writes the list.
pub fn save(path: &Path, list: &[String]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, list.join("\n") + "\n")
}
