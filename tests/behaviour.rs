//! Tests of the behaviour of the launcher at the exact edge of each limit: the settings, the size limit of what is read, the file browser, the arguments of the window,
//! the words of the eight languages, the restart policy, the manifests of the programs, the list of recent projects and the project list. Damaged input is refused, never a panic.

use eh3d_suite_launcher::browse::{list, matches_extension, size_text, Browser};
use eh3d_suite_launcher::lang::{fill, Lang, Msg};
use eh3d_suite_launcher::launch::sim::{crashes, supervise_script};
use eh3d_suite_launcher::launch::{
    configured_workspace, is_inside, message, policy_for, resolve, starts_by_itself,
    supervise_with, Msg as LaunchMsg, Policy, CONFIG_FILE,
};
use eh3d_suite_launcher::limit::{self, read_text_limited};
use eh3d_suite_launcher::project::{
    add_layer_to_file, create_file, remove_layer_from_file, Kind, Project, MAX_NAME, SCHEMA_VERSION,
};
use eh3d_suite_launcher::recent;
use eh3d_suite_launcher::registry::{
    find_module, is_known, is_newer, program_name, read_manifest, valid_bin_name, version_numbers,
    Module, KNOWN_PROGRAMS,
};
use eh3d_suite_launcher::settings::{
    Settings, DEFAULT_BACKOFF_MS, DEFAULT_MAX_RESTARTS, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
};
use eh3d_suite_launcher::shot::{
    parse_size, parse_window_args, wants_picture, ShotTheme, DEFAULT_SIZE, DEFAULT_WAIT,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A folder of its own for each call, so that tests that run at the same time never share a file.
fn temp(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "eh3d-launcher-behaviour-{}-{n}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn module(name: &str, role: &str, target: &str, path: &Path) -> Module {
    Module {
        name: name.into(),
        version: "0.1.0".into(),
        role: role.into(),
        maturity: "prototype".into(),
        deployment_target: target.into(),
        path: path.to_path_buf(),
    }
}

fn manifest_text(ecosystem: &str, version: &str, role: &str) -> String {
    format!(
        "{{\"ecosystem\":\"{ecosystem}\",\"name\":\"EH3D-TEST\",\"version\":\"{version}\",\"role\":\"{role}\",\"maturity\":\"prototype\",\"deployment_target\":\"user-pc\"}}"
    )
}

// ---------------------------------------------------------------- settings

#[test]
fn an_empty_settings_file_gives_the_defaults() {
    let s = Settings::parse("").unwrap();
    assert_eq!(s, Settings::default());
    assert_eq!(
        (
            s.max_restarts,
            s.backoff_ms,
            s.window_width,
            s.window_height
        ),
        (
            DEFAULT_MAX_RESTARTS,
            DEFAULT_BACKOFF_MS,
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT
        )
    );
    assert_eq!(s.language, None);
}

#[test]
fn the_width_of_the_window_is_bounded_at_both_edges() {
    assert!(Settings::parse("window_width = 639\n").is_err());
    assert_eq!(
        Settings::parse("window_width = 640\n")
            .unwrap()
            .window_width,
        640
    );
    assert_eq!(
        Settings::parse("window_width = 7680\n")
            .unwrap()
            .window_width,
        7680
    );
    assert!(Settings::parse("window_width = 7681\n").is_err());
}

#[test]
fn the_height_of_the_window_is_bounded_at_both_edges() {
    assert!(Settings::parse("window_height = 419\n").is_err());
    assert_eq!(
        Settings::parse("window_height = 420\n")
            .unwrap()
            .window_height,
        420
    );
    assert_eq!(
        Settings::parse("window_height = 4320\n")
            .unwrap()
            .window_height,
        4320
    );
    assert!(Settings::parse("window_height = 4321\n").is_err());
}

#[test]
fn the_restarts_and_the_wait_are_bounded_at_both_edges() {
    assert_eq!(
        Settings::parse("max_restarts = 0\n").unwrap().max_restarts,
        0
    );
    assert_eq!(
        Settings::parse("max_restarts = 10\n").unwrap().max_restarts,
        10
    );
    assert!(Settings::parse("max_restarts = 11\n").is_err());
    assert_eq!(
        Settings::parse("backoff_ms = 30000\n").unwrap().backoff_ms,
        30_000
    );
    assert!(Settings::parse("backoff_ms = 30001\n").is_err());
}

#[test]
fn the_size_limit_of_the_files_is_bounded_in_mebibytes() {
    assert!(Settings::parse("max_file_mib = 0\n").is_err());
    assert_eq!(
        Settings::parse("max_file_mib = 1\n")
            .unwrap()
            .max_file_bytes(),
        1_048_576
    );
    assert_eq!(
        Settings::parse("max_file_mib = 1024\n")
            .unwrap()
            .max_file_bytes(),
        1_073_741_824
    );
    assert!(Settings::parse("max_file_mib = 1025\n").is_err());
}

#[test]
fn a_number_that_is_negative_or_not_a_number_is_refused_with_the_line_and_the_key() {
    for text in [
        "max_restarts = -1\n",
        "max_restarts = two\n",
        "max_restarts = 1.5\n",
        "max_restarts =\n",
    ] {
        let e = Settings::parse(&format!("# first\n{text}")).unwrap_err();
        assert!(
            e.starts_with("launcher.cfg line 2: max_restarts is '"),
            "{e}"
        );
        assert!(e.ends_with("a whole number from 0 to 10 is needed"), "{e}");
    }
}

#[test]
fn quotes_comments_blank_lines_a_byte_order_mark_and_unknown_keys_are_tolerated() {
    let text =
        "\u{feff}# a comment\n\nunknown_key = 5\nno equals sign here\nwindow_width = \"900\"\n";
    let s = Settings::parse(text).unwrap();
    assert_eq!(s.window_width, 900);
    assert_eq!(s.window_height, DEFAULT_WINDOW_HEIGHT);
}

#[test]
fn the_last_line_of_a_key_wins() {
    let s = Settings::parse("max_restarts = 3\nmax_restarts = 4\n").unwrap();
    assert_eq!(s.max_restarts, 4);
}

#[test]
fn the_language_is_read_in_any_letter_case_and_auto_means_the_system() {
    assert_eq!(
        Settings::parse("language = ES\n").unwrap().language,
        Some(Lang::Es)
    );
    assert_eq!(
        Settings::parse("language = \"pt\"\n").unwrap().language,
        Some(Lang::Pt)
    );
    assert_eq!(Settings::parse("language = auto\n").unwrap().language, None);
    assert_eq!(
        Settings::parse("language = de\nlanguage = auto\n")
            .unwrap()
            .language,
        None
    );
}

#[test]
fn a_language_that_is_not_one_of_the_eight_is_refused_and_the_message_lists_them() {
    for text in [
        "language = xx\n",
        "language = pt-pt\n",
        "language =\n",
        "language = english\n",
    ] {
        let e = Settings::parse(text).unwrap_err();
        assert!(e.contains("launcher.cfg line 1: language is"), "{e}");
        assert!(e.contains("auto, en, es, de, fr, it, ja, zh or pt"), "{e}");
    }
}

#[test]
fn the_settings_of_a_folder_without_the_file_are_the_defaults() {
    let dir = temp("no-settings");
    assert_eq!(Settings::load(&dir).unwrap(), Settings::default());
}

#[test]
fn the_settings_file_is_read_and_its_size_is_limited_to_one_mebibyte() {
    let dir = temp("settings");
    fs::write(dir.join(CONFIG_FILE), "max_restarts = 7\n").unwrap();
    assert_eq!(Settings::load(&dir).unwrap().max_restarts, 7);
    let big = "#".repeat(limit::MAX_CONFIG_BYTES as usize + 1);
    fs::write(dir.join(CONFIG_FILE), big).unwrap();
    let e = Settings::load(&dir).unwrap_err();
    assert!(e.starts_with(limit::TOO_LARGE), "{e}");
}

#[test]
fn the_settings_give_the_restart_policy() {
    let s = Settings::parse("max_restarts = 5\nbackoff_ms = 250\n").unwrap();
    assert_eq!(
        s.policy(),
        Policy {
            max_restarts: 5,
            backoff_ms: 250
        }
    );
    assert_eq!(Policy::default().max_restarts, DEFAULT_MAX_RESTARTS);
}

// ---------------------------------------------------------------- the size limit of what is read

#[test]
fn a_mebibyte_count_never_overflows() {
    assert_eq!(limit::mib(0), 0);
    assert_eq!(limit::mib(1), 1_048_576);
    assert_eq!(limit::mib(u64::MAX), u64::MAX);
}

#[test]
fn a_file_of_exactly_the_limit_is_read_and_one_byte_more_is_refused() {
    let dir = temp("limit");
    let file = dir.join("f.txt");
    fs::write(&file, "abcd").unwrap();
    assert_eq!(read_text_limited(&file, 4).unwrap(), "abcd");
    let e = read_text_limited(&file, 3).unwrap_err();
    assert!(
        e.starts_with(limit::TOO_LARGE) && e.contains("max_file_mib"),
        "{e}"
    );
}

#[test]
fn an_empty_file_is_read_even_with_a_limit_of_zero() {
    let dir = temp("limit-empty");
    let file = dir.join("f.txt");
    fs::write(&file, "").unwrap();
    assert_eq!(read_text_limited(&file, 0).unwrap(), "");
}

#[test]
fn a_missing_file_and_a_file_that_is_not_text_have_their_own_error_words() {
    let dir = temp("limit-kinds");
    let e = read_text_limited(&dir.join("missing"), 10).unwrap_err();
    assert!(e.starts_with(limit::UNREADABLE), "{e}");
    let bin = dir.join("b.bin");
    fs::write(&bin, [0xFFu8, 0xFE, 0x00]).unwrap();
    let e = read_text_limited(&bin, 10).unwrap_err();
    assert!(e.starts_with(limit::NOT_TEXT), "{e}");
}

// ---------------------------------------------------------------- the file browser

#[test]
fn an_empty_list_of_extensions_matches_every_name() {
    assert!(matches_extension("anything", &[]));
    assert!(matches_extension(".hidden", &[]));
}

#[test]
fn an_extension_matches_in_any_letter_case_and_only_after_a_stem() {
    assert!(matches_extension("a.GBR", &["gbr"]));
    assert!(matches_extension("archive.tar.gz", &["gz"]));
    assert!(!matches_extension(".gbr", &["gbr"]));
    assert!(!matches_extension("noext", &["gbr"]));
    assert!(!matches_extension("a.", &["gbr"]));
    assert!(!matches_extension("a.gbrx", &["gbr"]));
}

#[test]
fn a_size_is_written_in_bytes_kibibytes_or_mebibytes_with_the_edges_exact() {
    assert_eq!(size_text(0), "0 B");
    assert_eq!(size_text(1023), "1023 B");
    assert_eq!(size_text(1024), "1.0 KiB");
    assert_eq!(size_text(1536), "1.5 KiB");
    assert_eq!(size_text(1_048_575), "1024.0 KiB");
    assert_eq!(size_text(1_048_576), "1.0 MiB");
}

#[test]
fn the_list_puts_folders_first_hides_dotted_names_and_sorts_without_regard_to_case() {
    let dir = temp("browse");
    fs::create_dir(dir.join("Zeta")).unwrap();
    fs::create_dir(dir.join("alpha")).unwrap();
    fs::create_dir(dir.join(".git")).unwrap();
    fs::write(dir.join("b.gbr"), b"12").unwrap();
    fs::write(dir.join("A.gbr"), b"1").unwrap();
    fs::write(dir.join(".hidden.gbr"), b"x").unwrap();
    fs::write(dir.join("note.txt"), b"x").unwrap();
    let entries = list(&dir, &["gbr"]).unwrap();
    let names: Vec<(&str, bool)> = entries
        .iter()
        .map(|e| (e.name.as_str(), e.is_dir))
        .collect();
    assert_eq!(
        names,
        vec![
            ("alpha", true),
            ("Zeta", true),
            ("A.gbr", false),
            ("b.gbr", false)
        ]
    );
    assert_eq!(entries[3].size, 2);
    assert_eq!(entries[0].size, 0);
}

#[test]
fn listing_a_missing_folder_is_an_error_that_names_it() {
    let dir = temp("browse-missing");
    let e = list(&dir.join("nope"), &[]).unwrap_err();
    assert!(e.contains("nope"), "{e}");
}

#[test]
fn a_browser_on_a_missing_folder_shows_the_error_and_no_entries() {
    let dir = temp("browser-missing");
    let b = Browser::new(dir.join("nope"), &["gbr"], false);
    assert!(b.error.is_some() && b.entries.is_empty());
}

#[test]
fn a_browser_shows_every_file_when_it_is_told_to() {
    let dir = temp("browser-all");
    fs::write(dir.join("a.gbr"), b"x").unwrap();
    fs::write(dir.join("b.txt"), b"x").unwrap();
    assert_eq!(Browser::new(dir.clone(), &["gbr"], false).entries.len(), 1);
    assert_eq!(Browser::new(dir, &["gbr"], true).entries.len(), 2);
}

#[test]
fn entering_a_folder_goes_in_and_entering_a_file_returns_its_path() {
    let dir = temp("browser-enter");
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("a.gbr"), b"x").unwrap();
    let mut b = Browser::new(dir.clone(), &["gbr"], false);
    assert_eq!(b.enter("a.gbr"), Some(dir.join("a.gbr")));
    assert_eq!(b.dir, dir);
    assert_eq!(b.enter("sub"), None);
    assert_eq!(b.dir, dir.join("sub"));
    b.up();
    assert_eq!(b.dir, dir);
    assert_eq!(b.path_text, dir.display().to_string());
}

#[test]
fn a_typed_path_that_does_not_exist_is_an_error_and_a_file_is_returned() {
    let dir = temp("browser-typed");
    fs::write(dir.join("a.gbr"), b"x").unwrap();
    let mut b = Browser::new(dir.clone(), &[], true);
    b.path_text = dir.join("nothing").display().to_string();
    assert_eq!(b.go_typed(), None);
    assert!(b.error.as_deref().unwrap().ends_with("not found"));
    b.path_text = format!("  {}  ", dir.join("a.gbr").display());
    assert_eq!(b.go_typed(), Some(dir.join("a.gbr")));
    b.path_text = dir.display().to_string();
    assert_eq!(b.go_typed(), None);
    assert!(b.error.is_none());
}

// ---------------------------------------------------------------- the arguments of the window

#[test]
fn a_size_is_width_x_height_inside_the_limits() {
    assert_eq!(parse_size("320x320"), Some((320, 320)));
    assert_eq!(parse_size("8192X8192"), Some((8192, 8192)));
    for text in [
        "319x500", "500x319", "8193x500", "500x8193", "1280", "axb", "-5x500", "1280x", "x720", "",
    ] {
        assert_eq!(parse_size(text), None, "{text:?}");
    }
}

#[test]
fn a_theme_is_read_in_any_letter_case_and_forge_is_the_default() {
    assert_eq!(ShotTheme::parse(" DARK "), Some(ShotTheme::Dark));
    assert_eq!(ShotTheme::parse("Light"), Some(ShotTheme::Light));
    assert_eq!(ShotTheme::parse("blue"), None);
    assert_eq!(ShotTheme::default(), ShotTheme::Forge);
    for t in [ShotTheme::Forge, ShotTheme::Dark, ShotTheme::Light] {
        assert_eq!(ShotTheme::parse(t.name()), Some(t));
    }
}

#[test]
fn no_arguments_give_an_empty_request_and_the_word_gui_is_skipped() {
    let none = parse_window_args(&[], 5).unwrap();
    assert!(none.files.is_empty() && none.lang.is_none() && none.shot.is_none());
    let gui = parse_window_args(&strings(&["gui", "a.eh3dlist"]), 5).unwrap();
    assert_eq!(gui.files, vec![PathBuf::from("a.eh3dlist")]);
}

#[test]
fn the_number_of_files_is_limited_and_the_message_says_how_many() {
    let one = parse_window_args(&strings(&["a", "b"]), 1).unwrap_err();
    assert_eq!(one, "only one file can be opened");
    let three = parse_window_args(&strings(&["a", "b", "c", "d"]), 3).unwrap_err();
    assert_eq!(three, "at most 3 files can be opened");
    assert_eq!(
        parse_window_args(&strings(&["a", "--open", "b", "c"]), 3)
            .unwrap()
            .files
            .len(),
        3
    );
}

#[test]
fn an_unknown_option_and_an_option_without_its_value_are_refused() {
    assert_eq!(
        parse_window_args(&strings(&["--nope"]), 3).unwrap_err(),
        "unknown option --nope"
    );
    for (arg, word) in [
        ("--shot", "--shot needs"),
        ("--open", "--open needs"),
        ("--lang", "--lang needs"),
    ] {
        let e = parse_window_args(&strings(&[arg]), 3).unwrap_err();
        assert!(e.starts_with(word), "{e}");
    }
}

#[test]
fn a_picture_must_be_a_png_file_in_any_letter_case() {
    assert!(parse_window_args(&strings(&["--shot", "x.jpg"]), 3).is_err());
    assert!(parse_window_args(&strings(&["--shot", "x"]), 3).is_err());
    assert!(parse_window_args(&strings(&["--shot", "X.PNG"]), 3).is_ok());
    assert!(wants_picture(&strings(&["--shot", "x.png"])));
    assert!(!wants_picture(&strings(&["--open", "x.png"])));
}

#[test]
fn a_picture_has_its_defaults_and_the_options_of_the_picture_need_it() {
    let p = parse_window_args(&strings(&["--shot", "x.png"]), 3).unwrap();
    let shot = p.shot.unwrap();
    assert_eq!(shot.size, DEFAULT_SIZE);
    assert_eq!(
        (shot.wait, shot.theme, shot.replace),
        (DEFAULT_WAIT, ShotTheme::Forge, false)
    );
    for opt in [
        &["--size", "800x600"][..],
        &["--theme", "dark"],
        &["--wait", "3"],
        &["--replace"],
    ] {
        assert!(parse_window_args(&strings(opt), 3).is_err(), "{opt:?}");
    }
}

#[test]
fn the_wait_of_a_picture_is_bounded_in_frames() {
    let ok = parse_window_args(&strings(&["--shot", "x.png", "--wait", "3600"]), 3).unwrap();
    assert_eq!(ok.shot.unwrap().wait, 3600);
    for bad in ["3601", "-1", "x", ""] {
        assert!(
            parse_window_args(&strings(&["--shot", "x.png", "--wait", bad]), 3).is_err(),
            "{bad:?}"
        );
    }
}

#[test]
fn the_language_of_the_window_comes_from_its_code_and_an_unknown_one_is_refused() {
    let ok = parse_window_args(&strings(&["--lang", "ja_JP.UTF-8"]), 3).unwrap();
    assert_eq!(ok.lang, Some(Lang::Ja));
    assert_eq!(
        parse_window_args(&strings(&["--lang", "xx"]), 3).unwrap_err(),
        "unknown language code"
    );
}

// ---------------------------------------------------------------- the words

#[test]
fn the_eight_languages_have_distinct_codes_and_names_in_a_fixed_order() {
    let codes: Vec<&str> = Lang::ALL.iter().map(|l| l.code()).collect();
    assert_eq!(codes, vec!["en", "es", "de", "fr", "it", "ja", "zh", "pt"]);
    let mut names: Vec<&str> = Lang::ALL.iter().map(|l| l.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 8);
}

#[test]
fn a_code_of_the_system_is_read_by_its_first_two_letters_only() {
    assert_eq!(Lang::from_code("ZH_CN"), Some(Lang::Zh));
    assert_eq!(Lang::from_code("pt_BR"), Some(Lang::Pt));
    assert_eq!(Lang::from_code("e"), None);
    assert_eq!(Lang::from_code("日本"), None);
}

#[test]
fn filling_a_template_replaces_every_use_of_a_place_and_leaves_the_others() {
    assert_eq!(fill("{0}{0}", &["ab"]), "abab");
    assert_eq!(fill("{0} {1}", &["a"]), "a {1}");
    assert_eq!(fill("{0}", &["a", "b", "c"]), "a");
    assert_eq!(fill("", &["a"]), "");
}

#[test]
fn a_sentence_is_never_the_english_one_by_accident_in_the_other_seven_languages() {
    let differs = |m: Msg| {
        Lang::ALL
            .iter()
            .skip(1)
            .filter(|l| m.tr(**l) != m.tr(Lang::En))
            .count()
    };
    let total: usize = Msg::ALL.iter().map(|m| differs(*m)).sum();
    assert!(total > Msg::ALL.len() * 3, "{total}");
}

#[test]
fn the_error_messages_of_the_command_line_carry_their_argument_in_all_eight_languages() {
    for lang in ["en", "es", "de", "fr", "it", "ja", "zh", "pt"] {
        for msg in [LaunchMsg::NoWorkspace, LaunchMsg::UnknownProgram] {
            let text = message(msg, lang, "THE-ARG");
            assert!(
                text.contains("THE-ARG") && !text.contains("{0}"),
                "{lang}: {text}"
            );
        }
    }
}

#[test]
fn an_unknown_language_falls_back_to_english_and_the_others_differ_from_it() {
    let en = message(LaunchMsg::UnknownProgram, "en", "x");
    assert_eq!(message(LaunchMsg::UnknownProgram, "xx", "x"), en);
    assert_eq!(message(LaunchMsg::UnknownProgram, "", "x"), en);
    for lang in ["es", "de", "fr", "it", "ja", "zh", "pt"] {
        assert_ne!(message(LaunchMsg::UnknownProgram, lang, "x"), en, "{lang}");
    }
}

// ---------------------------------------------------------------- starting and keeping alive

#[test]
fn only_the_two_start_options_make_a_program_start_by_itself() {
    assert!(starts_by_itself(&strings(&["a", "--start"])));
    assert!(starts_by_itself(&strings(&["--start-real"])));
    assert!(!starts_by_itself(&strings(&["--starts", "start"])));
    assert!(!starts_by_itself(&[]));
}

#[test]
fn a_program_that_starts_by_itself_is_never_started_again_and_keeps_its_wait() {
    let base = Policy {
        max_restarts: 4,
        backoff_ms: 123,
    };
    let own = policy_for(&strings(&["--start"]), base);
    assert_eq!((own.max_restarts, own.backoff_ms), (0, 123));
    assert_eq!(policy_for(&strings(&["x"]), base), base);
}

#[test]
fn the_wait_doubles_and_never_passes_thirty_seconds() {
    let policy = Policy {
        max_restarts: 12,
        backoff_ms: 20_000,
    };
    let script: Vec<_> = (0..13).map(|_| Ok(Some(3))).collect();
    let (result, waits) = supervise_script(policy, &script);
    let outcome = result.unwrap();
    assert_eq!(
        (outcome.runs, outcome.last_code, outcome.gave_up),
        (13, Some(3), true)
    );
    assert_eq!(waits.len(), 12);
    assert_eq!(waits[0], 20_000);
    assert!(waits[1..].iter().all(|w| *w == 30_000));
}

#[test]
fn a_negative_exit_code_is_a_crash_and_is_started_again() {
    let (result, waits) = supervise_script(Policy::default(), &[Ok(Some(-1)), Ok(Some(0))]);
    let outcome = result.unwrap();
    assert_eq!((outcome.runs, outcome.last_code), (2, Some(0)));
    assert_eq!(waits, vec![500]);
}

#[test]
fn with_no_restarts_allowed_a_crash_is_given_up_at_once() {
    let policy = Policy {
        max_restarts: 0,
        backoff_ms: 1,
    };
    let (result, waits) = supervise_script(policy, &[Ok(Some(9)), Ok(Some(0))]);
    let outcome = result.unwrap();
    assert_eq!((outcome.runs, outcome.gave_up), (1, true));
    assert!(waits.is_empty());
}

#[test]
fn an_error_of_the_start_is_passed_on_and_nothing_is_waited() {
    let mut waited = 0;
    let result = supervise_with(
        Policy::default(),
        || Err("cannot start".to_string()),
        |_| waited += 1,
    );
    assert_eq!(result, Err("cannot start".to_string()));
    assert_eq!(waited, 0);
}

#[test]
fn the_simulated_crashes_are_repeatable_and_exact_at_the_extremes() {
    assert_eq!(crashes(3, 300, 40, 9).len(), 40);
    let again = crashes(3, 300, 40, 9);
    assert_eq!(
        format!("{:?}", crashes(3, 300, 40, 9)),
        format!("{again:?}")
    );
    assert!(crashes(3, 0, 20, 9).iter().all(|r| *r == Ok(Some(0))));
    assert!(crashes(3, 1000, 20, 9).iter().all(|r| *r == Ok(Some(9))));
    assert!(crashes(0, 500, 20, 9).contains(&Ok(Some(9))));
}

#[test]
fn a_run_with_a_seeded_crash_rate_never_runs_more_than_the_policy_allows() {
    for seed in 0..30 {
        let script = crashes(seed, 600, 40, 7);
        let (result, waits) = supervise_script(Policy::default(), &script);
        let outcome = result.unwrap();
        assert!(outcome.runs <= 3, "{seed}");
        assert_eq!(waits.len() as u32, outcome.runs - 1, "{seed}");
    }
}

// ---------------------------------------------------------------- the workspace and the programs

fn write_cfg(dir: &Path, text: &str) {
    fs::write(dir.join(CONFIG_FILE), text).unwrap();
}

#[test]
fn the_workspace_is_the_absolute_path_of_the_configuration_with_or_without_quotes() {
    let dir = temp("workspace");
    let abs = dir.display().to_string();
    write_cfg(&dir, &format!("# a comment\n  workspace  =  {abs}  \n"));
    assert_eq!(configured_workspace(&dir, "en").unwrap(), dir);
    write_cfg(&dir, &format!("\u{feff}workspace = \"{abs}\"\n"));
    assert_eq!(configured_workspace(&dir, "en").unwrap(), dir);
}

#[test]
fn a_relative_empty_or_missing_workspace_is_refused_in_the_language_asked() {
    let dir = temp("workspace-bad");
    for text in [
        "workspace = relative/path\n",
        "workspace =\n",
        "other = 1\n",
        "# workspace = /x\n",
        "",
    ] {
        write_cfg(&dir, text);
        let e = configured_workspace(&dir, "es").unwrap_err();
        assert!(e.starts_with("launch necesita"), "{text:?}: {e}");
        assert!(e.contains(CONFIG_FILE));
    }
    let empty = temp("workspace-none");
    assert!(configured_workspace(&empty, "en")
        .unwrap_err()
        .starts_with("launch needs"));
}

#[test]
fn a_file_is_inside_a_folder_only_when_both_exist_and_it_is_below_it() {
    let dir = temp("inside");
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/a.txt"), b"x").unwrap();
    fs::write(dir.join("b.txt"), b"x").unwrap();
    assert!(is_inside(&dir.join("sub/a.txt"), &dir));
    assert!(is_inside(&dir.join("sub/a.txt"), &dir.join("sub")));
    assert!(!is_inside(&dir.join("b.txt"), &dir.join("sub")));
    assert!(!is_inside(&dir.join("missing"), &dir));
    assert!(!is_inside(&dir, &dir.join("missing")));
}

#[test]
fn a_program_that_is_not_for_this_computer_is_refused_and_says_what_it_is() {
    let dir = temp("resolve");
    let m = module("EH3D-NODE-X", "firmware", "esp32", &dir);
    let e = resolve(&m, &dir, "en").unwrap_err();
    assert_eq!(
        e,
        "EH3D-NODE-X: it is a firmware for 'esp32', not a program for this computer"
    );
}

#[test]
fn a_desktop_program_that_is_not_on_the_list_of_the_suite_is_not_started() {
    let dir = temp("resolve-unknown");
    let m = module("EH3D-STRANGER", "tool", "user-pc", &dir);
    let e = resolve(&m, &dir, "en").unwrap_err();
    assert!(e.contains("not a known program of the suite"), "{e}");
}

#[test]
fn a_known_program_outside_the_workspace_is_not_started() {
    let work = temp("resolve-work");
    let other = temp("resolve-other");
    let m = module("EH3D-ROUTEPRO", "tool", "user-pc", &other);
    assert!(resolve(&m, &work, "de")
        .unwrap_err()
        .contains("kein bekanntes Programm"));
}

#[test]
fn a_known_program_that_is_not_built_says_how_to_build_it() {
    let work = temp("resolve-unbuilt");
    let path = work.join("EH3D-ROUTEPRO");
    fs::create_dir(&path).unwrap();
    let m = module("EH3D-ROUTEPRO", "tool", "user-pc", &path);
    let e = resolve(&m, &work, "en").unwrap_err();
    assert!(
        e.contains("it has not been built") && e.contains("build.bat or build.sh"),
        "{e}"
    );
}

#[test]
fn the_list_of_known_programs_has_no_repeats_and_every_name_is_a_valid_program_name() {
    let mut sorted = KNOWN_PROGRAMS.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), KNOWN_PROGRAMS.len());
    for name in KNOWN_PROGRAMS {
        assert!(name.starts_with("EH3D-"), "{name}");
        assert!(valid_bin_name(&program_name(name)), "{name}");
    }
    assert!(KNOWN_PROGRAMS.contains(&"EH3D-SUITE-LAUNCHER"));
    let dir = temp("known");
    assert!(is_known(&module("EH3D-DRILLPRO", "tool", "user-pc", &dir)));
    assert!(!is_known(&module("eh3d-drillpro", "tool", "user-pc", &dir)));
}

#[test]
fn a_binary_name_has_at_most_sixty_four_safe_characters() {
    assert!(valid_bin_name("a"));
    assert!(valid_bin_name("a-b_c.d"));
    assert!(valid_bin_name(&"a".repeat(64)));
    for bad in ["", ".a", "a..b", "a b", "a/b", "a\\b", "ü", "a;b"] {
        assert!(!valid_bin_name(bad), "{bad:?}");
    }
    assert!(!valid_bin_name(&"a".repeat(65)));
}

#[test]
fn the_program_name_drops_the_prefix_once_and_lowers_the_case() {
    assert_eq!(program_name("EH3D-ROUTEPRO"), "routepro");
    assert_eq!(program_name("OTHER-Thing"), "other-thing");
    assert_eq!(program_name("EH3D-"), "");
    assert_eq!(program_name("EH3D-EH3D-X"), "eh3d-x");
}

#[test]
fn a_version_is_numbers_between_dots_and_nothing_else() {
    assert_eq!(version_numbers("1.2.3"), Some(vec![1, 2, 3]));
    assert_eq!(version_numbers("01.2"), Some(vec![1, 2]));
    for bad in [
        "",
        "1..2",
        "1.2.x",
        ".1",
        "1.",
        "-1.2",
        "99999999999999999999.1",
    ] {
        assert_eq!(version_numbers(bad), None, "{bad:?}");
    }
}

#[test]
fn a_candidate_is_newer_by_numbers_not_by_text_and_missing_parts_count_as_zero() {
    assert!(is_newer("1.10.0", "1.9.9"));
    assert!(!is_newer("1.0.0", "1.0.0"));
    assert!(!is_newer("1.0", "1.0.0"));
    assert!(is_newer("1.0.1", "1.0"));
    assert!(!is_newer("1.0.0", "1.0.1"));
    assert!(!is_newer("x", "1.0.0") && !is_newer("1.0.0", "x"));
}

#[test]
fn a_module_is_found_by_its_name_or_its_program_name_in_any_letter_case() {
    let dir = temp("find");
    let list = [
        module("EH3D-ROUTEPRO", "tool", "user-pc", &dir),
        module("EH3D-ISOCAM", "tool", "user-pc", &dir),
    ];
    assert_eq!(
        find_module(&list, "routepro").unwrap().name,
        "EH3D-ROUTEPRO"
    );
    assert_eq!(
        find_module(&list, "eh3d-isocam").unwrap().name,
        "EH3D-ISOCAM"
    );
    assert_eq!(find_module(&list, "IsoCam").unwrap().name, "EH3D-ISOCAM");
    assert!(find_module(&list, "other").is_none());
    assert!(find_module(&[], "routepro").is_none());
}

#[test]
fn a_manifest_of_another_ecosystem_or_with_a_bad_version_is_refused_with_the_reason() {
    let dir = temp("manifest");
    let cases = [
        (
            manifest_text("ARMOR", "1.0.0", "tool"),
            "the ecosystem is not EH3D",
        ),
        (
            manifest_text("EH3D", "1.0", "tool"),
            "is not a version of numbers separated by dots",
        ),
        (manifest_text("EH3D", "1.0.x", "tool"), "is not a version"),
        (manifest_text("EH3D", "1..0", "tool"), "is not a version"),
        ("not json".to_string(), "not JSON"),
        (
            "{\"ecosystem\":\"EH3D\"}".to_string(),
            "the field 'name' is missing",
        ),
    ];
    for (text, reason) in cases {
        fs::write(dir.join("eh3d.project.json"), &text).unwrap();
        let e = read_manifest(&dir).unwrap_err();
        assert!(e.contains(reason), "{text}: {e}");
    }
}

#[test]
fn a_good_manifest_gives_the_module_with_its_folder() {
    let dir = temp("manifest-ok");
    fs::write(
        dir.join("eh3d.project.json"),
        manifest_text("EH3D", "2.10.3", "ui"),
    )
    .unwrap();
    let m = read_manifest(&dir).unwrap();
    assert_eq!(
        (m.version.as_str(), m.role.as_str(), m.path.clone()),
        ("2.10.3", "ui", dir)
    );
    assert!(m.is_desktop());
}

// ---------------------------------------------------------------- the recent projects

#[test]
fn a_missing_recent_file_is_an_empty_list() {
    let dir = temp("recent-missing");
    assert!(recent::load(&dir.join("recent.txt")).is_empty());
    assert_eq!(
        recent::load_checked(&dir.join("recent.txt"), 10),
        Ok(vec![])
    );
}

#[test]
fn a_recent_line_of_4095_bytes_is_kept_and_one_of_4096_is_dropped() {
    let dir = temp("recent-length");
    let file = dir.join("recent.txt");
    let (keep, drop) = ("k".repeat(4095), "d".repeat(4096));
    fs::write(
        &file,
        format!("{keep}\n{drop}\nbefore\0after\n  trimmed  \n\n"),
    )
    .unwrap();
    let list = recent::load(&file);
    assert_eq!(list, vec![keep, "trimmed".to_string()]);
}

#[test]
fn the_recent_list_keeps_the_first_ten_of_a_longer_file_in_order() {
    let dir = temp("recent-ten");
    let file = dir.join("recent.txt");
    let text: String = (0..25).map(|i| format!("p{i}\np{i}\n")).collect();
    fs::write(&file, text).unwrap();
    let list = recent::load(&file);
    assert_eq!(list.len(), recent::MAX_RECENT);
    assert_eq!(list[0], "p0");
    assert_eq!(list[9], "p9");
}

#[test]
fn a_recent_file_above_the_limit_is_an_error_when_checked_and_empty_when_not() {
    let dir = temp("recent-big");
    let file = dir.join("recent.txt");
    fs::write(&file, "0123456789\n").unwrap();
    assert!(recent::load_checked(&file, 10)
        .unwrap_err()
        .starts_with(limit::TOO_LARGE));
    assert_eq!(recent::load_checked(&file, 11).unwrap().len(), 1);
    assert!(recent::load_with_limit(&file, 10).is_empty());
}

#[test]
fn remembering_puts_the_entry_at_the_head_and_cuts_the_list_at_ten() {
    let mut list: Vec<String> = (0..10).map(|i| format!("p{i}")).collect();
    recent::remember(&mut list, "new");
    assert_eq!(list.len(), 10);
    assert_eq!((list[0].as_str(), list[9].as_str()), ("new", "p8"));
    recent::remember(&mut list, "p5");
    assert_eq!(list[0], "p5");
    assert_eq!(list.iter().filter(|e| *e == "p5").count(), 1);
}

#[test]
fn saving_makes_the_folder_and_the_list_is_read_back_unchanged() {
    let dir = temp("recent-save");
    let file = dir.join("deep/er/recent.txt");
    let list = strings(&["one", "two words", "three"]);
    recent::save(&file, &list).unwrap();
    assert_eq!(recent::load(&file), list);
    assert!(fs::read_to_string(&file).unwrap().ends_with("three\n"));
    recent::save(&file, &[]).unwrap();
    assert!(recent::load(&file).is_empty());
}

#[test]
fn adding_a_file_stores_its_full_path_once_and_leaves_no_lock_behind() {
    let dir = temp("recent-add");
    let list_file = dir.join("recent.txt");
    let project = dir.join("a.eh3dlist");
    fs::write(&project, "x").unwrap();
    let one = recent::add(&list_file, project.to_str().unwrap()).unwrap();
    let two = recent::add(&list_file, project.to_str().unwrap()).unwrap();
    assert_eq!(one, two);
    assert_eq!(one.len(), 1);
    assert!(!one[0].starts_with(r"\\?\"));
    assert!(Path::new(&one[0]).is_absolute());
    let left: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        left.iter()
            .all(|n| !n.ends_with(".lock") && !n.ends_with(".tmp")),
        "{left:?}"
    );
}

#[test]
fn adding_a_file_that_does_not_exist_is_an_error_that_names_it() {
    let dir = temp("recent-add-missing");
    let e = recent::add(&dir.join("recent.txt"), "no-such-project.eh3dlist").unwrap_err();
    assert!(e.contains("no-such-project.eh3dlist"), "{e}");
    assert!(!dir.join("recent.txt").exists());
}

// ---------------------------------------------------------------- the project list

#[test]
fn the_name_of_a_project_is_trimmed_and_counted_in_characters() {
    assert_eq!(Project::new("  My board  ").unwrap().name, "My board");
    assert!(Project::new(&"é".repeat(MAX_NAME)).is_ok());
    assert!(Project::new(&"é".repeat(MAX_NAME + 1)).is_err());
    assert!(Project::new("   ").unwrap_err().contains("empty"));
    assert!(Project::new("a\u{7}b")
        .unwrap_err()
        .contains("control character"));
}

#[test]
fn a_project_goes_to_json_and_back_with_its_zero_and_its_layers() {
    let dir = temp("project-json");
    fs::write(dir.join("a.gtl"), b"abc").unwrap();
    let mut p = Project::new("P").unwrap();
    p.add_layer(&dir, &dir.join("a.gtl")).unwrap();
    p.zero = (1.5, -2.25);
    let text = p.to_json();
    assert!(text.ends_with("}\n"));
    assert!(text.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")));
    assert_eq!(Project::from_json(&text).unwrap(), p);
    assert_eq!(p.layers[0].kind, Kind::Gerber);
    assert_eq!(p.layers[0].sha256.len(), 64);
}

#[test]
fn the_same_layer_twice_and_a_layer_that_is_not_there_are_refused() {
    let dir = temp("project-dup");
    fs::write(dir.join("a.gtl"), b"abc").unwrap();
    let mut p = Project::new("P").unwrap();
    p.add_layer(&dir, Path::new("a.gtl")).unwrap();
    assert!(p
        .add_layer(&dir, &dir.join("a.gtl"))
        .unwrap_err()
        .contains("already in the project"));
    assert!(p
        .remove_layer("missing")
        .unwrap_err()
        .contains("not in the project"));
    assert!(p.remove_layer("a.gtl").is_ok());
    assert!(p.layers.is_empty());
}

#[test]
fn a_project_file_of_a_newer_format_or_without_a_name_is_refused() {
    let newer = format!(
        "{{\"schema_version\": {}, \"name\": \"x\"}}",
        SCHEMA_VERSION + 1
    );
    assert!(Project::from_json(&newer)
        .unwrap_err()
        .contains("this program reads up to"));
    assert!(Project::from_json("{}")
        .unwrap_err()
        .contains("has no name"));
    assert!(Project::from_json("{\"name\": 5}")
        .unwrap_err()
        .contains("has no name"));
    assert!(Project::from_json("[").unwrap_err().contains("not JSON"));
    assert!(Project::from_json("").unwrap_err().contains("not JSON"));
}

#[test]
fn layers_that_are_not_a_list_are_refused_and_so_is_a_layer_without_a_path() {
    for layers in ["5", "\"a.gtl\"", "{\"path\":\"a.gtl\"}", "true"] {
        let text = format!("{{\"name\":\"x\",\"layers\":{layers}}}");
        let e = Project::from_json(&text).unwrap_err();
        assert!(e.contains("not a list"), "{layers}: {e}");
    }
    assert!(Project::from_json("{\"name\":\"x\",\"layers\":null}")
        .unwrap()
        .layers
        .is_empty());
    assert!(Project::from_json("{\"name\":\"x\"}")
        .unwrap()
        .layers
        .is_empty());
    let e = Project::from_json("{\"name\":\"x\",\"layers\":[{\"kind\":\"gerber\"}]}").unwrap_err();
    assert!(e.contains("a layer has no path"));
}

#[test]
fn a_layer_path_that_cannot_be_a_file_name_on_every_system_is_refused() {
    for path in [
        "C:/x.gbr",
        "a\\\\b.gbr",
        "/abs.gbr",
        "../x.gbr",
        "a//b.gbr",
        "a/./b.gbr",
        "CON.gbr",
        "nul",
        "a/COM1.txt",
        "LPT9.x",
        "name.",
        "name ",
        "a/aux",
    ] {
        let text = format!("{{\"name\":\"x\",\"layers\":[{{\"path\":\"{path}\"}}]}}");
        assert!(Project::from_json(&text).is_err(), "{path}");
    }
    for path in ["COM0.txt", "CONSOLE.txt", "a/b.c.d", "x y.gbr"] {
        let text = format!("{{\"name\":\"x\",\"layers\":[{{\"path\":\"{path}\"}}]}}");
        assert!(Project::from_json(&text).is_ok(), "{path}");
    }
}

#[test]
fn two_layers_that_differ_only_in_case_are_one_and_the_first_is_kept() {
    let text = "{\"name\":\"x\",\"layers\":[{\"path\":\"A.gbr\"},{\"path\":\"a.GBR\"},{\"path\":\"b.gbr\"}]}";
    let p = Project::from_json(text).unwrap();
    let paths: Vec<&str> = p.layers.iter().map(|l| l.path.as_str()).collect();
    assert_eq!(paths, vec!["A.gbr", "b.gbr"]);
}

#[test]
fn a_zero_that_is_not_a_number_is_refused_and_an_unknown_kind_is_other() {
    for zero in [
        "{\"x\":\"a\",\"y\":1}",
        "{\"x\":1,\"y\":null}",
        "{\"x\":1}",
        "{\"y\":1}",
        "{}",
        "[1,2]",
        "7",
        "\"0,0\"",
    ] {
        let e = Project::from_json(&format!("{{\"name\":\"x\",\"zero\":{zero}}}")).unwrap_err();
        assert!(e.contains("work zero"), "{zero}: {e}");
    }
    for text in ["{\"name\":\"x\"}", "{\"name\":\"x\",\"zero\":null}"] {
        assert_eq!(Project::from_json(text).unwrap().zero, (0.0, 0.0), "{text}");
    }
    let text = "{\"name\":\"x\",\"layers\":[{\"path\":\"a.gbr\",\"kind\":\"weird\"}]}";
    let p = Project::from_json(text).unwrap();
    assert_eq!(p.layers[0].kind, Kind::Other);
    let p = Project::from_json("{\"name\":\"x\",\"zero\":{\"x\":1e3,\"y\":-0.5}}").unwrap();
    assert_eq!(p.zero, (1000.0, -0.5));
}

#[test]
fn the_layers_without_a_checksum_are_listed_as_unchecked() {
    let text = "{\"name\":\"x\",\"layers\":[\"old.gbr\",{\"path\":\"new.gbr\",\"sha256\":\"ab\"}]}";
    let p = Project::from_json(text).unwrap();
    assert_eq!(p.unchecked(), vec!["old.gbr"]);
}

#[test]
fn verify_names_the_file_that_is_missing_and_the_one_that_changed() {
    let dir = temp("project-verify");
    fs::write(dir.join("a.gtl"), b"abc").unwrap();
    fs::write(dir.join("b.gtl"), b"abc").unwrap();
    let mut p = Project::new("P").unwrap();
    p.add_layer(&dir, &dir.join("a.gtl")).unwrap();
    p.add_layer(&dir, &dir.join("b.gtl")).unwrap();
    assert!(p.verify(&dir).is_empty());
    fs::write(dir.join("a.gtl"), b"changed").unwrap();
    fs::remove_file(dir.join("b.gtl")).unwrap();
    let problems = p.verify(&dir);
    assert_eq!(
        problems,
        vec![
            "a.gtl: the file has changed since it was added".to_string(),
            "b.gtl: the file is missing".to_string()
        ]
    );
}

#[test]
fn a_project_file_that_is_an_archive_or_not_text_or_missing_is_refused_by_its_name() {
    let dir = temp("project-load");
    let archive = dir.join("a.eh3dproj");
    fs::write(&archive, b"PK\x03\x04rest").unwrap();
    assert!(Project::load(&archive)
        .unwrap_err()
        .contains("archive of EH3D-COMMON"));
    let binary = dir.join("b.eh3dlist");
    fs::write(&binary, [0xFFu8, 0xFE, 0x00]).unwrap();
    assert!(Project::load(&binary).unwrap_err().contains("not text"));
    assert!(Project::load(&dir.join("none.eh3dlist"))
        .unwrap_err()
        .contains("none.eh3dlist"));
}

#[test]
fn a_new_project_file_needs_the_right_extension_and_never_replaces_a_file() {
    let dir = temp("project-create");
    let wrong = dir.join("a.eh3dproj");
    assert!(create_file(wrong.to_str().unwrap(), "P")
        .unwrap_err()
        .contains(".eh3dlist"));
    assert!(!wrong.exists());
    let file = dir.join("a.eh3dlist");
    create_file(file.to_str().unwrap(), "P").unwrap();
    assert_eq!(Project::load(&file).unwrap().name, "P");
    let again = create_file(file.to_str().unwrap(), "Q").unwrap_err();
    assert!(again.contains("already exists"));
    assert_eq!(Project::load(&file).unwrap().name, "P");
    assert!(create_file(dir.join("b.eh3dlist").to_str().unwrap(), "  ").is_err());
    assert!(!dir.join("b.eh3dlist").exists());
}

#[test]
fn a_damaged_list_of_layers_is_never_written_over() {
    let dir = temp("layers-damaged");
    fs::write(dir.join("a.gtl"), "x").unwrap();
    let file = dir.join("p.eh3dlist");
    let damaged = "{\"name\":\"p\",\"layers\":{\"path\":\"kept.gtl\"}}";
    fs::write(&file, damaged).unwrap();
    for result in [
        add_layer_to_file(&file, &dir.join("a.gtl")),
        remove_layer_from_file(&file, "kept.gtl"),
    ] {
        assert!(result.unwrap_err().contains("not a list"));
    }
    assert_eq!(fs::read_to_string(&file).unwrap(), damaged);
}

#[test]
fn a_damaged_work_zero_or_one_that_is_not_a_number_is_never_written_over() {
    let dir = temp("zero-damaged");
    fs::write(dir.join("a.gtl"), "x").unwrap();
    let file = dir.join("p.eh3dlist");
    let damaged = "{\"name\":\"p\",\"layers\":[],\"zero\":{\"x\":\"12,5\",\"y\":3}}";
    fs::write(&file, damaged).unwrap();
    for result in [
        add_layer_to_file(&file, &dir.join("a.gtl")),
        remove_layer_from_file(&file, "a.gtl"),
    ] {
        let e = result.unwrap_err();
        assert!(e.contains("work zero") && e.contains("for x"), "{e}");
    }
    assert_eq!(fs::read_to_string(&file).unwrap(), damaged);
    let mut p = Project::new("p").unwrap();
    for zero in [(f64::NAN, 0.0), (0.0, f64::INFINITY)] {
        p.zero = zero;
        assert!(p.save(&file).unwrap_err().contains("work zero"));
    }
    assert_eq!(fs::read_to_string(&file).unwrap(), damaged);
}

#[test]
fn a_layer_is_hashed_before_the_project_is_locked() {
    let dir = temp("hash-before-lock");
    // a folder cannot be hashed
    fs::create_dir_all(dir.join("folder.gtl")).unwrap();
    let file = dir.join("p.eh3dlist");
    create_file(file.to_str().unwrap(), "p").unwrap();
    let lock = dir.join("p.eh3dlist.lock");
    fs::write(&lock, "").unwrap();
    // with the lock held by somebody else, the failure is the one of the file: it was read without asking for the lock
    let e = add_layer_to_file(&file, &dir.join("folder.gtl")).unwrap_err();
    assert!(e.contains("folder.gtl") && !e.contains("lock"), "{e}");
    assert!(lock.exists(), "the lock of the other run was removed");
}

#[test]
fn adding_and_removing_a_layer_in_the_file_counts_the_layers_and_leaves_no_lock_or_temporary_file()
{
    let dir = temp("project-file");
    let file = dir.join("p.eh3dlist");
    create_file(file.to_str().unwrap(), "P").unwrap();
    fs::write(dir.join("a.gtl"), b"1").unwrap();
    fs::write(dir.join("b.drl"), b"2").unwrap();
    assert_eq!(add_layer_to_file(&file, &dir.join("a.gtl")).unwrap(), 1);
    assert_eq!(add_layer_to_file(&file, &dir.join("b.drl")).unwrap(), 2);
    assert!(add_layer_to_file(&file, &dir.join("a.gtl")).is_err());
    assert_eq!(remove_layer_from_file(&file, "a.gtl").unwrap(), 1);
    assert!(remove_layer_from_file(&file, "a.gtl").is_err());
    let p = Project::load(&file).unwrap();
    assert_eq!(p.layers[0].path, "b.drl");
    assert_eq!(p.layers[0].kind, Kind::Drill);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["a.gtl", "b.drl", "p.eh3dlist"]);
}

#[test]
fn the_first_file_is_remembered_when_the_folder_of_the_launcher_is_not_there_yet() {
    let dir = temp("recent-first");
    let project = dir.join("a.eh3dlist");
    fs::write(&project, "x").unwrap();
    let list_file = dir.join("not-yet").join("eh3d-launcher").join("recent.txt");
    let list = recent::add(&list_file, project.to_str().unwrap()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(recent::load(&list_file), list);
}

#[test]
fn writers_at_the_same_time_leave_one_whole_file_and_no_file_of_the_person_is_taken() {
    let dir = temp("atomic-together");
    let file = dir.join("p.eh3dlist");
    let theirs = dir.join("p.eh3dlist.tmp");
    fs::write(&theirs, "mine").unwrap();
    let texts: Vec<String> = (0..8)
        .map(|k| format!("line {k}\n").repeat(2_000 + k * 100))
        .collect();
    let written: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = texts
            .iter()
            .map(|t| {
                let file = &file;
                scope.spawn(move || {
                    usize::from(eh3d_suite_launcher::project::write_atomic(file, t).is_ok())
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    // on Windows a rename over a file that another rename holds at that moment can be refused: the file is whole all the same
    assert!(written >= 1);
    assert!(texts.contains(&fs::read_to_string(&file).unwrap()));
    assert_eq!(fs::read_to_string(&theirs).unwrap(), "mine");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
}
