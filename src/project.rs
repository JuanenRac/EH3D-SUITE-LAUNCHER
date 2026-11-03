//! The project list (`.eh3dlist`; the files that older versions wrote as `.eh3dproj` are still read): a JSON document with the name of the project, the files it uses (each with its kind and its SHA-256) and the work zero.
//! The files stay where they are; the project says which ones and checks that they have not changed.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
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
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// A layer path as the project writes it: relative, with `/`, no empty part, no `..` part (a file name that merely contains two dots is fine), no drive or stream colon.
fn is_inside_path(path: &str) -> bool {
    !path.contains(':')
        && !path.contains('\\')
        && path.split('/').all(|part| !part.is_empty() && part != "..")
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
        fs::write(file, self.to_json()).map_err(|e| format!("{}: {e}", file.display()))
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
