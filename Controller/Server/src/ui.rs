use std::net::IpAddr;
use std::time::{Duration, Instant};

use eframe::egui;
use if_addrs::get_if_addrs;

use crate::core::{
    get_components, get_steering, has_data,
    set_axis, set_max_angle, set_zero,
};
use crate::network::output::osc_output::{OSC_ADDRESS, UNITY_IP, UNITY_PORT};

const OSC_PORT: u16    = 50000;
const PREPARE_SECS: f32 = 2.0;
const HOLD_SECS: f32    = 1.5;

// ── Détection des IPs ─────────────────────────────────────────────────────────

struct IpEntry {
    label: String,
    addr:  String,
}

fn classify(name: &str, ip: IpAddr) -> Option<IpEntry> {
    let v4 = match ip {
        IpAddr::V4(v) => v,
        IpAddr::V6(_) => return None,
    };
    if v4.is_loopback() { return None; }
    let [a, b, ..] = v4.octets();
    if a == 169 && b == 254 { return None; } // APIPA

    let n = name.to_lowercase();
    let label = if n.contains("wi-fi") || n.contains("wlan") || n.contains("wireless") {
        format!("WiFi — {}", name)
    } else if n.contains("hotspot")
        || (n.contains("local area connection") && n.contains('*'))
        || n.contains("softap")
        || n.contains("virtual")
    {
        format!("Point d'acces WiFi — {}", name)
    } else if n.contains("ethernet") || n.starts_with("eth") || n.contains("local area connection") {
        format!("Ethernet — {}", name)
    } else if n.contains("bluetooth") {
        format!("Bluetooth — {}", name)
    } else if n.starts_with("lo") || n.contains("loopback") {
        return None;
    } else {
        format!("Reseau — {}", name)
    };

    Some(IpEntry { label, addr: v4.to_string() })
}

fn collect_ips() -> Vec<IpEntry> {
    let Ok(ifaces) = get_if_addrs() else { return vec![]; };
    ifaces.into_iter().filter_map(|i| classify(&i.name, i.ip())).collect()
}

// ── Machine à états de calibration ───────────────────────────────────────────

enum CalibStep {
    Waiting,
    /// Étape 1 : tourner à droite — on enregistre le peak positif
    TurnRight { peak: f32 },
    /// Étape 2 : tourner à gauche — on enregistre le peak négatif
    TurnLeft { peak_right: f32, peak: f32 },
    /// Préparation : compte à rebours avant de devoir tenir droit
    PrepareHold { peak_right: f32, peak_left: f32, since: Instant },
    /// Étape 3 : tenir droit pendant HOLD_SECS
    HoldStraight { peak_right: f32, peak_left: f32, since: Instant },
    /// Calibration terminée, affichage live
    Done,
}

/// Actions collectées pendant le rendu pour être appliquées après
enum Action {
    None,
    GoTurnLeft  { peak_right: f32 },
    GoPrep      { peak_right: f32, peak_left: f32 },
    SetAxis     (u8),
    Recalibrate,
}

// ── Application egui ─────────────────────────────────────────────────────────

pub struct DriveForestApp {
    ips:  Vec<IpEntry>,
    step: CalibStep,
    axis: u8,
}

impl DriveForestApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { ips: collect_ips(), step: CalibStep::Waiting, axis: 2 }
    }
}

impl eframe::App for DriveForestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Rafraichissement permanent (~60 fps)
        ctx.request_repaint_after(Duration::from_millis(16));

        let steering  = get_steering();
        let connected = has_data();

        // ── Transitions d'état ────────────────────────────────────────────────

        // Dès réception des premières données : on pose une référence temporaire
        if matches!(self.step, CalibStep::Waiting) && connected {
            set_zero();
            self.step = CalibStep::TurnRight { peak: 0.0 };
        }

        // Mise à jour des peaks en temps réel
        match &mut self.step {
            CalibStep::TurnRight { peak }     => *peak = peak.max(steering),
            CalibStep::TurnLeft  { peak, .. } => *peak = peak.max(-steering),
            _ => {}
        }

        // Fin du compte à rebours de préparation → passage à HoldStraight
        if let CalibStep::PrepareHold { peak_right, peak_left, since } = &self.step {
            if since.elapsed().as_secs_f32() >= PREPARE_SECS {
                let (pr, pl) = (*peak_right, *peak_left);
                self.step = CalibStep::HoldStraight {
                    peak_right: pr,
                    peak_left:  pl,
                    since:      Instant::now(),
                };
            }
        }

        // Fin du compte à rebours → calibration finale
        if let CalibStep::HoldStraight { peak_right, peak_left, since } = &self.step {
            if since.elapsed().as_secs_f32() >= HOLD_SECS {
                let (pr, pl) = (*peak_right, *peak_left);
                set_zero();
                // Normalisation : on prend la moyenne des amplitudes observées
                let observed = ((pr + pl) * 0.5 * std::f32::consts::PI)
                    .max(std::f32::consts::FRAC_PI_4);
                set_max_angle(observed);
                set_axis(self.axis);
                self.step = CalibStep::Done;
            }
        }

        // ── Données pour le rendu (avant emprunts) ────────────────────────────
        let axis = self.axis;
        let (ci, cj, ck) = get_components();

        // ── Barre du haut ────────────────────────────────────────────────────
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("DriveForest");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if connected {
                        ui.colored_label(egui::Color32::GREEN, "Connecte");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(220, 80, 50), "En attente...");
                    }
                });
            });
        });

        // ── Panneau IPs (gauche) ──────────────────────────────────────────────
        let ips = &self.ips;
        egui::SidePanel::left("ips")
            .resizable(false)
            .exact_width(230.0)
            .show(ctx, |ui| {
                ui.heading("Connexion OSC");
                ui.label(format!("Port : {OSC_PORT}"));
                ui.separator();
                ui.add_space(4.0);

                if ips.is_empty() {
                    ui.colored_label(egui::Color32::RED, "Aucune interface detectee.");
                }
                for e in ips {
                    ui.label(egui::RichText::new(&e.label).small().color(egui::Color32::GRAY));
                    ui.label(
                        egui::RichText::new(format!("{}:{OSC_PORT}", e.addr))
                            .monospace()
                            .strong(),
                    );
                    ui.add_space(6.0);
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.heading("Sortie Unity");
                ui.label(egui::RichText::new("OSC UDP local").small().color(egui::Color32::GRAY));
                ui.label(
                    egui::RichText::new(format!("{}:{}", UNITY_IP, UNITY_PORT))
                        .monospace()
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(OSC_ADDRESS)
                        .monospace()
                        .color(egui::Color32::LIGHT_GREEN),
                );
            });

        // ── Panneau principal ─────────────────────────────────────────────────

        let mut action = Action::None;

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.step {

                // ── Attente téléphone ─────────────────────────────────────────
                CalibStep::Waiting => {
                    ui.add_space(60.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("En attente du telephone...")
                                .size(18.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(12.0);
                        ui.label("Lancez ZIG SIM et entrez l'une des adresses ci-contre.");
                        ui.label(format!("Protocole : OSC UDP | Port : {OSC_PORT}"));
                    });
                }

                // ── Étape 1 : droite ──────────────────────────────────────────
                CalibStep::TurnRight { peak } => {
                    let peak = *peak;

                    step_header(ui, 1, "Tournez le volant a DROITE au maximum");
                    ui.add_space(10.0);
                    steering_bar(ui, steering);
                    ui.add_space(6.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Peak droite : {:+.3}", peak))
                                .monospace(),
                        );
                    });
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("  Suivant  >").clicked() {
                            action = Action::GoTurnLeft { peak_right: peak };
                        }
                    });
                }

                // ── Étape 2 : gauche ──────────────────────────────────────────
                CalibStep::TurnLeft { peak_right, peak } => {
                    let (pr, pl) = (*peak_right, *peak);

                    step_header(ui, 2, "Tournez le volant a GAUCHE au maximum");
                    ui.add_space(10.0);
                    steering_bar(ui, steering);
                    ui.add_space(6.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Peak gauche : {:+.3}", -pl))
                                .monospace(),
                        );
                    });
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("  Suivant  >").clicked() {
                            action = Action::GoPrep { peak_right: pr, peak_left: pl };
                        }
                    });
                }

                // ── Préparation : compte à rebours avant de tenir droit ──────
                CalibStep::PrepareHold { since, .. } => {
                    let remaining = (PREPARE_SECS - since.elapsed().as_secs_f32()).max(0.0);
                    let progress  = (since.elapsed().as_secs_f32() / PREPARE_SECS).min(1.0);

                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Preparez-vous...")
                                .size(20.0)
                                .color(egui::Color32::YELLOW),
                        );
                        ui.add_space(8.0);
                        ui.label("Revenez en position droite, puis tenez.");
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new(format!("{:.1}", remaining))
                                .size(64.0)
                                .strong()
                                .color(egui::Color32::YELLOW),
                        );
                    });
                    ui.add_space(16.0);
                    ui.add(egui::ProgressBar::new(progress).animate(true));
                }

                // ── Étape 3 : tenir droit ─────────────────────────────────────
                CalibStep::HoldStraight { since, .. } => {
                    let elapsed  = since.elapsed().as_secs_f32();
                    let remaining = (HOLD_SECS - elapsed).max(0.0);
                    let progress  = (elapsed / HOLD_SECS).min(1.0);

                    step_header(ui, 3, "Tenez le volant DROIT");
                    ui.add_space(20.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}", remaining))
                                .size(64.0)
                                .strong(),
                        );
                    });

                    ui.add_space(16.0);
                    ui.add(egui::ProgressBar::new(progress).animate(true));
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.small("La position actuelle sera enregistree comme position neutre.");
                    });
                }

                // ── Affichage live ────────────────────────────────────────────
                CalibStep::Done => {
                    ui.heading("Braquage");
                    ui.add_space(10.0);
                    steering_bar(ui, steering);
                    ui.add_space(6.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:+.4}", steering))
                                .monospace()
                                .size(22.0),
                        );
                    });

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Sélecteur d'axe
                    ui.label("Axe de braquage :");
                    ui.horizontal(|ui| {
                        let mut a = axis;
                        ui.radio_value(&mut a, 0u8, "X  (cote telephone)");
                        ui.radio_value(&mut a, 1u8, "Y  (haut telephone)");
                        ui.radio_value(&mut a, 2u8, "Z  (normal ecran)");
                        if a != axis {
                            action = Action::SetAxis(a);
                        }
                    });

                    ui.add_space(8.0);

                    // Composantes brutes (diagnostic)
                    egui::CollapsingHeader::new("Composantes quaternion (diagnostic)")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::Grid::new("qcomp").spacing([16.0, 4.0]).show(ui, |ui| {
                                for (lbl, val) in [("X (i)", ci), ("Y (j)", cj), ("Z (k)", ck)] {
                                    ui.label(lbl);
                                    ui.label(
                                        egui::RichText::new(format!("{:+.5}", val)).monospace(),
                                    );
                                    component_bar(ui, val);
                                    ui.end_row();
                                }
                            });
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    if ui.button("Recalibrer").clicked() {
                        action = Action::Recalibrate;
                    }
                }
            }
        });

        // ── Application des actions ───────────────────────────────────────────
        match action {
            Action::None => {}
            Action::GoTurnLeft { peak_right } => {
                self.step = CalibStep::TurnLeft { peak_right, peak: 0.0 };
            }
            Action::GoPrep { peak_right, peak_left } => {
                self.step = CalibStep::PrepareHold {
                    peak_right,
                    peak_left,
                    since: Instant::now(),
                };
            }
            Action::SetAxis(a) => {
                self.axis = a;
                set_axis(a);
            }
            Action::Recalibrate => {
                set_zero();
                self.step = CalibStep::TurnRight { peak: 0.0 };
            }
        }
    }
}

// ── Widgets réutilisables ─────────────────────────────────────────────────────

/// En-tête d'étape avec numéro et description
fn step_header(ui: &mut egui::Ui, n: u8, text: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Etape {}/3", n))
                .strong()
                .color(egui::Color32::LIGHT_BLUE),
        );
        ui.separator();
        ui.label(text);
    });
    ui.add_space(4.0);
}

/// Barre de braquage centrée, valeur dans [-1, 1]
fn steering_bar(ui: &mut egui::Ui, value: f32) {
    let size   = egui::vec2(ui.available_width(), 34.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    if !ui.is_rect_visible(rect) { return; }

    let p = ui.painter();
    p.rect_filled(rect, 6.0, egui::Color32::from_gray(25));

    let cx = rect.center().x;
    // Ligne centrale
    p.line_segment(
        [egui::pos2(cx, rect.top() + 3.0), egui::pos2(cx, rect.bottom() - 3.0)],
        egui::Stroke::new(1.5, egui::Color32::from_gray(100)),
    );

    // Barre de valeur depuis le centre
    let vx = (cx + value * rect.width() * 0.5)
        .clamp(rect.left() + 2.0, rect.right() - 2.0);
    let (x0, x1) = if value >= 0.0 { (cx, vx) } else { (vx, cx) };
    let bar = egui::Rect::from_min_max(
        egui::pos2(x0, rect.top() + 3.0),
        egui::pos2(x1.max(x0 + 1.0), rect.bottom() - 3.0),
    );
    let color = if value.abs() > 0.85 {
        egui::Color32::from_rgb(220, 70, 50)
    } else if value.abs() > 0.5 {
        egui::Color32::from_rgb(220, 160, 40)
    } else {
        egui::Color32::from_rgb(50, 150, 220)
    };
    p.rect_filled(bar, 4.0, color);

    // Label centré
    p.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:+.3}", value),
        egui::FontId::monospace(12.0),
        egui::Color32::WHITE,
    );
}

/// Petite barre de composante dans [-1, 1] (pour le diagnostic quaternion)
fn component_bar(ui: &mut egui::Ui, value: f32) {
    let size  = egui::vec2(120.0, 14.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) { return; }

    let p = ui.painter();
    p.rect_filled(rect, 3.0, egui::Color32::from_gray(30));

    let cx = rect.center().x;
    let vx = (cx + value * rect.width() * 0.5).clamp(rect.left(), rect.right());
    let (x0, x1) = if value >= 0.0 { (cx, vx) } else { (vx, cx) };
    let bar = egui::Rect::from_min_max(
        egui::pos2(x0, rect.top() + 1.0),
        egui::pos2(x1.max(x0 + 1.0), rect.bottom() - 1.0),
    );
    p.rect_filled(bar, 2.0, egui::Color32::from_rgb(80, 180, 120));
}

// ── Point d'entrée ────────────────────────────────────────────────────────────

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DriveForest")
            .with_inner_size([740.0, 460.0])
            .with_min_inner_size([600.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DriveForest",
        options,
        Box::new(|cc| Ok(Box::new(DriveForestApp::new(cc)))),
    )
    .ok();
}
