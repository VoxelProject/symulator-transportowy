use eframe::egui;
//use std::fmt::format;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize)]
struct Node {
    id: usize,
    x: f32,
    y: f32,

    node_type: u8, // 1 = normalny, 2 = ignorowany

    name: String,
    location: String,

    color: [f32; 3],

    meta: Option<String>,
}
// const MAPA_STARTOWA_JSON: &str = r#"
// [
//   { "id": 0, "x": -7, "y": 6, "meta": null },
//   { "id": 1, "x": -4, "y": -0, "meta": null },
//   { "id": 2, "x": 5, "y": 3, "meta": null },
//   { "id": 3, "x": -3, "y": -2, "meta": null },
//   { "id": 4, "x": 2, "y": 4, "meta": null },
//   { "id": 5, "x": -2, "y": -0, "meta": null }
// ]
// "#;
const MAPA_STARTOWA_JSON: &str = r#"
[
  {
    "id": 0,
    "x": -7.0,
    "y": 6.0,
    "node_type": 1,
    "name": "N0",
    "location": "user",
    "color": [
      0.0,
      1.5,
      1.0
    ],
    "meta": null
  },
  {
    "id": 1,
    "x": 0.0,
    "y": 0.0,
    "node_type": 1,
    "name": "N1",
    "location": "user",
    "color": [
      0.0,
      0.5,
      1.0
    ],
    "meta": null
  },
  {
    "id": 2,
    "x": 3.0,
    "y": -0.0,
    "node_type": 1,
    "name": "N2",
    "location": "user",
    "color": [
      0.0,
      0.5,
      1.0
    ],
    "meta": null
  },
  {
    "id": 3,
    "x": 2.0,
    "y": -2.0,
    "node_type": 1,
    "name": "N3",
    "location": "user",
    "color": [
      0.0,
      0.5,
      1.0
    ],
    "meta": null
  },
  {
    "id": 4,
    "x": 1.0,
    "y": 2.0,
    "node_type": 1,
    "name": "N4",
    "location": "user",
    "color": [
      0.0,
      0.5,
      1.0
    ],
    "meta": null
  },
  {
    "id": 5,
    "x": 3.0,
    "y": 2.0,
    "node_type": 1,
    "name": "N5",
    "location": "user",
    "color": [
      0.0,
      0.5,
      1.0
    ],
    "meta": null
  }
]
"#;
struct MyApp {
    x: f32,
    y: f32,

    punkty: Vec<Node>,
    next_id: usize,
    linie: Vec<(usize, usize)>,

    wybrany: Option<usize>,
    tryb_linii: bool,

    ruch_kratkowy: bool,
    tryb_myszki: bool,

    grid_scale: f32,
    panel_width: f32,
    hovered_node: Option<usize>,
}

impl MyApp {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,

            punkty: Self::load_from_json(MAPA_STARTOWA_JSON)
                .into_iter()
                .filter(|n| n.node_type == 1)
                .collect(),
            next_id: Self::load_from_json(MAPA_STARTOWA_JSON)
                .iter()
                .map(|n| n.id)
                .max()
                .unwrap_or(0)
                + 1,
            linie: Vec::new(),

            wybrany: None,
            tryb_linii: false,

            ruch_kratkowy: false,
            tryb_myszki: false,

            grid_scale: 40.0,
            panel_width: 300.0,
            hovered_node: None,
        }
    }
    fn load_from_json(json: &str) -> Vec<Node> {
        serde_json::from_str(json).unwrap_or_else(|_| Vec::new())
    }
    fn add_point(&mut self, x: f32, y: f32) {
        if self
            .punkty
            .iter()
            .any(|n| n.x == x && n.y == y && n.node_type == 1)
        {
            return;
        }

        let node = Node {
            id: self.next_id,
            x,
            y,
            node_type: 1,
            name: format!("N{}", self.next_id),
            location: "user".to_string(),
            color: [0.0, 0.5, 1.0],
            meta: None,
        };

        self.next_id += 1;

        self.on_point_added(&node);

        self.punkty.push(node);
    }
    fn on_point_added(&mut self, node: &Node) {
        println!(
            "Dodano punkt -> id: {}, x: {}, y: {}",
            node.id, node.x, node.y
        );
    }
    fn export_json(&self) -> String {
        let filtered: Vec<&Node> = self.punkty.iter().filter(|n| n.node_type == 1).collect();

        serde_json::to_string_pretty(&filtered).unwrap()
    }
}

//tu chciałbym te pamięć punktów

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
        let hovered_info = self
            .hovered_node
            .and_then(|id| self.punkty.iter().find(|n| n.id == id));

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .exact_width(self.panel_width)
            .show(ctx, |ui| {
                let panel_rect = ui.available_rect_before_wrap();

                let margin = 10.0;
                let spacing = 10.0;

                let content_rect = panel_rect.shrink(margin);

                let section_height = (content_rect.height() - spacing) / 2.0;

                // =========================
                // GÓRNA SEKCJA
                // =========================
                let top_rect = egui::Rect::from_min_size(
                    content_rect.min,
                    egui::vec2(content_rect.width(), section_height),
                );

                ui.painter()
                    .rect_filled(top_rect, 8.0, egui::Color32::from_gray(45));

                ui.allocate_ui_at_rect(top_rect, |ui| {
                    ui.label("DEBUG PANEL");
                    ui.label(format!("Player: ({:.1}, {:.1})", self.x, self.y));
                });

                // =========================
                // DOLNA SEKCJA (INFO O NODE)
                // =========================
                let bottom_rect = egui::Rect::from_min_size(
                    egui::pos2(content_rect.min.x, top_rect.max.y + spacing),
                    egui::vec2(content_rect.width(), section_height),
                );

                ui.painter()
                    .rect_filled(bottom_rect, 8.0, egui::Color32::from_gray(35));

                ui.allocate_ui_at_rect(bottom_rect, |ui| {
                    ui.label("INFO:");

                    if let Some(node) = hovered_info {
                        ui.separator();
                        ui.label(format!("ID: {}", node.id));
                        ui.label(format!("NAME: {}", node.name));
                        ui.label(format!("X: {:.1} Y: {:.1}", node.x, node.y));
                        ui.label(format!("TYPE: {}", node.node_type));
                        ui.label(format!("LOCATION: {}", node.location));
                    } else {
                        ui.label("Brak obiektu pod kursorem");
                    }
                });
            });

        // ==============================================
        // GŁÓWNY OBSZAR RYSOWANIA
        // ==============================================

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Symulator tramwaju");

            // ------------------------------------------
            // PRZYCISKI
            // ------------------------------------------

            ui.horizontal(|ui| {
                if ui.button("D").clicked() {
                    self.add_point(self.x, self.y);
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
                if ui.button("Z").clicked() {
                    println!("=== JSON MAPA ===");
                    println!("{}", self.export_json());
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
                    self.add_point(self.x, self.y);
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
                let mut best: Option<(usize, f32)> = None;

                for node in &self.punkty {
                    if node.node_type != 1 {
                        continue;
                    }

                    let dx = self.x - node.x;
                    let dy = self.y - node.y;
                    let dist = dx * dx + dy * dy;

                    if dist < 0.5 {
                        // tolerancja kliknięcia
                        match best {
                            Some((_, best_dist)) if best_dist <= dist => {}
                            _ => best = Some((node.id, dist)),
                        }
                    }
                }

                if let Some((id, _)) = best {
                    if let Some(start) = self.wybrany {
                        if start != id {
                            self.linie.push((start, id));
                        }
                        self.wybrany = None;
                    } else {
                        self.wybrany = Some(id);
                    }

                    self.tryb_linii = false;
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
            // LINIE (GRID PATHFINDING)(TYLKO ID → BEZ FLOATÓW)
            // ==========================================
            for (start_id, end_id) in &self.linie {
                let a_node = self.punkty.iter().find(|n| n.id == *start_id);
                let b_node = self.punkty.iter().find(|n| n.id == *end_id);

                // jeśli któryś punkt został usunięty
                let (Some(a_node), Some(b_node)) = (a_node, b_node) else {
                    continue;
                };

                let mut x = a_node.x;
                let mut y = a_node.y;

                let tx = b_node.x;
                let ty = b_node.y;

                // rysowanie „po kratce”
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

            self.hovered_node = None;

            for node in &self.punkty {
                let dx = self.x - node.x;
                let dy = self.y - node.y;

                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 0.6 {
                    self.hovered_node = Some(node.id);
                    break;
                }
            }

            for node in &self.punkty {
                if node.node_type != 1 {
                    continue;
                }

                let color = if Some(node.id) == self.wybrany {
                    egui::Color32::RED
                } else {
                    egui::Color32::from_rgb(
                        (node.color[0] * 255.0) as u8,
                        (node.color[1] * 255.0) as u8,
                        (node.color[2] * 255.0) as u8,
                    )
                };

                painter.circle_filled(
                    egui::pos2(
                        center.x + node.x * self.grid_scale,
                        center.y - node.y * self.grid_scale,
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
