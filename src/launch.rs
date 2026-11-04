//! Starting a program of the suite and keeping it alive: a program that dies with an error is started again a limited number of times, one that ends well is left alone.

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
    let mut runs = 0;
    let mut wait = policy.backoff_ms;
    loop {
        runs += 1;
        let status = Command::new(program)
            .args(args)
            .status()
            .map_err(|e| format!("{}: {e}", program.display()))?;
        let code = status.code();
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
        sleep(Duration::from_millis(wait));
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
    let text = std::fs::read_to_string(&file).map_err(|_| refused())?;
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
