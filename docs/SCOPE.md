# EH3D-SUITE-LAUNCHER: scope

Status: **host-test**. EH3D-SUITE-LAUNCHER is a library, a command line (`eh3d-launcher`) and a window (`eh3d-launcher-gui`) that find the programs of the suite, start them with a project and keep the list of projects. It has been tried on a computer only, with sample manifests and sample programs. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The hub of the suite: every repository says what it is in its `eh3d.project.json`; the launcher reads them, tells which programs are built, starts one of them with a project and starts it again after a crash. It also keeps the project file (`.eh3dlist`: the name, the files of a job with their kind and checksum and the work zero) and the list of recent projects.

## What it does

* Finds the repositories of the suite by their manifests, skips a damaged manifest with a reason instead of stopping, and compares versions number by number.
* Finds the binary of a program by the `[[bin]]` name of its `Cargo.toml`, in release before debug.
* Starts a program and, when it crashes, starts it again up to a set number of times with a wait that doubles and stops growing at thirty seconds; a program that ends well, ends by a signal or refuses its arguments is not started again.
* Writes and reads the project file with the name, the files of the job with their kind and SHA-256 and the work zero, refuses a path that leaves the project and migrates the first format.
* Keeps the last ten projects, without repeats, and notices a changed or missing file of a project when it verifies.
* Reads manifests and lists only within a size limit, and shows all of this in a window (built with `--features gui`) in eight languages.

## What it will not do

* No decision about a machine: starting a program does not move anything, and the limits, the interlocks and the emergency stop belong to the machine (see below).
* No discovery of the machines of the workshop on the network yet (see `NOT_YET.md`).
* No updating of the suite: that belongs to the updater.
* No archive of a project: the `.eh3dlist` points at the files and does not contain them.
* No claim that anything was proven on a workshop machine: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** the `eh3d.project.json` of the repositories next to it, `Cargo.toml` of a program for its binary name, the project files, the list of recent projects and its settings file (`launcher.cfg`).
* **It writes:** the project file and the list of recent projects, through a temporary file; it never writes to a manifest or to the files of a job.
* **It is read by:** nothing reads it: it starts the other programs of the suite, which exchange work through files or through the server and never depend on each other.
* **It depends on:** nothing of the suite at build time; at run time it looks for the programs built next to it.

## What it never decides

Whether a program is safe to start, and the limits, the interlocks and the emergency stop of any machine, belong to the machine and to the programs started, not to the launcher. Starting a program again after a crash is a convenience, not a safety function: a person decides whether a job that was running goes on. Nothing here replaces the emergency stop of a machine or the judgement of the person who stands at it.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
