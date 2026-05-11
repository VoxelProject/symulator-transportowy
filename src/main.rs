use eframe::egui;
use std::fmt::format;

/// ===============================
/// TRAM SIM
/// ===============================
///
/// Prosty edytor torów oparty o siatkę.
///
/// Funkcje:
/// - ruch gracza,
/// - tryb ruchu płynnego,
/// - tryb ruchu kratkowego,
/// - dodawanie punktów,
/// - łączenie punktów liniami,
/// - linie podążające po siatce,
/// - wyśrodkowana kratka.
///
/// Sterowanie:
/// W A S D / strzałki  -> ruch
/// Shift               -> zmiana trybu ruchu
/// +                   -> dodaj punkt
/// Enter lub /         -> wybór punktów i tworzenie linii
///
/// Tryby ruchu:
/// - płynny:
///     przytrzymanie klawisza = ciągły ruch
///
/// - kratkowy:
///     jedno kliknięcie = jeden ruch o kratkę

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Tram Sim", options, Box::new(|_cc| Box::new(MyApp::new())))
}

/// ===============================
/// GŁÓWNY STAN APLIKACJI
/// ===============================
struct MyApp {
    /// Aktualna pozycja kursora/gracza na siatce
    x: f32,
    y: f32,

    /// Lista wszystkich punktów
    ///
    /// Każdy punkt posiada współrzędne:
    /// (x, y)
    punkty: Vec<(f32, f32)>,

    /// Lista połączeń pomiędzy punktami
    ///
    /// Przykład:
    /// (0, 2)
    ///
    /// oznacza połączenie:
    /// punkt 0 -> punkt 2
    linie: Vec<(usize, usize)>,

    /// Aktualnie wybrany punkt
    ///
    /// Potrzebny podczas tworzenia linii.
    wybrany: Option<usize>,

    /// Czy aktywny jest tryb tworzenia linii
    tryb_linii: bool,

    /// Tryb ruchu:
    ///
    /// false -> płynny
    /// true  -> kratkowy
    ruch_kratkowy: bool,
}

impl MyApp {
    /// Tworzy nową aplikację
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,

            punkty: Vec::new(),
            linie: Vec::new(),

            wybrany: None,
            tryb_linii: false,

            ruch_kratkowy: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // =====================================
        // ZMIANA TRYBU RUCHU
        // =====================================
        //
        // Shift przełącza:
        // płynny <-> kratkowy
        //

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.ruch_kratkowy = !self.ruch_kratkowy;
        }

        // =====================================
        // SYSTEM RUCHU
        // =====================================

        if self.ruch_kratkowy {
            // -----------------------------
            // RUCH KRATKOWY
            // -----------------------------
            //
            // Jedno kliknięcie = jeden ruch
            //

            if ctx.input(|i| i.key_pressed(egui::Key::A) || i.key_pressed(egui::Key::ArrowLeft)) {
                self.x -= 1.0;
            }

            if ctx.input(|i| i.key_pressed(egui::Key::D) || i.key_pressed(egui::Key::ArrowRight)) {
                self.x += 1.0;
            }

            if ctx.input(|i| i.key_pressed(egui::Key::W) || i.key_pressed(egui::Key::ArrowUp)) {
                self.y += 1.0;
            }

            if ctx.input(|i| i.key_pressed(egui::Key::S) || i.key_pressed(egui::Key::ArrowDown)) {
                self.y -= 1.0;
            }
        } else {
            // -----------------------------
            // RUCH PŁYNNY
            // -----------------------------
            //
            // Przytrzymanie = ciągły ruch
            //

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
        }

        // =====================================
        // GUI
        // =====================================

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Symulator tramwaju");

            // ---------------------------------
            // PRZYCISKI
            // ---------------------------------

            ui.horizontal(|ui| {
                // Dodanie punktu
                if ui.button("+").clicked() {
                    self.punkty.push((self.x, self.y));
                }

                // Informacja o trybie ruchu
                if self.ruch_kratkowy {
                    ui.label("Tryb: kratkowy");
                } else {
                    ui.label("Tryb: płynny");
                }
            });

            // ---------------------------------
            // POZYCJA
            // ---------------------------------

            ui.label(format!("Pozycja X: {} Y: {}", self.x, self.y));

            // ---------------------------------
            // OBSZAR RYSOWANIA
            // ---------------------------------

            let (rect, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());

            // Rozmiar jednej kratki
            let skala = 20.0;

            // Środek ekranu = punkt (0,0)
            let centrum = rect.rect.center();

            // =====================================
            // SYSTEM ŁĄCZENIA PUNKTÓW
            // =====================================
            //
            // Enter lub /
            // wybiera punkty i tworzy linie
            //

            if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Slash)) {
                self.tryb_linii = true;
            }

            if self.tryb_linii {
                for (index, (px, py)) in self.punkty.iter().enumerate() {
                    // Czy gracz stoi na punkcie
                    if (self.x - px).abs() < 0.1 && (self.y - py).abs() < 0.1 {
                        // Drugi punkt
                        if let Some(start) = self.wybrany {
                            if start != index {
                                self.linie.push((start, index));
                            }

                            self.wybrany = None;
                            self.tryb_linii = false;
                        }
                        // Pierwszy punkt
                        else {
                            self.wybrany = Some(index);
                            self.tryb_linii = false;
                        }
                    }
                }
            }

            // =====================================
            // KRATKA TŁA
            // =====================================

            let kolor = egui::Color32::DARK_GRAY;

            // pionowe linie
            for i in -100..=100 {
                let x = centrum.x + i as f32 * skala;

                painter.line_segment(
                    [
                        egui::pos2(x, rect.rect.top()),
                        egui::pos2(x, rect.rect.bottom()),
                    ],
                    egui::Stroke::new(2.0, kolor),
                );
            }

            // poziome linie
            for j in -100..=100 {
                let y = centrum.y + j as f32 * skala;

                painter.line_segment(
                    [
                        egui::pos2(rect.rect.left(), y),
                        egui::pos2(rect.rect.right(), y),
                    ],
                    egui::Stroke::new(2.0, kolor),
                );
            }

            // =====================================
            // RYSOWANIE LINII
            // =====================================
            //
            // Linie:
            // - idą po kratkach,
            // - mogą używać przekątnych,
            // - wybierają najkrótszą drogę,
            // - działają matematycznie,
            // - NIE bazują na kolorach.
            //

            for (a, b) in &self.linie {
                let (mut x, mut y) = self.punkty[*a];
                let (tx, ty) = self.punkty[*b];

                while (x - tx).abs() > 0.1 || (y - ty).abs() > 0.1 {
                    let start = egui::pos2(centrum.x + x * skala, centrum.y - y * skala);

                    // różnica pozycji
                    let dx = tx - x;
                    let dy = ty - y;

                    // -------------------------
                    // RUCH PO PRZEKĄTNEJ
                    // -------------------------

                    if dx.abs() >= 1.0 && dy.abs() >= 1.0 {
                        x += dx.signum();
                        y += dy.signum();
                    }
                    // -------------------------
                    // RUCH POZIOMY
                    // -------------------------
                    else if dx.abs() > dy.abs() {
                        x += dx.signum();
                    }
                    // -------------------------
                    // RUCH PIONOWY
                    // -------------------------
                    else {
                        y += dy.signum();
                    }

                    let koniec = egui::pos2(centrum.x + x * skala, centrum.y - y * skala);

                    painter
                        .line_segment([start, koniec], egui::Stroke::new(4.0, egui::Color32::BLUE));
                }
            }

            // =====================================
            // RYSOWANIE PUNKTÓW
            // =====================================

            for (index, (px, py)) in self.punkty.iter().enumerate() {
                let kolor = if Some(index) == self.wybrany {
                    egui::Color32::RED
                } else {
                    egui::Color32::BLUE
                };

                painter.circle_filled(
                    egui::pos2(centrum.x + px * skala, centrum.y - py * skala),
                    10.0,
                    kolor,
                );
            }

            // =====================================
            // GRACZ
            // =====================================

            let pozycja = egui::pos2(centrum.x + self.x * skala, centrum.y - self.y * skala);

            painter.circle_filled(pozycja, 8.0, egui::Color32::WHITE);
        });

        // Odświeżanie ekranu
        ctx.request_repaint();
    }
}
