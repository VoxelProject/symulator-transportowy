use eframe::egui;
use std::fmt::format;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Tram Sim", options, Box::new(|_cc| Box::new(MyApp::new())))
}
struct MyApp {
    x: f32,
    y: f32,
    docelowe: Option<(f32, f32)>,
}
impl MyApp {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            docelowe: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_down(egui::Key::A) || i.key_down(egui::Key::ArrowLeft)) {
            self.x -= 2.0;
        }
        if ctx.input(|i| i.key_down(egui::Key::D) || i.key_down(egui::Key::ArrowRight)) {
            self.x += 2.0;
        }
        if ctx.input(|i| i.key_down(egui::Key::W) || i.key_down(egui::Key::ArrowUp)) {
            self.y += 2.0;
        }
        if ctx.input(|i| i.key_down(egui::Key::S) || i.key_down(egui::Key::ArrowDown)) {
            self.y -= 2.0;
        }

        if ctx.input(|i| i.key_down(egui::Key::Space)) {
            self.docelowe = Some((self.x, self.y))
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Symulator tramwaju");
            // ułożone obok siebie
            if ui.button("+").clicked() {
                //dodawanie punktu niebieska kulka w miejscu gdzie przesunięto sie wskaźnikiem
            }
            if ui.button("/").clicked() {
                //dodawanie lini między punktami niebieskimi, trzeba zaznaczyć kulke enterem ona zacznie pulsować na czerwono i jak znajdzie i kliknie się 2 kulke enterem to między nimi powstanie linia
            }
            ui.label(format!("Pozycja X: {} Y: {}", self.x, self.y));

            //rysowanie
            let (rect, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());

            let skala: f32 = 20.0;
            let centrum = rect.rect.center();

            let pozycja = egui::pos2(centrum.x + self.x * skala, centrum.y - (self.y * skala));
            painter.circle_filled(pozycja, 10.0, egui::Color32::BLUE);

            painter.circle_stroke(centrum, 10.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
            if let Some((tx, ty)) = self.docelowe {
                painter.line_segment(
                    [
                        egui::pos2(centrum.x, centrum.y),
                        egui::pos2(centrum.x + tx * skala, centrum.y - ty * skala),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            }
            //kratka z lini
            let odstęp = skala;
            let kolor = egui::Color32::DARK_GRAY;
            let grubość = 2.0;

            // pionowe linie
            let mut i = -100;

            while i <= 100 {
                let x = centrum.x + i as f32 * odstęp;

                painter.line_segment(
                    [
                        egui::pos2(x, rect.rect.top()),
                        egui::pos2(x, rect.rect.bottom()),
                    ],
                    egui::Stroke::new(grubość, kolor),
                );

                i += 1;
            }

            // poziome linie
            let mut j = -100;

            while j <= 100 {
                let y = centrum.y + j as f32 * odstęp;

                painter.line_segment(
                    [
                        egui::pos2(rect.rect.left(), y),
                        egui::pos2(rect.rect.right(), y),
                    ],
                    egui::Stroke::new(grubość, kolor),
                );

                j += 1;
            }
            //schemat
            painter.circle_filled(
                egui::pos2(centrum.x + 2.0 * skala, centrum.y - 2.0 * skala),
                10.0,
                egui::Color32::BLUE,
            );
            painter.line_segment(
                [
                    egui::pos2(centrum.x + 2.0 * skala, centrum.y - 2.0 * skala),
                    egui::pos2(centrum.x + 2.0 * skala, centrum.y - 4.0 * skala),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLUE),
            );
            painter.line_segment(
                [
                    egui::pos2(centrum.x + 2.0 * skala, centrum.y - 4.0 * skala),
                    egui::pos2(centrum.x + 4.0 * skala, centrum.y - 6.0 * skala),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLUE),
            );
            painter.line_segment(
                [
                    egui::pos2(centrum.x + 5.0 * skala, centrum.y - 6.0 * skala),
                    egui::pos2(centrum.x + 6.0 * skala, centrum.y - 6.0 * skala),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLUE),
            );
            painter.circle_filled(
                egui::pos2(centrum.x + 6.0 * skala, centrum.y - 6.0 * skala),
                10.0,
                egui::Color32::BLUE,
            );
        });
        ctx.request_repaint();
    }
}
