//! The project list (`.eh3dlist`; the files that older versions wrote as `.eh3dproj` are still read): a JSON document with the name of the project, the files it uses (each with its kind and its SHA-256) and the work zero.
//! The files stay where they are; the project says which ones and checks that they have not changed.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

/// The version of the format this program writes; an older one is migrated when it is read, a newer one is refused.
pub const SCHEMA_VERSION: u64 = 1;
pub const MAX_NAME: usize = 120;
/// The extension of the files this program writes. `.eh3dproj` is the archive of EH3D-COMMON, a different format.
pub const EXTENSION: &str = "eh3dlist";
const MAX_LAYER_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Gerber,
    Drill,
    Hpgl,
    Gcode,
    PickPlace,
    Image,
    Other,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Gerber => "gerber",
            Kind::Drill => "drill",
            Kind::Hpgl => "hpgl",
            Kind::Gcode => "gcode",
            Kind::PickPlace => "pick-place",
            Kind::Image => "image",
            Kind::Other => "other",
        }
    }

    pub fn from_key(key: &str) -> Kind {
        [
            Kind::Gerber,
            Kind::Drill,
            Kind::Hpgl,
            Kind::Gcode,
            Kind::PickPlace,
            Kind::Image,
        ]
        .into_iter()
        .find(|k| k.key() == key)
        .unwrap_or(Kind::Other)
    }
}

/// What a file is, from its extension.
pub fn kind_of_path(path: &Path) -> Kind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "gbr" | "gtl" | "gbl" | "gts" | "gbs" | "gto" | "gbo" | "gtp" | "gbp" | "gko" | "gm1"
        | "gml" | "ger" => Kind::Gerber,
        "drl" | "ncd" | "xln" | "exc" => Kind::Drill,
        "plt" | "hpgl" | "hgl" => Kind::Hpgl,
        "nc" | "gcode" | "ngc" | "tap" | "cnc" => Kind::Gcode,
        "pos" | "csv" => Kind::PickPlace,
        "png" | "jpg" | "jpeg" | "pgm" | "bmp" => Kind::Image,
        _ => Kind::Other,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// Relative to the folder of the project file, with `/` as separator.
    pub path: String,
    pub kind: Kind,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub layers: Vec<Layer>,
    pub zero: (f64, f64),
}

/// Writes `text` to a temporary file beside `file` and renames it over `file`, so that a cut or a full disk leaves the old file whole.
pub fn write_atomic(file: &Path, text: &str) -> std::io::Result<()> {
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    if let Err(e) = fs::write(&tmp, text) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = fs::rename(&tmp, file) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

pub fn sha256_hex(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_LAYER_BYTES {
        return Err(format!(
            "{}: {} bytes is more than the {} bytes a layer may have",
            path.display(),
            meta.len(),
            MAX_LAYER_BYTES
        ));
    }
    // read in blocks of 64 KiB: a layer of many megabytes is never held whole in memory
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut block = vec![0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut block)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&block[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Runs `work` while this program holds the lock `file.lock` (a file made with `create_new`, which only one process can do), so that two runs that read a file, change it and write it
/// back do not lose each other's changes. It waits up to five seconds; a lock older than ten seconds belongs to a run that died and is taken over.
pub fn with_lock<T>(file: &Path, work: impl FnOnce() -> T) -> Result<T, String> {
    let mut name = file.as_os_str().to_owned();
    name.push(".lock");
    let lock = std::path::PathBuf::from(name);
    let mut tries = 0;
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = fs::metadata(&lock)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|age| age.as_secs() > 10);
                if stale {
                    let _ = fs::remove_file(&lock);
                    continue;
                }
                tries += 1;
                if tries > 250 {
                    return Err(format!("{}: another run holds the lock", lock.display()));
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(e) => return Err(format!("{}: {e}", lock.display())),
        }
    }
    let result = work();
    let _ = fs::remove_file(&lock);
    Ok(result)
}

/// A layer path as the project writes it: relative, with `/`, no empty part, no `..` part (a file name that merely contains two dots is fine), no drive or stream colon.
fn is_inside_path(path: &str) -> bool {
    !path.contains(':')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != ".." && part != "." && usable_part(part))
}

/// Whether a part of a path is a name every system takes: Windows refuses `CON`, `PRN`, `AUX`, `NUL`, `COM1` to `COM9` and `LPT1` to `LPT9` (with any extension) and a name that ends
/// in a dot or a space.
fn usable_part(part: &str) -> bool {
    if part.ends_with('.') || part.ends_with(' ') {
        return false;
    }
    let stem = part
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end()
        .to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    !reserved
}

fn check_name(name: &str) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("the name of the project is empty".into());
    }
    if n.chars().count() > MAX_NAME {
        return Err(format!(
            "the name of the project has more than {MAX_NAME} characters"
        ));
    }
    if n.chars().any(|c| c.is_control()) {
        return Err("the name of the project has a control character".into());
    }
    Ok(())
}

impl Project {
    pub fn new(name: &str) -> Result<Project, String> {
        check_name(name)?;
        Ok(Project {
            name: name.trim().to_string(),
            layers: Vec::new(),
            zero: (0.0, 0.0),
        })
    }

    /// Adds a file that lies under `root` (the folder of the project file); a file outside of it, or one already added, is an error.
    pub fn add_layer(&mut self, root: &Path, file: &Path) -> Result<(), String> {
        let abs = if file.is_absolute() {
            file.to_path_buf()
        } else {
            root.join(file)
        };
        let rel = abs.strip_prefix(root).map_err(|_| {
            format!(
                "{}: the file is not under the folder of the project",
                abs.display()
            )
        })?;
        let rel = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if rel.split('/').any(|p| p == ".." || p.is_empty()) {
            return Err(format!("{rel}: not a path inside the project"));
        }
        if self.layers.iter().any(|l| l.path == rel) {
            return Err(format!("{rel}: already in the project"));
        }
        let sha256 = sha256_hex(&abs)?;
        self.layers.push(Layer {
            kind: kind_of_path(&abs),
            path: rel,
            sha256,
        });
        Ok(())
    }

    /// Takes a file out of the list (the file itself is not touched); one that is not in the list is an error.
    pub fn remove_layer(&mut self, path: &str) -> Result<(), String> {
        let before = self.layers.len();
        self.layers.retain(|l| l.path != path);
        if self.layers.len() == before {
            return Err(format!("{path}: not in the project"));
        }
        Ok(())
    }

    pub fn to_json(&self) -> String {
        let layers: Vec<Value> = self
            .layers
            .iter()
            .map(|l| json!({"path": l.path, "kind": l.kind.key(), "sha256": l.sha256}))
            .collect();
        serde_json::to_string_pretty(&json!({"schema_version": SCHEMA_VERSION, "name": self.name, "layers": layers, "zero": {"x": self.zero.0, "y": self.zero.1}})).unwrap_or_default() + "\n"
    }

    pub fn from_json(text: &str) -> Result<Project, String> {
        let v: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|e| format!("the project file is not JSON: {e}"))?;
        let schema = v
            .get("schema_version")
            .and_then(|s| s.as_u64())
            .unwrap_or(0);
        if schema > SCHEMA_VERSION {
            return Err(format!(
                "the project file has format {schema}; this program reads up to {SCHEMA_VERSION}"
            ));
        }
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("the project file has no name")?;
        check_name(name)?;
        let mut layers = Vec::new();
        for item in v
            .get("layers")
            .and_then(|l| l.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            if let Some(path) = item.as_str() {
                // format 0: a list of paths without kind or checksum
                if !is_inside_path(path) {
                    return Err(format!(
                        "{path}: a layer must be a relative path inside the project"
                    ));
                }
                layers.push(Layer {
                    path: path.to_string(),
                    kind: kind_of_path(Path::new(path)),
                    sha256: String::new(),
                });
            } else {
                let path = item
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("a layer has no path")?;
                if !is_inside_path(path) {
                    return Err(format!(
                        "{path}: a layer must be a relative path inside the project"
                    ));
                }
                layers.push(Layer {
                    path: path.to_string(),
                    kind: Kind::from_key(
                        item.get("kind").and_then(|k| k.as_str()).unwrap_or("other"),
                    ),
                    sha256: item
                        .get("sha256")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
        // two layers whose paths differ only in case are one file on Windows and on macOS: the first is kept
        let mut seen: Vec<String> = Vec::new();
        layers.retain(|l: &Layer| {
            let key = l.path.to_lowercase();
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        });
        let zero = v.get("zero");
        let num = |key: &str| {
            zero.and_then(|z| z.get(key))
                .and_then(|n| n.as_f64())
                .filter(|n| n.is_finite())
                .unwrap_or(0.0)
        };
        Ok(Project {
            name: name.trim().to_string(),
            layers,
            zero: (num("x"), num("y")),
        })
    }

    pub fn save(&self, file: &Path) -> Result<(), String> {
        write_atomic(file, &self.to_json()).map_err(|e| format!("{}: {e}", file.display()))
    }

    pub fn load(file: &Path) -> Result<Project, String> {
        let bytes = fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
        if bytes.starts_with(b"PK") {
            return Err(format!(
                "{}: this is an archive of EH3D-COMMON (.eh3dproj), not a project list (.{EXTENSION})",
                file.display()
            ));
        }
        let text = String::from_utf8(bytes).map_err(|_| format!("{}: not text", file.display()))?;
        Project::from_json(&text)
    }

    /// The layers that have no checksum (a list of format 0, or an entry written without one): `verify` can only look for their files, not tell if they changed.
    pub fn unchecked(&self) -> Vec<&str> {
        self.layers
            .iter()
            .filter(|l| l.sha256.is_empty())
            .map(|l| l.path.as_str())
            .collect()
    }

    /// Checks every layer against the disk: a missing file or a changed checksum is one message. A layer of format 0 has no checksum and is only looked for.
    pub fn verify(&self, root: &Path) -> Vec<String> {
        let mut problems = Vec::new();
        for l in &self.layers {
            let file = root.join(&l.path);
            if !file.is_file() {
                problems.push(format!("{}: the file is missing", l.path));
            } else if !l.sha256.is_empty() {
                match sha256_hex(&file) {
                    Ok(h) if h == l.sha256 => {}
                    Ok(_) => problems.push(format!(
                        "{}: the file has changed since it was added",
                        l.path
                    )),
                    Err(e) => problems.push(e),
                }
            }
        }
        problems
    }
}

/// Writes an empty project list `file` called `name`. The file must end in `.eh3dlist` and must not exist: nothing is overwritten.
pub fn create_file(file: &str, name: &str) -> Result<(), String> {
    if Path::new(file).extension().and_then(|e| e.to_str()) != Some(EXTENSION) {
        return Err(format!(
            "{file}: a project list ends in .{EXTENSION} (.eh3dproj is the archive of EH3D-COMMON)"
        ));
    }
    if Path::new(file).exists() {
        return Err(format!("{file}: already exists; nothing was overwritten"));
    }
    Project::new(name)?.save(Path::new(file))
}

/// Adds `layer` to the project list `file` (kind and SHA-256 are recorded) and says how many files the project has now. The file must lie under the folder of the project.
/// Reading, adding and writing happen under a lock: two runs at the same time do not lose a layer.
pub fn add_layer_to_file(file: &Path, layer: &Path) -> Result<usize, String> {
    let root = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    let layer_abs = layer
        .canonicalize()
        .map_err(|e| format!("{}: {e}", layer.display()))?;
    with_lock(file, || {
        let mut p = Project::load(file)?;
        p.add_layer(&root, &layer_abs)?;
        p.save(file)?;
        Ok::<usize, String>(p.layers.len())
    })?
}

/// Takes the layer `layer` (as the project writes it) out of the project list `file` and says how many files are left; under the same lock as `add_layer_to_file`.
pub fn remove_layer_from_file(file: &Path, layer: &str) -> Result<usize, String> {
    with_lock(file, || {
        let mut p = Project::load(file)?;
        p.remove_layer(layer)?;
        p.save(file)?;
        Ok::<usize, String>(p.layers.len())
    })?
}
