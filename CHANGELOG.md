# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

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
