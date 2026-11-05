//! The programs of the suite: each repository says what it is in its `eh3d.project.json`, so nothing here is a fixed list of names.

use std::fs;
use std::path::{Path, PathBuf};

/// One repository of the suite found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub version: String,
    pub role: String,
    pub maturity: String,
    pub deployment_target: String,
    pub path: PathBuf,
}

impl Module {
    /// The name of the program a person starts: `EH3D-DRILLPRO` -> `drillpro`.
    pub fn program(&self) -> String {
        program_name(&self.name)
    }

    /// A program for the computer of the person (not a firmware, a service or a phone app).
    pub fn is_desktop(&self) -> bool {
        self.deployment_target == "user-pc" && matches!(self.role.as_str(), "tool" | "ui" | "cli")
    }
}

/// The programs of the suite that `launch` starts: only these, and only from the folder the configuration file names.
pub const KNOWN_PROGRAMS: &[&str] = &[
    "EH3D-AUTOLEVEL",
    "EH3D-BUNGARD-GUI",
    "EH3D-CAM-DEBUGGER",
    "EH3D-DESIGN-EXPLORER",
    "EH3D-DESIGN-VERIFIER",
    "EH3D-DRILLPRO",
    "EH3D-GERBERVIEWER",
    "EH3D-ISOCAM",
    "EH3D-PHOENIX-CAD",
    "EH3D-PHOENIX-CNC",
    "EH3D-PHOENIX-LASER",
    "EH3D-PHOENIX-PCB",
    "EH3D-PHOENIX-PNP",
    "EH3D-PROJECT-PASSPORT",
    "EH3D-ROUTEPRO",
    "EH3D-STENCILCAM",
    "EH3D-SUITE-LAUNCHER",
    "EH3D-THERMAL-LAB",
    "EH3D-TOOL-MANAGER",
    "EH3D-UPDATER",
    "EH3D-VALIDATION-LAB",
    "EH3D-VISION-STUDIO",
];

/// True when the module is one of the programs of the suite (the name of the manifest is on the list).
pub fn is_known(module: &Module) -> bool {
    KNOWN_PROGRAMS.contains(&module.name.as_str())
}

/// A name a program file can have: letters, digits, `_`, `-` and `.` inside the name, never a separator, never `..`, never a drive.
pub fn valid_bin_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

/// `EH3D-DRILLPRO` -> `drillpro`; `EH3D-BUNGARD-GUI` -> `bungard-gui`.
pub fn program_name(repo: &str) -> String {
    repo.strip_prefix("EH3D-")
        .unwrap_or(repo)
        .to_ascii_lowercase()
}

/// Reads the manifest of one repository. Anything that is not an EH3D manifest is an error that says why.
pub fn read_manifest(dir: &Path) -> Result<Module, String> {
    let file = dir.join("eh3d.project.json");
    let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    // a manifest saved with a byte order mark is still a manifest
    let text = text.trim_start_matches('\u{feff}').to_string();
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: not JSON: {e}", file.display()))?;
    let get = |key: &str| -> Result<String, String> {
        json.get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                format!(
                    "{}: the field '{key}' is missing or is not text",
                    file.display()
                )
            })
    };
    if get("ecosystem")? != "EH3D" {
        return Err(format!("{}: the ecosystem is not EH3D", file.display()));
    }
    let name = get("name")?;
    let version = get("version")?;
    if version.split('.').count() < 3
        || version
            .split('.')
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(format!(
            "{}: '{version}' is not a version of numbers separated by dots",
            file.display()
        ));
    }
    Ok(Module {
        name,
        version,
        role: get("role")?,
        maturity: get("maturity")?,
        deployment_target: get("deployment_target")?,
        path: dir.to_path_buf(),
    })
}

/// Every EH3D repository that is a direct child of `workspace`, in alphabetical order, with the folders that could not be read as a list of reasons.
pub fn discover(workspace: &Path) -> (Vec<Module>, Vec<String>) {
    let mut modules = Vec::new();
    let mut problems = Vec::new();
    let Ok(entries) = fs::read_dir(workspace) else {
        return (
            modules,
            vec![format!(
                "{}: the folder cannot be read",
                workspace.display()
            )],
        );
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("eh3d.project.json").is_file())
        .collect();
    dirs.sort();
    for dir in dirs {
        match read_manifest(&dir) {
            Ok(m) => modules.push(m),
            Err(e) => problems.push(e),
        }
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    (modules, problems)
}

/// The names a module's program can have: the ones its `Cargo.toml` gives to its `[[bin]]` targets first, then the name of the repository
/// (`EH3D-DRILLPRO` -> `drillpro`) with and without the `eh3d-` prefix.
pub fn binary_names(module: &Module) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Ok(text) = fs::read_to_string(module.path.join("Cargo.toml")) {
        let mut in_bin = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_bin = line.split('#').next().unwrap_or("").trim() == "[[bin]]";
            } else if in_bin {
                if let Some(rest) = line.strip_prefix("name") {
                    if let Some(value) = rest.trim_start().strip_prefix('=') {
                        // the text between the quotes; whatever follows (a comment) is not part of the name
                        let value = value.trim();
                        let name = match value.chars().next() {
                            Some(q @ ('"' | '\'')) => {
                                value[1..].split(q).next().unwrap_or_default().to_string()
                            }
                            _ => value.split('#').next().unwrap_or("").trim().to_string(),
                        };
                        if valid_bin_name(&name) && !names.contains(&name) {
                            names.push(name);
                        }
                    }
                }
            }
        }
    }
    for name in [module.program(), format!("eh3d-{}", module.program())] {
        if valid_bin_name(&name) && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// The module that a name stands for: the name of its repository or the name of its program, without regard to case.
pub fn find_module<'a>(modules: &'a [Module], name: &str) -> Option<&'a Module> {
    let n = name.to_ascii_lowercase();
    modules
        .iter()
        .find(|m| m.name.to_ascii_lowercase() == n || m.program() == n)
}

/// Where the built program of a module is: `target/release` first, then `target/debug`; `None` when it has not been built.
pub fn find_binary(module: &Module) -> Option<PathBuf> {
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    ["release", "debug"].iter().find_map(|profile| {
        binary_names(module)
            .into_iter()
            .map(|name| {
                module
                    .path
                    .join("target")
                    .join(profile)
                    .join(format!("{name}{suffix}"))
            })
            .find(|p| p.is_file())
    })
}

/// The version that a text such as `0.0.7` stands for, as numbers; `None` when it is not one.
pub fn version_numbers(text: &str) -> Option<Vec<u64>> {
    text.split('.').map(|p| p.parse::<u64>().ok()).collect()
}

/// True when `candidate` is a newer version than `installed` (a missing component counts as zero).
pub fn is_newer(candidate: &str, installed: &str) -> bool {
    match (version_numbers(candidate), version_numbers(installed)) {
        (Some(mut a), Some(mut b)) => {
            let n = a.len().max(b.len());
            a.resize(n, 0);
            b.resize(n, 0);
            a > b
        }
        _ => false,
    }
}
