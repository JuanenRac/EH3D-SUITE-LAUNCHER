//! The size limit of the files the launcher reads: a manifest or a list of recent projects within `max_file_mib` is read, a bigger one is refused with a code and its name.

use eh3d_suite_launcher::limit::{read_text_limited, NOT_TEXT, TOO_LARGE, UNREADABLE};
use eh3d_suite_launcher::registry::{discover_with_limit, read_manifest_with_limit};
use eh3d_suite_launcher::{limit, recent, settings::Settings};
use std::fs;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("eh3d_launcher_limit_{}_{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

const MANIFEST: &str = r#"{"ecosystem":"EH3D","name":"EH3D-DRILLPRO","version":"0.1.2","role":"cnc","maturity":"alpha","deployment_target":"desktop"}"#;

#[test]
fn a_file_within_the_limit_is_read_and_a_bigger_one_is_refused_with_its_code() {
    let dir = scratch("size");
    let file = dir.join("a.txt");
    fs::write(&file, "twelve bytes").unwrap();
    assert_eq!(read_text_limited(&file, 12).unwrap(), "twelve bytes");
    let why = read_text_limited(&file, 11).unwrap_err();
    assert!(
        why.starts_with("file_too_large: ") && why.contains("a.txt") && why.contains("11 bytes"),
        "{why}"
    );
    assert!(why.starts_with(TOO_LARGE));
    assert_eq!(limit::mib(2), 2_097_152);
    assert_eq!(limit::mib(u64::MAX), u64::MAX);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_file_and_a_file_that_is_not_text_have_their_own_codes() {
    let dir = scratch("codes");
    let why = read_text_limited(&dir.join("nothing.json"), 10).unwrap_err();
    assert!(
        why.starts_with(UNREADABLE) && why.contains("nothing.json"),
        "{why}"
    );
    let file = dir.join("bin.json");
    fs::write(&file, [0xff, 0xfe]).unwrap();
    let why = read_text_limited(&file, 10).unwrap_err();
    assert!(
        why.starts_with(NOT_TEXT) && why.contains("bin.json"),
        "{why}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_manifest_over_the_limit_is_refused_and_the_others_are_still_found() {
    let ws = scratch("manifests");
    let good = ws.join("EH3D-DRILLPRO");
    let big = ws.join("EH3D-BIG");
    fs::create_dir_all(&good).unwrap();
    fs::create_dir_all(&big).unwrap();
    fs::write(good.join("eh3d.project.json"), MANIFEST).unwrap();
    let padded = format!("{MANIFEST}{}", " ".repeat(200));
    fs::write(big.join("eh3d.project.json"), &padded).unwrap();
    let limit_bytes = MANIFEST.len() as u64;
    assert_eq!(
        read_manifest_with_limit(&good, limit_bytes).unwrap().name,
        "EH3D-DRILLPRO"
    );
    let why = read_manifest_with_limit(&big, limit_bytes).unwrap_err();
    assert!(
        why.starts_with("file_too_large: ") && why.contains("eh3d.project.json"),
        "{why}"
    );
    let (modules, problems) = discover_with_limit(&ws, limit_bytes);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "EH3D-DRILLPRO");
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("file_too_large: "),
        "{}",
        problems[0]
    );
    // with room for both, both are found
    let (modules, problems) = discover_with_limit(&ws, padded.len() as u64);
    assert_eq!((modules.len(), problems.len()), (2, 0));
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn the_list_of_recent_projects_over_the_limit_is_an_empty_list_or_an_error_with_its_code() {
    let dir = scratch("recent");
    let file = dir.join("recent.txt");
    fs::write(&file, "a.eh3dlist\nb.eh3dlist\n").unwrap();
    assert_eq!(
        recent::load_with_limit(&file, 22),
        ["a.eh3dlist", "b.eh3dlist"]
    );
    assert!(recent::load_with_limit(&file, 21).is_empty());
    let why = recent::load_checked(&file, 21).unwrap_err();
    assert!(
        why.starts_with("file_too_large: ") && why.contains("recent.txt"),
        "{why}"
    );
    assert_eq!(recent::load_checked(&file, 22).unwrap().len(), 2);
    // a list that is not there is an empty list, not an error
    assert_eq!(
        recent::load_checked(&dir.join("none.txt"), 22).unwrap(),
        Vec::<String>::new()
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_limit_of_the_settings_is_read_with_its_range() {
    assert_eq!(Settings::default().max_file_mib, 16);
    assert_eq!(Settings::default().max_file_bytes(), 16_777_216);
    assert_eq!(
        Settings::parse("max_file_mib = 1")
            .unwrap()
            .max_file_bytes(),
        1_048_576
    );
    assert_eq!(
        Settings::parse("max_file_mib = 1024").unwrap().max_file_mib,
        1024
    );
    for bad in [
        "max_file_mib = 0",
        "max_file_mib = 1025",
        "max_file_mib = many",
    ] {
        let e = Settings::parse(bad).unwrap_err();
        assert!(e.contains("max_file_mib") && e.contains("1 to 1024"), "{e}");
    }
}

#[test]
fn the_sample_file_has_the_default_values() {
    let sample = include_str!("../launcher.cfg.example");
    assert_eq!(Settings::parse(sample).unwrap(), Settings::default());
    assert!(sample.contains("max_file_mib = 16"));
}
