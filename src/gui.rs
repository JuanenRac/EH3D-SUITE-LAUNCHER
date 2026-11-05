//! The window itself (built with the `gui` feature): it draws the state of `window::Window` and passes on what the person does. Everything that can be decided without a screen is
//! decided there, in code that is tested. A program is started on a thread of its own, so the window keeps answering while the program runs.

use crate::browse;
use crate::lang::{fill, Lang, Msg};
use crate::launch::Outcome;
use crate::settings::MIN_WINDOW;
use crate::window::{run_job, Dialog, DialogKind, Window};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Stroke};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

// the colours of the theme Forge, the one the other programs of the suite start with: amber on charcoal
const PANEL: Color32 = Color32::from_rgb(0x17, 0x13, 0x10);
const RAISED: Color32 = Color32::from_rgb(0x22, 0x1c, 0x17);
const TEXT: Color32 = Color32::from_rgb(0xf4, 0xec, 0xe0);
const DIM: Color32 = Color32::from_rgb(0x9c, 0x8d, 0x7a);
const ACCENT: Color32 = Color32::from_rgb(0xff, 0xb0, 0x00);
const OK: Color32 = Color32::from_rgb(0x9b, 0xe3, 0x4a);
const DANGER: Color32 = Color32::from_rgb(0xff, 0x54, 0x70);

/// What a program that ended tells the window: its name and how it went.
type Finished = (String, Result<Outcome, String>);

/// What the person did in a dialog.
enum DialogResult {
    Cancel,
    Accept(Option<PathBuf>),
}

struct App {
    win: Window,
    dialog: Option<Dialog>,
    tx: Sender<Finished>,
    rx: Receiver<Finished>,
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().color(ACCENT));
}

impl App {
    fn new(win: Window) -> App {
        let (tx, rx) = channel();
        App {
            win,
            dialog: None,
            tx,
            rx,
        }
    }

    fn open_dialog(&mut self, kind: DialogKind) {
        self.dialog = Some(self.win.dialog(kind));
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("EH3D-SUITE-LAUNCHER")
                        .strong()
                        .size(18.0)
                        .color(ACCENT),
                );
                ui.separator();
                if ui.button(Msg::NewProject.tr(lang)).clicked() {
                    self.open_dialog(DialogKind::NewProject);
                }
                if ui.button(Msg::OpenProject.tr(lang)).clicked() {
                    self.open_dialog(DialogKind::OpenProject);
                }
                if ui.button(Msg::Refresh.tr(lang)).clicked() {
                    self.win.refresh();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(self.win.lang.name())
                        .show_ui(ui, |ui| {
                            for l in Lang::ALL {
                                ui.selectable_value(&mut self.win.lang, l, l.name());
                            }
                        });
                    ui.label(egui::RichText::new(Msg::Language.tr(lang)).color(DIM));
                });
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::Panel::bottom("status").show(ui, |ui| {
            let colour = if self.win.status.is_problem() {
                DANGER
            } else {
                OK
            };
            ui.label(egui::RichText::new(self.win.status.text(lang)).color(colour));
        });
    }

    fn recent_panel(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::Panel::left("recent")
            .default_size(260.0)
            .show(ui, |ui| {
                ui.add_space(4.0);
                heading(ui, Msg::RecentProjects.tr(lang));
                ui.separator();
                if self.win.recent.is_empty() {
                    ui.label(egui::RichText::new(Msg::NoRecent.tr(lang)).color(DIM));
                }
                let mut open = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, entry) in self.win.recent.iter().enumerate() {
                        let name = Path::new(entry)
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| entry.clone());
                        if ui
                            .selectable_label(false, name)
                            .on_hover_text(entry.as_str())
                            .clicked()
                        {
                            open = Some(i);
                        }
                    }
                });
                if let Some(i) = open {
                    self.win.open_recent(i);
                }
            });
    }

    fn project_section(&mut self, ui: &mut egui::Ui, lang: Lang) {
        heading(ui, Msg::CurrentProject.tr(lang));
        ui.separator();
        let Some(open) = &self.win.project else {
            ui.label(egui::RichText::new(Msg::NoProjectHint.tr(lang)).color(DIM));
            return;
        };
        let title = format!("{} - {}", open.project.name, open.file.display());
        let rows: Vec<(String, String)> = open
            .project
            .layers
            .iter()
            .map(|l| (l.path.clone(), l.kind.key().to_string()))
            .collect();
        ui.label(egui::RichText::new(title).strong());
        let (mut add, mut verify, mut remove) = (false, false, None);
        ui.horizontal(|ui| {
            add = ui.button(Msg::AddLayer.tr(lang)).clicked();
            verify = ui.button(Msg::VerifyFiles.tr(lang)).clicked();
            ui.checkbox(&mut self.win.with_project, Msg::WithProject.tr(lang));
        });
        if rows.is_empty() {
            ui.label(egui::RichText::new(Msg::NoLayers.tr(lang)).color(DIM));
        }
        ui.label(egui::RichText::new(Msg::Layers.tr(lang)).color(DIM));
        for (i, (path, kind)) in rows.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(path.as_str());
                ui.label(egui::RichText::new(kind.as_str()).color(DIM));
                if ui.small_button(Msg::Remove.tr(lang)).clicked() {
                    remove = Some(i);
                }
            });
        }
        if add {
            self.open_dialog(DialogKind::AddLayer);
        }
        if verify {
            self.win.verify();
        }
        if let Some(i) = remove {
            self.win.remove_layer(i);
        }
    }

    fn programs_section(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Lang) {
        heading(ui, Msg::Programs.tr(lang));
        ui.separator();
        ui.checkbox(&mut self.win.show_all, Msg::ShowAll.tr(lang));
        if let Some(ws) = &self.win.workspace {
            let folder = ws.display().to_string();
            ui.label(
                egui::RichText::new(fill(Msg::Workspace.tr(lang), &[folder.as_str()])).color(DIM),
            );
        }
        if let Some(why) = &self.win.workspace_error {
            ui.colored_label(DANGER, why.as_str());
        }
        let rows = self.win.visible_programs();
        if rows.is_empty() && self.win.workspace.is_some() {
            ui.label(egui::RichText::new(Msg::NoPrograms.tr(lang)).color(DIM));
        }
        let mut launch = None;
        egui::Grid::new("programs")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                ui.label("");
                for head in [Msg::Version, Msg::Maturity, Msg::State] {
                    ui.label(egui::RichText::new(head.tr(lang)).color(DIM));
                }
                ui.label("");
                ui.end_row();
                for &i in &rows {
                    let p = &self.win.programs[i];
                    ui.label(p.module.name.as_str());
                    ui.label(p.module.version.as_str());
                    ui.label(p.module.maturity.as_str());
                    let (word, colour) = if p.built {
                        (Msg::Built, OK)
                    } else {
                        (Msg::NotBuilt, DIM)
                    };
                    ui.label(egui::RichText::new(word.tr(lang)).color(colour));
                    let free = !self.win.running.contains(&p.module.name);
                    if ui
                        .add_enabled(
                            p.launchable() && free,
                            egui::Button::new(Msg::Launch.tr(lang)),
                        )
                        .clicked()
                    {
                        launch = Some(i);
                    }
                    ui.end_row();
                }
            });
        if let Some(job) = launch.and_then(|i| self.win.begin_launch(i)) {
            let tx = self.tx.clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let result = run_job(&job);
                let _ = tx.send((job.name.clone(), result));
                ctx.request_repaint();
            });
        }
    }

    fn details_section(&self, ui: &mut egui::Ui, lang: Lang) {
        if !self.win.details.is_empty() {
            ui.add_space(8.0);
            heading(ui, Msg::Details.tr(lang));
            ui.separator();
            for line in &self.win.details {
                ui.colored_label(DANGER, line.as_str());
            }
        }
        if !self.win.skipped.is_empty() {
            ui.add_space(8.0);
            heading(ui, Msg::Skipped.tr(lang));
            ui.separator();
            for line in &self.win.skipped {
                ui.label(egui::RichText::new(line.as_str()).color(DIM));
            }
        }
    }

    fn centre(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.project_section(ui, lang);
                ui.add_space(12.0);
                self.programs_section(ui, &ctx, lang);
                self.details_section(ui, lang);
            });
        });
    }

    fn dialog_window(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        match show_dialog(ctx, &mut dialog, self.win.lang) {
            None => self.dialog = Some(dialog),
            Some(DialogResult::Cancel) => {}
            Some(DialogResult::Accept(picked)) => {
                // a refused action says why in the status bar and the dialog stays so that another try is possible
                if !self.win.accept(&dialog, picked) {
                    self.dialog = Some(dialog);
                }
            }
        }
    }
}

/// Draws the dialog that opens a file, adds one to the project or asks for the file and the name of a new project, and says what the person chose.
fn show_dialog(ctx: &egui::Context, d: &mut Dialog, lang: Lang) -> Option<DialogResult> {
    let mut result = None;
    let mut open = true;
    let title = match d.kind {
        DialogKind::OpenProject => Msg::OpenTitle,
        DialogKind::NewProject => Msg::NewTitle,
        DialogKind::AddLayer => Msg::AddTitle,
    };
    let kind = d.kind;
    egui::Window::new(title.tr(lang))
        .open(&mut open)
        .collapsible(false)
        .default_size([620.0, 460.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(Msg::Up.tr(lang)).clicked() {
                    d.browser.up();
                }
                let edit = egui::TextEdit::singleline(&mut d.browser.path_text)
                    .desired_width(ui.available_width());
                let response = ui.add(edit);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(p) = d.browser.go_typed() {
                        if kind != DialogKind::NewProject {
                            result = Some(DialogResult::Accept(Some(p)));
                        }
                    }
                }
            });
            if let Some(e) = &d.browser.error {
                ui.colored_label(DANGER, e.as_str());
            }
            if ui
                .checkbox(&mut d.browser.show_all, Msg::AllFiles.tr(lang))
                .changed()
            {
                d.browser.refresh();
            }
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    if d.browser.entries.is_empty() {
                        ui.label(egui::RichText::new(Msg::EmptyFolder.tr(lang)).color(DIM));
                    }
                    for e in d.browser.entries.clone() {
                        let label = if e.is_dir {
                            format!("{}/", e.name)
                        } else {
                            format!("{}    ({})", e.name, browse::size_text(e.size))
                        };
                        if !ui.selectable_label(false, label).clicked() {
                            continue;
                        }
                        if e.is_dir {
                            d.browser.enter(&e.name);
                        } else if kind == DialogKind::NewProject {
                            d.file_name = e.name.clone();
                        } else if let Some(p) = d.browser.enter(&e.name) {
                            result = Some(DialogResult::Accept(Some(p)));
                        }
                    }
                });
            ui.separator();
            if kind == DialogKind::NewProject {
                ui.horizontal(|ui| {
                    ui.label(Msg::FileName.tr(lang));
                    ui.add(egui::TextEdit::singleline(&mut d.file_name).desired_width(240.0));
                });
                ui.horizontal(|ui| {
                    ui.label(Msg::ProjectName.tr(lang));
                    ui.add(egui::TextEdit::singleline(&mut d.project_name).desired_width(240.0));
                });
            }
            ui.horizontal(|ui| {
                if kind == DialogKind::NewProject && ui.button(Msg::Create.tr(lang)).clicked() {
                    result = Some(DialogResult::Accept(None));
                }
                if ui.button(Msg::Cancel.tr(lang)).clicked() {
                    result = Some(DialogResult::Cancel);
                }
            });
        });
    if !open && result.is_none() {
        result = Some(DialogResult::Cancel);
    }
    result
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // the programs that ended since the last frame
        while let Ok((name, result)) = self.rx.try_recv() {
            self.win.finish_launch(&name, result);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_dialog(DialogKind::OpenProject);
        }
        self.top_bar(ui);
        self.status_bar(ui);
        self.recent_panel(ui);
        self.centre(ui);
        self.dialog_window(&ctx);
    }
}

/// The icon of the program for the window, from the file of the brand.
fn window_icon() -> Option<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../images/brand/png/icon-256.png");
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some(egui::IconData {
        rgba: buf,
        width: info.width,
        height: info.height,
    })
}

/// The fonts of the system that draw Japanese and Chinese are added to the ones of egui, which have no such letters.
fn install_fonts(ctx: &egui::Context) {
    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/simsun.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/System/Library/Fonts/PingFang.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(FontData::from_owned(bytes)));
            for family in [FontFamily::Monospace, FontFamily::Proportional] {
                fonts.families.entry(family).or_default().push("cjk".into());
            }
            ctx.set_fonts(fonts);
            return;
        }
    }
}

/// The colours of the theme Forge for the controls of egui.
fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = RAISED;
    v.extreme_bg_color = Color32::from_rgb(0x14, 0x11, 0x0e);
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0xff, 0xb0, 0x00, 110);
    v.selection.stroke = Stroke::new(1.5, ACCENT);
    v.widgets.inactive.bg_stroke =
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xb0, 0x00, 110));
    v.widgets.inactive.weak_bg_fill = RAISED;
    v.widgets.hovered.bg_stroke = Stroke::new(1.5, ACCENT);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x33, 0x28, 0x1c);
    v.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    v.widgets.inactive.corner_radius = 6.0.into();
    v.widgets.hovered.corner_radius = 6.0.into();
    v.widgets.active.corner_radius = 6.0.into();
    ctx.set_visuals(v);
}

/// Opens the window for the launcher whose files are in `data_dir`. The size of the window and the language come from `launcher.cfg` there.
pub fn run(data_dir: PathBuf) -> eframe::Result {
    let win = Window::new(data_dir);
    let size = [
        win.settings.window_width as f32,
        win.settings.window_height as f32,
    ];
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(size)
        .with_min_inner_size([MIN_WINDOW.0 as f32, MIN_WINDOW.1 as f32])
        .with_title("EH3D-SUITE-LAUNCHER");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "EH3D-SUITE-LAUNCHER",
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(App::new(win)))
        }),
    )
}
