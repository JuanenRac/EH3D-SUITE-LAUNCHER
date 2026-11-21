# EH3D-SUITE-LAUNCHER: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with the real programs of the suite on a workshop machine.

The register has 28 requirements: 20 implemented, 3 prototype, 5 planned and 0 validated. By progress: 20 done, 3 partial, 1 pending and 4 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `SLCH-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a simulated program that crashes or ends well, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The registry

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| SLCH-001 | The suite is found by the manifests, and a broken manifest is reported and not fatal. | implemented | done | host-test | `launcher::the_suite_is_found_by_the_manifests_and_a_broken_one_is_reported_not_fatal` | - |
| SLCH-002 | A manifest with a bad version or a missing field says which. | implemented | done | host-test | `launcher::a_manifest_with_a_bad_version_or_a_missing_field_says_which` | - |
| SLCH-003 | The name of a program is the repository without the prefix in lower case. | implemented | done | host-test | `launcher::the_program_name_is_the_repository_without_the_prefix_in_lowercase` | - |
| SLCH-004 | A built program is found in release before debug. | implemented | done | host-test | `launcher::a_built_program_is_found_in_release_before_debug` | - |
| SLCH-005 | Versions are compared number by number and not by their text. | implemented | done | host-test | `more::a_version_is_newer_by_its_numbers_and_not_by_its_text` | - |
| SLCH-006 | A manifest over the size limit is refused and the others are still found. | implemented | done | host-test | `read_limit::a_manifest_over_the_limit_is_refused_and_the_others_are_still_found` | - |

## Starting and restarting

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| SLCH-007 | A program that ends well is run once and nothing is waited. | implemented | done | host-test | `src/launch::a_program_that_ends_well_is_run_once_and_nothing_is_waited` | - |
| SLCH-008 | A crash is started again after the first wait, and the recovery is not a failure. | implemented | done | host-test | `src/launch::a_crash_is_started_again_after_the_first_wait_and_the_recovery_is_not_a_failure` | - |
| SLCH-009 | A program that keeps crashing gives up after its restarts with the wait doubled. | implemented | done | host-test | `src/launch::a_program_that_keeps_crashing_gives_up_after_its_restarts_with_the_wait_doubled` | - |
| SLCH-010 | An error of use and a program ended by a signal are never started again. | implemented | done | host-test | `src/launch::an_error_of_use_is_never_started_again` | - |
| SLCH-011 | Zero restarts means never, and the wait stops growing at thirty seconds. | implemented | done | host-test | `src/launch::zero_restarts_means_never_and_the_wait_stops_growing_at_thirty_seconds` | - |
| SLCH-012 | A seeded campaign of crashes follows the policy for every seed and repeats exactly. | prototype | partial | simulator | `src/launch::a_seeded_campaign_follows_the_policy_for_every_seed_and_repeats_exactly` | A simulated program that crashes; no program of the suite has been run under it in a workshop. |

## Projects

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| SLCH-013 | A project records its layers with a checksum and notices a change. | implemented | done | host-test | `launcher::a_project_records_its_layers_with_a_checksum_and_notices_a_change` | - |
| SLCH-014 | A file outside the project folder is refused. | implemented | done | host-test | `launcher::a_file_outside_the_project_folder_is_refused` | - |
| SLCH-015 | The first format with a plain list of paths is migrated. | implemented | done | host-test | `launcher::the_first_format_with_a_plain_list_of_paths_is_migrated` | - |
| SLCH-016 | The project file is read with care. | implemented | done | host-test | `launcher::the_project_file_is_read_with_care` | - |
| SLCH-017 | The recent list drops repeats and keeps the order, and is cut to the limit. | implemented | done | host-test | `more::the_recent_list_is_cut_to_the_limit_when_read_and_when_a_project_is_remembered` | - |
| SLCH-018 | The list of recent projects over the limit is an empty list or an error with its code. | implemented | done | host-test | `read_limit::the_list_of_recent_projects_over_the_limit_is_an_empty_list_or_an_error_with_its_code` | - |

## Window

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| SLCH-019 | The window finds the programs and marks the built ones. | prototype | partial | host-test | `src/window::the_window_finds_the_programs_and_marks_the_built_ones` | The logic is tested without a screen; no person has looked at the drawing. |
| SLCH-020 | A program is launched with the project and only once at a time. | prototype | partial | host-test | `src/window::a_program_is_launched_with_the_project_and_only_once_at_a_time` | The logic is tested without a screen; no person has looked at the drawing. |
| SLCH-021 | Files are added and removed with the checks of the command line. | implemented | done | host-test | `src/window::files_are_added_and_removed_with_the_checks_of_the_command_line` | - |
| SLCH-022 | A new project is written, opened and remembered. | implemented | done | host-test | `src/window::a_new_project_is_written_opened_and_remembered` | - |
| SLCH-023 | Every sentence of the window has a text in each of the eight languages. | implemented | done | host-test | `src/lang::every_sentence_has_a_text_in_each_of_the_eight_languages` | - |

## Not written yet

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| SLCH-024 | The machines of the workshop are discovered on the network. | planned | blocked | none | `planned:discovery::the_machines_of_the_workshop_are_discovered` | Needs the machine-control programs to announce themselves. |
| SLCH-025 | The updater is called from the window. | planned | blocked | none | `planned:updater::the_window_calls_the_updater` | Needs the updater to be startable with a project. |
| SLCH-026 | A project is an archive that contains its files. | planned | pending | none | `planned:archive::a_project_travels_as_one_archive` | The project points at the files today. |
| SLCH-027 | The launcher runs the real programs of the suite in a workshop. | planned | blocked | none | `planned:workshop::the_real_programs_are_started_and_restarted_in_a_workshop` | Needs a workshop with the programs built. |
| SLCH-028 | The window is looked at on a screen of each system. | planned | blocked | none | `planned:window_screen::the_window_matches_the_reference_picture` | Needs a person at a screen. |
