// EH3D-SUITE-LAUNCHER - command line.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]

use eh3d_suite_launcher::launch::{
    configured_workspace, data_dir, language, message, resolve, supervise, Msg, CONFIG_FILE,
};
use eh3d_suite_launcher::project::Project;
use eh3d_suite_launcher::recent;
use eh3d_suite_launcher::registry::{discover, find_binary, find_module};
use eh3d_suite_launcher::settings::Settings;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP: &str = "eh3d-launcher: the hub of the EH3D Fabrication Suite

  eh3d-launcher list [--workspace DIR] [--all]     the programs found (--all also lists firmware, services and apps)
  eh3d-launcher launch NAME [-- ARGS...]           start a program of the suite, again up to 2 times if it fails
  eh3d-launcher new-project FILE.eh3dlist NAME     write an empty project list
  eh3d-launcher add-layer FILE.eh3dlist LAYER      add a file under the folder of the project (kind and SHA-256 are recorded)
  eh3d-launcher verify FILE.eh3dlist               check that the files of the project are there and unchanged
  eh3d-launcher recent [add FILE]                  show the recent projects, or put one at the head

For list, the workspace is the folder that holds the repositories of the suite: --workspace, or the variable EH3D_WORKSPACE, or '..'.
launch starts only the known programs of the suite, from the folder named by the line 'workspace = FULL_PATH' of launcher.cfg in the folder below.
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
            // what is started comes from the folder of the configuration file only: not from the current directory, the command line or the environment
            let lang = language();
            let config = data_dir();
            let ws = configured_workspace(&config, lang)?;
            // only the options before `--` are the launcher's: the rest belongs to the program that is started
            let own = args.iter().position(|a| a == "--").unwrap_or(args.len());
            if args[..own].iter().any(|a| a == "--workspace") {
                return Err(message(
                    Msg::NoWorkspace,
                    lang,
                    &config.join(CONFIG_FILE).display().to_string(),
                ));
            }
            let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
            let rest: Vec<String> = args.split_off(split).into_iter().skip(1).collect();
            let name = args
                .first()
                .ok_or("launch needs the name of a program (see list)")?;
            let (modules, _) = discover(&ws);
            let module = find_module(&modules, name)
                .ok_or_else(|| format!("{name}: no such program in {}", ws.display()))?;
            let bin = resolve(module, &ws, lang)?;
            let out = supervise(&bin, &rest, Settings::load(&config)?.policy())?;
            if out.signalled {
                // the operator (or the system) ended it: that is not a failure and it is not started again
                eprintln!("{}: it was ended by a signal", module.name);
                return Ok(());
            }
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
            eh3d_suite_launcher::project::create_file(file, name)?;
            recent::add(&data_dir().join("recent.txt"), file)?;
            println!("written {file}");
            Ok(())
        }
        "add-layer" => {
            let (file, layer) = (
                args.first().ok_or("add-layer needs a project file")?,
                args.get(1).ok_or("add-layer needs a layer file")?,
            );
            let count =
                eh3d_suite_launcher::project::add_layer_to_file(Path::new(file), Path::new(layer))?;
            println!("added {layer} ({count} files in the project)");
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
                let unchecked = p.unchecked();
                if unchecked.is_empty() {
                    println!(
                        "{}: {} files, all there and unchanged",
                        p.name,
                        p.layers.len()
                    );
                } else {
                    // a layer with no checksum cannot be said to be unchanged
                    println!(
                        "{}: {} files, all there; {} without a checksum, not verified: {}",
                        p.name,
                        p.layers.len(),
                        unchecked.len(),
                        unchecked.join(", ")
                    );
                }
                Ok(())
            } else {
                Err(problems.join("\n"))
            }
        }
        "recent" => {
            let path = data_dir().join("recent.txt");
            let list = if args.first().map(String::as_str) == Some("add") {
                let file = args.get(1).ok_or("recent add needs a file")?;
                recent::add(&path, file)?
            } else {
                recent::load(&path)
            };
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
