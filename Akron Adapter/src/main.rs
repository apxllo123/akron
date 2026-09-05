use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use akron_analyzer::manifest::GameManifest;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 680.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Akron",
        options,
        Box::new(|_cc| Ok(Box::new(AkronApp::default()))),
    )
}

#[derive(Default)]
struct AkronApp {
    selected_path: Option<PathBuf>,
    manifest: Option<GameManifest>,
    error: Option<String>,
    receiver: Option<Receiver<Result<GameManifest, String>>>,
    analyzing: bool,
}

impl eframe::App for AkronApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_analysis(ui);

        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("AKRON");
                ui.separator();
                ui.label("Universal game analysis and adaptation");
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading("Game Analyzer");
            ui.label(
                "Select a game directory. Akron will inspect it without modifying the source files.",
            );
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.analyzing, egui::Button::new("Select game folder"))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    self.start_analysis(path);
                }

                if let Some(path) = &self.selected_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.weak("No game selected");
                }
            });

            ui.add_space(14.0);

            if self.analyzing {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label("Analyzing game files…");
                });
            }

            if let Some(error) = &self.error {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::RED, error);
            }

            if let Some(manifest) = &self.manifest {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                let total_bytes = manifest.files.iter().map(|file| file.size).sum::<u64>();
                let pe_count = manifest
                    .executables
                    .iter()
                    .filter(|executable| executable.format == "PE")
                    .count();

                ui.horizontal_wrapped(|ui| {
                    stat_card(ui, "Files", manifest.files.len().to_string());
                    stat_card(ui, "Executables", manifest.executables.len().to_string());
                    stat_card(ui, "PE binaries", pe_count.to_string());
                    stat_card(ui, "Total size", format_bytes(total_bytes));
                });

                ui.add_space(12.0);
                ui.collapsing("Executables", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            for executable in &manifest.executables {
                                ui.horizontal(|ui| {
                                    ui.label(executable.path.display().to_string());
                                    ui.weak(&executable.format);
                                    if let Some(architecture) = &executable.architecture {
                                        ui.weak(architecture);
                                    }
                                });
                            }
                        });
                });
            }
        });
    }
}

impl AkronApp {
    fn poll_analysis(&mut self, ui: &mut egui::Ui) {
        let Some(receiver) = self.receiver.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.analyzing = false;
                match result {
                    Ok(manifest) => {
                        self.error = None;
                        self.manifest = Some(manifest);
                    }
                    Err(error) => {
                        self.manifest = None;
                        self.error = Some(error);
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.receiver = Some(receiver);
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.analyzing = false;
                self.error = Some("Analyzer worker stopped unexpectedly.".to_owned());
            }
        }
    }

    fn start_analysis(&mut self, path: PathBuf) {
        self.selected_path = Some(path.clone());
        self.manifest = None;
        self.error = None;
        self.analyzing = true;

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        thread::spawn(move || {
            let result = akron_analyzer::scanner::analyze_game(&path)
                .map_err(|error| format!("Analysis failed: {error:#}"));
            let _ = sender.send(result);
        });
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: String) {
    ui.group(|ui| {
        ui.label(label);
        ui.strong(value);
    });
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    }
}
