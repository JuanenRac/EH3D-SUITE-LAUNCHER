// EH3D-SUITE-LAUNCHER - command line.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]

use eh3d_suite_launcher::launch::{supervise, Policy};
use eh3d_suite_launcher::project::Project;
use eh3d_suite_launcher::recent;
use eh3d_suite_launcher::registry::{discover, find_binary, Module};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "eh3d-launcher: the hub of the EH3D Fabrication Suite

  eh3d-launcher list [--workspace DIR] [--all]     the programs found (--all also lists firmware, services and apps)
  eh3d-launcher launch NAME [--workspace DIR] [-- ARGS...]    start a program, again up to 2 times if it fails
  eh3d-launcher new-project FILE.eh3dproj NAME     write an empty project file
  eh3d-launcher add-layer FILE.eh3dproj LAYER      add a file under the folder of the project (kind and SHA-256 are recorded)
  eh3d-launcher verify FILE.eh3dproj               check that the files of the project are there and unchanged
  eh3d-launcher recent [add FILE]                  show the recent projects, or put one at the head

The workspace is the folder that holds the repositories of the suite: --workspace, or the variable EH3D_WORKSPACE, or '..'.
The list of recent projects is kept in the folder of the variable EH3D_LAUNCHER_DIR (default: the application data folder).";

fn workspace(args: &mut Vec<String>) -> PathBuf {
    if let Some(i) = args.iter().position(|a| a == "--workspace") {
        if i + 1 < args.len() {
            let dir = args.remove(i + 1);
            args.remove(i);
            return PathBuf::from(dir);
        }
    }
    env::var_os("EH3D_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".."))
}

fn data_dir() -> PathBuf {
    if let Some(d) = env::var_os("EH3D_LAUNCHER_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = env::var_os("APPDATA") {
        return PathBuf::from(d).join("eh3d-launcher");
    }
    if let Some(d) = env::var_os("HOME") {
        return PathBuf::from(d).join(".config").join("eh3d-launcher");
    }
    PathBuf::from(".")
}

fn find<'a>(modules: &'a [Module], name: &str) -> Option<&'a Module> {
    let n = name.to_ascii_lowercase();
    modules
        .iter()
        .find(|m| m.name.to_ascii_lowercase() == n || m.program() == n)
}

fn run(mut args: Vec<String>) -> Result<(), String> {
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        println!("{HELP}");
        return Ok(());
    }
    let cmd = args.remove(0);
    match cmd.as_str() {
        "list" => {
            let all = if let Some(i) = args.iter().position(|a| a == "--all") {
                args.remove(i);
                true
            } else {
                false
            };
            let ws = workspace(&mut args);
            let (modules, problems) = discover(&ws);
            for m in modules.iter().filter(|m| all || m.is_desktop()) {
                let built = if find_binary(m).is_some() {
                    "built"
                } else {
                    "not built"
                };
                println!(
                    "{:<28} {:<10} {:<12} {:<9} {built}",
                    m.name, m.version, m.maturity, m.role
                );
            }
            for p in problems {
                eprintln!("skipped: {p}");
            }
            Ok(())
        }
        "launch" => {
            let ws = workspace(&mut args);
            let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
            let rest: Vec<String> = args.split_off(split).into_iter().skip(1).collect();
            let name = args
                .first()
                .ok_or("launch needs the name of a program (see list)")?;
            let (modules, _) = discover(&ws);
            let module = find(&modules, name)
                .ok_or_else(|| format!("{name}: no such program in {}", ws.display()))?;
            if !module.is_desktop() {
                return Err(format!(
                    "{}: it is a {} for '{}', not a program for this computer",
                    module.name, module.role, module.deployment_target
                ));
            }
            let bin = find_binary(module).ok_or_else(|| {
                format!(
                    "{}: it has not been built; run build.bat or build.sh in {}",
                    module.name,
                    module.path.display()
                )
            })?;
            let out = supervise(&bin, &rest, Policy::default())?;
            if out.gave_up {
                return Err(format!(
                    "{}: it failed {} times in a row (last code {:?})",
                    module.name, out.runs, out.last_code
                ));
            }
            Ok(())
        }
        "new-project" => {
            let (file, name) = (
                args.first().ok_or("new-project needs a file")?,
                args.get(1).ok_or("new-project needs a name")?,
            );
            if Path::new(file).exists() {
                return Err(format!("{file}: already exists; nothing was overwritten"));
            }
            Project::new(name)?.save(Path::new(file))?;
            let mut list = recent::load(&data_dir().join("recent.txt"));
            recent::remember(&mut list, file);
            recent::save(&data_dir().join("recent.txt"), &list).map_err(|e| e.to_string())?;
            println!("written {file}");
            Ok(())
        }
        "add-layer" => {
            let (file, layer) = (
                args.first().ok_or("add-layer needs a project file")?,
                args.get(1).ok_or("add-layer needs a layer file")?,
            );
            let root = Path::new(file)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let root = root
                .canonicalize()
                .map_err(|e| format!("{}: {e}", root.display()))?;
            let mut p = Project::load(Path::new(file))?;
            let layer_abs = Path::new(layer)
                .canonicalize()
                .map_err(|e| format!("{layer}: {e}"))?;
            p.add_layer(&root, &layer_abs)?;
            p.save(Path::new(file))?;
            println!("added {layer} ({} files in the project)", p.layers.len());
            Ok(())
        }
        "verify" => {
            let file = args.first().ok_or("verify needs a project file")?;
            let root = Path::new(file)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let p = Project::load(Path::new(file))?;
            let problems = p.verify(root);
            if problems.is_empty() {
                println!(
                    "{}: {} files, all there and unchanged",
                    p.name,
                    p.layers.len()
                );
                Ok(())
            } else {
                Err(problems.join("\n"))
            }
        }
        "recent" => {
            let path = data_dir().join("recent.txt");
            let mut list = recent::load(&path);
            if args.first().map(String::as_str) == Some("add") {
                let file = args.get(1).ok_or("recent add needs a file")?;
                recent::remember(&mut list, file);
                recent::save(&path, &list).map_err(|e| e.to_string())?;
            }
            for e in &list {
                println!("{e}");
            }
            Ok(())
        }
        other => Err(format!("{other}: unknown command (see help)")),
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
