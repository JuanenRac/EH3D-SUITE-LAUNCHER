// EH3D-SUITE-LAUNCHER - the window, as a program of its own.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
//! `eh3d-launcher-gui` opens the window. It is built with `--features gui`. On Windows it is a program of the graphical subsystem, so no console window opens behind it; for that
//! reason it prints nothing, and an error that keeps the window from opening is written to `last_error.txt` in the folder of the launcher. The command line (`eh3d-launcher`) keeps
//! its console, so its output and its exit codes work in a terminal and in a script.
#![forbid(unsafe_code)]
#![cfg_attr(windows, windows_subsystem = "windows")]

use eh3d_suite_launcher::launch::data_dir;
use eh3d_suite_launcher::shot;
use std::process::ExitCode;

/// The picture mode: `--shot FILE.png` opens the window with the default settings, shows the folder of programs and the project list given (`--open`), saves what it draws as a PNG
/// file and closes. The exit code is 0 when the file was written, 1 when it was not and 2 when the arguments are wrong; the reason is written on the error output, which a script
/// that redirects it can read.
fn take_picture(args: &[String]) -> ExitCode {
    let parsed = match shot::parse_window_args(args, 2) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let Some(request) = parsed.shot else {
        eprintln!("error: --shot needs a file to write");
        return ExitCode::from(2);
    };
    let lang = parsed
        .lang
        .unwrap_or_else(eh3d_suite_launcher::lang::Lang::from_env);
    match eh3d_suite_launcher::gui::run_shot(parsed.files, lang, request) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: the picture could not be taken: {e}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if shot::wants_picture(&args) {
        return take_picture(&args);
    }
    let dir = data_dir();
    match eh3d_suite_launcher::gui::run(dir.clone()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // there is no console to print to: the reason goes to a file the person can open
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                dir.join("last_error.txt"),
                format!("the window could not open: {e}\n"),
            );
            ExitCode::from(1)
        }
    }
}
