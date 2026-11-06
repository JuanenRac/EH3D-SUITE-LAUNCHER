# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- Update the SHA-256 library to its newer version.

## [0.0.5] - The window with recent projects, layers and the launcher of the suite programs

- **A window:** built with the `gui` feature (egui, the look the other programs of the suite share), program `eh3d-launcher-gui`, which opens without a console on Windows. It lists the recent projects, opens or creates a project, adds and removes its files with the same checks as the command line, verifies the files, lists the programs of the suite and starts each one (with the file of the project as its argument when the person wants), in the eight languages. A program runs on its own thread, so the window keeps answering, and every error is the message of the library.
- **Settings in `launcher.cfg`:** `language`, `max_restarts`, `backoff_ms`, `window_width` and `window_height` (limits checked, a bad line is an error that names it); `launcher.cfg.example` lists every key with its default. The command line reads the restart settings too.
- **One implementation for the command line and the window:** creating a project, adding and removing a file, finding a program by name and the checks before a program is started are functions of the library that both use; the state and the actions of the window live in a module with tests and the screen only draws them.
- **Release binaries workflow:** builds the command line and the window for Windows (64 and 32 bits), Linux and macOS, with their hashes and attestations; the CI also runs the lints with the `gui` feature.
- **The validation job is the shared one of the documentation hub:** the checks of the manifest, the documents and the pictures run through the reusable workflow, fixed to a full commit of EH3D-DOCS whose actions are pinned by commit.
- **A layer path is checked harder:** `.`, a name that Windows refuses (`CON`, `AUX`, `COM1`...) and a part that ends in a dot or a space are refused, and two layers that differ only in case are one.
- **The recent list keeps full paths and `recent add` needs the file to exist:** the same file is not listed twice under two spellings.
- **Two runs at the same time do not lose a change:** `recent` and `add-layer` hold a lock file while they read, change and write.
- **A layer is hashed in blocks of 64 KiB** instead of being read whole.
- **A program ended by a signal is not a failure:** `launch` says it was ended by a signal and does not report that it failed.
- **The commands of the README are run by a test.**

## [0.0.4] - Audit fixes, the new reading limits, tests and the application icon

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **Safer files and options:** the project list and the recent list are written through a temporary file, a manifest with a byte order mark is read, and `--workspace` after `--` is left to the program that is started.
- **`launch` starts only what it should:** it runs only the known programs of the suite, from the folder named by `workspace` in `launcher.cfg` (not the current directory), and refuses a binary name with separators or dots; the refusal is written in the eight languages.
- **`verify` is honest:** a layer with no checksum is reported as not verified instead of "unchanged".
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.
- **More tests:** how files are told apart, the versions that are compared, the list of recent projects, and the project files that are damaged or hold files outside their folder.

## [0.0.3] - Reusable validation and CI hardening

- **Faster, bounded CI:** the Cargo build is cached, a run that a newer push supersedes is cancelled, every job has a time limit, and the Rust jobs live in their own workflow that is skipped when only documents or images change.
- **A fixed Rust toolchain:** `rust-toolchain.toml` names the compiler version (with rustfmt and clippy) that the CI installs, so a new stable release cannot turn the lints red without a change here.
- **The shared validation job:** the repository calls the reusable validation workflow of the documentation hub instead of carrying its own copy of the steps.
- **Monthly dependency updates:** Dependabot opens one grouped pull request a month for the GitHub Actions and the Cargo dependencies.
- **A line for every change:** a pull request that changes code must add a line under `[Unreleased]` of this file (`tools/changelog_pr_check.py`; the label `no-changelog` skips it).
- **Shared checks updated:** the validator of the suite checks the text of the banner against the manifest and reads the files that git tracks.
- **The banner of the README** shows the role and the technologies of this program (it showed the text of the shared library).
- **More tests:** the project list is checked with paths that climb out of the folder, file names with two dots, a byte order mark and the first format; the manifests, the list of recent projects, the restarts of a program that crashes and every command of the command line. A path in the first format of the project list is now checked like any other, and a file name with two dots no longer makes a saved project unreadable, a byte order mark is skipped, and a name of a program in Cargo.toml followed by a comment is read.

## [0.0.2] - Projects in .eh3dlist, programs found by the name of their binary and no restart of a program that was ended on purpose

- **The project list is `.eh3dlist`:** the file this program writes no longer shares the extension `.eh3dproj` with the archive of EH3D-COMMON. `new-project` asks for `.eh3dlist`, the old files are still read, and an archive of EH3D-COMMON given by mistake is told apart with a clear message.

- **The programs are found by the name of their binary:** the `[[bin]]` names of the `Cargo.toml` of each repository are tried first, so `eh3d-camdebug`, `eh3d-isocam`, `eh3d-pnp` and the rest are found and started.
- **A program the operator ended on purpose is not started again:** only a crash is repeated, not a signal or an exit code 1 or 2.

## [0.0.1] - The programs of the suite found by their manifests, started and kept alive, and the project file

- **The registry:** the EH3D repositories of a folder are read by their `eh3d.project.json`; a damaged or foreign manifest is reported with its reason and does not stop the rest; a built program is found in `target/release` before `target/debug`.
- **The launcher:** a program that fails is started again up to two times with a doubling wait; one that ends well runs once.
- **The project file:** name, files with kind and SHA-256, work zero; paths that leave the project are refused, the first format is migrated and a file that changed or went missing is reported by `verify`.
- **Recent projects:** the last ten, without repeats; a damaged list is an empty list, not an error.
- **The command line** `eh3d-launcher` with `list`, `launch`, `new-project`, `add-layer`, `verify` and `recent`.
- **15 tests** run on a computer, with damaged input.
