use crate::api::ElgatoClient;
use crate::config::ConfigManager;
use crate::discovery::DiscoveryManager;
use crate::models::{
    kelvin_to_mired, kelvin_to_rgb, mired_to_kelvin, AppConfig, DeviceConfig, LightState,
    KELVIN_MAX, KELVIN_MIN,
};
use eframe::egui::{self, Color32, Layout, RichText, Slider, Vec2};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

pub struct ElgatoGuiApp {
    config_mgr: ConfigManager,
    config: AppConfig,
    client: ElgatoClient,
    rt: Runtime,

    device_states: HashMap<String, Option<LightState>>,
    is_scanning: bool,
    manual_ip_input: String,
    last_poll: Instant,

    master_brightness: u8,
    master_kelvin: u16,

    tx: Sender<GuiAsyncMsg>,
    rx: Receiver<GuiAsyncMsg>,
}

enum GuiAsyncMsg {
    Discovered(Vec<DeviceConfig>),
    StateFetched(String, Option<LightState>),
}

impl ElgatoGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_mgr = ConfigManager::new();
        let config = config_mgr.load();
        let client = ElgatoClient::new();
        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        let (tx, rx) = channel();

        let mut app = Self {
            config_mgr,
            config,
            client,
            rt,
            device_states: HashMap::new(),
            is_scanning: false,
            manual_ip_input: String::new(),
            last_poll: Instant::now(),
            master_brightness: 80,
            master_kelvin: 4500,
            tx,
            rx,
        };

        app.refresh_all_devices();
        app
    }

    fn refresh_all_devices(&mut self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let ips: Vec<String> = self.config.devices.iter().map(|d| d.ip.clone()).collect();

        self.rt.spawn(async move {
            for ip in ips {
                let state = match client.get_lights(&ip).await {
                    Ok(resp) => resp.lights.first().cloned(),
                    Err(_) => None,
                };
                let _ = tx.send(GuiAsyncMsg::StateFetched(ip, state));
            }
        });
    }

    fn start_discovery(&mut self) {
        self.is_scanning = true;
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            let devs = DiscoveryManager::discover_mdns(3).await;
            let _ = tx.send(GuiAsyncMsg::Discovered(devs));
        });
    }

    fn apply_preset(&mut self, preset_name: &str) {
        if let Some(preset) = self
            .config
            .presets
            .iter()
            .find(|p| p.name == preset_name)
            .cloned()
        {
            let client = self.client.clone();
            let ips: Vec<String> = self.config.devices.iter().map(|d| d.ip.clone()).collect();
            let tx = self.tx.clone();

            self.master_brightness = preset.brightness;
            self.master_kelvin = preset.kelvin;

            // Optimistic update
            for ip in &ips {
                if let Some(Some(st)) = self.device_states.get_mut(ip) {
                    st.on = if preset.on { 1 } else { 0 };
                    st.brightness = preset.brightness;
                    st.temperature = Some(kelvin_to_mired(preset.kelvin));
                }
            }

            self.rt.spawn(async move {
                for ip in ips {
                    let _ = client
                        .set_settings(
                            &ip,
                            Some(preset.on),
                            Some(preset.brightness),
                            Some(preset.kelvin),
                        )
                        .await;
                    let state = match client.get_lights(&ip).await {
                        Ok(resp) => resp.lights.first().cloned(),
                        Err(_) => None,
                    };
                    let _ = tx.send(GuiAsyncMsg::StateFetched(ip, state));
                }
            });
        }
    }
}

impl eframe::App for ElgatoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain pending async messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GuiAsyncMsg::Discovered(devs) => {
                    self.is_scanning = false;
                    for dev in devs {
                        let _ = self.config_mgr.add_or_update_device(dev);
                    }
                    self.config = self.config_mgr.load();
                    self.refresh_all_devices();
                }
                GuiAsyncMsg::StateFetched(ip, state) => {
                    self.device_states.insert(ip, state);
                }
            }
        }

        // Intercept close button to minimize/hide to system tray
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Periodic background poll every 1.5 seconds
        if self.last_poll.elapsed() >= Duration::from_millis(1500) {
            self.last_poll = Instant::now();
            self.refresh_all_devices();
        }

        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("💡 OpenCenter").strong().size(22.0));
                ui.label(
                    RichText::new("(Elgato Control)")
                        .italics()
                        .color(Color32::GRAY),
                );
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.is_scanning {
                            "⏳ Scanning..."
                        } else {
                            "🔍 Auto-Discover"
                        })
                        .clicked()
                        && !self.is_scanning
                    {
                        self.start_discovery();
                    }
                    if ui.button("🔄 Refresh").clicked() {
                        self.refresh_all_devices();
                    }
                });
            });

            ui.separator();

            // Presets Bar
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Presets:").strong());
                    let presets = self.config.presets.clone();
                    for p in presets {
                        if ui.button(format!("✨ {}", p.name)).clicked() {
                            self.apply_preset(&p.name);
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Master Controls
            ui.group(|ui| {
                ui.label(
                    RichText::new("🎛️ Master Control (All Lights)")
                        .strong()
                        .size(16.0),
                );
                ui.horizontal(|ui| {
                    if ui.button("🟢 Turn All ON").clicked() {
                        let client = self.client.clone();
                        let ips: Vec<String> =
                            self.config.devices.iter().map(|d| d.ip.clone()).collect();
                        let tx = self.tx.clone();

                        for ip in &ips {
                            if let Some(Some(st)) = self.device_states.get_mut(ip) {
                                st.on = 1;
                            }
                        }

                        self.rt.spawn(async move {
                            for ip in ips {
                                let _ = client.set_power(&ip, true).await;
                                let state = match client.get_lights(&ip).await {
                                    Ok(resp) => resp.lights.first().cloned(),
                                    Err(_) => None,
                                };
                                let _ = tx.send(GuiAsyncMsg::StateFetched(ip, state));
                            }
                        });
                    }
                    if ui.button("🔴 Turn All OFF").clicked() {
                        let client = self.client.clone();
                        let ips: Vec<String> =
                            self.config.devices.iter().map(|d| d.ip.clone()).collect();
                        let tx = self.tx.clone();

                        for ip in &ips {
                            if let Some(Some(st)) = self.device_states.get_mut(ip) {
                                st.on = 0;
                            }
                        }

                        self.rt.spawn(async move {
                            for ip in ips {
                                let _ = client.set_power(&ip, false).await;
                                let state = match client.get_lights(&ip).await {
                                    Ok(resp) => resp.lights.first().cloned(),
                                    Err(_) => None,
                                };
                                let _ = tx.send(GuiAsyncMsg::StateFetched(ip, state));
                            }
                        });
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Master Brightness:");
                    let mut b = self.master_brightness;
                    if ui.add(Slider::new(&mut b, 0..=100).suffix("%")).changed() {
                        self.master_brightness = b;
                        let client = self.client.clone();
                        let ips: Vec<String> =
                            self.config.devices.iter().map(|d| d.ip.clone()).collect();
                        let tx = self.tx.clone();

                        for ip in &ips {
                            if let Some(Some(s)) = self.device_states.get_mut(ip) {
                                s.brightness = b;
                            }
                        }

                        self.rt.spawn(async move {
                            for ip in ips {
                                let _ = client.set_settings(&ip, None, Some(b), None).await;
                                let confirmed = match client.get_lights(&ip).await {
                                    Ok(resp) => resp.lights.first().cloned(),
                                    Err(_) => None,
                                };
                                let _ = tx.send(GuiAsyncMsg::StateFetched(ip, confirmed));
                            }
                        });
                    }

                    ui.label("Master Temp:");
                    let mut k = self.master_kelvin;

                    // Color indicator box for Kelvin temperature
                    let (r, g, b_val) = kelvin_to_rgb(k);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, 4.0, Color32::from_rgb(r, g, b_val));

                    if ui
                        .add(Slider::new(&mut k, KELVIN_MIN..=KELVIN_MAX).suffix("K"))
                        .changed()
                    {
                        self.master_kelvin = k;
                        let client = self.client.clone();
                        let ips: Vec<String> =
                            self.config.devices.iter().map(|d| d.ip.clone()).collect();
                        let tx = self.tx.clone();

                        let target_mired = kelvin_to_mired(k);
                        for ip in &ips {
                            if let Some(Some(s)) = self.device_states.get_mut(ip) {
                                s.temperature = Some(target_mired);
                            }
                        }

                        self.rt.spawn(async move {
                            for ip in ips {
                                let _ = client.set_settings(&ip, None, None, Some(k)).await;
                                let confirmed = match client.get_lights(&ip).await {
                                    Ok(resp) => resp.lights.first().cloned(),
                                    Err(_) => None,
                                };
                                let _ = tx.send(GuiAsyncMsg::StateFetched(ip, confirmed));
                            }
                        });
                    }
                });
            });

            ui.add_space(10.0);
            ui.heading("Devices");

            if self.config.devices.is_empty() {
                ui.label(
                    "No devices configured. Click 'Auto-Discover' or enter an IP address below.",
                );
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let devices = self.config.devices.clone();
                    for dev in devices {
                        let state = self.device_states.get(&dev.ip).cloned().flatten();

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let is_on = state.as_ref().map(|s| s.on == 1).unwrap_or(false);
                                let pwr_text = if is_on { "🟢 ON" } else { "🔴 OFF" };
                                ui.label(RichText::new(&dev.name).strong().size(15.0));
                                ui.label(format!("({})", dev.ip));

                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑️ Remove").clicked() {
                                        let _ = self.config_mgr.remove_device(&dev.ip);
                                        self.config = self.config_mgr.load();
                                    }
                                    if ui.button("⚡ Identify").clicked() {
                                        let client = self.client.clone();
                                        let ip = dev.ip.clone();
                                        self.rt.spawn(async move {
                                            let _ = client.identify(&ip).await;
                                        });
                                    }
                                    if ui.button(pwr_text).clicked() {
                                        let client = self.client.clone();
                                        let ip = dev.ip.clone();
                                        let tx = self.tx.clone();

                                        if let Some(Some(st)) = self.device_states.get_mut(&dev.ip)
                                        {
                                            st.on = if st.on == 1 { 0 } else { 1 };
                                        }

                                        self.rt.spawn(async move {
                                            let _ = client.toggle_power(&ip).await;
                                            let new_st = match client.get_lights(&ip).await {
                                                Ok(resp) => resp.lights.first().cloned(),
                                                Err(_) => None,
                                            };
                                            let _ = tx.send(GuiAsyncMsg::StateFetched(ip, new_st));
                                        });
                                    }
                                });
                            });

                            if let Some(mut st) = state {
                                ui.horizontal(|ui| {
                                    ui.label("Brightness:");
                                    let mut b_val = st.brightness;
                                    if ui
                                        .add(Slider::new(&mut b_val, 0..=100).suffix("%"))
                                        .changed()
                                    {
                                        st.brightness = b_val;
                                        let client = self.client.clone();
                                        let ip = dev.ip.clone();
                                        let state_to_send = st.clone();
                                        let tx = self.tx.clone();

                                        if let Some(Some(s)) = self.device_states.get_mut(&dev.ip) {
                                            s.brightness = b_val;
                                        }

                                        self.rt.spawn(async move {
                                            let _ = client.set_lights(&ip, &state_to_send).await;
                                            let confirmed = match client.get_lights(&ip).await {
                                                Ok(resp) => resp.lights.first().cloned(),
                                                Err(_) => None,
                                            };
                                            let _ =
                                                tx.send(GuiAsyncMsg::StateFetched(ip, confirmed));
                                        });
                                    }

                                    let mired = st.temperature.unwrap_or(250);
                                    let mut kelvin = mired_to_kelvin(mired);
                                    ui.label("Temp:");

                                    // Visual color temperature box
                                    let (r, g, b_col) = kelvin_to_rgb(kelvin);
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(16.0, 16.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        4.0,
                                        Color32::from_rgb(r, g, b_col),
                                    );

                                    if ui
                                        .add(
                                            Slider::new(&mut kelvin, KELVIN_MIN..=KELVIN_MAX)
                                                .suffix("K"),
                                        )
                                        .changed()
                                    {
                                        let client = self.client.clone();
                                        let ip = dev.ip.clone();
                                        let tx = self.tx.clone();

                                        self.rt.spawn(async move {
                                            let _ = client
                                                .set_settings(&ip, None, None, Some(kelvin))
                                                .await;
                                            let confirmed = match client.get_lights(&ip).await {
                                                Ok(resp) => resp.lights.first().cloned(),
                                                Err(_) => None,
                                            };
                                            let _ =
                                                tx.send(GuiAsyncMsg::StateFetched(ip, confirmed));
                                        });
                                    }
                                });
                            } else {
                                ui.label(
                                    RichText::new("Connecting / Offline")
                                        .italics()
                                        .color(Color32::GRAY),
                                );
                            }
                        });
                        ui.add_space(4.0);
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Add IP Manually:");
                ui.text_edit_singleline(&mut self.manual_ip_input);
                if ui.button("➕ Add Device").clicked() && !self.manual_ip_input.is_empty() {
                    let ip = self.manual_ip_input.trim().to_string();
                    let dev = DeviceConfig {
                        ip: ip.clone(),
                        name: format!("Elgato Light ({})", ip),
                        serial: None,
                        model: None,
                        enabled: true,
                    };
                    let _ = self.config_mgr.add_or_update_device(dev);
                    self.config = self.config_mgr.load();
                    self.manual_ip_input.clear();
                    self.refresh_all_devices();
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(300));
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([680.0, 560.0])
            .with_title("OpenCenter - Elgato Control Center"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenCenter",
        options,
        Box::new(|cc| Ok(Box::new(ElgatoGuiApp::new(cc)))),
    )
}
