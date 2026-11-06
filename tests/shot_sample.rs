//! The samples of the repository are what the pictures of the window show: the folder of programs and the project list must load in the window of a picture, which reads and
//! writes nothing of the person's files.

use eh3d_suite_launcher::lang::Lang;
use eh3d_suite_launcher::settings::Settings;
use eh3d_suite_launcher::window::{Status, Window};
use std::path::PathBuf;

fn sample(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    for part in parts {
        path.push(part);
    }
    path
}

#[test]
fn the_window_of_a_picture_shows_the_sample_programs_and_project() {
    let w = Window::for_shot(
        Some(sample(&["workspace"])),
        Some(sample(&["demo", "demo.eh3dlist"])),
        Lang::Fr,
    );
    assert_eq!(w.lang, Lang::Fr);
    assert_eq!(w.settings, Settings::default());
    assert_eq!(w.programs.len(), 8);
    assert!(w.skipped.is_empty(), "{:?}", w.skipped);
    assert!(w.workspace_error.is_none());
    let open = w.project.as_ref().expect("the sample project opens");
    assert_eq!(open.project.name, "Demo board");
    assert_eq!(open.project.layers.len(), 3);
    assert!(matches!(w.status, Status::Opened { files: 3, .. }));
    assert_eq!(w.recent.len(), 1);
}

#[test]
fn the_files_of_the_sample_project_match_their_checksums() {
    let w = Window::for_shot(None, Some(sample(&["demo", "demo.eh3dlist"])), Lang::En);
    let open = w.project.as_ref().expect("the sample project opens");
    assert_eq!(
        open.project.verify(&sample(&["demo"])),
        Vec::<String>::new()
    );
}

#[test]
fn without_a_folder_the_window_of_a_picture_says_so_and_reads_no_configuration() {
    let w = Window::for_shot(None, None, Lang::En);
    assert!(w.programs.is_empty());
    assert!(w.workspace.is_none());
    assert!(w.workspace_error.is_some());
    assert!(w.recent.is_empty());
    assert!(w.project.is_none());
}

#[test]
fn a_project_that_is_not_there_is_a_failure_and_not_a_recent_project() {
    let w = Window::for_shot(None, Some(sample(&["demo", "missing.eh3dlist"])), Lang::En);
    assert!(w.project.is_none());
    assert!(w.recent.is_empty());
    assert!(matches!(w.status, Status::Failed(_)));
}
