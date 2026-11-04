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

#[cfg(unix)]
#[test]
fn a_program_ended_by_a_signal_did_not_fail_and_is_not_started_again() {
    let out = supervise(
        Path::new("sh"),
        &["-c".to_string(), "kill -9 $$".to_string()],
        Policy {
            max_restarts: 3,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!(
        (out.runs, out.last_code, out.gave_up, out.signalled),
        (1, None, false, true)
    );
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

// ---- more edges: layer paths, byte order marks, comments in Cargo.toml, restarts and the command line ----

use eh3d_suite_launcher::project::{sha256_hex, MAX_NAME};
use eh3d_suite_launcher::registry::{binary_names, version_numbers};
use std::process::{Command, Output};

fn cli(envs: &[(&str, &Path)], args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_eh3d-launcher"));
    c.args(args);
    for (k, v) in envs {
        c.env(k, v);
    }
    c.output().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn the_checksum_is_the_known_sha256_of_the_bytes() {
    let dir = temp("sha");
    fs::write(dir.join("abc"), b"abc").unwrap();
    fs::write(dir.join("empty"), b"").unwrap();
    assert_eq!(
        sha256_hex(&dir.join("abc")).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(&dir.join("empty")).unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert!(sha256_hex(&dir.join("none")).unwrap_err().contains("none"));
    assert!(sha256_hex(&dir).is_err(), "a folder has no checksum");
}

#[test]
fn a_layer_that_climbs_out_of_the_folder_or_is_a_folder_is_refused_when_added() {
    let dir = temp("climb");
    let other = temp("climb-other");
    fs::write(other.join("x.gtl"), "x").unwrap();
    fs::create_dir_all(dir.join("sub")).unwrap();
    let mut p = Project::new("p").unwrap();
    let up = Path::new("sub")
        .join("..")
        .join("..")
        .join(other.file_name().unwrap())
        .join("x.gtl");
    assert!(p.add_layer(&dir, &up).is_err());
    assert!(
        p.add_layer(&dir, Path::new("sub")).is_err(),
        "a folder is not a layer"
    );
    assert!(p.layers.is_empty(), "a refused layer is not half added");
}

#[test]
fn a_file_name_with_two_dots_survives_the_round_trip_but_a_dotdot_part_does_not() {
    let dir = temp("dots");
    fs::write(dir.join("board..v2.gtl"), "x").unwrap();
    let mut p = Project::new("p").unwrap();
    p.add_layer(&dir, Path::new("board..v2.gtl")).unwrap();
    let back = Project::from_json(&p.to_json()).unwrap();
    assert_eq!(back, p);
    for bad in ["a/../b.gtl", "../b.gtl", "a//b.gtl", "", "a/", "./../x"] {
        let text = format!(r#"{{"name":"p","layers":[{{"path":"{bad}"}}]}}"#);
        assert!(Project::from_json(&text).is_err(), "{bad:?}");
    }
}

#[test]
fn the_first_format_cannot_smuggle_in_a_path_outside_the_project() {
    for bad in [
        "../../secret.txt",
        "/etc/passwd",
        "C:\\\\x.gtl",
        "a/../../b",
    ] {
        let text = format!(r#"{{"name":"old","layers":["{bad}"]}}"#);
        assert!(Project::from_json(&text).is_err(), "{bad}");
    }
}

#[test]
fn a_project_file_with_a_byte_order_mark_is_read() {
    let dir = temp("bom");
    let file = dir.join("p.eh3dlist");
    let p = Project::new("ñandú 日本").unwrap();
    fs::write(&file, format!("\u{feff}{}", p.to_json())).unwrap();
    assert_eq!(Project::load(&file).unwrap(), p);
}

#[test]
fn the_name_of_a_project_has_its_limits_counted_in_characters_after_trimming() {
    assert!(Project::new(&"é".repeat(MAX_NAME)).is_ok());
    assert!(Project::new(&"é".repeat(MAX_NAME + 1)).is_err());
    assert!(Project::new(&format!("  {}  ", "x".repeat(MAX_NAME))).is_ok());
    for bad in ["", "   ", "\t", "a\nb"] {
        assert!(Project::new(bad).is_err(), "{bad:?}");
    }
    let read = Project::from_json(r#"{"name":"  spaced  "}"#).unwrap();
    assert_eq!(read.name, "spaced");
}

#[test]
fn a_damaged_project_file_names_the_file_and_never_panics() {
    let dir = temp("damaged");
    let file = dir.join("p.eh3dlist");
    let good = Project::new("board").unwrap().to_json();
    for cut in 0..good.len() {
        if !good.is_char_boundary(cut) {
            continue;
        }
        fs::write(&file, &good[..cut]).unwrap();
        let _ = Project::load(&file);
    }
    fs::write(&file, b"").unwrap();
    assert!(Project::load(&file).is_err());
    fs::write(&file, [0xff, 0xfe, 0xfd]).unwrap();
    let e = Project::load(&file).unwrap_err();
    assert!(e.contains("p.eh3dlist") && e.contains("not text"), "{e}");
    assert!(Project::load(&dir.join("absent.eh3dlist"))
        .unwrap_err()
        .contains("absent"));
}

#[test]
fn the_project_json_is_the_same_text_every_time_and_unknown_kinds_become_other() {
    let text = r#"{"name":"p","layers":[{"path":"a.zzz","kind":"hologram","sha256":"ab"}],"zero":{"x":1.5,"y":-2}}"#;
    let p = Project::from_json(text).unwrap();
    assert_eq!((p.layers[0].kind, p.zero), (Kind::Other, (1.5, -2.0)));
    assert_eq!(
        p.to_json(),
        Project::from_json(&p.to_json()).unwrap().to_json()
    );
    let zero = Project::from_json(r#"{"name":"p","zero":{"x":1e999}}"#);
    assert!(zero.is_err(), "a number out of range is damaged JSON");
}

#[test]
fn verify_says_a_folder_where_a_layer_should_be_is_missing_and_keeps_going() {
    let dir = temp("verify-dir");
    fs::create_dir_all(dir.join("a.gtl")).unwrap();
    fs::write(dir.join("b.gtl"), "x").unwrap();
    let p = Project::from_json(r#"{"name":"p","layers":["a.gtl","b.gtl","c.gtl"]}"#).unwrap();
    let problems = p.verify(&dir);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("a.gtl") && problems[1].contains("c.gtl"));
}

#[test]
fn the_recent_list_keeps_the_newest_ten_of_a_longer_file_and_the_length_limit_is_exact() {
    let dir = temp("recent-long");
    let file = dir.join("r.txt");
    let lines: Vec<String> = (0..25).map(|i| format!("p{i}")).collect();
    fs::write(&file, lines.join("\r\n")).unwrap();
    let list = recent::load(&file);
    assert_eq!(list.len(), recent::MAX_RECENT);
    assert_eq!(
        (list[0].as_str(), list[9].as_str()),
        ("p0", "p9"),
        "the file is newest first: the head stays"
    );
    fs::write(
        &file,
        format!("{}\n{}\n", "y".repeat(4095), "z".repeat(4096)),
    )
    .unwrap();
    assert_eq!(recent::load(&file).len(), 1);
    // a path with non-ASCII letters makes the trip
    let mut one = Vec::new();
    recent::remember(&mut one, "C:\\proyectos\\diseño\\placa ñ.eh3dlist");
    recent::save(&file, &one).unwrap();
    assert_eq!(recent::load(&file), one);
}

#[test]
fn remembering_an_entry_already_in_the_list_moves_it_without_growing_the_list() {
    let mut list: Vec<String> = ["c", "b", "a"].iter().map(|s| s.to_string()).collect();
    recent::remember(&mut list, "a");
    assert_eq!(list, ["a", "c", "b"]);
    recent::remember(&mut list, "a");
    assert_eq!(list, ["a", "c", "b"]);
}

#[test]
fn versions_with_a_different_number_of_parts_or_signs_are_compared_sensibly() {
    assert!(
        !is_newer("0.8.0.0", "0.8.0"),
        "a missing part counts as zero"
    );
    assert!(!is_newer("0.8", "0.8.0.0"));
    assert!(is_newer("1", "0.9.9.9"));
    assert!(is_newer("0.0.7.1", "0.0.7"));
    assert!(!is_newer("", "0.0.1") && !is_newer("0.0.1", ""));
    assert!(!is_newer("1..2", "1.0.1") && !is_newer("1.0.2", "1..1"));
    assert!(!is_newer("0.0.9-beta", "0.0.1"));
    assert_eq!(version_numbers("0.8.0.1"), Some(vec![0, 8, 0, 1]));
    assert_eq!(version_numbers("1.2.x"), None);
    assert_eq!(version_numbers(" 1"), None);
    assert_eq!(
        version_numbers("99999999999999999999999"),
        None,
        "too big for a number"
    );
}

#[test]
fn a_manifest_needs_text_fields_and_a_version_of_at_least_three_numbers() {
    let ws = temp("manifest-edges");
    let dir = ws.join("m");
    fs::create_dir_all(&dir).unwrap();
    let write = |body: &str| fs::write(dir.join("eh3d.project.json"), body).unwrap();
    let full = |version: &str| {
        format!(
            r#"{{"ecosystem":"EH3D","name":"EH3D-M","version":{version},"role":"tool","maturity":"x","deployment_target":"user-pc"}}"#
        )
    };
    for bad in [
        "\"0.1\"",
        "\"1.2.3.\"",
        "\"1..3\"",
        "\"v1.2.3\"",
        "\"1.2.3-rc1\"",
        "7",
        "null",
        "\"\"",
    ] {
        write(&full(bad));
        assert!(read_manifest(&dir).is_err(), "{bad}");
    }
    for good in ["\"0.0.1\"", "\"0.8.0.1\"", "\"10.20.30.40.50\""] {
        write(&full(good));
        assert!(read_manifest(&dir).is_ok(), "{good}");
    }
    write(r#"["EH3D"]"#);
    assert!(read_manifest(&dir).unwrap_err().contains("missing"));
    write(&full("\"0.0.1\"").replace(r#""EH3D","name""#, r#""eh3d","name""#));
    assert!(
        read_manifest(&dir).unwrap_err().contains("not EH3D"),
        "the ecosystem is compared exactly"
    );
}

#[test]
fn a_workspace_that_does_not_exist_is_one_problem_and_no_modules() {
    let (modules, problems) = discover(&temp("ws-none").join("nowhere"));
    assert!(modules.is_empty());
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("cannot be read"), "{problems:?}");
}

#[test]
fn the_modules_come_out_in_alphabetical_order_whatever_the_folders_are_called() {
    let ws = temp("order");
    manifest(&ws.join("zzz"), "EH3D-AAA", "tool", "user-pc", "0.0.1");
    manifest(&ws.join("aaa"), "EH3D-ZZZ", "tool", "user-pc", "0.0.1");
    manifest(&ws.join("mmm"), "EH3D-MMM", "cli", "user-pc", "0.0.1");
    let (modules, problems) = discover(&ws);
    assert!(problems.is_empty());
    assert_eq!(
        modules.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
        ["EH3D-AAA", "EH3D-MMM", "EH3D-ZZZ"]
    );
}

#[test]
fn only_a_tool_ui_or_cli_for_the_user_pc_is_a_desktop_program() {
    let ws = temp("desktop");
    for (i, (role, target, want)) in [
        ("tool", "user-pc", true),
        ("ui", "user-pc", true),
        ("cli", "user-pc", true),
        ("service", "user-pc", false),
        ("tool", "mcu", false),
        ("tool", "cm5", false),
        ("firmware", "mcu", false),
    ]
    .into_iter()
    .enumerate()
    {
        let d = ws.join(format!("m{i}"));
        manifest(&d, &format!("EH3D-M{i}"), role, target, "0.0.1");
        assert_eq!(
            read_manifest(&d).unwrap().is_desktop(),
            want,
            "{role}/{target}"
        );
    }
}

#[test]
fn a_binary_name_in_cargo_toml_may_have_a_comment_single_quotes_and_come_after_other_tables() {
    let ws = temp("binnames");
    let dir = ws.join("EH3D-FOO");
    manifest(&dir, "EH3D-FOO", "tool", "user-pc", "0.0.1");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"not-this\"\n\n[[bin]] # the program\nname = \"foo-cmd\" # what a person types\npath = \"src/main.rs\"\n\n[[bin]]\nname = 'second'\n\n[dependencies]\nname = \"nor-this\"\n",
    )
    .unwrap();
    let names = binary_names(&read_manifest(&dir).unwrap());
    assert_eq!(names, ["foo-cmd", "second", "foo", "eh3d-foo"]);
}

#[test]
fn a_folder_named_like_the_program_is_not_the_program() {
    let ws = temp("folderbin");
    let dir = ws.join("EH3D-BAR");
    manifest(&dir, "EH3D-BAR", "tool", "user-pc", "0.0.1");
    let exe = if cfg!(windows) { "bar.exe" } else { "bar" };
    fs::create_dir_all(dir.join("target/release").join(exe)).unwrap();
    assert!(find_binary(&read_manifest(&dir).unwrap()).is_none());
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug").join(exe), b"").unwrap();
    assert!(find_binary(&read_manifest(&dir).unwrap())
        .unwrap()
        .to_string_lossy()
        .contains("debug"));
}

#[test]
fn a_program_that_crashes_once_and_then_works_is_run_twice_and_the_wait_doubles() {
    let dir = temp("flaky");
    let marker = dir.join("seen");
    let script = dir.join(if cfg!(windows) {
        "flaky.cmd"
    } else {
        "flaky.sh"
    });
    let (prog, args) = if cfg!(windows) {
        fs::write(
            &script,
            "@if exist \"%~dp0seen\" exit /b 0\r\n@type nul > \"%~dp0seen\"\r\n@exit /b 3\r\n",
        )
        .unwrap();
        ("cmd", vec!["/C".to_string(), script.display().to_string()])
    } else {
        fs::write(
            &script,
            format!(
                "if [ -f '{0}' ]; then exit 0; else touch '{0}'; exit 3; fi\n",
                marker.display()
            ),
        )
        .unwrap();
        ("sh", vec![script.display().to_string()])
    };
    let t = std::time::Instant::now();
    let o = supervise(
        Path::new(prog),
        &args,
        Policy {
            max_restarts: 2,
            backoff_ms: 60,
        },
    )
    .unwrap();
    assert_eq!((o.runs, o.last_code, o.gave_up), (2, Some(0), false));
    assert!(
        t.elapsed() >= std::time::Duration::from_millis(60),
        "it waited before the second run"
    );
    // three crashes with a wait of 40 then 80 milliseconds take at least 120
    let (p, a) = shell(7);
    let t = std::time::Instant::now();
    let o = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 2,
            backoff_ms: 40,
        },
    )
    .unwrap();
    assert_eq!((o.runs, o.gave_up), (3, true));
    assert!(
        t.elapsed() >= std::time::Duration::from_millis(120),
        "{:?}",
        t.elapsed()
    );
}

#[test]
fn a_program_that_ends_with_code_two_is_not_started_again() {
    let (p, a) = shell(2);
    let o = supervise(
        &p,
        &a,
        Policy {
            max_restarts: 5,
            backoff_ms: 1,
        },
    )
    .unwrap();
    assert_eq!((o.runs, o.last_code, o.gave_up), (1, Some(2), true));
}

#[test]
fn the_command_line_writes_a_project_adds_layers_and_notices_a_change() {
    let dir = temp("cli_project");
    let data = temp("cli_project_data");
    let envs = [("EH3D_LAUNCHER_DIR", data.as_path())];
    fs::write(dir.join("top.gtl"), "G04*\nM02*\n").unwrap();
    let file = dir.join("board.eh3dlist");
    let (f, l) = (file.to_str().unwrap(), dir.join("top.gtl"));
    let l = l.to_str().unwrap();
    assert!(cli(&envs, &["new-project", f, "ñandú"]).status.success());
    let again = cli(&envs, &["new-project", f, "other"]);
    assert_eq!(again.status.code(), Some(1));
    assert!(err(&again).contains("already exists"));
    assert_eq!(
        Project::load(&file).unwrap().name,
        "ñandú",
        "the second call did not overwrite"
    );
    let wrong = cli(
        &envs,
        &["new-project", dir.join("x.eh3dproj").to_str().unwrap(), "x"],
    );
    assert_eq!(wrong.status.code(), Some(1));
    assert!(err(&wrong).contains("eh3dlist"));
    assert!(cli(&envs, &["add-layer", f, l]).status.success());
    let twice = cli(&envs, &["add-layer", f, l]);
    assert!(
        !twice.status.success() && err(&twice).contains("already"),
        "{}",
        err(&twice)
    );
    let ok = cli(&envs, &["verify", f]);
    assert!(
        ok.status.success() && out(&ok).contains("1 files"),
        "{}",
        out(&ok)
    );
    fs::write(dir.join("top.gtl"), "changed").unwrap();
    let bad = cli(&envs, &["verify", f]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(err(&bad).contains("top.gtl") && err(&bad).contains("changed"));
    let recent = out(&cli(&envs, &["recent"]));
    assert_eq!(recent.lines().count(), 1, "{recent}");
}

#[test]
fn the_command_line_refuses_a_layer_from_another_folder_and_a_missing_file() {
    let dir = temp("cli_outside");
    let other = temp("cli_outside_other");
    let data = temp("cli_outside_data");
    let envs = [("EH3D_LAUNCHER_DIR", data.as_path())];
    fs::write(other.join("x.gtl"), "x").unwrap();
    let file = dir.join("p.eh3dlist");
    let f = file.to_str().unwrap();
    cli(&envs, &["new-project", f, "p"]);
    let r = cli(
        &envs,
        &["add-layer", f, other.join("x.gtl").to_str().unwrap()],
    );
    assert_eq!(r.status.code(), Some(1));
    assert!(err(&r).contains("not under"));
    let r = cli(
        &envs,
        &["add-layer", f, dir.join("none.gtl").to_str().unwrap()],
    );
    assert_eq!(r.status.code(), Some(1));
    assert!(err(&r).contains("none.gtl"));
    assert_eq!(Project::load(&file).unwrap().layers.len(), 0);
    for args in [
        vec!["verify"],
        vec!["add-layer", f],
        vec!["new-project"],
        vec!["recent", "add"],
        vec!["nonsense"],
    ] {
        assert_eq!(cli(&envs, &args).status.code(), Some(1), "{args:?}");
    }
}

#[test]
fn the_command_line_lists_only_desktop_programs_unless_asked_and_launch_explains_why_not() {
    let ws = temp("cli_list");
    let data = temp("cli_list_data");
    manifest(
        &ws.join("EH3D-TOOLX"),
        "EH3D-TOOLX",
        "tool",
        "user-pc",
        "0.1.2",
    );
    manifest(
        &ws.join("EH3D-NODE-Y"),
        "EH3D-NODE-Y",
        "firmware",
        "mcu",
        "0.0.3",
    );
    fs::create_dir_all(ws.join("BAD")).unwrap();
    fs::write(ws.join("BAD/eh3d.project.json"), "{").unwrap();
    manifest(
        &ws.join("EH3D-DRILLPRO"),
        "EH3D-DRILLPRO",
        "tool",
        "user-pc",
        "0.1.2",
    );
    // launch takes the folder from the configuration file, not from the environment
    fs::write(
        data.join("launcher.cfg"),
        format!(
            "workspace = {}
",
            ws.display()
        ),
    )
    .unwrap();
    let envs = [
        ("EH3D_WORKSPACE", ws.as_path()),
        ("EH3D_LAUNCHER_DIR", data.as_path()),
        ("LC_ALL", Path::new("en")),
    ];
    let list = cli(&envs, &["list"]);
    assert!(list.status.success());
    assert!(
        out(&list).contains("EH3D-TOOLX")
            && out(&list).contains("not built")
            && !out(&list).contains("NODE-Y"),
        "{}",
        out(&list)
    );
    assert!(
        err(&list).contains("skipped"),
        "a broken manifest is reported, not fatal"
    );
    assert!(out(&cli(&envs, &["list", "--all"])).contains("EH3D-NODE-Y"));
    let node = cli(&envs, &["launch", "node-y"]);
    assert_eq!(node.status.code(), Some(1));
    assert!(
        err(&node).contains("not a program for this computer"),
        "{}",
        err(&node)
    );
    let unbuilt = cli(&envs, &["launch", "DRILLPRO"]);
    assert!(
        err(&unbuilt).contains("has not been built"),
        "{}",
        err(&unbuilt)
    );
    let unknown = cli(&envs, &["launch", "TOOLX"]);
    assert!(
        err(&unknown).contains("not a known program of the suite"),
        "{}",
        err(&unknown)
    );
    let nobody = cli(&envs, &["launch", "ghost"]);
    assert!(err(&nobody).contains("no such program"), "{}", err(&nobody));
    assert_eq!(cli(&envs, &["launch"]).status.code(), Some(1));
}

#[test]
fn the_recent_command_puts_a_file_at_the_head_once() {
    let data = temp("cli_recent");
    let envs = [("EH3D_LAUNCHER_DIR", data.as_path())];
    for f in ["a.eh3dlist", "b.eh3dlist", "a.eh3dlist"] {
        assert!(cli(&envs, &["recent", "add", f]).status.success());
    }
    assert_eq!(
        out(&cli(&envs, &["recent"])).lines().collect::<Vec<_>>(),
        ["a.eh3dlist", "b.eh3dlist"]
    );
}

#[test]
fn launch_runs_nothing_from_a_folder_the_configuration_does_not_name() {
    let ws = temp("launch_cfg_ws");
    let data = temp("launch_cfg_data");
    manifest(
        &ws.join("EH3D-DRILLPRO"),
        "EH3D-DRILLPRO",
        "tool",
        "user-pc",
        "0.1.2",
    );
    let envs = [
        ("EH3D_WORKSPACE", ws.as_path()),
        ("EH3D_LAUNCHER_DIR", data.as_path()),
        ("LC_ALL", Path::new("en")),
    ];
    // no configuration file: the folder from the environment or the current directory is not used
    let none = cli(&envs, &["launch", "drillpro"]);
    assert_eq!(none.status.code(), Some(1));
    assert!(
        err(&none).contains("launcher.cfg") && err(&none).contains("workspace = FULL_PATH"),
        "{}",
        err(&none)
    );
    // a folder that is not a full path is refused
    fs::write(
        data.join("launcher.cfg"),
        "workspace = ..
",
    )
    .unwrap();
    assert!(err(&cli(&envs, &["launch", "drillpro"])).contains("launcher.cfg"));
    // the folder of the command line is not accepted either
    fs::write(
        data.join("launcher.cfg"),
        format!(
            "workspace = {}
",
            ws.display()
        ),
    )
    .unwrap();
    let flag = cli(
        &envs,
        &["launch", "drillpro", "--workspace", ws.to_str().unwrap()],
    );
    assert_eq!(flag.status.code(), Some(1));
    // the message of an unknown program comes in the language of the person
    manifest(
        &ws.join("EH3D-SOMETHING"),
        "EH3D-SOMETHING",
        "tool",
        "user-pc",
        "0.1.2",
    );
    for (lang, word) in [
        ("es", "no es un programa conocido"),
        ("de", "kein bekanntes Programm"),
        ("fr", "programme inconnu"),
        ("it", "non è un programma noto"),
        ("ja", "既知のプログラムではありません"),
        ("zh", "不是套件已知的程序"),
        ("pt", "não é um programa conhecido"),
        ("en", "not a known program"),
    ] {
        let envs = [
            ("EH3D_LAUNCHER_DIR", data.as_path()),
            ("LC_ALL", Path::new(lang)),
        ];
        let e = err(&cli(&envs, &["launch", "something"]));
        assert!(e.contains(word), "{lang}: {e}");
    }
}

#[test]
fn a_binary_name_with_a_separator_or_dots_is_not_a_name() {
    use eh3d_suite_launcher::registry::valid_bin_name;
    for bad in [
        "",
        "..",
        "..\\..\\x",
        "../x",
        "a/b",
        "C:x",
        ".hidden",
        "a..b",
    ] {
        assert!(!valid_bin_name(bad), "{bad}");
    }
    for good in ["eh3d-tools", "drillpro", "tool_2.exe"] {
        assert!(valid_bin_name(good), "{good}");
    }
    let dir = temp("bin_names");
    manifest(&dir, "EH3D-DRILLPRO", "tool", "user-pc", "0.1.2");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]
name = \"x\"
[[bin]]
name = \"..\\..\\evil\"
[[bin]]
name = \"good-bin\"
",
    )
    .unwrap();
    let m = read_manifest(&dir).unwrap();
    let names = binary_names(&m);
    assert!(names.contains(&"good-bin".to_string()));
    assert!(
        names.iter().all(|n| !n.contains('\\') && !n.contains("..")),
        "{names:?}"
    );
}

#[test]
fn verify_does_not_say_unchanged_for_a_layer_without_a_checksum() {
    let dir = temp("verify_unchecked");
    let list = dir.join("p.eh3dlist");
    fs::write(dir.join("a.gbr"), "changed").unwrap();
    fs::write(
        &list,
        r#"{"name":"p","layers":[{"path":"a.gbr","kind":"gerber"}]}"#,
    )
    .unwrap();
    let envs: [(&str, &Path); 0] = [];
    let o = cli(&envs, &["verify", list.to_str().unwrap()]);
    assert!(!out(&o).contains("unchanged"), "{}{}", out(&o), err(&o));
    assert!(
        out(&o).contains("not verified") && out(&o).contains("a.gbr"),
        "{}{}",
        out(&o),
        err(&o)
    );
}

#[test]
fn a_manifest_with_a_byte_order_mark_is_read() {
    let dir = temp("bom_manifest");
    manifest(&dir, "EH3D-DRILLPRO", "tool", "user-pc", "0.1.2");
    let text = fs::read_to_string(dir.join("eh3d.project.json")).unwrap();
    fs::write(dir.join("eh3d.project.json"), format!("\u{feff}{text}")).unwrap();
    assert_eq!(read_manifest(&dir).unwrap().name, "EH3D-DRILLPRO");
}

#[test]
fn the_options_after_the_double_dash_belong_to_the_program_that_is_started() {
    let ws = temp("dashdash_ws");
    let data = temp("dashdash_data");
    manifest(
        &ws.join("EH3D-DRILLPRO"),
        "EH3D-DRILLPRO",
        "tool",
        "user-pc",
        "0.1.2",
    );
    fs::write(
        data.join("launcher.cfg"),
        format!("workspace = {}\n", ws.display()),
    )
    .unwrap();
    let envs = [
        ("EH3D_LAUNCHER_DIR", data.as_path()),
        ("LC_ALL", Path::new("en")),
    ];
    // `--workspace` after `--` is an argument of drillpro, not of the launcher: the program is simply not built
    let o = cli(&envs, &["launch", "drillpro", "--", "--workspace", "x"]);
    assert!(err(&o).contains("has not been built"), "{}", err(&o));
}

#[test]
fn the_project_list_and_the_recent_list_are_written_through_a_temporary_file() {
    let dir = temp("atomic_save");
    let list = dir.join("p.eh3dlist");
    Project::new("p").unwrap().save(&list).unwrap();
    assert!(list.is_file() && !dir.join("p.eh3dlist.tmp").exists());
    let recent_file = dir.join("recent.txt");
    recent::save(&recent_file, &["a.eh3dlist".to_string()]).unwrap();
    assert!(recent_file.is_file() && !dir.join("recent.txt.tmp").exists());
    assert_eq!(recent::load(&recent_file), vec!["a.eh3dlist".to_string()]);
}
