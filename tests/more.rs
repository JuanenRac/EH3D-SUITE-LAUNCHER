//! More tests: how files are told apart, the versions that are compared, the list of recent projects, and the project files that are damaged or hold files outside their folder.

use eh3d_suite_launcher::project::{
    kind_of_path, Kind, Project, EXTENSION, MAX_NAME, SCHEMA_VERSION,
};
use eh3d_suite_launcher::recent::{load, remember, save, MAX_RECENT};
use eh3d_suite_launcher::registry::{is_newer, program_name, version_numbers};
use std::fs;
use std::path::{Path, PathBuf};

fn temp(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("eh3d-launcher-more-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_file_is_told_by_its_extension_in_any_case() {
    for (name, kind) in [
        ("a.GTL", Kind::Gerber),
        ("a.gm1", Kind::Gerber),
        ("a.drl", Kind::Drill),
        ("a.XLN", Kind::Drill),
        ("a.plt", Kind::Hpgl),
        ("a.nc", Kind::Gcode),
        ("a.tap", Kind::Gcode),
        ("a.pos", Kind::PickPlace),
        ("a.csv", Kind::PickPlace),
        ("a.PNG", Kind::Image),
        ("a.pgm", Kind::Image),
        ("a.txt", Kind::Other),
        ("noextension", Kind::Other),
        (".hidden", Kind::Other),
    ] {
        assert_eq!(kind_of_path(Path::new(name)), kind, "{name}");
    }
    // only the last extension counts
    assert_eq!(kind_of_path(Path::new("board.gtl.bak")), Kind::Other);
    assert_eq!(kind_of_path(Path::new("dir.gtl/readme")), Kind::Other);
}

#[test]
fn every_kind_has_a_key_that_reads_back_and_an_unknown_key_is_other() {
    for k in [
        Kind::Gerber,
        Kind::Drill,
        Kind::Hpgl,
        Kind::Gcode,
        Kind::PickPlace,
        Kind::Image,
        Kind::Other,
    ] {
        assert_eq!(Kind::from_key(k.key()), k);
    }
    assert_eq!(Kind::from_key(""), Kind::Other);
    assert_eq!(Kind::from_key("GERBER"), Kind::Other);
    assert_eq!(Kind::from_key("something new"), Kind::Other);
}

#[test]
fn the_name_of_a_program_is_the_repository_in_lower_case_without_the_prefix_and_only_the_first_one()
{
    assert_eq!(program_name("EH3D-DRILLPRO"), "drillpro");
    assert_eq!(program_name("EH3D-BUNGARD-GUI"), "bungard-gui");
    assert_eq!(program_name("OTHER-THING"), "other-thing");
    assert_eq!(program_name("EH3D-EH3D-X"), "eh3d-x");
    assert_eq!(program_name(""), "");
}

#[test]
fn a_version_is_a_list_of_numbers_and_anything_else_is_not_a_version() {
    assert_eq!(version_numbers("0.0.7"), Some(vec![0, 0, 7]));
    assert_eq!(version_numbers("1"), Some(vec![1]));
    assert_eq!(version_numbers("0.8.0.1"), Some(vec![0, 8, 0, 1]));
    for bad in [
        "", "a.b", "1.", ".1", "1..2", "1.2-rc1", "v1.0", "-1.0", "1.0 ",
    ] {
        assert_eq!(version_numbers(bad), None, "{bad:?}");
    }
}

#[test]
fn a_version_is_newer_by_its_numbers_and_not_by_its_text() {
    assert!(is_newer("0.0.10", "0.0.9"));
    assert!(!is_newer("0.0.9", "0.0.10"));
    assert!(is_newer("0.5.0", "0.4.9"));
    assert!(is_newer("1.0.0", "0.9.9.9"));
    // the same version, even with a component that is missing, is not newer
    assert!(!is_newer("1.0", "1.0.0"));
    assert!(!is_newer("1.0.0", "1.0"));
    assert!(is_newer("1.0.0.1", "1.0.0"));
    // four components, as the odometer from 0.8.0 has them
    assert!(is_newer("0.8.0.1", "0.7.9"));
    assert!(is_newer("0.8.1.0", "0.8.0.9"));
}

#[test]
fn a_version_that_is_not_one_is_never_newer_and_nothing_is_newer_than_something_that_is_not_one() {
    assert!(!is_newer("abc", "1.0.0"));
    assert!(!is_newer("1.0.0", "abc"));
    assert!(!is_newer("", ""));
    assert!(!is_newer("2.0.0-rc1", "1.0.0"));
}

#[test]
fn a_file_of_recent_projects_that_is_missing_or_empty_is_an_empty_list() {
    let d = temp("recent-missing");
    assert!(load(&d.join("nothing.txt")).is_empty());
    fs::write(d.join("empty.txt"), "").unwrap();
    assert!(load(&d.join("empty.txt")).is_empty());
    fs::write(d.join("blank.txt"), "\n   \n\n").unwrap();
    assert!(load(&d.join("blank.txt")).is_empty());
}

#[test]
fn the_recent_list_drops_repeats_blank_lines_long_lines_and_lines_with_a_nul_and_keeps_the_order() {
    let d = temp("recent-clean");
    let long = "x".repeat(5000);
    fs::write(d.join("r.txt"), format!("a\n\n  b  \na\n{long}\nc\0d\nc\n")).unwrap();
    assert_eq!(load(&d.join("r.txt")), vec!["a", "b", "c"]);
}

#[test]
fn the_recent_list_is_cut_to_the_limit_when_read_and_when_a_project_is_remembered() {
    let d = temp("recent-limit");
    let many: Vec<String> = (0..MAX_RECENT + 5).map(|i| format!("p{i}")).collect();
    fs::write(d.join("r.txt"), many.join("\n")).unwrap();
    let list = load(&d.join("r.txt"));
    assert_eq!(list.len(), MAX_RECENT);
    assert_eq!(list[0], "p0");
    let mut list = list;
    remember(&mut list, "new");
    assert_eq!(list.len(), MAX_RECENT);
    assert_eq!(list[0], "new");
    assert!(!list.contains(&format!("p{}", MAX_RECENT - 1)));
}

#[test]
fn a_project_that_is_remembered_again_moves_to_the_head_and_is_not_listed_twice() {
    let mut list = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    remember(&mut list, "c");
    assert_eq!(list, ["c", "a", "b"]);
    remember(&mut list, "b");
    assert_eq!(list, ["b", "c", "a"]);
    let mut empty: Vec<String> = Vec::new();
    remember(&mut empty, "x");
    assert_eq!(empty, ["x"]);
}

#[test]
fn the_recent_list_is_saved_in_folders_that_did_not_exist_and_reads_back() {
    let d = temp("recent-save");
    let file = d.join("deep").join("er").join("recent.txt");
    let list = vec!["one".to_string(), "two".to_string()];
    save(&file, &list).unwrap();
    assert_eq!(load(&file), list);
    assert!(fs::read_to_string(&file).unwrap().ends_with('\n'));
}

#[test]
fn a_project_needs_a_name_of_a_sensible_length_without_control_characters() {
    assert!(Project::new("board").is_ok());
    assert!(Project::new(&"x".repeat(MAX_NAME)).is_ok());
    assert!(Project::new(&"x".repeat(MAX_NAME + 1)).is_err());
    assert!(Project::new("").is_err());
    assert!(Project::new("   ").is_err());
    assert!(Project::new("a\nb").is_err());
    assert_eq!(Project::new("  spaced  ").unwrap().name, "spaced");
}

#[test]
fn a_file_outside_the_folder_of_the_project_or_added_twice_is_refused() {
    let root = temp("project-root");
    let outside = temp("project-outside");
    fs::write(root.join("top.gtl"), "x").unwrap();
    fs::write(outside.join("far.gtl"), "x").unwrap();
    let mut p = Project::new("p").unwrap();
    p.add_layer(&root, &root.join("top.gtl")).unwrap();
    assert!(p
        .add_layer(&root, &root.join("top.gtl"))
        .unwrap_err()
        .contains("already"));
    // the same file by a relative path is the same file
    assert!(p
        .add_layer(&root, Path::new("top.gtl"))
        .unwrap_err()
        .contains("already"));
    assert!(p
        .add_layer(&root, &outside.join("far.gtl"))
        .unwrap_err()
        .contains("not under"));
    assert_eq!(p.layers.len(), 1);
    assert_eq!(p.layers[0].kind, Kind::Gerber);
    assert_eq!(p.layers[0].path, "top.gtl");
    assert_eq!(p.layers[0].sha256.len(), 64);
}

#[test]
fn a_file_that_does_not_exist_cannot_be_added() {
    let root = temp("project-missing");
    let mut p = Project::new("p").unwrap();
    assert!(p.add_layer(&root, &root.join("ghost.gtl")).is_err());
    assert!(p.layers.is_empty());
}

#[test]
fn a_project_is_saved_and_read_back_with_its_layers_and_the_json_of_a_newer_schema_is_refused() {
    let root = temp("project-roundtrip");
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.gtl"), "copper").unwrap();
    fs::write(root.join("sub").join("b.drl"), "holes").unwrap();
    let mut p = Project::new("round trip").unwrap();
    p.add_layer(&root, &root.join("a.gtl")).unwrap();
    p.add_layer(&root, &root.join("sub").join("b.drl")).unwrap();
    p.zero = (1.5, -2.25);
    let file = root.join(format!("job.{EXTENSION}"));
    p.save(&file).unwrap();
    let back = Project::load(&file).unwrap();
    assert_eq!(back, p);
    assert_eq!(back.layers[1].path, "sub/b.drl");
    assert!(p.verify(&root).is_empty());
    // a file that changed since the project was made is told
    fs::write(root.join("a.gtl"), "changed").unwrap();
    let told = p.verify(&root);
    assert_eq!(told.len(), 1);
    assert!(told[0].contains("a.gtl"), "{told:?}");
    fs::remove_file(root.join("sub").join("b.drl")).unwrap();
    assert_eq!(p.verify(&root).len(), 2);
    let newer = p.to_json().replace(
        &format!("\"schema_version\": {SCHEMA_VERSION}"),
        &format!("\"schema_version\": {}", SCHEMA_VERSION + 1),
    );
    if newer != p.to_json() {
        assert!(Project::from_json(&newer).is_err());
    }
}

#[test]
fn a_project_json_that_is_damaged_is_an_error_and_never_a_panic() {
    for text in [
        "",
        "{",
        "[]",
        "null",
        "{\"name\": 3}",
        "{\"schema_version\": \"x\"}",
        "\u{0}\u{0}",
    ] {
        assert!(Project::from_json(text).is_err(), "{text:?}");
    }
}

#[test]
fn the_layers_of_the_old_format_are_a_list_of_paths_and_a_path_that_leaves_the_project_is_refused()
{
    let old =
        Project::from_json("\u{feff}{\"name\": \"p\", \"layers\": [\"a.gtl\", \"sub/b.drl\"]}")
            .unwrap();
    assert_eq!(old.layers.len(), 2);
    assert_eq!(old.layers[0].kind, Kind::Gerber);
    assert_eq!(old.layers[1].kind, Kind::Drill);
    assert!(old.layers.iter().all(|l| l.sha256.is_empty()));
    assert_eq!(old.zero, (0.0, 0.0));
    for bad in ["../x.gtl", "/etc/passwd", "a/../../x"] {
        let text = format!("{{\"name\": \"p\", \"layers\": [\"{bad}\"]}}");
        assert!(Project::from_json(&text).is_err(), "{bad}");
    }
    assert!(
        Project::from_json("{\"name\": \"p\", \"layers\": [{\"kind\": \"gerber\"}]}")
            .unwrap_err()
            .contains("no path")
    );
}

#[test]
fn an_archive_of_the_common_container_is_told_apart_from_a_project_list_and_a_binary_file_is_not_text(
) {
    let d = temp("project-load");
    fs::write(d.join("zip.eh3dlist"), b"PK\x03\x04rest").unwrap();
    assert!(Project::load(&d.join("zip.eh3dlist"))
        .unwrap_err()
        .contains("archive"));
    fs::write(d.join("bin.eh3dlist"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    assert!(Project::load(&d.join("bin.eh3dlist"))
        .unwrap_err()
        .contains("not text"));
    assert!(Project::load(&d.join("nothing.eh3dlist")).is_err());
}

#[test]
fn a_layer_is_hashed_in_blocks_and_gives_the_same_digest_as_a_whole_read() {
    use sha2::{Digest, Sha256};
    let dir = temp("hash_blocks");
    let file = dir.join("big.gtl");
    // longer than one block of 64 KiB and not a multiple of it
    let bytes: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    fs::write(&file, &bytes).unwrap();
    let whole: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        eh3d_suite_launcher::project::sha256_hex(&file).unwrap(),
        whole
    );
}

#[test]
fn names_a_system_cannot_have_are_not_layers_and_a_layer_is_not_listed_twice_in_two_cases() {
    for bad in [
        "CON",
        "nul.txt",
        "sub/aux.gtl",
        "com1",
        "LPT9.drl",
        ".",
        "a/./b",
        "name.",
        "name ",
        "sub/x. ",
    ] {
        let json = format!(r#"{{"name":"p","layers":["{bad}"]}}"#);
        assert!(Project::from_json(&json).is_err(), "{bad} was accepted");
    }
    for good in ["console.gtl", "com10.gtl", "board..v2.gtl", "a/b.gtl"] {
        let json = format!(r#"{{"name":"p","layers":["{good}"]}}"#);
        assert!(Project::from_json(&json).is_ok(), "{good} was refused");
    }
    let p = Project::from_json(
        r#"{"name":"p","layers":["Top.GTL","sub/x.drl","top.gtl","SUB/X.DRL"]}"#,
    )
    .unwrap();
    let paths: Vec<&str> = p.layers.iter().map(|l| l.path.as_str()).collect();
    assert_eq!(paths, vec!["Top.GTL", "sub/x.drl"]);
}

#[test]
fn the_recent_list_keeps_the_full_path_of_an_existing_file_and_refuses_one_that_is_not_there() {
    let dir = temp("recent_canonical");
    let list = dir.join("recent.txt");
    let file = dir.join("a.eh3dlist");
    fs::write(&file, "{}").unwrap();
    let plain = file.to_string_lossy().into_owned();
    let dotted = dir
        .join(".")
        .join("a.eh3dlist")
        .to_string_lossy()
        .into_owned();
    eh3d_suite_launcher::recent::add(&list, &plain).unwrap();
    let after = eh3d_suite_launcher::recent::add(&list, &dotted).unwrap();
    assert_eq!(
        after.len(),
        1,
        "the same file under two spellings: {after:?}"
    );
    assert!(Path::new(&after[0]).is_absolute());
    let err =
        eh3d_suite_launcher::recent::add(&list, &dir.join("missing.eh3dlist").to_string_lossy())
            .unwrap_err();
    assert!(err.contains("missing"), "{err}");
    assert_eq!(load(&list).len(), 1);
}

#[test]
fn runs_at_the_same_time_do_not_lose_an_entry_of_the_recent_list() {
    let dir = temp("recent_lock");
    let list = dir.join("recent.txt");
    let mut files = Vec::new();
    for i in 0..MAX_RECENT {
        let f = dir.join(format!("p{i}.eh3dlist"));
        fs::write(&f, "{}").unwrap();
        files.push(f.to_string_lossy().into_owned());
    }
    let handles: Vec<_> = files
        .iter()
        .cloned()
        .map(|f| {
            let list = list.clone();
            std::thread::spawn(move || eh3d_suite_launcher::recent::add(&list, &f).unwrap())
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(load(&list).len(), MAX_RECENT, "an entry was lost");
    assert!(
        !dir.join("recent.txt.lock").exists(),
        "the lock was left behind"
    );
}
