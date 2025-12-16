#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::time::{Duration, Instant};

fn fmt_hhmmss(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

struct AppState {
    // Set duration controls
    set_minutes: u32,
    set_seconds: u32,

    // Timer runtime state
    running: bool,
    remaining: Duration,
    last_tick: Option<Instant>,

    finished_modal: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let initial = Duration::from_secs(5 * 60);
        Self {
            set_minutes: 5,
            set_seconds: 0,
            running: false,
            remaining: initial,
            last_tick: None,
            finished_modal: false,
        }
    }
}

impl AppState {
    fn apply_set_duration(&mut self) {
        let secs = (self.set_minutes as u64) * 60 + (self.set_seconds as u64);
        self.remaining = Duration::from_secs(secs.max(1)); // avoid 0
        self.finished_modal = false;
        self.last_tick = None;
    }

    fn reset(&mut self) {
        self.running = false;
        self.apply_set_duration();
    }

    fn toggle(&mut self) {
        self.running = !self.running;
        self.last_tick = Some(Instant::now());
    }

    fn tick(&mut self) {
        if !self.running {
            self.last_tick = None;
            return;
        }

        let now = Instant::now();
        let Some(prev) = self.last_tick else {
            self.last_tick = Some(now);
            return;
        };

        let dt = now.saturating_duration_since(prev);
        self.last_tick = Some(now);

        if dt >= self.remaining {
            self.remaining = Duration::from_secs(0);
            self.running = false;
            self.finished_modal = true;
        } else {
            self.remaining -= dt;
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        let space = ctx.input(|i| i.key_pressed(egui::Key::Space));
        let r = ctx.input(|i| i.key_pressed(egui::Key::R));
        let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if space {
            self.toggle();
        }
        if r {
            self.reset();
        }
        if esc && self.finished_modal {
            self.finished_modal = false;
        }

        // Advance timer
        self.tick();

        // If running, keep UI smooth
        if self.running {
            ctx.request_repaint_after(Duration::from_millis(16));
        }

        // UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.heading("LightTimer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Space: Start/Pause   R: Reset");
                });
            });

            ui.add_space(12.0);

            // Big time display
            let secs = self.remaining.as_secs();
            let time_str = fmt_hhmmss(secs);

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(time_str)
                        .size(64.0)
                        .strong()
                        .monospace(),
                );
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            // Duration controls
            ui.horizontal(|ui| {
                ui.label("Set:");
                ui.add(
                    egui::DragValue::new(&mut self.set_minutes)
                        .clamp_range(0..=999)
                        .suffix(" min"),
                );
                ui.add(
                    egui::DragValue::new(&mut self.set_seconds)
                        .clamp_range(0..=59)
                        .suffix(" sec"),
                );

                if ui.button("Apply").clicked() {
                    self.apply_set_duration();
                }
            });

            ui.add_space(6.0);

            // Presets
            ui.horizontal(|ui| {
                ui.label("Presets:");
                for (label, mins) in [("1", 1), ("5", 5), ("10", 10), ("25", 25), ("50", 50)] {
                    if ui.button(label).clicked() {
                        self.set_minutes = mins;
                        self.set_seconds = 0;
                        self.apply_set_duration();
                    }
                }
            });

            ui.add_space(14.0);

            // Controls
            ui.horizontal(|ui| {
                let start_label = if self.running { "Pause" } else { "Start" };
                if ui.add_sized([110.0, 34.0], egui::Button::new(start_label)).clicked() {
                    self.toggle();
                }
                if ui.add_sized([110.0, 34.0], egui::Button::new("Reset")).clicked() {
                    self.reset();
                }
                if ui.add_sized([110.0, 34.0], egui::Button::new("Set = Remaining")).clicked() {
                    // convenient: copy current remaining into set controls
                    let total = self.remaining.as_secs();
                    self.set_minutes = (total / 60) as u32;
                    self.set_seconds = (total % 60) as u32;
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new("Tip: Press Space to start/pause, R to reset.")
                    .weak()
                    .size(12.0),
            );
        });

        // Finished modal
        if self.finished_modal {
            egui::Window::new("Time's up")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Timer finished.").size(18.0).strong());
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.finished_modal = false;
                        }
                        ui.label(egui::RichText::new("(Esc closes)").weak());
                    });
                });

            // keep repainting while modal is visible
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 320.0])
            .with_min_inner_size([420.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "LightTimer",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Box::<AppState>::default()
        }),
    )
}
