# EH3D-SUITE-LAUNCHER: status

## Current state

EH3D-SUITE-LAUNCHER is a working program on a computer: a library (`eh3d_suite_launcher`), a command line (`eh3d-launcher`) and a window (`eh3d-launcher-gui`, built with `--features gui`). It reads the manifests of the repositories, starts programs with a project and keeps the projects, and a seeded campaign of crashes checks that the restart policy holds. The suite of tests runs on Windows and Linux in the CI. The window has not been looked at on a screen, and no real workshop has used it.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 28 requirements: 20 implemented, 3 prototype, 5 planned and 0 validated. By progress: 20 done, 3 partial, 1 pending and 4 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* How the machines of the workshop should be discovered on the network and what the launcher shows about them.
* Whether the launcher should call the updater from its window.
* Whether the project should become an archive that contains its files.

## What it needs from other repositories

* The `eh3d.project.json` manifest of every program of the suite, to know what exists.
* The updater of the suite, for the call from the window.
* A workshop with the programs built, for the only validation that counts.

## Known technical debt and omitted tests

* The window is written and its logic is tested, but it has not been looked at on a screen.
* The `.eh3dlist` is a JSON file that points at the files; it is not an archive.
* Tests that need a workshop or a screen are not written: the register names each as `planned:` with the test that will show it.
