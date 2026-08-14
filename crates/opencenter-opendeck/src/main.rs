use async_trait::async_trait;
use openaction::*;
use opencenter_core::api::ElgatoClient;
use opencenter_core::config::ConfigManager;
use opencenter_core::models::{mired_to_kelvin, KELVIN_MAX, KELVIN_MIN};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct TargetSettings {
    pub target: String,
}

pub struct ToggleAction;

#[async_trait]
impl Action for ToggleAction {
    const UUID: ActionUuid = "com.akitafuki.opencenter.toggle";
    type Settings = TargetSettings;

    async fn will_appear(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        let ips = resolve_target_ips(&settings.target);
        if let Some(first_ip) = ips.first() {
            let client = ElgatoClient::new();
            if let Ok(state) = client.get_lights(first_ip).await {
                if let Some(light) = state.lights.first() {
                    let _ = instance.set_state(light.on as u16).await;
                }
            }
        }
        Ok(())
    }

    async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        let ips = resolve_target_ips(&settings.target);
        let client = ElgatoClient::new();
        let mut any_on = false;

        for ip in &ips {
            if let Ok(new_state) = client.toggle_power(ip).await {
                if new_state {
                    any_on = true;
                }
            }
        }

        let _ = instance.set_state(if any_on { 1 } else { 0 }).await;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PresetSettings {
    pub name: String,
}

pub struct PresetAction;

#[async_trait]
impl Action for PresetAction {
    const UUID: ActionUuid = "com.akitafuki.opencenter.preset";
    type Settings = PresetSettings;

    async fn will_appear(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        if !settings.name.is_empty() {
            let _ = instance.set_title(Some(settings.name.clone()), None).await;
        }
        Ok(())
    }

    async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        let cfg = ConfigManager::new().load();
        let preset_name = if settings.name.is_empty() {
            "Focus"
        } else {
            &settings.name
        };

        if let Some(preset) = cfg
            .presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(preset_name))
        {
            let client = ElgatoClient::new();
            for dev in &cfg.devices {
                let _ = client
                    .set_settings(
                        &dev.ip,
                        Some(preset.on),
                        Some(preset.brightness),
                        Some(preset.kelvin),
                    )
                    .await;
            }
            let _ = instance.show_ok().await;
        } else {
            let _ = instance.show_alert().await;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct BrightnessSettings {
    pub target: String,
    pub delta: Option<i8>,
    pub set_value: Option<u8>,
}

pub struct BrightnessAction;

#[async_trait]
impl Action for BrightnessAction {
    const UUID: ActionUuid = "com.akitafuki.opencenter.brightness";
    type Settings = BrightnessSettings;

    async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        let ips = resolve_target_ips(&settings.target);
        let client = ElgatoClient::new();

        for ip in &ips {
            if let Ok(state) = client.get_lights(ip).await {
                if let Some(light) = state.lights.first() {
                    let mut b = light.brightness as i16;
                    if let Some(delta) = settings.delta {
                        b = (b + delta as i16).clamp(0, 100);
                    } else if let Some(val) = settings.set_value {
                        b = val as i16;
                    }
                    let final_b = b as u8;
                    let _ = client
                        .set_settings(ip, Some(true), Some(final_b), None)
                        .await;
                    let _ = instance
                        .set_title(Some(format!("{}%", final_b)), None)
                        .await;
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct TemperatureSettings {
    pub target: String,
    pub delta: Option<i16>,
    pub set_value: Option<u16>,
}

pub struct TemperatureAction;

#[async_trait]
impl Action for TemperatureAction {
    const UUID: ActionUuid = "com.akitafuki.opencenter.temperature";
    type Settings = TemperatureSettings;

    async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        let ips = resolve_target_ips(&settings.target);
        let client = ElgatoClient::new();

        for ip in &ips {
            if let Ok(state) = client.get_lights(ip).await {
                if let Some(light) = state.lights.first() {
                    let current_k = light.temperature.map(mired_to_kelvin).unwrap_or(4000);
                    let mut k = current_k as i32;
                    if let Some(delta) = settings.delta {
                        k = (k + delta as i32).clamp(KELVIN_MIN as i32, KELVIN_MAX as i32);
                    } else if let Some(val) = settings.set_value {
                        k = val as i32;
                    }
                    let final_k = (k as u16).clamp(KELVIN_MIN, KELVIN_MAX);
                    let _ = client
                        .set_settings(ip, Some(true), None, Some(final_k))
                        .await;
                    let _ = instance
                        .set_title(Some(format!("{}K", final_k)), None)
                        .await;
                }
            }
        }
        Ok(())
    }
}

fn resolve_target_ips(target: &str) -> Vec<String> {
    let cfg = ConfigManager::new().load();
    if target.is_empty() || target.eq_ignore_ascii_case("all") {
        return cfg.devices.iter().map(|d| d.ip.clone()).collect();
    }
    if let Some(group) = cfg
        .groups
        .iter()
        .find(|g| g.name.eq_ignore_ascii_case(target))
    {
        return group.device_ips.clone();
    }
    if let Some(dev) = cfg
        .devices
        .iter()
        .find(|d| d.name.eq_ignore_ascii_case(target) || d.ip == target)
    {
        return vec![dev.ip.clone()];
    }
    vec![target.to_string()]
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
    register_action(ToggleAction).await;
    register_action(PresetAction).await;
    register_action(BrightnessAction).await;
    register_action(TemperatureAction).await;

    run(std::env::args().collect()).await
}
