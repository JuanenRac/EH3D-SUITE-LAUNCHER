<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-SUITE-LAUNCHER banner" width="100%">
</p>

# 📖 EH3D-SUITE-LAUNCHER

<p align="center">🇺🇸 <b>English</b> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### The hub of the EH3D Fabrication Suite: finds the programs, starts them and keeps the project file

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Honesty check - what actually runs today:** a Rust library and a command line program (`eh3d-launcher`) with 113 tests that run on a computer: finds the programs of the suite by their manifests, starts them and starts them again when they fail, keeps the recent projects and writes and checks the project file with the SHA-256 of every file. A window (`eh3d-launcher-gui`, built with the `gui` feature) is written and its logic is tested, but it has not been built or looked at on a screen; nothing has been tried on a machine and nothing has been checked against a collection of real files.

---

> **Safety and security status:** this program does not control any machine and is not a safety system. What it produces must be checked by a person before anything is run on a machine; nothing here has run on real hardware.


## 📋 At a glance

| | |
|---|---|
| Version | `0.0.5` |
| Maturity | Scaffolding |
| Tests | 113 |
| Stack | Rust |
| On real hardware | Not tried: nothing here has run on a machine |

**Jump to:** [Overview](#-overview) · [Architecture](#-architecture) · [Quick start](#-quick-start) · [Testing](#-testing) · [Structure](#-repository-structure) · [Related](#-related-projects)

## 🚦 Project status

Each line is one part of this repository and the light says how far it is today. It is written from what the repository can show (tests, builds, runs), not from what is planned.

🟢 Done and verified · 🟡 Works, with known limits · 🟠 Partly done or not verified yet · 🔴 Not done or not working

| Area | State | What it means here |
|---|---|---|
| Core | 🟡 | finds the programs of the suite by their manifests, starts them and starts them again when they fail, keeps the recent projects and writes and checks the project file with the SHA-256 of every file; tested on a computer with normal and damaged input, not with a collection of real files. |
| Command line | 🟡 | `eh3d-launcher` runs the same functions as the library and prints the result; every error comes back as a message and a non-zero exit code. |
| Tests | 🟡 | 113 tests pass (`cargo test`), with normal cases, edge cases and damaged input. |
| Security | 🟡 | Plain safe Rust (`forbid(unsafe_code)`): nothing panics on the files it is given and a bad value is an error with its reason; no independent review. |
| Window | 🟠 | Written (egui, `--features gui`, program `eh3d-launcher-gui`): the recent projects, open or create a project, add and remove its files with the checks of the command line, and the programs of the suite with a button to start each one; its logic is tested, but it has not been built or looked at on a screen yet. |
| Real hardware | 🔴 | Not tried: nothing here has been run on a machine. |
| Documentation | 🟡 | README in eight languages, the changelog and the two pictures drawn from `docs/`. |

## 🎯 Overview

* **The registry** (`src/registry.rs`): every repository of the suite says what it is in its `eh3d.project.json`; the launcher reads them, skips the damaged ones with a reason and tells which programs are built.
* **The launcher** (`src/launch.rs`): starts a program of the suite and, if it ends with an error, starts it again up to two times with a waiting time that doubles; a program that ends well is left alone.
* **The project file** (`src/project.rs`, `.eh3dlist`): the name, the files of the job with their kind and SHA-256, and the work zero; reading refuses paths that leave the project and migrates the first format. **Recent projects** (`src/recent.rs`) keeps the last ten.
* **Finding and restarting:** the binary of a program is found by the `[[bin]]` name of its `Cargo.toml` (`eh3d-camdebug`, `eh3d-pnp`...), and a program the operator ended on purpose (a signal) or that refused its arguments (exit code 1 or 2) is not started again: only a crash is.
* **The window** (`src/window.rs`, `src/gui.rs`; `--features gui`, program `eh3d-launcher-gui`): the recent projects, open or create a project, add and remove its files with the same checks as `add-layer`, and the programs of the suite with a button to start each one with the project. The state, the actions, the settings and the words are plain code with tests; the drawing is thin. Written, not seen on a screen yet.

## 🧩 Role in EH3D

EH3D-SUITE-LAUNCHER is one program of the EH3D Fabrication Suite. It works as a library, a command line and, built with the `gui` feature, a window (`eh3d-launcher-gui`) in the look the other programs of the suite share. It reads and writes plain files, so it can be used on its own or between the other programs.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-SUITE-LAUNCHER architecture: who uses it, what it holds and what it depends on" width="100%">
</p>


How the parts of this repository fit together: who uses it, what it holds and what it depends on. The picture is drawn from `docs/architecture.json` and checked against the code.

## ✨ Features

* **A library and a command line:** every command of `eh3d-launcher` is a function of the library, so another program of the suite can call it without starting a process.
* **Tests with damaged input:** the tests also feed empty files, garbage, huge numbers and cut files, and none of them may panic.
* **No guessing:** a value that does not make sense is an error that says which value and why, never a result that only looks right.
* **No unsafe code:** `#![forbid(unsafe_code)]` in the library and the program.
* **Safer lists and projects:** a layer path may not be `.`, a name that Windows refuses (`CON`, `AUX`, `COM1`...) or end in a dot or a space, two layers that differ only in case are one, the recent list keeps the full path of an existing file and two runs at the same time (`recent`, `add-layer`) do not lose each other's change because the file is locked while it is changed, a layer is hashed in blocks of 64 KiB, and a program ended by a signal is told as such and not as a failure.

## 🔒 Security & Safety

| Threat | Defense |
|---|---|
| A damaged, truncated or empty file | no panic: the tests cut valid input at every position and feed garbage |
| A number that makes no sense (negative, not finite, absurdly large) | refused with an error that names the value; nothing is rounded into something plausible |
| Unsafe memory use | plain safe Rust with `forbid(unsafe_code)` |

**What is not defended:** a result that was produced cleanly is not a verified result. Nothing here knows the limits of a machine, and the program has had no independent review. Report a vulnerability privately (see [SECURITY.md](SECURITY.md)).

## 🔌 Interfaces

The library is `eh3d_suite_launcher` (`use eh3d_suite_launcher::...`); the command line is `eh3d-launcher`. The commands are these; each prints its result on the standard output and reports an error on the standard error with a non-zero exit code. There is no network access.

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                the list of files of the window that opens a file
│   ├── gui.rs                                   the window (egui, with the gui feature): draws the state and passes on what the person does
│   ├── gui_main.rs                              the window as a program of its own, without a console on Windows
│   ├── lang.rs                                  the words of the window in the eight languages
│   ├── launch.rs                                a part of the logic
│   ├── lib.rs                                   the library: what every command uses
│   ├── main.rs                                  the command line
│   ├── project.rs                               a part of the logic
│   ├── recent.rs                                a part of the logic
│   ├── registry.rs                              a part of the logic
│   ├── settings.rs                              the settings of launcher.cfg: language, restarts and window size
│   ├── shot.rs                                  the picture mode of the window (--shot): its arguments, read and checked without a screen
│   └── window.rs                                the state of the window, no screen needed: programs, projects, files and launching
├── tests/
│   ├── launcher.rs                              tests of the library and the command line
│   └── shot_sample.rs                           tests of the samples in the window of a picture
├── examples/
│   ├── workspace/                               sample manifests of programs of the suite, to show in the window of a picture
│   └── demo/                                    a sample project list with three files and their checksums
├── docs/
│   ├── architecture.json                        data of the Architecture picture
│   └── structure.json                           data of the Repository Structure picture
├── tools/                                       CI check, README check and the picture tools
├── images/                                      banner and the Architecture and Repository Structure pictures
├── .github/                                     CI and release workflows, issue and pull-request templates
├── eh3d.project.json                            manifest: version, role, maturity and build command
├── Cargo.toml Cargo.lock                        the workspace and its locked dependencies
├── rust-toolchain.toml                          the pinned Rust toolchain
├── CHANGELOG.md                                 what changed in each version
├── SECURITY.md SUPPORT.md                       how to report a vulnerability and where to ask
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   how to contribute, the code of conduct and the licence
└── launcher.cfg.example                         every setting of launcher.cfg with its default
```

## ⚙️ Configuration

The configuration file is `launcher.cfg` in the folder of the launcher, read by `launch` and by the window: its line `workspace = FULL_PATH` is the folder the programs are started from (only the known programs of the suite are started), and the optional lines `language`, `max_restarts`, `backoff_ms`, `window_width` and `window_height` set the language of the window, how many times a program that crashed is started again, the waiting time and the size of the window. `launcher.cfg.example` lists every key with its default and its limits. Apart from it, what a person chooses is given as arguments of the command line (see Interfaces) and every default is written in the help of the command (`eh3d-launcher help`). The limits of a value are named constants of the code and are covered by tests.

## 🚀 Quick start

You need a stable Rust toolchain (1.95 or newer).

```bash
cargo build --release
cargo test
cargo run --release -- list --workspace ..
# the window (needs the gui feature)
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-launcher-gui
cargo run --release --features gui --bin eh3d-launcher-gui -- --shot screen.png --open examples/workspace --open examples/demo/demo.eh3dlist   # a picture of the window
```

To take a picture of the window without opening it by hand, run it with `--shot FILE.png` (also `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`): it opens at that size, saves the PNG, keeps no settings and closes by itself (exit code 0 when the file was written, 1 when it was not, 2 for wrong arguments).

## 🧪 Testing

The suite has 113 tests (`cargo test`, run on Windows and Linux in the CI). They check the normal cases, the edge cases and damaged input: empty files, garbage and cut files never panic. The CI also runs `cargo fmt --check`, `cargo clippy` (also with `--features gui`) with warnings as errors and `python -I tools/ci_validate.py`.

## 🏗️ Build & Release

The version lives in `Cargo.toml` and in `eh3d.project.json`; each version has its entry in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` build the program, `test.bat` / `test.sh` run the tests and `run.bat` / `run.sh` run it with a sample. Releases are published on GitHub.

### Downloads

Each release has the command line program and the window for Windows (64 and 32 bits), Linux (64 bits) and macOS (Apple silicon and Intel), as `<name>-<version>-<system>.zip` (Windows) or `.tar.gz` with the two programs, the licence, the readme, the icon and `launcher.cfg.example`. The files are built by GitHub Actions and listed in `SHA256SUMS`. To check a download, compare its hash with the `.sha256` file next to it (`sha256sum -c`).

## 📂 Repository Structure

<p align="center">
  <img src="images/repository-structure.svg" alt="Repository structure of EH3D-SUITE-LAUNCHER" width="100%">
</p>

<details>
<summary>Text version of the tree</summary>

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                the list of files of the window that opens a file
│   ├── gui.rs                                   the window (egui, with the gui feature): draws the state and passes on what the person does
│   ├── gui_main.rs                              the window as a program of its own, without a console on Windows
│   ├── lang.rs                                  the words of the window in the eight languages
│   ├── launch.rs                                a part of the logic
│   ├── lib.rs                                   the library: what every command uses
│   ├── main.rs                                  the command line
│   ├── project.rs                               a part of the logic
│   ├── recent.rs                                a part of the logic
│   ├── registry.rs                              a part of the logic
│   ├── settings.rs                              the settings of launcher.cfg: language, restarts and window size
│   ├── shot.rs                                  the picture mode of the window (--shot): its arguments, read and checked without a screen
│   └── window.rs                                the state of the window, no screen needed: programs, projects, files and launching
├── tests/
│   ├── launcher.rs                              tests of the library and the command line
│   └── shot_sample.rs                           tests of the samples in the window of a picture
├── examples/
│   ├── workspace/                               sample manifests of programs of the suite, to show in the window of a picture
│   └── demo/                                    a sample project list with three files and their checksums
├── docs/
│   ├── architecture.json                        data of the Architecture picture
│   └── structure.json                           data of the Repository Structure picture
├── tools/                                       CI check, README check and the picture tools
├── images/                                      banner and the Architecture and Repository Structure pictures
├── .github/                                     CI and release workflows, issue and pull-request templates
├── eh3d.project.json                            manifest: version, role, maturity and build command
├── Cargo.toml Cargo.lock                        the workspace and its locked dependencies
├── rust-toolchain.toml                          the pinned Rust toolchain
├── CHANGELOG.md                                 what changed in each version
├── SECURITY.md SUPPORT.md                       how to report a vulnerability and where to ask
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   how to contribute, the code of conduct and the licence
└── launcher.cfg.example                         every setting of launcher.cfg with its default
```

</details>

## 🗺️ Roadmap & Known limits

**Known limits today**

* The window is written and its logic is tested, but it has not been built or looked at on a screen yet.
* It has not been tried on a machine nor with a collection of real files, and it has had no independent review.
* It does not discover machines on the network yet, does not keep the suite up to date (EH3D-UPDATER does) and the `.eh3dlist` is a JSON file that points at the files, not an archive that contains them.

**Next**

* Discover the machines of the workshop on the network and call EH3D-UPDATER from the window.
* Look at the window on a screen of each system and adjust what is found.
* Try it with real files and with the other programs of the suite.

## 🌍 Languages

README and texts exist in English, Spanish, French, Italian, German, Chinese, Japanese and European Portuguese.

## 🔗 Related Projects

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): the project container and the shared types of the suite
* [EH3D-UPDATER](https://github.com/JuanenRac/EH3D-UPDATER): the updater of the suite
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): the drilling program
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): the contour milling program
* [EH3D-BUNGARD-GUI](https://github.com/JuanenRac/EH3D-BUNGARD-GUI): the window of the Bungard machines
* [EH3D-PHOENIX-CNC](https://github.com/JuanenRac/EH3D-PHOENIX-CNC): the machine control program

## 📚 Documentation

* [Changelog](CHANGELOG.md)
* [Architecture and structure data](docs/architecture.json): the data the two pictures are drawn from.

## 🤝 Contributing & Community

* Before a pull request run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `python -I tools/ci_validate.py`: all must pass. See [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md).
* The README exists in eight languages: a change to the text is made in all eight files, and the two pictures are drawn from `docs/architecture.json` and `docs/structure.json` by `tools/make_architecture.py` and `tools/make_structure.py`.
* Report a vulnerability privately: [SECURITY.md](SECURITY.md). Questions, ideas and reports: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTHOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENSE

Software: GPL-3.0-or-later. See [LICENSE](LICENSE). Hardware designs of the suite use CERN OHL-S.
