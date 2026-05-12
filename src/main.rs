use eframe::egui;
use eframe::egui::Key;
use rfd::FileDialog;
use std::fs;
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
#[derive(Clone, Serialize, Deserialize)]
struct Edge {
    from: usize,
    to: usize,

    numer: Option<i32>, // jeśli brak → nieprzypisane
    meta: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct SaveData {
    punkty: Vec<Node>,
    linie: LiniaPakiet,
}

#[derive(Clone, Serialize, Deserialize)]
struct LiniaPakiet {
    przypisane: std::collections::HashMap<i32, Vec<Edge>>,
    nieprzypisane: Vec<Edge>,
}
const SAVE_FILE: &str = "mapa.json";
struct MyApp {
    x: f32,
    y: f32,

    punkty: Vec<Node>,
    next_id: usize,
    linie: Vec<Edge>,

    wybrany: Option<usize>,

    ruch_kratkowy: bool,
    tryb_myszki: bool,
    show_import_dialog: bool,

    grid_scale: f32,
    panel_width: f32,
    hovered_node: Option<usize>,
}

impl MyApp {
    fn new() -> Self {
        let data = Self::load_file();
        let nodes = data.punkty;
        let linie = data.linie;

        let next_id = nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;

        Self {
            x: 0.0,
            y: 0.0,

            punkty: nodes.into_iter().filter(|n| n.node_type == 1).collect(),

            next_id,

            linie: {
                let mut all = Vec::new();
                for (_, v) in linie.przypisane {
                    all.extend(v);
                }
                all.extend(linie.nieprzypisane);
                all
            },

            wybrany: None,

            ruch_kratkowy: false,
            tryb_myszki: false,
            show_import_dialog: false,

            grid_scale: 40.0,
            panel_width: 300.0,
            hovered_node: None,
        }
    }
    fn handle_selection(&mut self) {
        let mut best: Option<(usize, f32)> = None;

        for node in &self.punkty {
            if node.node_type != 1 {
                continue;
            }

            let dx = self.x - node.x;
            let dy = self.y - node.y;
            let dist = dx * dx + dy * dy;

            if dist < 0.5 {
                match best {
                    Some((_, best_dist)) if best_dist <= dist => {}
                    _ => best = Some((node.id, dist)),
                }
            }
        }

        // kliknięto poza punktem
        let Some((id, _)) = best else {
            return;
        };

        // odznaczenie
        if self.wybrany == Some(id) {
            self.wybrany = None;
            return;
        }

        // tworzenie linii
        if let Some(start) = self.wybrany {
            self.linie.push(Edge {
                from: start,
                to: id,
                numer: None,
                meta: None,
            });

            self.wybrany = None;
        } else {
            self.wybrany = Some(id);
        }
    }
    fn handle_movement(
        &mut self,
        ctx: &egui::Context,
        key: egui::Key,
        arrow: egui::Key,
        dx: f32,
        dy: f32,
        step: f32,
    ) {
        let active = ctx.input(|i| i.key_down(key) || i.key_down(arrow));

        if !active {
            return;
        }

        let should_move = if self.ruch_kratkowy {
            ctx.input(|i| i.key_pressed(key) || i.key_pressed(arrow))
        } else {
            true
        };

        if should_move {
            self.x += dx * step;
            self.y += dy * step;
        }
    }
    fn load_file() -> SaveData {
        if let Ok(content) = fs::read_to_string(SAVE_FILE) {
            serde_json::from_str(&content).unwrap_or(SaveData {
                punkty: Vec::new(),
                linie: LiniaPakiet {
                    przypisane: std::collections::HashMap::new(),
                    nieprzypisane: Vec::new(),
                },
            })
        } else {
            SaveData {
                punkty: Vec::new(),
                linie: LiniaPakiet {
                    przypisane: std::collections::HashMap::new(),
                    nieprzypisane: Vec::new(),
                },
            }
        }
    }
    
    fn add_point(&mut self, x: f32, y: f32) {
        const EPS: f32 = 0.001;
        if self
            .punkty
            .iter()
            .any(|n| (n.x - x).abs() < EPS && (n.y - y).abs() < EPS)
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
        let mut przypisane: std::collections::HashMap<i32, Vec<Edge>> =
            std::collections::HashMap::new();

        let mut nieprzypisane: Vec<Edge> = Vec::new();

        for e in &self.linie {
            if let Some(num) = e.numer {
                przypisane.entry(num).or_default().push(e.clone());
            } else {
                nieprzypisane.push(e.clone());
            }
        }

        let data = SaveData {
            punkty: self.punkty.clone(),
            linie: LiniaPakiet {
                przypisane,
                nieprzypisane,
            },
        };

        serde_json::to_string_pretty(&data).unwrap()
    }
    fn import_json(&mut self, json: &str) {
        if let Ok(data) = serde_json::from_str::<SaveData>(json) {
            self.punkty = data.punkty;

            self.linie.clear();

            for (_, vec) in &data.linie.przypisane {
                self.linie.extend(vec.clone());
            }

            self.linie.extend(data.linie.nieprzypisane);
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

        self.handle_movement(ctx, Key::A, Key::ArrowLeft, -1.0, 0.0, step);
        self.handle_movement(ctx, Key::D, Key::ArrowRight, 1.0, 0.0, step);
        self.handle_movement(ctx, Key::W, Key::ArrowUp, 0.0, 1.0, step);
        self.handle_movement(ctx, Key::S, Key::ArrowDown, 0.0, -1.0, step);
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
                    ui.set_min_size(top_rect.size());

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .id_source("top_panel_scroll")
                        .show(ui, |ui| {
                            ui.heading("WYBRANY PUNKT");

                            if let Some(selected_id) = self.wybrany {
                                if let Some(node) = self.punkty.iter().find(|n| n.id == selected_id)
                                {
                                    ui.separator();

                                    ui.label(format!("ID: {}", node.id));
                                    ui.label(format!("NAZWA: {}", node.name));

                                    ui.label(format!("POZYCJA: ({:.1}, {:.1})", node.x, node.y));

                                    ui.label(format!("TYP: {}", node.node_type));
                                    ui.label(format!("LOKALIZACJA: {}", node.location));

                                    if let Some(meta) = &node.meta {
                                        ui.label(format!("META: {}", meta));
                                    } else {
                                        ui.label("META: brak");
                                    }

                                    // ======================================
                                    // PODGLĄD LINII
                                    // ======================================

                                    if let Some(hover_id) = self.hovered_node {
                                        if hover_id != node.id {
                                            if let Some(target) =
                                                self.punkty.iter().find(|n| n.id == hover_id)
                                            {
                                                ui.separator();
                                                ui.heading("PODGLĄD LINII");

                                                let mut x = node.x;
                                                let mut y = node.y;

                                                let tx = target.x;
                                                let ty = target.y;

                                                let mut visited = 0;
                                                let mut crossed_nodes: Vec<String> = Vec::new();

                                                let mut safety = 0;

                                                while (x - tx).abs() > 0.1 || (y - ty).abs() > 0.1 {
                                                    safety += 1;

                                                    if safety > 1000 {
                                                        break;
                                                    }

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

                                                    visited += 1;

                                                    for other in &self.punkty {
                                                        if other.id == node.id
                                                            || other.id == target.id
                                                        {
                                                            continue;
                                                        }

                                                        if (other.x - x).abs() < 0.1
                                                            && (other.y - y).abs() < 0.1
                                                        {
                                                            crossed_nodes.push(format!(
                                                                "{} (id:{})",
                                                                other.name, other.id
                                                            ));
                                                        }
                                                    }
                                                }

                                                ui.label(format!("DŁUGOŚĆ: {} kratek", visited));

                                                if crossed_nodes.is_empty() {
                                                    ui.label("PRZECINANE PUNKTY: brak");
                                                } else {
                                                    ui.label("PRZECINANE PUNKTY:");

                                                    for name in crossed_nodes {
                                                        ui.label(format!("• {}", name));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    ui.label("Brak zaznaczonego punktu");
                                }
                            } else {
                                ui.label("Brak zaznaczonego punktu");
                            }
                        });
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
                    ui.set_min_size(top_rect.size()); // KLUCZOWE

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .id_source("bottom_panel_scroll")
                        .show(ui, |ui| {
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
                    if let Some(path) = FileDialog::new().set_file_name("mapa.json").save_file() {
                        let _ = fs::write(path, self.export_json());
                    }
                }
                if ui.button("W").clicked() {
                    self.show_import_dialog = true;
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

                // LPM -> tworzenie punktu
                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.add_point(self.x, self.y);
                }

                // PPM -> zaznaczanie / linie
                if ctx.input(|i| i.pointer.secondary_clicked()) {
                    self.handle_selection();
                }
            }
            // ==========================================
            // OBSŁUGA ENTERA / ŁĄCZENIA PUNKTÓW
            // ==========================================

            if !self.tryb_myszki {
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut found = false;

                    for node in &self.punkty {
                        let dx = self.x - node.x;
                        let dy = self.y - node.y;

                        if (dx * dx + dy * dy) < 0.5 {
                            found = true;
                            break;
                        }
                    }

                    // pusty obszar -> dodaj punkt
                    if !found {
                        self.add_point(self.x, self.y);
                    } else {
                        // punkt -> zaznacz / połącz
                        self.handle_selection();
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
            // LINIE (GRID PATHFINDING)(TYLKO ID → BEZ FLOATÓW)
            // ==========================================
            for edge in &self.linie {
                let start_id = edge.from;
                let end_id = edge.to;
                let a_node = self.punkty.iter().find(|n| n.id == start_id);
                let b_node = self.punkty.iter().find(|n| n.id == end_id);

                // jeśli któryś punkt został usunięty
                let (Some(a_node), Some(b_node)) = (a_node, b_node) else {
                    continue;
                };

                let mut x = a_node.x;
                let mut y = a_node.y;

                let tx = b_node.x;
                let ty = b_node.y;

                // rysowanie „po kratce”
                let mut safety = 0;
                while (x - tx).abs() > 0.1 || (y - ty).abs() > 0.1 {
                    safety += 1;
                    if safety > 1000 {
                        break;
                    }

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

                // dystans od białej kropki
                if (dx * dx + dy * dy) < 0.25 {
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

        if self.show_import_dialog {
            self.show_import_dialog = false;

            if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
                if let Ok(content) = fs::read_to_string(path) {
                    self.import_json(&content);
                }
            }
        }

        ctx.request_repaint();
    }
}
