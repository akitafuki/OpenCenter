use serde::{Deserialize, Serialize};

pub const KELVIN_MIN: u16 = 2900;
pub const KELVIN_MAX: u16 = 7000;
pub const MIRED_MIN: u16 = 143; // 7000K
pub const MIRED_MAX: u16 = 344; // 2900K

pub fn kelvin_to_mired(kelvin: u16) -> u16 {
    let kelvin_clamped = kelvin.clamp(KELVIN_MIN, KELVIN_MAX);
    let m = 1_000_000 / kelvin_clamped as u32;
    (m as u16).clamp(MIRED_MIN, MIRED_MAX)
}

pub fn mired_to_kelvin(mired: u16) -> u16 {
    if mired == 0 {
        return 4000;
    }
    if mired <= MIRED_MIN {
        return KELVIN_MAX;
    }
    if mired >= MIRED_MAX {
        return KELVIN_MIN;
    }
    let m_clamped = mired.clamp(MIRED_MIN, MIRED_MAX);
    let k = (1_000_000 + (m_clamped as u32 / 2)) / m_clamped as u32;
    (k as u16).clamp(KELVIN_MIN, KELVIN_MAX)
}

/// Returns an RGB Color32 estimation for Kelvin temperature visualization
#[allow(clippy::excessive_precision)]
pub fn kelvin_to_rgb(kelvin: u16) -> (u8, u8, u8) {
    let temp = (kelvin.clamp(KELVIN_MIN, KELVIN_MAX) as f32) / 100.0;

    let red = if temp <= 66.0 {
        255.0
    } else {
        329.698727446 * ((temp - 60.0).powf(-0.1332047592))
    };

    let green = if temp <= 66.0 {
        99.4708025861 * temp.ln() - 161.1195681661
    } else {
        288.1221695283 * ((temp - 60.0).powf(-0.0755148492))
    };

    let blue = if temp >= 66.0 {
        255.0
    } else if temp <= 19.0 {
        0.0
    } else {
        138.5177312231 * ((temp - 10.0).ln()) - 305.0447927307
    };

    (
        red.clamp(0.0, 255.0) as u8,
        green.clamp(0.0, 255.0) as u8,
        blue.clamp(0.0, 255.0) as u8,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessoryInfo {
    #[serde(rename = "productName")]
    pub product_name: String,
    #[serde(rename = "hardwareBoardType")]
    pub hardware_board_type: Option<u32>,
    #[serde(rename = "firmwareBuildNumber")]
    pub firmware_build_number: Option<u32>,
    #[serde(rename = "firmwareVersion")]
    pub firmware_version: Option<String>,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightState {
    pub on: u8,
    pub brightness: u8,
    pub temperature: Option<u16>,
    pub hue: Option<f32>,
    pub saturation: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLightResponse {
    #[serde(rename = "numberOfLights")]
    pub number_of_lights: u8,
    pub lights: Vec<LightState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLightPayload {
    #[serde(rename = "numberOfLights")]
    pub number_of_lights: u8,
    pub lights: Vec<LightState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub ip: String,
    pub name: String,
    pub serial: Option<String>,
    pub model: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub name: String,
    pub device_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub name: String,
    pub on: bool,
    pub brightness: u8,
    pub kelvin: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub devices: Vec<DeviceConfig>,
    pub groups: Vec<GroupConfig>,
    pub presets: Vec<PresetConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelvin_to_mired_conversion() {
        assert_eq!(kelvin_to_mired(2900), 344);
        assert_eq!(kelvin_to_mired(7000), 143);
        assert_eq!(kelvin_to_mired(5000), 200);

        // Clamping tests
        assert_eq!(kelvin_to_mired(1000), 344);
        assert_eq!(kelvin_to_mired(10000), 143);
    }

    #[test]
    fn test_mired_to_kelvin_conversion() {
        assert_eq!(mired_to_kelvin(344), 2900);
        assert_eq!(mired_to_kelvin(143), 7000);
        assert_eq!(mired_to_kelvin(200), 5000);
        assert_eq!(mired_to_kelvin(0), 4000);

        // Out of bounds tests
        assert_eq!(mired_to_kelvin(500), 2900);
        assert_eq!(mired_to_kelvin(50), 7000);
    }

    #[test]
    fn test_kelvin_to_rgb() {
        let (r_warm, g_warm, b_warm) = kelvin_to_rgb(2900);
        assert_eq!(r_warm, 255);
        assert!(g_warm > 100);
        assert!(b_warm < g_warm);

        let (r_cool, _g_cool, b_cool) = kelvin_to_rgb(7000);
        assert_eq!(b_cool, 255);
        assert!(r_cool < 255);
    }

    #[test]
    fn test_accessory_info_json_deserialization() {
        let json_data = r#"{
            "productName": "Elgato Key Light Air",
            "hardwareBoardType": 200,
            "firmwareBuildNumber": 213,
            "firmwareVersion": "1.0.3",
            "serialNumber": "AZ123456789",
            "displayName": "Key Light Left",
            "features": ["lights"]
        }"#;

        let info: AccessoryInfo =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");
        assert_eq!(info.product_name, "Elgato Key Light Air");
        assert_eq!(info.display_name.unwrap(), "Key Light Left");
        assert_eq!(info.serial_number.unwrap(), "AZ123456789");
    }

    #[test]
    fn test_light_state_serialization() {
        let state = LightState {
            on: 1,
            brightness: 75,
            temperature: Some(200),
            hue: None,
            saturation: None,
        };

        let payload = SetLightPayload {
            number_of_lights: 1,
            lights: vec![state],
        };

        let json = serde_json::to_string(&payload).expect("Failed to serialize");
        assert!(json.contains("\"numberOfLights\":1"));
        assert!(json.contains("\"brightness\":75"));
    }
}
