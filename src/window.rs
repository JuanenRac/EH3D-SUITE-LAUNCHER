//! The state of the window, without any screen: the programs found, the recent projects, the project that is open, what was last said to the person and the programs that are
//! running. The screen (`gui`, built with the `gui` feature) only draws this and passes on what the person does, so everything here is tested as plain code. Every check is the
//! one the command line makes (the same functions of the library), and an error is the message of the library.

use crate::browse::{self, Browser};
use crate::lang::{fill, Lang, Msg};
use crate::launch::{
    configured_workspace, message, resolve, supervise, Msg as LaunchMsg, Outcome, Policy,
};
use crate::project::{self, Project};
use crate::recent;
use crate::registry::{discover_with_limit, find_binary, is_known, Module};
use crate::settings::Settings;
use std::path::{Path, PathBuf};

/// The repository of this program: the window does not start itself (the file it would find is the command line).
pub const OWN_NAME: &str = "EH3D-SUITE-LAUNCHER";
/// The file of the recent projects, in the folder of the launcher.
pub const RECENT_FILE: &str = "recent.txt";
/// The extensions the dialog offers for a project and for the files of a project.
pub const PROJECT_EXTENSIONS: [&str; 1] = [project::EXTENSION];
pub const LAYER_EXTENSIONS: [&str; 22] = [
    "gbr", "gtl", "gbl", "gts", "gbs", "gto", "gbo", "gtp", "gbp", "gko", "gm1", "ger", "drl",
    "ncd", "xln", "exc", "plt", "hpgl", "nc", "gcode", "pos", "csv",
];

/// One program of the suite found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub module: Module,
    pub built: bool,
}

impl Program {
    /// Whether the button Launch is offered: a built program of this computer that is on the list of the suite (and is not this program).
    pub fn launchable(&self) -> bool {
        self.built
            && self.module.is_desktop()
            && is_known(&self.module)
            && self.module.name != OWN_NAME
    }
}

/// The project that is open: its file and what it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenProject {
    pub file: PathBuf,
    pub project: Project,
}

/// What the window last said to the person.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Ready,
    Opened {
        name: String,
        files: usize,
    },
    Created {
        file: String,
    },
    LayerAdded {
        layer: String,
        files: usize,
    },
    LayerRemoved {
        layer: String,
        files: usize,
    },
    Verified {
        name: String,
        files: usize,
    },
    VerifiedPartly {
        name: String,
        files: usize,
        unchecked: Vec<String>,
    },
    VerifyFailed {
        problems: usize,
    },
    Running {
        name: String,
    },
    Ended {
        name: String,
    },
    Signalled {
        name: String,
    },
    GaveUp {
        name: String,
        runs: u32,
        code: Option<i32>,
    },
    /// The message of the library, as it is (it may already be in the language of the person).
    Failed(String),
    NoProjectOpen,
    Busy {
        name: String,
    },
    Refreshed {
        programs: usize,
    },
    BadSettings(String),
    NeedFileName,
}

impl Status {
    /// The sentence, in the language of the person.
    pub fn text(&self, lang: Lang) -> String {
        let n = |v: usize| v.to_string();
        match self {
            Status::Ready => Msg::Ready.tr(lang).to_string(),
            Status::Opened { name, files } => {
                fill(Msg::Opened.tr(lang), &[name.as_str(), n(*files).as_str()])
            }
            Status::Created { file } => fill(Msg::Created.tr(lang), &[file.as_str()]),
            Status::LayerAdded { layer, files } => fill(
                Msg::LayerAdded.tr(lang),
                &[layer.as_str(), n(*files).as_str()],
            ),
            Status::LayerRemoved { layer, files } => fill(
                Msg::LayerRemoved.tr(lang),
                &[layer.as_str(), n(*files).as_str()],
            ),
            Status::Verified { name, files } => {
                fill(Msg::Verified.tr(lang), &[name.as_str(), n(*files).as_str()])
            }
            Status::VerifiedPartly {
                name,
                files,
                unchecked,
            } => fill(
                Msg::VerifiedPartly.tr(lang),
                &[
                    name.as_str(),
                    n(*files).as_str(),
                    n(unchecked.len()).as_str(),
                    unchecked.join(", ").as_str(),
                ],
            ),
            Status::VerifyFailed { problems } => {
                fill(Msg::VerifyFailed.tr(lang), &[n(*problems).as_str()])
            }
            Status::Running { name } => fill(Msg::Running.tr(lang), &[name.as_str()]),
            Status::Ended { name } => fill(Msg::Ended.tr(lang), &[name.as_str()]),
            Status::Signalled { name } => fill(Msg::Signalled.tr(lang), &[name.as_str()]),
            Status::GaveUp { name, runs, code } => {
                let code = code.map_or_else(|| "-".to_string(), |c| c.to_string());
                fill(
                    Msg::GaveUp.tr(lang),
                    &[name.as_str(), runs.to_string().as_str(), code.as_str()],
                )
            }
            Status::Failed(why) => fill(Msg::Failed.tr(lang), &[why.as_str()]),
            Status::NoProjectOpen => Msg::NoProjectOpen.tr(lang).to_string(),
            Status::Busy { name } => fill(Msg::Busy.tr(lang), &[name.as_str()]),
            Status::Refreshed { programs } => {
                fill(Msg::Refreshed.tr(lang), &[n(*programs).as_str()])
            }
            Status::BadSettings(why) => fill(Msg::BadSettings.tr(lang), &[why.as_str()]),
            Status::NeedFileName => Msg::NeedFileName.tr(lang).to_string(),
        }
    }

    /// Whether the sentence is a problem or a warning (the window shows it in another colour).
    pub fn is_problem(&self) -> bool {
        matches!(
            self,
            Status::VerifyFailed { .. }
                | Status::GaveUp { .. }
                | Status::Failed(_)
                | Status::NoProjectOpen
                | Status::Busy { .. }
                | Status::BadSettings(_)
                | Status::NeedFileName
        )
    }
}

/// What a dialog of files is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    OpenProject,
    NewProject,
    AddLayer,
}

/// The dialog that opens a project, asks for the file and the name of a new one, or adds a file to the open project.
#[derive(Debug, Clone)]
pub struct Dialog {
    pub kind: DialogKind,
    pub browser: Browser,
    pub file_name: String,
    pub project_name: String,
}

/// A program ready to be started: the checks have passed. It is run by `run_job`, on a thread of its own so that the window does not stop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchJob {
    pub name: String,
    pub bin: PathBuf,
    pub args: Vec<String>,
    pub policy: Policy,
}

/// Starts the program of a job and waits for it, the way `eh3d-launcher launch` does (a program that crashed is started again as many times as the settings say).
pub fn run_job(job: &LaunchJob) -> Result<Outcome, String> {
    supervise(&job.bin, &job.args, job.policy)
}

pub struct Window {
    pub lang: Lang,
    pub settings: Settings,
    /// The folder of the launcher: `launcher.cfg` and the list of recent projects live here.
    pub data_dir: PathBuf,
    pub workspace: Option<PathBuf>,
    /// Why the folder of the programs is not known (the message of the library, in the language of the person).
    pub workspace_error: Option<String>,
    pub programs: Vec<Program>,
    /// The folders of the workspace whose manifest could not be read, each with its reason.
    pub skipped: Vec<String>,
    pub recent: Vec<String>,
    pub project: Option<OpenProject>,
    /// The lines that go with the last status: the problems found by verifying.
    pub details: Vec<String>,
    pub status: Status,
    pub show_all: bool,
    pub with_project: bool,
    /// The programs that are running now.
    pub running: Vec<String>,
    /// True in the window of a picture: nothing is read from or written to the files of the person (no `launcher.cfg`, no recent projects).
    ephemeral: bool,
    /// The folder of the programs of the window of a picture, taken from its arguments and not from `launcher.cfg`.
    fixed_workspace: Option<PathBuf>,
}

impl Window {
    /// A window for the launcher whose files are in `data_dir`. The language is the one of the settings, or the one of the system.
    pub fn new(data_dir: PathBuf) -> Window {
        let (settings, bad) = match Settings::load(&data_dir) {
            Ok(s) => (s, None),
            Err(e) => (Settings::default(), Some(e)),
        };
        let lang = settings.language.unwrap_or_else(Lang::from_env);
        let mut w = Window {
            lang,
            settings,
            data_dir,
            workspace: None,
            workspace_error: None,
            programs: Vec::new(),
            skipped: Vec::new(),
            recent: Vec::new(),
            project: None,
            details: Vec::new(),
            status: Status::Ready,
            show_all: false,
            with_project: true,
            running: Vec::new(),
            ephemeral: false,
            fixed_workspace: None,
        };
        w.reload();
        if let Some(e) = bad {
            w.status = Status::BadSettings(e);
        }
        w
    }

    /// The window of a picture: the default settings, the folder of the programs and the project list that are given (either may be missing) and a list of recent projects that
    /// lives only in memory, so that nothing of the person's files is read or changed. The paths stay as they were given.
    pub fn for_shot(workspace: Option<PathBuf>, project: Option<PathBuf>, lang: Lang) -> Window {
        let mut w = Window {
            lang,
            settings: Settings::default(),
            data_dir: PathBuf::new(),
            workspace: None,
            workspace_error: None,
            programs: Vec::new(),
            skipped: Vec::new(),
            recent: Vec::new(),
            project: None,
            details: Vec::new(),
            status: Status::Ready,
            show_all: false,
            with_project: true,
            running: Vec::new(),
            ephemeral: true,
            fixed_workspace: workspace,
        };
        w.reload();
        if let Some(file) = project {
            w.open_project(&file);
        }
        w
    }

    fn recent_path(&self) -> PathBuf {
        self.data_dir.join(RECENT_FILE)
    }

    fn fail(&mut self, why: String) -> bool {
        self.details.clear();
        self.status = Status::Failed(why);
        false
    }

    /// Reads the programs of the workspace and the recent projects again.
    fn reload(&mut self) {
        let found = if self.ephemeral {
            self.fixed_workspace.clone().ok_or_else(|| {
                message(
                    LaunchMsg::NoWorkspace,
                    self.lang.code(),
                    crate::launch::CONFIG_FILE,
                )
            })
        } else {
            self.recent =
                recent::load_with_limit(&self.recent_path(), self.settings.max_file_bytes());
            configured_workspace(&self.data_dir, self.lang.code())
        };
        match found {
            Ok(ws) => {
                let (modules, problems) = discover_with_limit(&ws, self.settings.max_file_bytes());
                self.programs = modules
                    .into_iter()
                    .map(|module| Program {
                        built: find_binary(&module).is_some(),
                        module,
                    })
                    .collect();
                self.skipped = problems;
                self.workspace = Some(ws);
                self.workspace_error = None;
            }
            Err(e) => {
                self.programs.clear();
                self.skipped.clear();
                self.workspace = None;
                self.workspace_error = Some(e);
            }
        }
    }

    /// The button Refresh: the programs are looked for again (a program may have been built since).
    pub fn refresh(&mut self) {
        self.reload();
        self.details.clear();
        self.status = Status::Refreshed {
            programs: self.visible_programs().len(),
        };
    }

    /// The places in `programs` of the programs that are listed: the ones for this computer, or all of them when `show_all` is on.
    pub fn visible_programs(&self) -> Vec<usize> {
        self.programs
            .iter()
            .enumerate()
            .filter(|(_, p)| self.show_all || p.module.is_desktop())
            .map(|(i, _)| i)
            .collect()
    }

    /// The folder where a dialog starts: the one of the open project, else the one of the latest recent project, else the current one.
    fn start_folder(&self) -> PathBuf {
        let from_file = |f: &Path| {
            f.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_path_buf)
        };
        self.project
            .as_ref()
            .and_then(|p| from_file(&p.file))
            .or_else(|| self.recent.first().and_then(|r| from_file(Path::new(r))))
            .unwrap_or_else(browse::start_dir)
    }

    /// A dialog of the given kind, ready to be drawn.
    pub fn dialog(&self, kind: DialogKind) -> Dialog {
        let folder = self.start_folder();
        let browser = match kind {
            DialogKind::OpenProject | DialogKind::NewProject => {
                Browser::new(folder, &PROJECT_EXTENSIONS, false)
            }
            DialogKind::AddLayer => Browser::new(folder, &LAYER_EXTENSIONS, true),
        };
        Dialog {
            kind,
            browser,
            file_name: String::new(),
            project_name: String::new(),
        }
    }

    /// The person chose in a dialog (`picked` is the file chosen in the list, when there is one). Returns true when the dialog is done and should close; when the action is
    /// refused the status says why and the dialog stays so that another try is possible.
    pub fn accept(&mut self, dialog: &Dialog, picked: Option<PathBuf>) -> bool {
        match dialog.kind {
            DialogKind::OpenProject => picked.is_some_and(|p| self.open_project(&p)),
            DialogKind::AddLayer => picked.is_some_and(|p| self.add_layer(&p)),
            DialogKind::NewProject => {
                let name = dialog.file_name.trim();
                if name.is_empty() {
                    self.details.clear();
                    self.status = Status::NeedFileName;
                    return false;
                }
                let mut file = dialog.browser.dir.join(name);
                // a name with no extension at all is a project list; a name with another extension is refused by the same check as the command line
                if file.extension().is_none() {
                    file.set_extension(project::EXTENSION);
                }
                self.new_project(&file, &dialog.project_name)
            }
        }
    }

    /// Opens a project list and puts it at the head of the recent ones.
    pub fn open_project(&mut self, file: &Path) -> bool {
        let full = if self.ephemeral {
            // the window of a picture shows the path as it was given
            if !file.is_file() {
                return self.fail(format!("{}: not a file", file.display()));
            }
            file.to_string_lossy().into_owned()
        } else {
            match recent::canonical(&file.to_string_lossy()) {
                Ok(f) => f,
                Err(e) => return self.fail(e),
            }
        };
        let loaded = match Project::load(Path::new(&full)) {
            Ok(p) => p,
            Err(e) => return self.fail(e),
        };
        self.details.clear();
        self.status = Status::Opened {
            name: loaded.name.clone(),
            files: loaded.layers.len(),
        };
        self.project = Some(OpenProject {
            file: PathBuf::from(&full),
            project: loaded,
        });
        if self.ephemeral {
            recent::remember(&mut self.recent, &full);
        } else {
            match recent::add_with_limit(&self.recent_path(), &full, self.settings.max_file_bytes())
            {
                Ok(list) => self.recent = list,
                Err(e) => {
                    self.status = Status::Failed(e);
                }
            }
        }
        true
    }

    /// Opens the recent project at `index` of the list.
    pub fn open_recent(&mut self, index: usize) -> bool {
        match self.recent.get(index).cloned() {
            Some(file) => self.open_project(Path::new(&file)),
            None => false,
        }
    }

    /// Writes an empty project list (the checks of `new-project`) and opens it.
    pub fn new_project(&mut self, file: &Path, name: &str) -> bool {
        if let Err(e) = project::create_file(&file.to_string_lossy(), name) {
            return self.fail(e);
        }
        if !self.open_project(file) {
            return false;
        }
        if !self.status.is_problem() {
            self.status = Status::Created {
                file: file.display().to_string(),
            };
        }
        true
    }

    /// Reads the open project again from its file.
    fn reload_project(&mut self) -> bool {
        let Some(open) = &self.project else {
            return false;
        };
        match Project::load(&open.file) {
            Ok(p) => {
                if let Some(open) = self.project.as_mut() {
                    open.project = p;
                }
                true
            }
            Err(e) => self.fail(e),
        }
    }

    /// Adds a file to the open project (the checks of `add-layer`: it must lie under the folder of the project and not be in it yet).
    pub fn add_layer(&mut self, layer: &Path) -> bool {
        let Some(open) = &self.project else {
            self.details.clear();
            self.status = Status::NoProjectOpen;
            return false;
        };
        let file = open.file.clone();
        match project::add_layer_to_file(&file, layer) {
            Ok(files) => {
                if !self.reload_project() {
                    return false;
                }
                self.details.clear();
                self.status = Status::LayerAdded {
                    layer: layer.display().to_string(),
                    files,
                };
                true
            }
            Err(e) => self.fail(e),
        }
    }

    /// Takes the file at `index` out of the open project (the file itself is not touched).
    pub fn remove_layer(&mut self, index: usize) -> bool {
        let Some(open) = &self.project else {
            self.details.clear();
            self.status = Status::NoProjectOpen;
            return false;
        };
        let Some(layer) = open.project.layers.get(index).map(|l| l.path.clone()) else {
            return false;
        };
        let file = open.file.clone();
        match project::remove_layer_from_file(&file, &layer) {
            Ok(files) => {
                if !self.reload_project() {
                    return false;
                }
                self.details.clear();
                self.status = Status::LayerRemoved { layer, files };
                true
            }
            Err(e) => self.fail(e),
        }
    }

    /// Checks the files of the open project against the disk (`verify`).
    pub fn verify(&mut self) {
        let Some(open) = &self.project else {
            self.details.clear();
            self.status = Status::NoProjectOpen;
            return;
        };
        let root = open
            .file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let problems = open.project.verify(root);
        let name = open.project.name.clone();
        let files = open.project.layers.len();
        let unchecked: Vec<String> = open
            .project
            .unchecked()
            .into_iter()
            .map(str::to_string)
            .collect();
        if !problems.is_empty() {
            self.status = Status::VerifyFailed {
                problems: problems.len(),
            };
            self.details = problems;
        } else if unchecked.is_empty() {
            self.details.clear();
            self.status = Status::Verified { name, files };
        } else {
            self.details.clear();
            self.status = Status::VerifiedPartly {
                name,
                files,
                unchecked,
            };
        }
    }

    /// The button Launch of the program at `index` of `programs`: every check of `launch` is made, and a job is given back when they pass. The caller runs it with `run_job` on
    /// a thread and reports the end with `finish_launch`. With the box "with the project" on and a project open, the file of the project is the argument of the program.
    pub fn begin_launch(&mut self, index: usize) -> Option<LaunchJob> {
        let module = self.programs.get(index)?.module.clone();
        if self.running.contains(&module.name) {
            self.details.clear();
            self.status = Status::Busy { name: module.name };
            return None;
        }
        let Some(workspace) = self.workspace.clone() else {
            let why = self.workspace_error.clone().unwrap_or_default();
            self.fail(why);
            return None;
        };
        let lang = self.lang.code();
        let found = if module.name == OWN_NAME {
            Err(message(LaunchMsg::UnknownProgram, lang, &module.name))
        } else {
            resolve(&module, &workspace, lang)
        };
        let bin = match found {
            Ok(bin) => bin,
            Err(e) => {
                self.fail(e);
                return None;
            }
        };
        let args = match (&self.project, self.with_project) {
            (Some(open), true) => vec![open.file.to_string_lossy().into_owned()],
            _ => Vec::new(),
        };
        self.running.push(module.name.clone());
        self.details.clear();
        self.status = Status::Running {
            name: module.name.clone(),
        };
        Some(LaunchJob {
            name: module.name,
            bin,
            args,
            policy: self.settings.policy(),
        })
    }

    /// The program of a job ended (or could not be started): the window says how it went.
    pub fn finish_launch(&mut self, name: &str, result: Result<Outcome, String>) {
        self.running.retain(|n| n != name);
        self.details.clear();
        let name = name.to_string();
        self.status = match result {
            Err(e) => Status::Failed(e),
            Ok(o) if o.signalled => Status::Signalled { name },
            Ok(o) if o.gave_up => Status::GaveUp {
                name,
                runs: o.runs,
                code: o.last_code,
            },
            Ok(_) => Status::Ended { name },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("eh3d-window-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn manifest(dir: &Path, name: &str, role: &str, target: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("eh3d.project.json"),
            format!(
                r#"{{"ecosystem":"EH3D","name":"{name}","version":"0.1.2","role":"{role}","maturity":"scaffolding","deployment_target":"{target}"}}"#
            ),
        )
        .unwrap();
    }

    /// The program file that a build leaves in the `target` folder of a repository.
    fn build(ws: &Path, repo: &str, program: &str) {
        let dir = ws.join(repo).join("target").join("release");
        fs::create_dir_all(&dir).unwrap();
        let suffix = if cfg!(windows) { ".exe" } else { "" };
        fs::write(dir.join(format!("{program}{suffix}")), "x").unwrap();
    }

    /// A workspace with four repositories, the folder of the launcher whose configuration names it, and a folder for projects.
    fn setup(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let base = scratch(tag);
        let (ws, data, jobs) = (base.join("ws"), base.join("data"), base.join("jobs"));
        for d in [&ws, &data, &jobs] {
            fs::create_dir_all(d).unwrap();
        }
        manifest(
            &ws.join("EH3D-DRILLPRO"),
            "EH3D-DRILLPRO",
            "tool",
            "user-pc",
        );
        manifest(&ws.join("EH3D-NODE-Y"), "EH3D-NODE-Y", "firmware", "mcu");
        manifest(&ws.join("EH3D-SUITE-LAUNCHER"), OWN_NAME, "tool", "user-pc");
        manifest(&ws.join("EH3D-TOOLX"), "EH3D-TOOLX", "tool", "user-pc");
        fs::write(
            data.join("launcher.cfg"),
            format!("workspace = {}\n", ws.display()),
        )
        .unwrap();
        (ws, data, jobs)
    }

    fn index_of(w: &Window, name: &str) -> usize {
        w.programs
            .iter()
            .position(|p| p.module.name == name)
            .unwrap()
    }

    fn english(data: &Path) -> Window {
        let mut w = Window::new(data.to_path_buf());
        w.lang = Lang::En;
        w
    }

    fn failure(w: &Window) -> String {
        match &w.status {
            Status::Failed(why) => why.clone(),
            other => panic!("expected a failure, the status is {other:?}"),
        }
    }

    #[test]
    fn the_window_finds_the_programs_and_marks_the_built_ones() {
        let (ws, data, _) = setup("find");
        build(&ws, "EH3D-DRILLPRO", "drillpro");
        let mut w = english(&data);
        assert_eq!(w.workspace.as_deref(), Some(ws.as_path()));
        assert_eq!(w.programs.len(), 4);
        let names: Vec<&str> = w.programs.iter().map(|p| p.module.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "EH3D-DRILLPRO",
                "EH3D-NODE-Y",
                "EH3D-SUITE-LAUNCHER",
                "EH3D-TOOLX"
            ]
        );
        assert!(w.programs[0].built && !w.programs[3].built);
        assert!(w.programs[0].launchable());
        assert!(!w.programs[3].launchable());
        // the firmware is listed only when the person asks for everything
        assert_eq!(w.visible_programs(), [0, 2, 3]);
        w.show_all = true;
        assert_eq!(w.visible_programs(), [0, 1, 2, 3]);
    }

    #[test]
    fn a_missing_configuration_is_told_and_nothing_is_listed() {
        let base = scratch("nocfg");
        let w = Window::new(base);
        assert!(w.workspace.is_none() && w.programs.is_empty());
        let why = w.workspace_error.clone().unwrap();
        assert!(why.contains("launcher.cfg"), "{why}");
    }

    #[test]
    fn the_settings_choose_the_language_and_a_bad_one_is_told() {
        let (ws, data, _) = setup("settings");
        fs::write(
            data.join("launcher.cfg"),
            format!(
                "workspace = {}\nlanguage = de\nmax_restarts = 1\n",
                ws.display()
            ),
        )
        .unwrap();
        let w = Window::new(data.clone());
        assert_eq!(w.lang, Lang::De);
        assert_eq!(w.settings.policy().max_restarts, 1);
        fs::write(
            data.join("launcher.cfg"),
            format!("workspace = {}\nmax_restarts = 99\n", ws.display()),
        )
        .unwrap();
        let w = Window::new(data);
        assert!(matches!(w.status, Status::BadSettings(_)) && w.status.is_problem());
        assert_eq!(w.settings, Settings::default());
        assert_eq!(w.programs.len(), 4, "the programs are still found");
    }

    #[test]
    fn a_new_project_is_written_opened_and_remembered() {
        let (_, data, jobs) = setup("create");
        let mut w = english(&data);
        let file = jobs.join("board.eh3dlist");
        assert!(w.new_project(&file, "Board"));
        assert!(file.is_file());
        assert_eq!(w.project.as_ref().unwrap().project.name, "Board");
        assert!(matches!(w.status, Status::Created { .. }));
        assert_eq!(w.recent.len(), 1);
        assert!(w.recent[0].ends_with("board.eh3dlist"));
        assert_eq!(recent::load(&data.join(RECENT_FILE)), w.recent);
        // the same refusals as the command line, with its messages
        assert!(!w.new_project(&file, "Again"));
        assert!(failure(&w).contains("already exists"));
        assert!(!w.new_project(&jobs.join("x.txt"), "X"));
        assert!(failure(&w).contains("a project list ends in"));
        assert!(!w.new_project(&jobs.join("y.eh3dlist"), "  "));
        assert!(failure(&w).contains("empty"));
        assert!(!jobs.join("y.eh3dlist").exists());
    }

    #[test]
    fn the_dialog_of_a_new_project_gives_the_name_an_extension() {
        let (_, data, jobs) = setup("dialog");
        let mut w = english(&data);
        let mut d = w.dialog(DialogKind::NewProject);
        d.browser.go(jobs.clone());
        assert!(!w.accept(&d, None));
        assert_eq!(w.status, Status::NeedFileName);
        d.file_name = "job".to_string();
        d.project_name = "Job".to_string();
        assert!(w.accept(&d, None));
        assert!(jobs.join("job.eh3dlist").is_file());
        // a dialog that opens a file does nothing until a file is chosen
        let open = w.dialog(DialogKind::OpenProject);
        assert!(!w.accept(&open, None));
        assert!(w.accept(&open, Some(jobs.join("job.eh3dlist"))));
        // the dialog of the files of a project shows every file
        assert!(w.dialog(DialogKind::AddLayer).browser.show_all);
        assert!(!open.browser.show_all);
    }

    #[test]
    fn files_are_added_and_removed_with_the_checks_of_the_command_line() {
        let (ws, data, jobs) = setup("layers");
        let mut w = english(&data);
        assert!(!w.add_layer(&jobs.join("top.gtl")));
        assert_eq!(w.status, Status::NoProjectOpen);
        assert!(w.new_project(&jobs.join("p.eh3dlist"), "P"));
        let top = jobs.join("top.gtl");
        fs::write(&top, "G04 test*").unwrap();
        assert!(w.add_layer(&top));
        let layers = &w.project.as_ref().unwrap().project.layers;
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].path, "top.gtl");
        assert!(!layers[0].sha256.is_empty());
        assert!(matches!(w.status, Status::LayerAdded { files: 1, .. }));
        // twice, outside the folder of the project, and a file that is not there are refused
        assert!(!w.add_layer(&top));
        assert!(failure(&w).contains("already in the project"));
        let outside = ws.join("other.gtl");
        fs::write(&outside, "x").unwrap();
        assert!(!w.add_layer(&outside));
        assert!(failure(&w).contains("not under the folder of the project"));
        assert!(!w.add_layer(&jobs.join("nothing.gtl")));
        assert!(failure(&w).contains("nothing.gtl"));
        // taking a file out
        assert!(!w.remove_layer(5));
        assert!(w.remove_layer(0));
        assert!(matches!(w.status, Status::LayerRemoved { files: 0, .. }));
        assert!(w.project.as_ref().unwrap().project.layers.is_empty());
        let on_disk = Project::load(&jobs.join("p.eh3dlist")).unwrap();
        assert!(on_disk.layers.is_empty());
        assert!(top.is_file(), "the file itself is not touched");
    }

    #[test]
    fn verifying_tells_a_changed_and_a_missing_file() {
        let (_, data, jobs) = setup("verify");
        let mut w = english(&data);
        w.verify();
        assert_eq!(w.status, Status::NoProjectOpen);
        assert!(w.new_project(&jobs.join("p.eh3dlist"), "P"));
        let top = jobs.join("top.gtl");
        fs::write(&top, "one").unwrap();
        assert!(w.add_layer(&top));
        w.verify();
        assert!(matches!(w.status, Status::Verified { files: 1, .. }));
        assert!(w.details.is_empty());
        fs::write(&top, "two").unwrap();
        w.verify();
        assert_eq!(w.status, Status::VerifyFailed { problems: 1 });
        assert!(w.status.is_problem() && w.details[0].contains("changed"));
        fs::remove_file(&top).unwrap();
        w.verify();
        assert!(w.details[0].contains("missing"));
    }

    #[test]
    fn a_recent_project_opens_again_and_a_vanished_one_is_told() {
        let (_, data, jobs) = setup("recent");
        let file = jobs.join("p.eh3dlist");
        assert!(english(&data).new_project(&file, "P"));
        let mut w = english(&data);
        assert_eq!(w.recent.len(), 1);
        assert!(!w.open_recent(9));
        assert!(w.open_recent(0));
        assert_eq!(w.project.as_ref().unwrap().project.name, "P");
        assert!(matches!(w.status, Status::Opened { files: 0, .. }));
        fs::remove_file(&file).unwrap();
        assert!(!w.open_recent(0));
        assert!(w.status.is_problem());
    }

    #[test]
    fn a_program_is_launched_with_the_project_and_only_once_at_a_time() {
        let (ws, data, jobs) = setup("launch");
        build(&ws, "EH3D-DRILLPRO", "drillpro");
        build(&ws, "EH3D-SUITE-LAUNCHER", "suite-launcher");
        let mut w = english(&data);
        let drill = index_of(&w, "EH3D-DRILLPRO");
        let job = w.begin_launch(drill).unwrap();
        assert_eq!(job.name, "EH3D-DRILLPRO");
        assert!(job.args.is_empty(), "no project is open");
        assert_eq!(job.policy, Policy::default());
        assert_eq!(w.running, ["EH3D-DRILLPRO"]);
        assert!(matches!(w.status, Status::Running { .. }));
        assert!(w.begin_launch(drill).is_none());
        assert!(matches!(w.status, Status::Busy { .. }));
        let done = Outcome {
            runs: 1,
            last_code: Some(0),
            gave_up: false,
            signalled: false,
        };
        w.finish_launch("EH3D-DRILLPRO", Ok(done.clone()));
        assert!(w.running.is_empty());
        assert!(matches!(w.status, Status::Ended { .. }) && !w.status.is_problem());
        // with a project open its file is the argument, unless the box is off
        assert!(w.new_project(&jobs.join("p.eh3dlist"), "P"));
        let file = w
            .project
            .as_ref()
            .unwrap()
            .file
            .to_string_lossy()
            .into_owned();
        let job = w.begin_launch(drill).unwrap();
        assert_eq!(job.args, [file]);
        w.finish_launch("EH3D-DRILLPRO", Ok(done));
        w.with_project = false;
        assert!(w.begin_launch(drill).unwrap().args.is_empty());
    }

    #[test]
    fn a_program_that_should_not_start_is_refused_with_the_reason_of_the_library() {
        let (ws, data, _) = setup("refuse");
        build(&ws, "EH3D-SUITE-LAUNCHER", "suite-launcher");
        build(&ws, "EH3D-TOOLX", "toolx");
        let mut w = english(&data);
        assert!(w.begin_launch(99).is_none());
        // not built
        assert!(w.begin_launch(index_of(&w, "EH3D-DRILLPRO")).is_none());
        assert!(failure(&w).contains("has not been built"));
        // not a program for this computer
        assert!(w.begin_launch(index_of(&w, "EH3D-NODE-Y")).is_none());
        assert!(failure(&w).contains("not a program for this computer"));
        // built but not on the list of the suite
        assert!(w.begin_launch(index_of(&w, "EH3D-TOOLX")).is_none());
        assert!(failure(&w).contains("not a known program of the suite"));
        // the window does not start itself
        let own = index_of(&w, OWN_NAME);
        assert!(!w.programs[own].launchable());
        assert!(w.begin_launch(own).is_none());
        assert!(w.running.is_empty());
        // the refusal comes in the language of the person
        w.lang = Lang::Es;
        assert!(w.begin_launch(index_of(&w, "EH3D-TOOLX")).is_none());
        assert!(failure(&w).contains("no es un programa conocido"));
    }

    #[test]
    fn the_end_of_a_program_is_told_with_its_code() {
        let (_, data, _) = setup("finish");
        let mut w = english(&data);
        let gave_up = Outcome {
            runs: 3,
            last_code: Some(139),
            gave_up: true,
            signalled: false,
        };
        w.finish_launch("EH3D-DRILLPRO", Ok(gave_up));
        let text = w.status.text(Lang::En);
        assert!(
            text.contains("failed 3 times") && text.contains("139"),
            "{text}"
        );
        assert!(w.status.is_problem());
        let signalled = Outcome {
            runs: 1,
            last_code: None,
            gave_up: false,
            signalled: true,
        };
        w.finish_launch("EH3D-DRILLPRO", Ok(signalled));
        assert!(matches!(w.status, Status::Signalled { .. }) && !w.status.is_problem());
        w.finish_launch("EH3D-DRILLPRO", Err("x: no such file".to_string()));
        assert!(w.status.text(Lang::En).contains("x: no such file"));
    }

    #[test]
    fn every_status_has_a_sentence_in_each_language() {
        let statuses = [
            Status::Ready,
            Status::Opened {
                name: "n".into(),
                files: 2,
            },
            Status::Created { file: "f".into() },
            Status::LayerAdded {
                layer: "l".into(),
                files: 1,
            },
            Status::LayerRemoved {
                layer: "l".into(),
                files: 0,
            },
            Status::Verified {
                name: "n".into(),
                files: 3,
            },
            Status::VerifiedPartly {
                name: "n".into(),
                files: 3,
                unchecked: vec!["a".into(), "b".into()],
            },
            Status::VerifyFailed { problems: 2 },
            Status::Running { name: "p".into() },
            Status::Ended { name: "p".into() },
            Status::Signalled { name: "p".into() },
            Status::GaveUp {
                name: "p".into(),
                runs: 3,
                code: None,
            },
            Status::Failed("why".into()),
            Status::NoProjectOpen,
            Status::Busy { name: "p".into() },
            Status::Refreshed { programs: 4 },
            Status::BadSettings("why".into()),
            Status::NeedFileName,
        ];
        for s in &statuses {
            for l in Lang::ALL {
                let text = s.text(l);
                assert!(
                    !text.is_empty() && !text.contains('{'),
                    "{s:?} in {}",
                    l.code()
                );
            }
        }
    }

    #[test]
    fn refreshing_sees_a_program_that_was_built_since() {
        let (ws, data, _) = setup("refresh");
        let mut w = english(&data);
        let drill = index_of(&w, "EH3D-DRILLPRO");
        assert!(!w.programs[drill].built);
        build(&ws, "EH3D-DRILLPRO", "drillpro");
        w.refresh();
        assert!(w.programs[drill].built);
        assert_eq!(w.status, Status::Refreshed { programs: 3 });
    }
}
