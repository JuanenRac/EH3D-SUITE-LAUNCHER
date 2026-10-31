# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.0.1] - The programs of the suite found by their manifests, started and kept alive, and the project file

- **The registry:** the EH3D repositories of a folder are read by their `eh3d.project.json`; a damaged or foreign manifest is reported with its reason and does not stop the rest; a built program is found in `target/release` before `target/debug`.
- **The launcher:** a program that fails is started again up to two times with a doubling wait; one that ends well runs once.
- **The project file:** name, files with kind and SHA-256, work zero; paths that leave the project are refused, the first format is migrated and a file that changed or went missing is reported by `verify`.
- **Recent projects:** the last ten, without repeats; a damaged list is an empty list, not an error.
- **The command line** `eh3d-launcher` with `list`, `launch`, `new-project`, `add-layer`, `verify` and `recent`.
- **15 tests** run on a computer, with damaged input.
