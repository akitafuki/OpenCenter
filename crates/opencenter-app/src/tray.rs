use ksni::menu::{MenuItem, StandardItem};
use ksni::{Handle, ToolTip, Tray, TrayService};
use opencenter_core::api::ElgatoClient;
use opencenter_core::config::ConfigManager;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct ElgatoTray {
    config_mgr: ConfigManager,
    client: ElgatoClient,
    rt: Arc<Runtime>,
}

impl ElgatoTray {
    pub fn new(rt: Arc<Runtime>) -> Self {
        Self {
            config_mgr: ConfigManager::new(),
            client: ElgatoClient::new(),
            rt,
        }
    }
}

impl Tray for ElgatoTray {
    fn id(&self) -> String {
        "opencenter".to_string()
    }

    fn title(&self) -> String {
        "Elgato Control Center".to_string()
    }

    fn icon_name(&self) -> String {
        "weather-clear".to_string()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Elgato Control Center".to_string(),
            description: "Linux Elgato Key Light Controller".to_string(),
            icon_name: "weather-clear".to_string(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let cfg = self.config_mgr.load();
        let client = self.client.clone();
        let rt = self.rt.clone();

        let mut items = vec![
            MenuItem::Standard(StandardItem {
                label: "⚡ Toggle All Lights".to_string(),
                activate: Box::new(move |_| {
                    let client_c = client.clone();
                    let cfg_c = ConfigManager::new().load();
                    rt.spawn(async move {
                        for dev in cfg_c.devices {
                            let _ = client_c.toggle_power(&dev.ip).await;
                        }
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
        ];

        for preset in &cfg.presets {
            let p_name = preset.name.clone();
            let p_on = preset.on;
            let p_bright = preset.brightness;
            let p_kelvin = preset.kelvin;
            let client_c = self.client.clone();
            let rt_c = self.rt.clone();

            items.push(MenuItem::Standard(StandardItem {
                label: format!("✨ Preset: {}", p_name),
                activate: Box::new(move |_| {
                    let client_cc = client_c.clone();
                    let cfg_cc = ConfigManager::new().load();
                    rt_c.spawn(async move {
                        for dev in cfg_cc.devices {
                            let _ = client_cc
                                .set_settings(&dev.ip, Some(p_on), Some(p_bright), Some(p_kelvin))
                                .await;
                        }
                    });
                }),
                ..Default::default()
            }));
        }

        items.push(MenuItem::Separator);

        items.push(MenuItem::Standard(StandardItem {
            label: "🖥️ Open GUI Window".to_string(),
            activate: Box::new(|_| {
                std::thread::spawn(|| {
                    let _ = crate::gui::run_gui();
                });
            }),
            ..Default::default()
        }));

        items.push(MenuItem::Standard(StandardItem {
            label: "❌ Quit".to_string(),
            activate: Box::new(|_| {
                std::process::exit(0);
            }),
            ..Default::default()
        }));

        items
    }
}

pub fn spawn_tray(rt: Arc<Runtime>) -> Handle<ElgatoTray> {
    let tray = ElgatoTray::new(rt);
    let service = TrayService::new(tray);
    let handle = service.handle();
    service.spawn();
    handle
}
