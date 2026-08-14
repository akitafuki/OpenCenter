use crate::models::{
    kelvin_to_mired, AccessoryInfo, GetLightResponse, LightState, SetLightPayload,
};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct ElgatoClient {
    http: Client,
}

impl Default for ElgatoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ElgatoClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(3))
            .connect_timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { http }
    }

    async fn retry_request<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        loop {
            match f().await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    attempts += 1;
                    if attempts >= 2 {
                        return Err(err);
                    }
                    sleep(Duration::from_millis(150)).await;
                }
            }
        }
    }

    pub async fn get_accessory_info(&self, ip: &str) -> Result<AccessoryInfo> {
        let url = format!("http://{}:9123/elgato/accessory-info", ip);
        self.retry_request(|| async {
            let resp = self.http.get(&url).send().await?;
            if !resp.status().is_success() {
                return Err(anyhow!("HTTP error status: {}", resp.status()));
            }
            let info: AccessoryInfo = resp.json().await?;
            Ok(info)
        })
        .await
    }

    pub async fn get_lights(&self, ip: &str) -> Result<GetLightResponse> {
        let url = format!("http://{}:9123/elgato/lights", ip);
        self.retry_request(|| async {
            let resp = self.http.get(&url).send().await?;
            if !resp.status().is_success() {
                return Err(anyhow!("HTTP error status: {}", resp.status()));
            }
            let state: GetLightResponse = resp.json().await?;
            Ok(state)
        })
        .await
    }

    pub async fn set_lights(&self, ip: &str, state: &LightState) -> Result<()> {
        let url = format!("http://{}:9123/elgato/lights", ip);
        let payload = SetLightPayload {
            number_of_lights: 1,
            lights: vec![state.clone()],
        };
        self.retry_request(|| async {
            let resp = self.http.put(&url).json(&payload).send().await?;
            if !resp.status().is_success() {
                return Err(anyhow!("HTTP PUT error status: {}", resp.status()));
            }
            Ok(())
        })
        .await
    }

    pub async fn identify(&self, ip: &str) -> Result<()> {
        let url = format!("http://{}:9123/elgato/identify", ip);
        self.retry_request(|| async {
            let resp = self.http.post(&url).send().await?;
            if !resp.status().is_success() {
                return Err(anyhow!("HTTP POST error status: {}", resp.status()));
            }
            Ok(())
        })
        .await
    }

    pub async fn set_power(&self, ip: &str, on: bool) -> Result<()> {
        let mut current = self.get_lights(ip).await?;
        if let Some(light) = current.lights.first_mut() {
            light.on = if on { 1 } else { 0 };
            self.set_lights(ip, light).await?;
        }
        Ok(())
    }

    pub async fn toggle_power(&self, ip: &str) -> Result<bool> {
        let mut current = self.get_lights(ip).await?;
        if let Some(light) = current.lights.first_mut() {
            let new_on = if light.on == 1 { 0 } else { 1 };
            light.on = new_on;
            self.set_lights(ip, light).await?;
            Ok(new_on == 1)
        } else {
            Err(anyhow!("No lights returned from device"))
        }
    }

    pub async fn set_settings(
        &self,
        ip: &str,
        on: Option<bool>,
        brightness: Option<u8>,
        kelvin: Option<u16>,
    ) -> Result<()> {
        let mut current = self.get_lights(ip).await?;
        if let Some(light) = current.lights.first_mut() {
            if let Some(o) = on {
                light.on = if o { 1 } else { 0 };
            }
            if let Some(b) = brightness {
                light.brightness = b.clamp(0, 100);
            }
            if let Some(k) = kelvin {
                light.temperature = Some(kelvin_to_mired(k));
            }
            self.set_lights(ip, light).await?;
        }
        Ok(())
    }

    pub async fn fade_transition(
        &self,
        ip: &str,
        target_on: bool,
        target_brightness: u8,
        target_kelvin: u16,
        duration_ms: u64,
    ) -> Result<()> {
        let current_res = self.get_lights(ip).await?;
        let current_light = match current_res.lights.first() {
            Some(l) => l.clone(),
            None => return Err(anyhow!("No lights found")),
        };

        let start_brightness = current_light.brightness as f32;
        let end_brightness = target_brightness.clamp(0, 100) as f32;

        let steps = (duration_ms / 50).max(1);
        let step_delay = Duration::from_millis(duration_ms / steps);

        let target_mired = kelvin_to_mired(target_kelvin);

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let current_b = (start_brightness + (end_brightness - start_brightness) * t) as u8;

            let updated_state = LightState {
                on: if target_on { 1 } else { 0 },
                brightness: current_b,
                temperature: Some(target_mired),
                hue: current_light.hue,
                saturation: current_light.saturation,
            };

            let _ = self.set_lights(ip, &updated_state).await;
            sleep(step_delay).await;
        }

        Ok(())
    }
}
