//! The list of files of the window that opens a file: the folder being looked at, its sub-folders first and then the files of the kinds asked for. It only reads folders, so it is
//! tested without a screen.

use std::path::{Path, PathBuf};

/// A folder or a file in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Whether a file name ends in one of the extensions (compared without regard to case); with no extensions in the list every name matches.
pub fn matches_extension(name: &str, extensions: &[&str]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => {
            extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        }
        _ => false,
    }
}

/// The contents of a folder: the sub-folders first, then the files that match, each group in alphabetical order. Names that start with a dot are left out.
pub fn list(dir: &Path, extensions: &[&str]) -> Result<Vec<Entry>, String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut folders = Vec::new();
    let mut files = Vec::new();
    for item in read.flatten() {
        let name = item.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = item.metadata() else {
            continue;
        };
        if meta.is_dir() {
            folders.push(Entry {
                name,
                is_dir: true,
                size: 0,
            });
        } else if meta.is_file() && matches_extension(&name, extensions) {
            files.push(Entry {
                name,
                is_dir: false,
                size: meta.len(),
            });
        }
    }
    folders.sort_by_key(|e| e.name.to_lowercase());
    files.sort_by_key(|e| e.name.to_lowercase());
    folders.extend(files);
    Ok(folders)
}

/// The folder where a window starts when nothing better is known: the current one, or the one of the person.
pub fn start_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// A size for the list: bytes, KiB or MiB.
pub fn size_text(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// The state of the window that opens a file.
#[derive(Debug, Clone)]
pub struct Browser {
    pub dir: PathBuf,
    pub entries: Vec<Entry>,
    pub error: Option<String>,
    /// The text of the box where a path can be typed.
    pub path_text: String,
    /// The extensions shown; every file is shown when `show_all` is on.
    pub extensions: Vec<&'static str>,
    pub show_all: bool,
}

impl Browser {
    pub fn new(dir: PathBuf, extensions: &[&'static str], show_all: bool) -> Browser {
        let mut b = Browser {
            dir: PathBuf::new(),
            entries: Vec::new(),
            error: None,
            path_text: String::new(),
            extensions: extensions.to_vec(),
            show_all,
        };
        b.go(dir);
        b
    }

    fn filter(&self) -> &[&'static str] {
        if self.show_all {
            &[]
        } else {
            &self.extensions
        }
    }

    /// Reads the folder again (after the filter changed).
    pub fn refresh(&mut self) {
        match list(&self.dir, self.filter()) {
            Ok(entries) => {
                self.entries = entries;
                self.error = None;
            }
            Err(e) => {
                self.entries.clear();
                self.error = Some(e);
            }
        }
    }

    /// Goes to a folder. When it cannot be read the list is empty and the error is kept for the window to show.
    pub fn go(&mut self, dir: PathBuf) {
        self.dir = dir;
        self.path_text = self.dir.display().to_string();
        self.refresh();
    }

    /// Goes to the folder above, if there is one.
    pub fn up(&mut self) {
        if let Some(parent) = self.dir.parent() {
            if !parent.as_os_str().is_empty() {
                let parent = parent.to_path_buf();
                self.go(parent);
            }
        }
    }

    /// What a click on a name does: a folder is entered (and nothing is returned); a file is returned with its whole path.
    pub fn enter(&mut self, name: &str) -> Option<PathBuf> {
        let path = self.dir.join(name);
        if path.is_dir() {
            self.go(path);
            None
        } else {
            Some(path)
        }
    }

    /// What the path typed in the box means: a folder is entered, a file is returned.
    pub fn go_typed(&mut self) -> Option<PathBuf> {
        let path = PathBuf::from(self.path_text.trim());
        if path.is_dir() {
            self.go(path);
            None
        } else if path.is_file() {
            Some(path)
        } else {
            self.error = Some(format!("{}: not found", path.display()));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("eh3d-browse-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extensions_are_compared_without_case() {
        assert!(matches_extension("A.EH3DLIST", &["eh3dlist"]));
        assert!(matches_extension("a.eh3dlist", &["EH3DLIST", "x"]));
        assert!(!matches_extension("a.txt", &["eh3dlist"]));
        assert!(!matches_extension(".eh3dlist", &["eh3dlist"]));
        assert!(matches_extension("anything", &[]));
    }

    #[test]
    fn folders_come_first_and_hidden_names_are_left_out() {
        let dir = scratch("list");
        fs::create_dir_all(dir.join("zeta")).unwrap();
        fs::write(dir.join("b.eh3dlist"), "x").unwrap();
        fs::write(dir.join("A.eh3dlist"), "x").unwrap();
        fs::write(dir.join("c.txt"), "x").unwrap();
        fs::write(dir.join(".hidden.eh3dlist"), "x").unwrap();
        let names: Vec<String> = list(&dir, &["eh3dlist"])
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, ["zeta", "A.eh3dlist", "b.eh3dlist"]);
        assert_eq!(list(&dir, &[]).unwrap().len(), 4);
        assert!(list(&dir.join("missing"), &[]).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_browser_enters_folders_and_returns_files() {
        let dir = scratch("browser");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("p.eh3dlist"), "x").unwrap();
        let mut b = Browser::new(dir.clone(), &["eh3dlist"], false);
        assert_eq!(b.enter("sub"), None);
        assert_eq!(b.dir, dir.join("sub"));
        assert_eq!(
            b.enter("p.eh3dlist"),
            Some(dir.join("sub").join("p.eh3dlist"))
        );
        b.up();
        assert_eq!(b.dir, dir);
        b.path_text = dir.join("nothing").display().to_string();
        assert_eq!(b.go_typed(), None);
        assert!(b.error.as_deref().unwrap_or("").contains("not found"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sizes_are_told_in_bytes_kib_and_mib() {
        assert_eq!(size_text(12), "12 B");
        assert_eq!(size_text(2048), "2.0 KiB");
        assert_eq!(size_text(3 * 1_048_576), "3.0 MiB");
    }
}
