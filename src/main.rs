use eframe::egui;
use std::fmt::format;

/// ======================================================
/// TRAM SIM – PROSTY EDYTOR SIECI TORÓW (GRID SYSTEM)
/// ======================================================
///
/// Projekt symuluje:
/// - ruch na siatce (grid),
/// - edycję punktów,
/// - łączenie punktów liniami,
/// - rysowanie tras po kratkach (pathfinding grid),
/// - tryb myszki i klawiatury.
///
/// ------------------------------------------------------
/// STEROWANIE:
/// ------------------------------------------------------
///
/// KEYBOARD:
/// W A S D / strzałki → ruch
/// SPACE             → tryb ruchu (płynny / kratkowy)
/// ENTER / /         → tryb łączenia punktów
/// M                 → tryb myszki
/// +                 → dodanie punktu
///
/// MOUSE:
/// LPM               → dodaj punkt
/// PPM               → tryb łączenia punktów
///
/// ------------------------------------------------------
/// TRYBY:
/// ------------------------------------------------------
///
/// ruch płynny  → ciągły ruch
/// ruch kratkowy → skoki o 1 kratkę
/// myszka       → sterowanie kursorem po gridzie
///
/// ======================================================
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Tram Sim", options, Box::new(|_cc| Box::new(MyApp::new())))
}

/// ======================================================
/// STAN APLIKACJI
/// ======================================================
struct MyApp {
    x: f32,
    y: f32,

    punkty: Vec<(f32, f32)>,
    linie: Vec<(usize, usize)>,

    wybrany: Option<usize>,
    tryb_linii: bool,

    ruch_kratkowy: bool,
    tryb_myszki: bool,

    grid_scale: f32,
}

impl MyApp {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,

            punkty: Vec::new(),
            linie: Vec::new(),

            wybrany: None,
            tryb_linii: false,

            ruch_kratkowy: false,
            tryb_myszki: false,

            grid_scale: 40.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ==============================================
        // ZMIANA TRYBU RUCHU
        // ==============================================

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.ruch_kratkowy = !self.ruch_kratkowy;
        }

        // ==============================================
        // RUCH POSTACI (POPRAWIONY)
        // ==============================================
        //
        // - ruch płynny: key_down
        // - ruch klatkowy: key_pressed (1 krok na kliknięcie)
        // ==============================================

        let step = if self.ruch_kratkowy { 1.0 } else { 2.0 };

        // LEWO
        if ctx.input(|i| i.key_down(egui::Key::A) || i.key_down(egui::Key::ArrowLeft)) {
            if self.ruch_kratkowy {
                if ctx.input(|i| i.key_pressed(egui::Key::A) || i.key_pressed(egui::Key::ArrowLeft))
                {
                    self.x -= step;
                }
            } else {
                self.x -= step;
            }
        }

        // PRAWO
        if ctx.input(|i| i.key_down(egui::Key::D) || i.key_down(egui::Key::ArrowRight)) {
            if self.ruch_kratkowy {
                if ctx
                    .input(|i| i.key_pressed(egui::Key::D) || i.key_pressed(egui::Key::ArrowRight))
                {
                    self.x += step;
                }
            } else {
                self.x += step;
            }
        }

        // GÓRA
        if ctx.input(|i| i.key_down(egui::Key::W) || i.key_down(egui::Key::ArrowUp)) {
            if self.ruch_kratkowy {
                if ctx.input(|i| i.key_pressed(egui::Key::W) || i.key_pressed(egui::Key::ArrowUp)) {
                    self.y += step;
                }
            } else {
                self.y += step;
            }
        }

        // DÓŁ
        if ctx.input(|i| i.key_down(egui::Key::S) || i.key_down(egui::Key::ArrowDown)) {
            if self.ruch_kratkowy {
                if ctx.input(|i| i.key_pressed(egui::Key::S) || i.key_pressed(egui::Key::ArrowDown))
                {
                    self.y -= step;
                }
            } else {
                self.y -= step;
            }
        }
        // ==============================================
        // UI
        // ==============================================

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Symulator tramwaju");

            // ------------------------------------------
            // PRZYCISKI
            // ------------------------------------------

            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.punkty.push((self.x, self.y));
                }

                if ui.button("M").clicked() {
                    self.tryb_myszki = !self.tryb_myszki;

                    // po wyjściu z trybu myszki wróć do środka
                    if !self.tryb_myszki {
                        self.x = 0.0;
                        self.y = 0.0;
                    }
                }

                ui.label(if self.ruch_kratkowy {
                    "TRYB: kratkowy"
                } else {
                    "TRYB: płynny"
                });
                if ui.button("+").clicked() {
                    self.grid_scale += 5.0;
                }

                if ui.button("-").clicked() {
                    self.grid_scale = (self.grid_scale - 5.0).max(5.0);
                }
            });

            ui.label(format!("X: {:.1}  Y: {:.1}", self.x, self.y));

            // ------------------------------------------
            // OBSZAR RYSOWANIA
            // ------------------------------------------

            let (rect, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());

            let center = rect.rect.center();

            let line_width = self.grid_scale * 0.10;
            let point_radius = self.grid_scale * 0.25;
            let player_radius = self.grid_scale * 0.20;

            // ==========================================
            // TRYB MYSZKI (GRID SNAP)
            // ==========================================

            if self.tryb_myszki {
                if let Some(mpos) = ctx.pointer_hover_pos() {
                    let gx = (mpos.x - center.x) / self.grid_scale;
                    let gy = -(mpos.y - center.y) / self.grid_scale;

                    self.x = gx.round();
                    self.y = gy.round();
                }

                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.punkty.push((self.x, self.y));
                }

                if ctx.input(|i| i.pointer.secondary_clicked()) {
                    self.tryb_linii = true;
                }
            }

            // ==========================================
            // TRYB ŁĄCZENIA PUNKTÓW
            // ==========================================

            if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Slash)) {
                self.tryb_linii = true;
            }

            if self.tryb_linii {
                for (i, (px, py)) in self.punkty.iter().enumerate() {
                    if (self.x - px).abs() < 0.1 && (self.y - py).abs() < 0.1 {
                        if let Some(start) = self.wybrany {
                            if start != i {
                                self.linie.push((start, i));
                            }
                            self.wybrany = None;
                            self.tryb_linii = false;
                        } else {
                            self.wybrany = Some(i);
                            self.tryb_linii = false;
                        }
                    }
                }
            }

            // ==========================================
            // KRATKA (TŁO)
            // ==========================================

            let grid_color = egui::Color32::DARK_GRAY;

            for i in -100..=100 {
                let x = center.x + i as f32 * self.grid_scale;

                painter.line_segment(
                    [
                        egui::pos2(x, rect.rect.top()),
                        egui::pos2(x, rect.rect.bottom()),
                    ],
                    egui::Stroke::new(line_width * 0.5, grid_color),
                );
            }

            for j in -100..=100 {
                let y = center.y + j as f32 * self.grid_scale;

                painter.line_segment(
                    [
                        egui::pos2(rect.rect.left(), y),
                        egui::pos2(rect.rect.right(), y),
                    ],
                    egui::Stroke::new(line_width * 0.5, grid_color),
                );
            }

            // ==========================================
            // LINIE (GRID PATHFINDING)
            // ==========================================

            for (a, b) in &self.linie {
                let (mut x, mut y) = self.punkty[*a];
                let (tx, ty) = self.punkty[*b];

                while (x - tx).abs() > 0.1 || (y - ty).abs() > 0.1 {
                    let start = egui::pos2(
                        center.x + x * self.grid_scale,
                        center.y - y * self.grid_scale,
                    );

                    let dx = tx - x;
                    let dy = ty - y;

                    if dx.abs() >= 1.0 && dy.abs() >= 1.0 {
                        x += dx.signum();
                        y += dy.signum();
                    } else if dx.abs() > dy.abs() {
                        x += dx.signum();
                    } else {
                        y += dy.signum();
                    }

                    let end = egui::pos2(
                        center.x + x * self.grid_scale,
                        center.y - y * self.grid_scale,
                    );

                    painter.line_segment(
                        [start, end],
                        egui::Stroke::new(line_width, egui::Color32::BLUE),
                    );
                }
            }

            // ==========================================
            // PUNKTY
            // ==========================================

            for (i, (px, py)) in self.punkty.iter().enumerate() {
                let color = if Some(i) == self.wybrany {
                    egui::Color32::RED
                } else {
                    egui::Color32::BLUE
                };

                painter.circle_filled(
                    egui::pos2(
                        center.x + px * self.grid_scale,
                        center.y - py * self.grid_scale,
                    ),
                    point_radius,
                    color,
                );
            }

            // ==========================================
            // GRACZ
            // ==========================================

            painter.circle_filled(
                egui::pos2(
                    center.x + self.x * self.grid_scale,
                    center.y - self.y * self.grid_scale,
                ),
                player_radius,
                egui::Color32::WHITE,
            );
        });

        ctx.request_repaint();
    }
}
