//! Starting a program of the suite and keeping it alive: a program that dies with an error is started again a limited number of times, one that ends well is left alone.

use crate::registry::{find_binary, is_known, Module};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// How many times a program that failed is started again (0 = never).
    pub max_restarts: u32,
    pub backoff_ms: u64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_restarts: 2,
            backoff_ms: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// How many times the program was started.
    pub runs: u32,
    /// The exit code of the last run; `None` when it was ended by a signal.
    pub last_code: Option<i32>,
    /// True when the last run failed and the restarts were used up. A program ended by a signal did not fail: the operator ended it.
    pub gave_up: bool,
    /// True when the last run was ended by a signal (no exit code): it is neither a failure nor started again.
    pub signalled: bool,
}

/// True when a run that ended with `code` is worth starting again: it crashed. A signal (`None`: the operator ended it), 1 and 2
/// (an error of use, or a program that found something to report) are not.
fn restarts_after(code: Option<i32>) -> bool {
    matches!(code, Some(c) if c != 0 && c != 1 && c != 2)
}

/// Runs `program` with `args` and waits. A run that crashed (a code other than 0, 1 and 2) is repeated up to `policy.max_restarts` times, with the waiting time doubling.
pub fn supervise(program: &Path, args: &[String], policy: Policy) -> Result<Outcome, String> {
    supervise_with(
        policy,
        || {
            Command::new(program)
                .args(args)
                .status()
                .map(|status| status.code())
                .map_err(|e| format!("{}: {e}", program.display()))
        },
        |ms| sleep(Duration::from_millis(ms)),
    )
}

/// The supervision with its two outside effects handed in: `run` starts the program once and waits for it (its exit code, `None` when a signal ended it, or the failure to
/// start it) and `wait_ms` waits. [`supervise`] gives it the real ones; the simulation gives it a script and a clock that only writes the waits down.
pub fn supervise_with(
    policy: Policy,
    mut run: impl FnMut() -> Result<Option<i32>, String>,
    mut wait_ms: impl FnMut(u64),
) -> Result<Outcome, String> {
    let mut runs = 0;
    let mut wait = policy.backoff_ms;
    loop {
        runs += 1;
        let code = run()?;
        if code == Some(0) {
            return Ok(Outcome {
                runs,
                last_code: code,
                gave_up: false,
                signalled: false,
            });
        }
        // a program that the operator ended on purpose (a signal) or that refused its arguments (1, 2) is not started again: only a crash is
        if !restarts_after(code) || runs > policy.max_restarts {
            return Ok(Outcome {
                runs,
                last_code: code,
                gave_up: code.is_some(),
                signalled: code.is_none(),
            });
        }
        wait_ms(wait);
        wait = wait.saturating_mul(2).min(30_000);
    }
}

/// The messages `launch` gives about the folder and the program it is asked to start, in the eight languages of the suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    /// The configuration file does not name the folder of the repositories (`{0}` is the file).
    NoWorkspace,
    /// The program is not on the list of the suite (`{0}` is its name).
    UnknownProgram,
}

/// The language of the person from the variables of the system: `en`, `es`, `de`, `fr`, `it`, `ja`, `zh` or `pt`; English when it is none of them.
pub fn language() -> &'static str {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            if v.is_empty() {
                continue;
            }
            let code: String = v.chars().take(2).collect::<String>().to_ascii_lowercase();
            for known in ["es", "de", "fr", "it", "ja", "zh", "pt"] {
                if code == known {
                    return known;
                }
            }
            return "en";
        }
    }
    "en"
}

/// The text of a message in `lang`, with `arg` in the place of `{0}`.
pub fn message(msg: Msg, lang: &str, arg: &str) -> String {
    let text = match (msg, lang) {
        (Msg::NoWorkspace, "es") => "launch necesita la carpeta que contiene los repositorios en el fichero de configuración ({0}): escriba 'workspace = RUTA_COMPLETA' en él",
        (Msg::NoWorkspace, "de") => "launch braucht den Ordner mit den Repositories in der Konfigurationsdatei ({0}): 'workspace = VOLLSTÄNDIGER_PFAD' eintragen",
        (Msg::NoWorkspace, "fr") => "launch a besoin du dossier qui contient les dépôts dans le fichier de configuration ({0}) : y écrire 'workspace = CHEMIN_COMPLET'",
        (Msg::NoWorkspace, "it") => "launch richiede la cartella che contiene i repository nel file di configurazione ({0}): scrivere 'workspace = PERCORSO_COMPLETO'",
        (Msg::NoWorkspace, "ja") => "launch には設定ファイル（{0}）でリポジトリを置くフォルダを指定する必要があります。'workspace = フルパス' と書いてください",
        (Msg::NoWorkspace, "zh") => "launch 需要在配置文件（{0}）中指定存放仓库的文件夹：写入 'workspace = 完整路径'",
        (Msg::NoWorkspace, "pt") => "launch precisa da pasta que contém os repositórios no ficheiro de configuração ({0}): escreva 'workspace = CAMINHO_COMPLETO'",
        (Msg::NoWorkspace, _) => "launch needs the folder that holds the repositories in the configuration file ({0}): write 'workspace = FULL_PATH' in it",
        (Msg::UnknownProgram, "es") => "{0}: no es un programa conocido de la suite; solo se inician los programas de la lista de la suite",
        (Msg::UnknownProgram, "de") => "{0}: kein bekanntes Programm der Suite; nur Programme aus der Liste der Suite werden gestartet",
        (Msg::UnknownProgram, "fr") => "{0} : programme inconnu de la suite ; seuls les programmes de la liste de la suite sont lancés",
        (Msg::UnknownProgram, "it") => "{0}: non è un programma noto della suite; vengono avviati solo i programmi dell'elenco della suite",
        (Msg::UnknownProgram, "ja") => "{0}: スイートの既知のプログラムではありません。スイートの一覧にあるプログラムだけを起動します",
        (Msg::UnknownProgram, "zh") => "{0}：不是套件已知的程序；只会启动套件列表中的程序",
        (Msg::UnknownProgram, "pt") => "{0}: não é um programa conhecido da suite; só se iniciam os programas da lista da suite",
        (Msg::UnknownProgram, _) => "{0}: not a known program of the suite; only the programs on the list of the suite are started",
    };
    text.replace("{0}", arg)
}

/// The name of the configuration file inside the folder of the launcher.
pub const CONFIG_FILE: &str = "launcher.cfg";

/// The folder of the repositories that `launch` may start programs from: the line `workspace = FULL_PATH` of the configuration file in `dir`. The current directory,
/// the command line and the environment do not choose it; a path that is not a full path is refused.
pub fn configured_workspace(dir: &Path, lang: &str) -> Result<PathBuf, String> {
    let file = dir.join(CONFIG_FILE);
    let refused = || message(Msg::NoWorkspace, lang, &file.display().to_string());
    let text = crate::limit::read_text_limited(&file, crate::limit::MAX_CONFIG_BYTES)
        .map_err(|_| refused())?;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "workspace" {
                let value = value.trim().trim_matches('"');
                let path = PathBuf::from(value);
                if value.is_empty() || !path.is_absolute() {
                    return Err(refused());
                }
                return Ok(path);
            }
        }
    }
    Err(refused())
}

/// True when `file` is inside `folder` once both are resolved (symbolic links and `..` included).
pub fn is_inside(file: &Path, folder: &Path) -> bool {
    match (file.canonicalize(), folder.canonicalize()) {
        (Ok(f), Ok(d)) => f.starts_with(d),
        _ => false,
    }
}

/// The folder where the launcher keeps its files (`launcher.cfg`, `recent.txt`): the variable `EH3D_LAUNCHER_DIR`, or the application data folder.
pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("EH3D_LAUNCHER_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = std::env::var_os("APPDATA") {
        return PathBuf::from(d).join("eh3d-launcher");
    }
    if let Some(d) = std::env::var_os("HOME") {
        return PathBuf::from(d).join(".config").join("eh3d-launcher");
    }
    PathBuf::from(".")
}

/// The built program of `module`, once every check that `launch` makes has passed: it is a program for this computer, it is on the list of the suite and inside `workspace`, and
/// the file found is inside its own `target` folder.
pub fn resolve(module: &Module, workspace: &Path, lang: &str) -> Result<PathBuf, String> {
    if !module.is_desktop() {
        return Err(format!(
            "{}: it is a {} for '{}', not a program for this computer",
            module.name, module.role, module.deployment_target
        ));
    }
    if !is_known(module) || !is_inside(&module.path, workspace) {
        return Err(message(Msg::UnknownProgram, lang, &module.name));
    }
    let bin = find_binary(module).ok_or_else(|| {
        format!(
            "{}: it has not been built; run build.bat or build.sh in {}",
            module.name,
            module.path.display()
        )
    })?;
    if !is_inside(&bin, &module.path.join("target")) {
        return Err(message(Msg::UnknownProgram, lang, &module.name));
    }
    Ok(bin)
}

/// The simulation of a program that crashes: the supervision runs against a script of runs (the exit code of each, a crash, a signal or a failure to start)
/// and a clock that only writes down the waits, so the restarts, the doubling of the wait and the giving up can be tried with no process and no time.
/// Nothing here has started a real program.
pub mod sim {
    use super::{supervise_with, Outcome, Policy};

    /// One run of the simulated program: its exit code (`None`: ended by a signal) or the failure to start it.
    pub type Run = Result<Option<i32>, String>;

    /// A small deterministic generator (xorshift64*). It reads no clock.
    #[derive(Debug, Clone)]
    pub struct SimRng(u64);

    impl SimRng {
        /// A generator for the seed (zero is replaced, because xorshift cannot start from zero).
        pub fn new(seed: u64) -> SimRng {
            SimRng(if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            })
        }

        /// A number below `n` (`n` must be above zero).
        pub fn below(&mut self, n: u64) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D) % n
        }
    }

    /// A script of `runs` runs in which each one crashes with `crash_code` with the chance `crash_per_mille` in a thousand and otherwise ends well.
    pub fn crashes(seed: u64, crash_per_mille: u64, runs: usize, crash_code: i32) -> Vec<Run> {
        let mut rng = SimRng::new(seed);
        (0..runs)
            .map(|_| {
                if rng.below(1000) < crash_per_mille {
                    Ok(Some(crash_code))
                } else {
                    Ok(Some(0))
                }
            })
            .collect()
    }

    /// Supervise the script: what the supervision made of it and the waits (in milliseconds) it asked for. A script that runs out is a failure to start.
    pub fn supervise_script(policy: Policy, script: &[Run]) -> (Result<Outcome, String>, Vec<u64>) {
        let mut next = 0;
        let mut waits = Vec::new();
        let result = supervise_with(
            policy,
            || {
                let run = script
                    .get(next)
                    .cloned()
                    .unwrap_or_else(|| Err("the script ran out".into()));
                next += 1;
                run
            },
            |ms| waits.push(ms),
        );
        (result, waits)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn policy(max_restarts: u32, backoff_ms: u64) -> Policy {
            Policy {
                max_restarts,
                backoff_ms,
            }
        }

        fn outcome(runs: u32, last_code: Option<i32>, gave_up: bool, signalled: bool) -> Outcome {
            Outcome {
                runs,
                last_code,
                gave_up,
                signalled,
            }
        }

        #[test]
        fn a_program_that_ends_well_is_run_once_and_nothing_is_waited() {
            let (result, waits) = supervise_script(Policy::default(), &[Ok(Some(0))]);
            assert_eq!(result, Ok(outcome(1, Some(0), false, false)));
            assert!(waits.is_empty());
        }

        #[test]
        fn a_crash_is_started_again_after_the_first_wait_and_the_recovery_is_not_a_failure() {
            let (result, waits) =
                supervise_script(Policy::default(), &[Ok(Some(139)), Ok(Some(0))]);
            assert_eq!(result, Ok(outcome(2, Some(0), false, false)));
            assert_eq!(waits, vec![500]);
        }

        #[test]
        fn a_program_that_keeps_crashing_gives_up_after_its_restarts_with_the_wait_doubled() {
            let script = vec![Ok(Some(139)); 10];
            let (result, waits) = supervise_script(Policy::default(), &script);
            assert_eq!(result, Ok(outcome(3, Some(139), true, false)));
            assert_eq!(waits, vec![500, 1000]);
        }

        #[test]
        fn an_error_of_use_is_never_started_again() {
            for code in [1, 2] {
                let (result, waits) =
                    supervise_script(Policy::default(), &[Ok(Some(code)), Ok(Some(0))]);
                assert_eq!(result, Ok(outcome(1, Some(code), true, false)));
                assert!(waits.is_empty());
            }
        }

        #[test]
        fn a_program_ended_by_a_signal_is_neither_a_failure_nor_started_again() {
            let (result, waits) = supervise_script(Policy::default(), &[Ok(None), Ok(Some(0))]);
            assert_eq!(result, Ok(outcome(1, None, false, true)));
            assert!(waits.is_empty());
        }

        #[test]
        fn a_program_that_cannot_be_started_is_an_error_and_nothing_is_waited() {
            let (result, waits) =
                supervise_script(Policy::default(), &[Err("no such file".into())]);
            assert_eq!(result, Err("no such file".to_string()));
            assert!(waits.is_empty());
            let (ran_out, _) = supervise_script(Policy::default(), &[Ok(Some(139))]);
            assert!(ran_out.is_err());
        }

        #[test]
        fn zero_restarts_means_never_and_the_wait_stops_growing_at_thirty_seconds() {
            let (result, waits) = supervise_script(policy(0, 500), &[Ok(Some(139)), Ok(Some(0))]);
            assert_eq!(result, Ok(outcome(1, Some(139), true, false)));
            assert!(waits.is_empty());
            let script = vec![Ok(Some(139)); 30];
            let (result, waits) = supervise_script(policy(12, 1000), &script);
            assert_eq!(result, Ok(outcome(13, Some(139), true, false)));
            assert_eq!(&waits[..6], &[1000, 2000, 4000, 8000, 16000, 30000]);
            assert!(waits[5..].iter().all(|&w| w == 30000));
            assert_eq!(waits.len(), 12);
        }

        #[test]
        fn a_seeded_campaign_follows_the_policy_for_every_seed_and_repeats_exactly() {
            for seed in 1..=40 {
                let script = crashes(seed, 600, 6, 139);
                let pol = policy(5, 10);
                let first = supervise_script(pol, &script);
                assert_eq!(first, supervise_script(pol, &script), "seed {seed}");
                let (result, waits) = first;
                let out = result.unwrap();
                let first_good = script.iter().position(|r| *r == Ok(Some(0)));
                let expected_runs = first_good.map_or(6, |i| i as u32 + 1);
                assert_eq!(out.runs, expected_runs, "seed {seed}");
                assert_eq!(out.gave_up, first_good.is_none(), "seed {seed}");
                assert_eq!(waits.len() as u32, expected_runs - 1, "seed {seed}");
                for (k, w) in waits.iter().enumerate() {
                    assert_eq!(*w, 10 << k, "seed {seed}");
                }
            }
        }

        #[test]
        fn the_generator_never_stalls_on_a_zero_seed() {
            let mut rng = SimRng::new(0);
            let drawn: Vec<u64> = (0..8).map(|_| rng.below(1_000_000)).collect();
            assert!(drawn.windows(2).any(|w| w[0] != w[1]));
        }
    }
}
