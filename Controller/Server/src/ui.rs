use std::net::IpAddr;
use std::time::{Duration, Instant};

use eframe::egui;
use if_addrs::get_if_addrs;

use crate::core::{
    get_accel, get_components, get_steering, has_data,
    set_accel_axis, set_steering_axis, set_zero, apply_calibration, AXIS_NONE,
};
use crate::network::input::{set_shake_threshold, take_honk_flash};
use crate::network::output::osc_output::{OSC_ADDRESS, UNITY_IP, UNITY_PORT};

const OSC_PORT:    u16  = 50000;
const ZERO_SECS:   f32  = 2.0;
const HOLD_SECS:   f32  = 2.5;
const RETURN_SECS: f32  = ZERO_SECS;

// ── IPs ───────────────────────────────────────────────────────────────────────

struct IpEntry { label: String, addr: String }

fn classify(name: &str, ip: IpAddr) -> Option<IpEntry> {
    let v4 = match ip { IpAddr::V4(v) => v, IpAddr::V6(_) => return None };
    if v4.is_loopback() { return None; }
    let [a, b, ..] = v4.octets();
    if a == 169 && b == 254 { return None; }

    let n = name.to_lowercase();
    let label = if n.contains("wi-fi") || n.contains("wlan") || n.contains("wireless") {
        format!("WiFi — {}", name)
    } else if n.contains("hotspot") || n.contains("softap") || n.contains("virtual")
        || (n.contains("local area connection") && n.contains('*'))
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
    let Ok(ifaces) = get_if_addrs() else { return vec![] };
    ifaces.into_iter().filter_map(|i| classify(&i.name, i.ip())).collect()
}

// ── Calibration ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum CalibPhase { TurnRight, TurnLeft, Accel, Brake }

impl CalibPhase {
    fn instruction(self) -> &'static str {
        match self {
            Self::TurnRight => "Tournez a DROITE et maintenez",
            Self::TurnLeft  => "Tournez a GAUCHE et maintenez",
            Self::Accel     => "Inclinez vers l'AVANT (accelerer) et maintenez",
            Self::Brake     => "Inclinez vers l'ARRIERE (freiner) et maintenez",
        }
    }
    fn color(self) -> egui::Color32 {
        match self {
            Self::TurnRight | Self::TurnLeft => egui::Color32::from_rgb(80, 160, 255),
            Self::Accel | Self::Brake        => egui::Color32::from_rgb(80, 220, 120),
        }
    }
}

enum CalibMode {
    Idle,
    /// Retour au neutre initial (set_zero à la fin)
    NeutralCountdown(Instant),
    /// Maintien d'une position pour mesure
    Holding { phase: CalibPhase, start: Instant, acc: [f64; 3], n: u32 },
    /// Retour au centre entre deux étapes (pas de set_zero)
    ReturnToCenter { next: CalibPhase, countdown: Instant },
    Done,
}

fn dominant_axis(avg: &[f32; 3]) -> u8 {
    avg.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

fn dominant_axis_except(avg: &[f32; 3], exclude: u8) -> u8 {
    avg.iter().enumerate()
        .filter(|(i, _)| *i as u8 != exclude)
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
        .map(|(i, _)| i as u8)
        .unwrap_or(if exclude == 0 { 1 } else { 0 })
}

fn axis_name(a: u8) -> &'static str {
    match a { 0 => "X", 1 => "Y", 2 => "Z", _ => "—" }
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct DriveForestApp {
    ips:              Vec<IpEntry>,
    ready:            bool,
    zero_countdown:   Option<Instant>,
    steering_axis:    u8,
    accel_axis:       u8,
    shake_threshold:  f32,
    honk_flash_until: Option<Instant>,
    // Calibration
    calib:            CalibMode,
    calib_right:      Option<[f32; 3]>,   // avg composantes pendant TurnRight
    calib_accel_fwd:  Option<[f32; 3]>,   // avg composantes pendant Accel
    calib_steer_sign: f32,
    calib_accel_sign: f32,
}

impl DriveForestApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            ips:              collect_ips(),
            ready:            false,
            zero_countdown:   None,
            steering_axis:    2,
            accel_axis:       AXIS_NONE,
            shake_threshold:  8.0,
            honk_flash_until: None,
            calib:            CalibMode::Idle,
            calib_right:      None,
            calib_accel_fwd:  None,
            calib_steer_sign: 1.0,
            calib_accel_sign: 1.0,
        }
    }
}

impl eframe::App for DriveForestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));

        let connected = has_data();

        if !self.ready && connected {
            set_zero();
            self.ready = true;
        }

        if let Some(start) = self.zero_countdown {
            if start.elapsed().as_secs_f32() >= ZERO_SECS {
                set_zero();
                self.zero_countdown = None;
            }
        }

        if take_honk_flash() {
            self.honk_flash_until = Some(Instant::now() + Duration::from_millis(400));
        }

        let steering = get_steering();
        let accel    = get_accel();
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

        // ── Panneau IPs ───────────────────────────────────────────────────────
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
                    ui.label(egui::RichText::new(format!("{}:{OSC_PORT}", e.addr)).monospace().strong());
                    ui.add_space(6.0);
                }
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                ui.heading("Sortie Unity");
                ui.label(egui::RichText::new("OSC UDP local").small().color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{}:{}", UNITY_IP, UNITY_PORT)).monospace().strong());
                ui.label(egui::RichText::new(OSC_ADDRESS).monospace().color(egui::Color32::LIGHT_GREEN));
            });

        // ── Panneau principal ─────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            if !connected {
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("En attente du telephone...").size(18.0).color(egui::Color32::GRAY));
                    ui.add_space(12.0);
                    ui.label("Lancez ZIG SIM et entrez l'une des adresses ci-contre.");
                    ui.label(format!("Protocole : OSC UDP | Port : {OSC_PORT}"));
                });
                return;
            }

            // ── Barres de valeurs (toujours visibles) ─────────────────────────
            ui.label(egui::RichText::new("Braquage").strong());
            value_bar(ui, steering);
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Acceleration").strong());
            value_bar(ui, accel);
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Calibration ───────────────────────────────────────────────────
            let calib_is_idle = matches!(self.calib, CalibMode::Idle);
            let calib_is_done = matches!(self.calib, CalibMode::Done);

            if !calib_is_idle {
                self.show_calibration(ui, ci, cj, ck);
                return; // masque les contrôles normaux pendant la calibration
            }

            // ── Contrôles normaux ─────────────────────────────────────────────
            egui::Grid::new("axes").spacing([40.0, 6.0]).show(ui, |ui| {
                ui.label("Axe braquage :");
                let mut sa = self.steering_axis;
                ui.radio_value(&mut sa, 0u8, "X");
                ui.radio_value(&mut sa, 1u8, "Y");
                ui.radio_value(&mut sa, 2u8, "Z");
                ui.radio_value(&mut sa, AXIS_NONE, "Aucun");
                if sa != self.steering_axis { self.steering_axis = sa; set_steering_axis(sa); }
                ui.end_row();

                ui.label("Axe acceleration :");
                let mut aa = self.accel_axis;
                ui.radio_value(&mut aa, 0u8, "X");
                ui.radio_value(&mut aa, 1u8, "Y");
                ui.radio_value(&mut aa, 2u8, "Z");
                ui.radio_value(&mut aa, AXIS_NONE, "Aucun");
                if aa != self.accel_axis { self.accel_axis = aa; set_accel_axis(aa); }
                ui.end_row();
            });

            ui.add_space(10.0);

            egui::CollapsingHeader::new("Composantes quaternion (diagnostic)")
                .default_open(false)
                .show(ui, |ui| {
                    egui::Grid::new("qcomp").spacing([16.0, 4.0]).show(ui, |ui| {
                        for (lbl, val) in [("X (i)", ci), ("Y (j)", cj), ("Z (k)", ck)] {
                            ui.label(lbl);
                            ui.label(egui::RichText::new(format!("{:+.5}", val)).monospace());
                            component_bar(ui, val);
                            ui.end_row();
                        }
                    });
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Sensibilité klaxon ────────────────────────────────────────────
            let honk_active = self.honk_flash_until.map_or(false, |t| Instant::now() < t);
            ui.horizontal(|ui| {
                ui.label("Sensibilite klaxon (secousse) :");
                let color = if honk_active { egui::Color32::from_rgb(255, 220, 0) } else { egui::Color32::from_gray(60) };
                let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 7.0, color);
                if honk_active { ui.colored_label(color, "KLAXON !"); }
            });
            let resp = ui.add(
                egui::Slider::new(&mut self.shake_threshold, 2.0f32..=20.0)
                    .suffix(" m/s²").text("seuil delta"),
            );
            if resp.changed() { set_shake_threshold(self.shake_threshold); }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Set Zero ──────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                match self.zero_countdown {
                    None => {
                        if ui.button("  Set Zero  ").clicked() {
                            self.zero_countdown = Some(Instant::now());
                        }
                    }
                    Some(start) => {
                        let elapsed   = start.elapsed().as_secs_f32();
                        let remaining = (ZERO_SECS - elapsed).max(0.0);
                        let progress  = (elapsed / ZERO_SECS).min(1.0);
                        ui.label(egui::RichText::new(format!("Set Zero dans {:.1}s...", remaining))
                            .color(egui::Color32::YELLOW));
                        if ui.small_button("Annuler").clicked() { self.zero_countdown = None; }
                        ui.add(egui::ProgressBar::new(progress).animate(true).desired_width(80.0));
                    }
                }

                ui.add_space(16.0);

                // ── Bouton Calibrer ───────────────────────────────────────────
                if ui.button("  Calibrer  ").clicked() {
                    self.calib_right     = None;
                    self.calib_accel_fwd = None;
                    self.calib           = CalibMode::NeutralCountdown(Instant::now());
                }

                // Résultat de la dernière calibration
                if calib_is_done {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::LIGHT_GREEN,
                        format!("Calib OK — braquage:{} accel:{}",
                            axis_name(self.steering_axis), axis_name(self.accel_axis)));
                }
            });
        });
    }
}

// ── Rendu calibration ─────────────────────────────────────────────────────────

impl DriveForestApp {
    fn show_calibration(&mut self, ui: &mut egui::Ui, ci: f32, cj: f32, ck: f32) {
        // Snapshot des données inter-étapes (nécessaire pour le borrow checker)
        let right_snap  = self.calib_right;
        let accel_snap  = self.calib_accel_fwd;

        // Accumuler les échantillons pendant Holding (avant le match)
        if let CalibMode::Holding { acc, n, .. } = &mut self.calib {
            acc[0] += ci as f64;
            acc[1] += cj as f64;
            acc[2] += ck as f64;
            *n += 1;
        }

        // Variable de transition (évite les conflits de borrow)
        let mut transition: Option<CalibMode> = None;
        let mut store_right: Option<[f32; 3]> = None;
        let mut store_accel: Option<[f32; 3]> = None;
        let mut do_apply: Option<(u8, f32, u8, f32)> = None;
        let mut cancel = false;

        match &self.calib {

            // ── Neutre initial ────────────────────────────────────────────────
            CalibMode::NeutralCountdown(start) => {
                let elapsed   = start.elapsed().as_secs_f32();
                let remaining = (ZERO_SECS - elapsed).max(0.0);
                let progress  = (elapsed / ZERO_SECS).min(1.0);

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("CALIBRATION").size(22.0).strong());
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Tenez le telephone en position NEUTRE").size(16.0)
                        .color(egui::Color32::YELLOW));
                    ui.add_space(4.0);
                    ui.label(format!("Enregistrement du zero dans {:.1}s...", remaining));
                    ui.add_space(8.0);
                    ui.add(egui::ProgressBar::new(progress).animate(true).desired_width(300.0));
                    ui.add_space(12.0);
                    if ui.button("Annuler").clicked() { cancel = true; }
                });

                if elapsed >= ZERO_SECS {
                    set_zero();
                    transition = Some(CalibMode::Holding {
                        phase: CalibPhase::TurnRight,
                        start: Instant::now(),
                        acc:   [0.0; 3],
                        n:     0,
                    });
                }
            }

            // ── Maintien d'une position ───────────────────────────────────────
            CalibMode::Holding { phase, start, acc, n } => {
                let phase     = *phase;
                let elapsed   = start.elapsed().as_secs_f32();
                let remaining = (HOLD_SECS - elapsed).max(0.0);
                let progress  = (elapsed / HOLD_SECS).min(1.0);

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("CALIBRATION").size(22.0).strong());
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(phase.instruction()).size(16.0)
                        .color(phase.color()));
                    ui.add_space(4.0);
                    ui.label(format!("Maintenez encore {:.1}s...", remaining));
                    ui.add_space(8.0);
                    ui.add(egui::ProgressBar::new(progress)
                        .fill(phase.color())
                        .desired_width(300.0));
                    ui.add_space(4.0);
                    // Composantes live
                    egui::Grid::new("calib_comp").spacing([8.0, 2.0]).show(ui, |ui| {
                        for (lbl, val) in [("i", ci), ("j", cj), ("k", ck)] {
                            ui.label(lbl);
                            ui.label(egui::RichText::new(format!("{:+.4}", val)).monospace());
                            component_bar(ui, val);
                            ui.end_row();
                        }
                    });
                    ui.add_space(12.0);
                    if ui.button("Annuler").clicked() { cancel = true; }
                });

                if elapsed >= HOLD_SECS {
                    let n_f = (*n as f64).max(1.0);
                    let avg = [
                        (acc[0] / n_f) as f32,
                        (acc[1] / n_f) as f32,
                        (acc[2] / n_f) as f32,
                    ];

                    match phase {
                        CalibPhase::TurnRight => {
                            store_right = Some(avg);
                            transition = Some(CalibMode::ReturnToCenter {
                                next: CalibPhase::TurnLeft,
                                countdown: Instant::now(),
                            });
                        }
                        CalibPhase::TurnLeft => {
                            transition = Some(CalibMode::ReturnToCenter {
                                next: CalibPhase::Accel,
                                countdown: Instant::now(),
                            });
                        }
                        CalibPhase::Accel => {
                            store_accel = Some(avg);
                            transition = Some(CalibMode::ReturnToCenter {
                                next: CalibPhase::Brake,
                                countdown: Instant::now(),
                            });
                        }
                        CalibPhase::Brake => {
                            let right     = right_snap.unwrap_or([0.0, 0.0, 1.0]);
                            let accel_fwd = accel_snap.unwrap_or([1.0, 0.0, 0.0]);

                            let steer_axis = dominant_axis(&right);
                            let steer_val  = right[steer_axis as usize];
                            // Formule: steering = sign * -(angle/max)
                            // Si steer_val > 0 quand on tourne à droite → sortie négative → on veut positif → sign = -1
                            let steer_sign = if steer_val > 0.0 { -1.0f32 } else { 1.0 };

                            let accel_axis = dominant_axis_except(&accel_fwd, steer_axis);
                            let accel_val  = accel_fwd[accel_axis as usize];
                            let accel_sign = if accel_val > 0.0 { -1.0f32 } else { 1.0 };

                            do_apply  = Some((steer_axis, steer_sign, accel_axis, accel_sign));
                            transition = Some(CalibMode::Done);
                        }
                    }
                }
            }

            // ── Retour au centre ──────────────────────────────────────────────
            CalibMode::ReturnToCenter { next, countdown } => {
                let next      = *next;
                let elapsed   = countdown.elapsed().as_secs_f32();
                let remaining = (RETURN_SECS - elapsed).max(0.0);
                let progress  = (elapsed / RETURN_SECS).min(1.0);

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("CALIBRATION").size(22.0).strong());
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Revenez en position NEUTRE").size(16.0)
                        .color(egui::Color32::YELLOW));
                    ui.add_space(4.0);
                    ui.label(format!("Prochaine etape dans {:.1}s : {}", remaining, next.instruction()));
                    ui.add_space(8.0);
                    ui.add(egui::ProgressBar::new(progress).animate(true).desired_width(300.0));
                    ui.add_space(4.0);
                    // Composantes live pour voir qu'on est bien revenu au neutre
                    egui::Grid::new("ret_comp").spacing([8.0, 2.0]).show(ui, |ui| {
                        for (lbl, val) in [("i", ci), ("j", cj), ("k", ck)] {
                            ui.label(lbl);
                            ui.label(egui::RichText::new(format!("{:+.4}", val)).monospace());
                            component_bar(ui, val);
                            ui.end_row();
                        }
                    });
                    ui.add_space(12.0);
                    if ui.button("Annuler").clicked() { cancel = true; }
                });

                if elapsed >= RETURN_SECS {
                    transition = Some(CalibMode::Holding {
                        phase: next,
                        start: Instant::now(),
                        acc:   [0.0; 3],
                        n:     0,
                    });
                }
            }

            // ── Résultat ──────────────────────────────────────────────────────
            CalibMode::Done => {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Calibration terminee !").size(22.0)
                        .color(egui::Color32::LIGHT_GREEN).strong());
                    ui.add_space(16.0);
                    egui::Grid::new("calib_res").spacing([20.0, 6.0]).show(ui, |ui| {
                        ui.label("Axe braquage :");
                        ui.label(egui::RichText::new(axis_name(self.steering_axis)).monospace().strong());
                        let dir = if self.calib_steer_sign < 0.0 { "(inverse)" } else { "(normal)" };
                        ui.label(dir);
                        ui.end_row();
                        ui.label("Axe acceleration :");
                        ui.label(egui::RichText::new(axis_name(self.accel_axis)).monospace().strong());
                        let dir = if self.calib_accel_sign < 0.0 { "(inverse)" } else { "(normal)" };
                        ui.label(dir);
                        ui.end_row();
                    });
                    ui.add_space(16.0);
                    if ui.button("  OK  ").clicked() {
                        // On reste en Done pour afficher "Calib OK" dans le bas de l'UI normale
                    }
                });
                // Clic OK → retour Idle
                if ui.button("Retour").clicked() { cancel = true; }
            }

            CalibMode::Idle => {}
        }

        // ── Appliquer les transitions ─────────────────────────────────────────
        if cancel {
            self.calib = CalibMode::Idle;
        } else if let Some(mode) = transition {
            self.calib = mode;
            if let Some(r) = store_right { self.calib_right = Some(r); }
            if let Some(a) = store_accel { self.calib_accel_fwd = Some(a); }
            if let Some((sa, ss, aa, acs)) = do_apply {
                apply_calibration(sa, ss, aa, acs);
                self.steering_axis    = sa;
                self.accel_axis       = aa;
                self.calib_steer_sign = ss;
                self.calib_accel_sign = acs;
            }
        }
    }
}

// ── Widgets ───────────────────────────────────────────────────────────────────

fn value_bar(ui: &mut egui::Ui, value: f32) {
    let size = egui::vec2(ui.available_width(), 30.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) { return; }

    let p = ui.painter();
    p.rect_filled(rect, 6.0, egui::Color32::from_gray(25));

    let cx = rect.center().x;
    p.line_segment(
        [egui::pos2(cx, rect.top() + 3.0), egui::pos2(cx, rect.bottom() - 3.0)],
        egui::Stroke::new(1.5, egui::Color32::from_gray(100)),
    );

    let vx = (cx + value * rect.width() * 0.5).clamp(rect.left() + 2.0, rect.right() - 2.0);
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
    p.text(rect.center(), egui::Align2::CENTER_CENTER,
        format!("{:+.3}", value), egui::FontId::monospace(12.0), egui::Color32::WHITE);
}

fn component_bar(ui: &mut egui::Ui, value: f32) {
    let size = egui::vec2(120.0, 14.0);
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
            .with_inner_size([740.0, 520.0])
            .with_min_inner_size([600.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native("DriveForest", options,
        Box::new(|cc| Ok(Box::new(DriveForestApp::new(cc))))).ok();
}
