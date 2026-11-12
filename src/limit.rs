//! Reading a file with a limit of size: the manifests, the list of recent projects and the settings are read only when they are not larger than the limit (`max_file_mib` of
//! `launcher.cfg` for the first two), so that a file by mistake does not fill the memory. Every refusal starts with a code that does not change (`file_too_large`,
//! `file_unreadable`, `file_not_text`), followed by the name of the file.

use std::io::Read;
use std::path::Path;

/// The file is larger than the limit.
pub const TOO_LARGE: &str = "file_too_large";
/// The file cannot be opened or read.
pub const UNREADABLE: &str = "file_unreadable";
/// The file is not text.
pub const NOT_TEXT: &str = "file_not_text";

/// The limit when `launcher.cfg` does not say otherwise, in MiB.
pub const DEFAULT_MAX_FILE_MIB: u64 = 16;
/// The largest `launcher.cfg` that is read, in bytes.
pub const MAX_CONFIG_BYTES: u64 = 1_048_576;

/// How many bytes `mib` MiB are.
pub fn mib(mib: u64) -> u64 {
    mib.saturating_mul(1_048_576)
}

/// The text of a file that is not larger than `max_bytes`. At most one byte more than the limit is ever read, also when the file grows while it is being read.
pub fn read_text_limited(path: &Path, max_bytes: u64) -> Result<String, String> {
    let shown = path.display();
    let file = std::fs::File::open(path).map_err(|e| format!("{UNREADABLE}: {shown}: {e}"))?;
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{UNREADABLE}: {shown}: {e}"))?;
    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "{TOO_LARGE}: {shown}: the file is larger than {max_bytes} bytes (max_file_mib in launcher.cfg)"
        ));
    }
    String::from_utf8(bytes).map_err(|_| format!("{NOT_TEXT}: {shown}: the file is not text"))
}
