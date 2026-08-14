use crate::models::{AppConfig, DeviceConfig, GroupConfig, PresetConfig};
use anyhow::Result;
use dirs::config_dir;
use std::fs::{create_dir_all, rename, File};
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct ConfigManager {
    pub path: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut path = config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("opencenter");
        let _ = create_dir_all(&path);
        path.push("config.json");
        Self { path }
    }

    #[allow(dead_code)]
    pub fn with_path(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        Self { path }
    }

    pub fn load(&self) -> AppConfig {
        if !self.path.exists() {
            let default_cfg = Self::default_config();
            let _ = self.save(&default_cfg);
            return default_cfg;
        }

        match File::open(&self.path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&contents) {
                        return cfg;
                    }
                }
                Self::default_config()
            }
            Err(_) => Self::default_config(),
        }
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        let tmp_path = self.path.with_extension("json.tmp");

        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(json.as_bytes())?;
            file.flush()?;
        }

        rename(&tmp_path, &self.path)?;
        Ok(())
    }

    pub fn default_config() -> AppConfig {
        AppConfig {
            devices: vec![],
            groups: vec![GroupConfig {
                name: "All Lights".to_string(),
                device_ips: vec![],
            }],
            presets: vec![
                PresetConfig {
                    name: "Focus".to_string(),
                    on: true,
                    brightness: 80,
                    kelvin: 5000,
                },
                PresetConfig {
                    name: "Studio Call".to_string(),
                    on: true,
                    brightness: 100,
                    kelvin: 4500,
                },
                PresetConfig {
                    name: "Warm Reading".to_string(),
                    on: true,
                    brightness: 40,
                    kelvin: 3000,
                },
                PresetConfig {
                    name: "Night Shift".to_string(),
                    on: true,
                    brightness: 10,
                    kelvin: 2900,
                },
                PresetConfig {
                    name: "All Off".to_string(),
                    on: false,
                    brightness: 0,
                    kelvin: 3500,
                },
            ],
        }
    }

    pub fn add_or_update_device(&self, device: DeviceConfig) -> Result<AppConfig> {
        let mut config = self.load();
        if let Some(existing) = config.devices.iter_mut().find(|d| d.ip == device.ip) {
            existing.name = device.name;
            if device.model.is_some() {
                existing.model = device.model;
            }
            if device.serial.is_some() {
                existing.serial = device.serial;
            }
        } else {
            config.devices.push(device);
        }
        self.save(&config)?;
        Ok(config)
    }

    pub fn remove_device(&self, ip: &str) -> Result<AppConfig> {
        let mut config = self.load();
        config.devices.retain(|d| d.ip != ip);
        for g in &mut config.groups {
            g.device_ips.retain(|i| i != ip);
        }
        self.save(&config)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = tempdir().expect("Failed to create tempdir");
        let cfg_path = dir.path().join("config.json");
        let mgr = ConfigManager::with_path(cfg_path.clone());

        let loaded_default = mgr.load();
        assert_eq!(loaded_default.presets.len(), 5);
        assert!(cfg_path.exists());

        // Add a device and save
        let dev = DeviceConfig {
            ip: "192.168.1.100".to_string(),
            name: "Test Light".to_string(),
            serial: Some("AZ999".to_string()),
            model: Some("Key Light Air".to_string()),
            enabled: true,
        };

        let updated = mgr.add_or_update_device(dev).expect("Failed to add device");
        assert_eq!(updated.devices.len(), 1);

        // Reload from file to verify persistence
        let mgr_reload = ConfigManager::with_path(cfg_path);
        let reloaded = mgr_reload.load();
        assert_eq!(reloaded.devices.len(), 1);
        assert_eq!(reloaded.devices[0].name, "Test Light");
        assert_eq!(reloaded.devices[0].ip, "192.168.1.100");
    }

    #[test]
    fn test_add_update_remove_device() {
        let dir = tempdir().expect("Failed to create tempdir");
        let cfg_path = dir.path().join("config.json");
        let mgr = ConfigManager::with_path(cfg_path);

        let dev1 = DeviceConfig {
            ip: "192.168.1.50".to_string(),
            name: "Left Light".to_string(),
            serial: None,
            model: None,
            enabled: true,
        };
        mgr.add_or_update_device(dev1).unwrap();

        // Update name
        let dev1_updated = DeviceConfig {
            ip: "192.168.1.50".to_string(),
            name: "Left Light Renamed".to_string(),
            serial: Some("SN123".to_string()),
            model: None,
            enabled: true,
        };
        let cfg = mgr.add_or_update_device(dev1_updated).unwrap();
        assert_eq!(cfg.devices.len(), 1);
        assert_eq!(cfg.devices[0].name, "Left Light Renamed");

        // Remove device
        let cfg_after_remove = mgr.remove_device("192.168.1.50").unwrap();
        assert_eq!(cfg_after_remove.devices.len(), 0);
    }
}
