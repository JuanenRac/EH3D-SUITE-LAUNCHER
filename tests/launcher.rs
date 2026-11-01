use eh3d_suite_launcher::launch::{supervise, Policy};
use eh3d_suite_launcher::project::{kind_of_path, Kind, Project};
use eh3d_suite_launcher::recent;
use eh3d_suite_launcher::registry::{discover, find_binary, is_newer, program_name, read_manifest};
use std::fs;
use std::path::{Path, PathBuf};

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("eh3d-launcher-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn manifest(dir: &Path, name: &str, role: &str, target: &str, version: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("eh3d.project.json"),
        format!(r#"{{"ecosystem":"EH3D","name":"{name}","version":"{version}","role":"{role}","maturity":"scaffolding","deployment_target":"{target}"}}"#),
    )
    .unwrap();
}

#[test]
fn the_program_name_is_the_repository_without_the_prefix_in_lowercase() {
    assert_eq!(program_name("EH3D-DRILLPRO"), "drillpro");
    assert_eq!(program_name("EH3D-BUNGARD-GUI"), "bungard-gui");
    assert_eq!(program_name("OTHER"), "other");
}

#[test]
fn the_suite_is_found_by_the_manifests_and_a_broken_one_is_reported_not_fatal() {
    let ws = temp("discover");
    manifest(
        &ws.join("EH3D-DRILLPRO"),
        "EH3D-DRILLPRO",
        "tool",
        "user-pc",
        "0.0.7",
    );
    manifest(
        &ws.join("EH3D-NODE-X"),
        "EH3D-NODE-X",
        "firmware",
        "mcu",
        "0.0.1",
    );
    fs::create_dir_all(ws.join("BROKEN")).unwrap();
    fs::write(ws.join("BROKEN/eh3d.project.json"), "{not json").unwrap();
    fs::create_dir_all(ws.join("NOT-EH3D")).unwrap();
    fs::write(ws.join("NOT-EH3D/eh3d.project.json"), r#"{"ecosystem":"ARMOR","name":"X","version":"1.0.0","role":"tool","maturity":"x","deployment_target":"user-pc"}"#).unwrap();
    fs::create_dir_all(ws.join("EMPTY")).unwrap();
    let (modules, problems) = discover(&ws);
    assert_eq!(
        modules.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
        ["EH3D-DRILLPRO", "EH3D-NODE-X"]
    );
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(modules[0].is_desktop() && !modules[1].is_desktop());
}

#[test]
fn a_manifest_with_a_bad_version_or_a_missing_field_says_which() {
    let ws = temp("badmanifest");
    manifest(&ws.join("a"), "EH3D-A", "tool", "user-pc", "0.x.1");
    assert!(read_manifest(&ws.join("a")).unwrap_err().contains("0.x.1"));
    fs::write(
        ws.join("a/eh3d.project.json"),
        r#"{"ecosystem":"EH3D","name":"EH3D-A"}"#,
    )
    .unwrap();
    assert!(read_manifest(&ws.join("a"))
        .unwrap_err()
        .contains("'version'"));
    assert!(read_manifest(&ws.join("missing")).is_err());
}

#[test]
fn a_built_program_is_found_in_release_before_debug() {
    let ws = temp("binary");
    let dir = ws.join("EH3D-DRILLPRO");
    manifest(&dir, "EH3D-DRILLPRO", "tool", "user-pc", "0.0.7");
    let module = read_manifest(&dir).unwrap();
    assert!(find_binary(&module).is_none());
    let exe = if cfg!(windows) {
        "drillpro.exe"
    } else {
        "drillpro"
    };
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug").join(exe), b"").unwrap();
    assert!(find_binary(&module)
        .unwrap()
        .to_string_lossy()
        .contains("debug"));
    fs::create_dir_all(dir.join("target/release")).unwrap();
    fs::write(dir.join("target/release").join(exe), b"").unwrap();
    assert!(find_binary(&module)
        .unwrap()
        .to_string_lossy()
        .contains("release"));
}

#[test]
fn versions_are_compared_number_by_number() {
    assert!(is_newer("0.0.10", "0.0.9"));
    assert!(is_newer("0.8.1", "0.8.0.9"));
    assert!(is_newer("0.8.0.1", "0.8.0"));
    assert!(!is_newer("0.0.7", "0.0.7"));
    assert!(!is_newer("0.0.6", "0.0.7"));
    assert!(!is_newer("junk", "0.0.7") && !is_newer("0.0.7", "junk"));
}

#[test]
fn the_kind_of_a_file_comes_from_its_extension() {
    assert_eq!(kind_of_path(Path::new("board.GTL")), Kind::Gerber);
    assert_eq!(kind_of_path(Path::new("holes.ncd")), Kind::Drill);
    assert_eq!(kind_of_path(Path::new("outline.plt")), Kind::Hpgl);
    assert_eq!(kind_of_path(Path::new("job.nc")), Kind::Gcode);
    assert_eq!(kind_of_path(Path::new("parts.pos")), Kind::PickPlace);
    assert_eq!(kind_of_path(Path::new("photo.png")), Kind::Image);
    assert_eq!(kind_of_path(Path::new("README")), Kind::Other);
}

#[test]
fn a_project_records_its_layers_with_a_checksum_and_notices_a_change() {
    let dir = temp("project");
    fs::write(dir.join("top.gtl"), "G04 copper*\nM02*\n").unwrap();
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/holes.ncd"), "M48\nM30\n").unwrap();
    let mut p = Project::new("  my board ").unwrap();
    assert_eq!(p.name, "my board");
    p.add_layer(&dir, Path::new("top.gtl")).unwrap();
    p.add_layer(&dir, &dir.join("sub/holes.ncd")).unwrap();
    assert!(p
        .add_layer(&dir, Path::new("top.gtl"))
        .unwrap_err()
        .contains("already"));
    assert!(p.add_layer(&dir, Path::new("absent.gtl")).is_err());
    assert_eq!(p.layers[1].path, "sub/holes.ncd");
    assert_eq!(p.layers[1].kind, Kind::Drill);
    assert_eq!(p.layers[0].sha256.len(), 64);
    let file = dir.join("board.eh3dlist");
    p.save(&file).unwrap();
    let back = Project::load(&file).unwrap();
    assert_eq!(back, p);
    assert!(back.verify(&dir).is_empty());
    fs::write(dir.join("top.gtl"), "changed").unwrap();
    fs::remove_file(dir.join("sub/holes.ncd")).unwrap();
    let problems = back.verify(&dir);
    assert!(
        problems
            .iter()
            .any(|m| m.contains("top.gtl") && m.contains("changed")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|m| m.contains("holes.ncd") && m.contains("missing")),
        "{problems:?}"
    );
}

#[test]
fn a_file_outside_the_project_folder_is_refused() {
    let dir = temp("outside");
    let other = temp("outside-other");
    fs::write(other.join("x.gtl"), "x").unwrap();
    let mut p = Project::new("p").unwrap();
    assert!(p
        .add_layer(&dir, &other.join("x.gtl"))
        .unwrap_err()
        .contains("not under"));
}

#[test]
fn the_project_file_is_read_with_care() {
    assert!(Project::from_json("").is_err());
    assert!(Project::from_json("[1,2]").is_err());
    assert!(Project::from_json(r#"{"schema_version":99,"name":"x"}"#)
        .unwrap_err()
        .contains("99"));
    assert!(Project::from_json(r#"{"schema_version":1}"#)
        .unwrap_err()
        .contains("name"));
    assert!(Project::from_json(r#"{"name":""}"#).is_err());
    assert!(Project::from_json(&format!(r#"{{"name":"{}"}}"#, "x".repeat(121))).is_err());
    assert!(Project::from_json(r#"{"name":"a\u0007b"}"#).is_err());
    for bad in ["/etc/passwd", "../x", "C:\\\\x", "a\\\\b", "c:x"] {
        let text = format!(r#"{{"name":"p","layers":[{{"path":"{bad}"}}]}}"#);
        assert!(Project::from_json(&text).is_err(), "{bad}");
    }
    // a zero that is not a number is the origin, not a failure
    let p = Project::from_json(r#"{"name":"p","zero":{"x":"a","y":2.5}}"#).unwrap();
    assert_eq!(p.zero, (0.0, 2.5));
}

#[test]
fn the_first_format_with_a_plain_list_of_paths_is_migrated() {
    let dir = temp("migrate");
    fs::write(dir.join("a.gtl"), "x").unwrap();
    let p = Project::from_json(r#"{"name":"old","layers":["a.gtl","b.ncd"]}"#).unwrap();
    assert_eq!(p.layers.len(), 2);
    assert_eq!(p.layers[0].kind, Kind::Gerber);
    let problems = p.verify(&dir);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("b.ncd"));
    assert!(p.to_json().contains("\"schema_version\": 1"));
}

#[test]
fn the_recent_list_has_no_repeats_and_a_limit() {
    let mut list = Vec::new();
    for i in 0..15 {
        recent::remember(&mut list, &format!("p{i}.eh3dproj"));
    }
    assert_eq!(list.len(), recent::MAX_RECENT);
    assert_eq!(list[0], "p14.eh3dproj");
    recent::remember(&mut list, "p10.eh3dproj");
    assert_eq!(list[0], "p10.eh3dproj");
    assert_eq!(list.iter().filter(|e| *e == "p10.eh3dproj").count(), 1);
    let dir = temp("recent");
    let file = dir.join("deep/recent.txt");
    recent::save(&file, &list).unwrap();
    assert_eq!(recent::load(&file), list);
}

#[test]
fn a_damaged_recent_file_is_an_empty_or_a_clean_list() {
    let dir = temp("recent-bad");
    assert!(recent::load(&dir.join("none.txt")).is_empty());
    let file = dir.join("r.txt");
    fs::write(&file, "a\n\n  \na\nb\u{0}c\nb\n").unwrap();
    assert_eq!(recent::load(&file), ["a", "b"]);
    fs::write(&file, vec![0xff, 0xfe, 0x00]).unwrap();
    assert!(recent::load(&file).is_empty());
    fs::write(&file, format!("{}\nok\n", "x".repeat(5000))).unwrap();
    assert_eq!(recent::load(&file), ["ok"]);
}

fn shell(code: i32) -> (PathBuf, Vec<String>) {
    if cfg!(windows) {
        (
            PathBuf::from("cmd"),
            vec!["/C".into(), format!("exit {code}")],
        )
    } else {
        (
            PathBuf::from("sh"),
            vec!["-c".into(), format!("exit {code}")],
        )
    }
}

#[test]
fn a_program_that_ends_well_is_run_once() {
    let (p, a) = shell(0);
    let out = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 3,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!((out.runs, out.last_code, out.gave_up), (1, Some(0), false));
}

#[test]
fn a_program_that_fails_is_started_again_a_limited_number_of_times() {
    let (p, a) = shell(3);
    let out = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 2,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!((out.runs, out.last_code, out.gave_up), (3, Some(3), true));
    let none = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 0,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!((none.runs, none.gave_up), (1, true));
}

#[test]
fn a_program_that_does_not_exist_is_an_error_that_names_it() {
    let e = supervise(
        Path::new("eh3d-no-such-program-xyz"),
        &[],
        Policy::default(),
    )
    .unwrap_err();
    assert!(e.contains("eh3d-no-such-program-xyz"), "{e}");
}

#[test]
fn a_program_is_found_by_the_name_its_cargo_toml_gives_to_the_binary() {
    let ws = temp("binname");
    let dir = ws.join("EH3D-CAM-DEBUGGER");
    manifest(&dir, "EH3D-CAM-DEBUGGER", "tool", "user-pc", "0.0.1");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"eh3d-cam-debugger\"\n\n[[bin]]\nname = \"eh3d-camdebug\"\npath = \"src/main.rs\"\n",
    )
    .unwrap();
    let module = read_manifest(&dir).unwrap();
    assert!(find_binary(&module).is_none());
    let exe = if cfg!(windows) {
        "eh3d-camdebug.exe"
    } else {
        "eh3d-camdebug"
    };
    fs::create_dir_all(dir.join("target/release")).unwrap();
    fs::write(dir.join("target/release").join(exe), b"").unwrap();
    assert!(find_binary(&module).unwrap().ends_with(exe));
}

#[test]
fn a_program_that_ended_with_an_error_of_use_is_not_started_again() {
    let (p, a) = shell(1);
    let out = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 3,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!((out.runs, out.last_code, out.gave_up), (1, Some(1), true));
}

#[test]
fn an_old_project_file_is_still_read_and_an_archive_of_common_is_told_apart() {
    let dir = temp("oldproj");
    let old = dir.join("old.eh3dproj");
    let p = Project::new("board").unwrap();
    p.save(&old).unwrap();
    assert_eq!(Project::load(&old).unwrap(), p);
    let zip = dir.join("archive.eh3dproj");
    fs::write(&zip, b"PK\x03\x04rest").unwrap();
    let err = Project::load(&zip).unwrap_err();
    assert!(err.contains("EH3D-COMMON"), "{err}");
    assert_eq!(eh3d_suite_launcher::project::EXTENSION, "eh3dlist");
}
