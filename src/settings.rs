//! The settings that a person may change, read from the lines of `launcher.cfg` (the file that also holds `workspace`): the language, how many times a failed program is
//! started again, the waiting time between starts and the size of the window. A key that is not here is left to the other readers of the file; a value that makes no sense
//! is an error that names the line, never a value that only looks right.

use crate::lang::Lang;
use crate::launch::{Policy, CONFIG_FILE};
use crate::limit::{self, DEFAULT_MAX_FILE_MIB, MAX_CONFIG_BYTES};
use std::path::Path;

/// The values used when the file does not say otherwise.
pub const DEFAULT_MAX_RESTARTS: u32 = 2;
pub const DEFAULT_BACKOFF_MS: u64 = 500;
pub const DEFAULT_WINDOW_WIDTH: u32 = 1100;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 700;
/// The limits a value of the file may have.
pub const MAX_RESTARTS_LIMIT: u32 = 10;
pub const MAX_BACKOFF_MS: u64 = 30_000;
pub const MIN_WINDOW: (u32, u32) = (640, 420);
pub const MAX_WINDOW: (u32, u32) = (7680, 4320);
pub const MAX_FILE_MIB_LIMITS: (u64, u64) = (1, 1024);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// The language of the window; `None` follows the system.
    pub language: Option<Lang>,
    pub max_restarts: u32,
    pub backoff_ms: u64,
    pub window_width: u32,
    pub window_height: u32,
    /// The largest manifest or list of recent projects that is read, in MiB; a bigger one is refused with the code `file_too_large`.
    pub max_file_mib: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: None,
            max_restarts: DEFAULT_MAX_RESTARTS,
            backoff_ms: DEFAULT_BACKOFF_MS,
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            max_file_mib: DEFAULT_MAX_FILE_MIB,
        }
    }
}

/// A whole number between `low` and `high`, or the reason it is not.
fn number<T: std::str::FromStr + PartialOrd + std::fmt::Display + Copy>(
    line: usize,
    key: &str,
    value: &str,
    low: T,
    high: T,
) -> Result<T, String> {
    match value.parse::<T>() {
        Ok(n) if (low..=high).contains(&n) => Ok(n),
        _ => Err(format!(
            "{CONFIG_FILE} line {line}: {key} is '{value}', a whole number from {low} to {high} is needed"
        )),
    }
}

impl Settings {
    /// Reads the text of the file. Empty lines and lines that start with `#` are skipped; `key = value`, with the value optionally in quotes.
    pub fn parse(text: &str) -> Result<Settings, String> {
        let mut s = Settings::default();
        for (n, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
            let line = n + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let Some((key, value)) = raw.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim().trim_matches('"'));
            match key {
                "language" => {
                    let v = value.to_ascii_lowercase();
                    s.language = if v == "auto" {
                        None
                    } else {
                        Some(
                            Lang::ALL
                                .into_iter()
                                .find(|l| l.code() == v)
                                .ok_or_else(|| {
                                    format!(
                                        "{CONFIG_FILE} line {line}: language is '{value}', one of auto, en, es, de, fr, it, ja, zh or pt is needed"
                                    )
                                })?,
                        )
                    };
                }
                "max_restarts" => {
                    s.max_restarts = number(line, key, value, 0, MAX_RESTARTS_LIMIT)?;
                }
                "backoff_ms" => {
                    s.backoff_ms = number(line, key, value, 0, MAX_BACKOFF_MS)?;
                }
                "window_width" => {
                    s.window_width = number(line, key, value, MIN_WINDOW.0, MAX_WINDOW.0)?;
                }
                "window_height" => {
                    s.window_height = number(line, key, value, MIN_WINDOW.1, MAX_WINDOW.1)?;
                }
                "max_file_mib" => {
                    s.max_file_mib = number(
                        line,
                        key,
                        value,
                        MAX_FILE_MIB_LIMITS.0,
                        MAX_FILE_MIB_LIMITS.1,
                    )?;
                }
                _ => {}
            }
        }
        Ok(s)
    }

    /// Reads `launcher.cfg` in `dir`; a file that is not there means the defaults.
    pub fn load(dir: &Path) -> Result<Settings, String> {
        let file = dir.join(CONFIG_FILE);
        if !file.exists() {
            return Ok(Settings::default());
        }
        Settings::parse(&limit::read_text_limited(&file, MAX_CONFIG_BYTES)?)
    }

    /// The size limit of the files that are read, in bytes.
    pub fn max_file_bytes(&self) -> u64 {
        limit::mib(self.max_file_mib)
    }

    /// How a failed program is started again.
    pub fn policy(&self) -> Policy {
        Policy {
            max_restarts: self.max_restarts,
            backoff_ms: self.backoff_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_gives_the_defaults() {
        assert_eq!(Settings::parse("").unwrap(), Settings::default());
        assert_eq!(Settings::default().policy(), Policy::default());
    }

    #[test]
    fn the_keys_are_read_and_the_other_lines_are_left_alone() {
        let s = Settings::parse(
            "\u{feff}# comment\nworkspace = C:\\suite\nlanguage = \"PT\"\nmax_restarts = 0\nbackoff_ms = 1200\nwindow_width = 1400\nwindow_height=900\nunknown = 1\n",
        )
        .unwrap();
        assert_eq!(s.language, Some(Lang::Pt));
        assert_eq!(s.max_restarts, 0);
        assert_eq!(s.backoff_ms, 1200);
        assert_eq!((s.window_width, s.window_height), (1400, 900));
        assert_eq!(
            s.policy(),
            Policy {
                max_restarts: 0,
                backoff_ms: 1200
            }
        );
        assert_eq!(Settings::parse("language = auto").unwrap().language, None);
    }

    #[test]
    fn a_value_that_makes_no_sense_names_its_line() {
        for (text, key) in [
            ("\nmax_restarts = 11", "max_restarts"),
            ("backoff_ms = -1", "backoff_ms"),
            ("window_width = 100", "window_width"),
            ("window_height = tall", "window_height"),
            ("language = klingon", "language"),
        ] {
            let e = Settings::parse(text).unwrap_err();
            assert!(e.contains(key) && e.contains("line"), "{e}");
        }
        assert!(Settings::parse("\nbackoff_ms = 99999")
            .unwrap_err()
            .contains("line 2"));
    }

    #[test]
    fn a_missing_file_is_the_defaults_and_a_file_is_read() {
        let dir = std::env::temp_dir().join(format!("eh3d-settings-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), Settings::default());
        std::fs::write(dir.join(CONFIG_FILE), "max_restarts = 4\n").unwrap();
        assert_eq!(Settings::load(&dir).unwrap().max_restarts, 4);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
