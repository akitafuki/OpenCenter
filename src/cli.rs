use crate::api::ElgatoClient;
use crate::config::ConfigManager;
use crate::discovery::DiscoveryManager;
use crate::models::{mired_to_kelvin, DeviceConfig, KELVIN_MAX, KELVIN_MIN};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "opencenter")]
#[command(about = "OpenCenter - Elgato Key Light Controller for Linux (CLI & Tray GUI)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Auto-discover Elgato Key Lights on local network
    Discover {
        /// Save discovered devices to config file automatically
        #[arg(short, long)]
        save: bool,
    },
    /// List configured lights and their current real-time status
    Status,
    /// Toggle power for a light, group, or all lights
    Toggle {
        /// Target IP address, group name, or 'all'
        #[arg(default_value = "all")]
        target: String,
    },
    /// Turn ON lights
    On {
        /// Target IP address, group name, or 'all'
        #[arg(default_value = "all")]
        target: String,
    },
    /// Turn OFF lights
    Off {
        /// Target IP address, group name, or 'all'
        #[arg(default_value = "all")]
        target: String,
    },
    /// Set brightness and/or color temperature
    Set {
        /// Target IP address, group name, or 'all'
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Brightness level (0 - 100)
        #[arg(short, long)]
        brightness: Option<u8>,
        /// Color temperature in Kelvin (2900 - 7000)
        #[arg(short, long)]
        kelvin: Option<u16>,
    },
    /// Smoothly fade brightness and temperature over time
    Fade {
        /// Target IP address, group name, or 'all'
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Target brightness level (0 - 100)
        #[arg(short, long, default_value_t = 80)]
        brightness: u8,
        /// Target color temperature in Kelvin (2900 - 7000)
        #[arg(short, long, default_value_t = 4000)]
        kelvin: u16,
        /// Fade duration in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        duration_ms: u64,
    },
    /// Preset management
    Preset {
        #[command(subcommand)]
        action: PresetCommands,
    },
    /// Trigger visual flash sequence to identify light location
    Identify {
        /// IP address of the target light
        ip: String,
    },
    /// Manually add a device by IP address
    AddIp {
        /// IP address of the Elgato device
        ip: String,
        /// Custom friendly name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove a device by IP address
    RemoveIp {
        /// IP address to remove
        ip: String,
    },
    /// Launch System Tray & Graphical Interface
    Gui,
}

#[derive(Subcommand)]
pub enum PresetCommands {
    /// List available saved presets
    List,
    /// Apply a saved preset by name
    Apply {
        /// Preset name (e.g. Focus, Warm Reading, Studio Call)
        name: String,
    },
    /// Save current state of all lights as a new preset
    Save {
        /// Preset name to create or update
        name: String,
    },
}

pub async fn handle_cli(cli: Cli) -> Result<()> {
    let cmd_mgr = ConfigManager::new();
    let client = ElgatoClient::new();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => Commands::Gui,
    };

    match command {
        Commands::Discover { save } => {
            println!("🔍 Scanning network for Elgato Key Lights...");
            let discovered = DiscoveryManager::discover_mdns(4).await;
            if discovered.is_empty() {
                println!("⚠️ No Elgato lights discovered.");
            } else {
                println!("✨ Found {} device(s):", discovered.len());
                for dev in &discovered {
                    println!(
                        "  • {} ({}) - IP: {}",
                        dev.name,
                        dev.model.as_deref().unwrap_or("Key Light"),
                        dev.ip
                    );
                    if save {
                        let _ = cmd_mgr.add_or_update_device(dev.clone());
                    }
                }
                if save {
                    println!("💾 Saved discovered devices to config.");
                }
            }
        }
        Commands::Status => {
            let cfg = cmd_mgr.load();
            if cfg.devices.is_empty() {
                println!("⚠️ No devices configured. Run `elgato-control discover --save` or `elgato-control add-ip <IP>`.");
                return Ok(());
            }

            println!("💡 Elgato Device Statuses:");
            for dev in &cfg.devices {
                match client.get_lights(&dev.ip).await {
                    Ok(state) => {
                        if let Some(light) = state.lights.first() {
                            let pwr = if light.on == 1 {
                                "🟢 ON "
                            } else {
                                "🔴 OFF"
                            };
                            let kelvin = light.temperature.map(mired_to_kelvin).unwrap_or(4000);
                            println!(
                                "  • {:<20} | IP: {:<15} | {} | Brightness: {:>3}% | Temp: {}K",
                                dev.name, dev.ip, pwr, light.brightness, kelvin
                            );
                        }
                    }
                    Err(_) => {
                        println!("  • {:<20} | IP: {:<15} | ⚪ UNREACHABLE", dev.name, dev.ip);
                    }
                }
            }
        }
        Commands::Toggle { target } => {
            let ips = resolve_target_ips(&cmd_mgr, &target);
            for ip in ips {
                if let Ok(new_state) = client.toggle_power(&ip).await {
                    let st = if new_state { "ON" } else { "OFF" };
                    println!("Toggled [{}] -> {}", ip, st);
                }
            }
        }
        Commands::On { target } => {
            let ips = resolve_target_ips(&cmd_mgr, &target);
            for ip in ips {
                let _ = client.set_power(&ip, true).await;
                println!("Turned ON [{}]", ip);
            }
        }
        Commands::Off { target } => {
            let ips = resolve_target_ips(&cmd_mgr, &target);
            for ip in ips {
                let _ = client.set_power(&ip, false).await;
                println!("Turned OFF [{}]", ip);
            }
        }
        Commands::Set {
            target,
            brightness,
            kelvin,
        } => {
            let ips = resolve_target_ips(&cmd_mgr, &target);
            let k_clamped = kelvin.map(|k| k.clamp(KELVIN_MIN, KELVIN_MAX));
            for ip in ips {
                let _ = client.set_settings(&ip, None, brightness, k_clamped).await;
                println!("Updated [{}]", ip);
            }
        }
        Commands::Fade {
            target,
            brightness,
            kelvin,
            duration_ms,
        } => {
            let ips = resolve_target_ips(&cmd_mgr, &target);
            println!("Fading {} device(s) over {}ms...", ips.len(), duration_ms);
            let mut tasks = Vec::new();
            for ip in ips {
                let client_c = client.clone();
                tasks.push(tokio::spawn(async move {
                    let _ = client_c
                        .fade_transition(&ip, true, brightness, kelvin, duration_ms)
                        .await;
                }));
            }
            for t in tasks {
                let _ = t.await;
            }
        }
        Commands::Preset { action } => match action {
            PresetCommands::List => {
                let cfg = cmd_mgr.load();
                println!("📋 Available Presets:");
                for p in &cfg.presets {
                    let pwr = if p.on { "ON" } else { "OFF" };
                    println!(
                        "  • {:<15} | Power: {:<3} | Brightness: {:>3}% | Temp: {}K",
                        p.name, pwr, p.brightness, p.kelvin
                    );
                }
            }
            PresetCommands::Apply { name } => {
                let cfg = cmd_mgr.load();
                if let Some(preset) = cfg
                    .presets
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(&name))
                {
                    println!("Applying preset '{}'...", preset.name);
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
                } else {
                    println!("❌ Preset '{}' not found.", name);
                }
            }
            PresetCommands::Save { name } => {
                let mut cfg = cmd_mgr.load();
                if let Some(first_dev) = cfg.devices.first() {
                    if let Ok(state) = client.get_lights(&first_dev.ip).await {
                        if let Some(light) = state.lights.first() {
                            let k = light.temperature.map(mired_to_kelvin).unwrap_or(4000);
                            let new_preset = crate::models::PresetConfig {
                                name: name.clone(),
                                on: light.on == 1,
                                brightness: light.brightness,
                                kelvin: k,
                            };
                            cfg.presets.retain(|p| !p.name.eq_ignore_ascii_case(&name));
                            cfg.presets.push(new_preset);
                            let _ = cmd_mgr.save(&cfg);
                            println!("💾 Saved preset '{}' from device state.", name);
                        }
                    }
                }
            }
        },
        Commands::Identify { ip } => {
            let _ = client.identify(&ip).await;
            println!("Flashing identify sequence on [{}]...", ip);
        }
        Commands::AddIp { ip, name } => {
            let custom_name = name.unwrap_or_else(|| format!("Elgato Light ({})", ip));
            let dev = DeviceConfig {
                ip: ip.clone(),
                name: custom_name,
                serial: None,
                model: None,
                enabled: true,
            };
            let _ = cmd_mgr.add_or_update_device(dev);
            println!("Added IP [{}] to configuration.", ip);
        }
        Commands::RemoveIp { ip } => {
            let _ = cmd_mgr.remove_device(&ip);
            println!("Removed IP [{}] from configuration.", ip);
        }
        Commands::Gui => {
            // Handled in main.rs
        }
    }
    Ok(())
}

fn resolve_target_ips(cfg_mgr: &ConfigManager, target: &str) -> Vec<String> {
    let cfg = cfg_mgr.load();
    if target.eq_ignore_ascii_case("all") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_subcommands() {
        // Test discover command
        let cli = Cli::try_parse_from(["opencenter", "discover", "--save"]).unwrap();
        match cli.command.unwrap() {
            Commands::Discover { save } => assert!(save),
            _ => panic!("Expected Discover subcommand"),
        }

        // Test set command with parameters
        let cli_set = Cli::try_parse_from([
            "opencenter",
            "set",
            "--brightness",
            "85",
            "--kelvin",
            "4500",
            "--target",
            "192.168.1.50",
        ])
        .unwrap();
        match cli_set.command.unwrap() {
            Commands::Set {
                target,
                brightness,
                kelvin,
            } => {
                assert_eq!(target, "192.168.1.50");
                assert_eq!(brightness, Some(85));
                assert_eq!(kelvin, Some(4500));
            }
            _ => panic!("Expected Set subcommand"),
        }

        // Test fade command defaults
        let cli_fade = Cli::try_parse_from(["opencenter", "fade", "--brightness", "90"]).unwrap();
        match cli_fade.command.unwrap() {
            Commands::Fade {
                brightness,
                duration_ms,
                ..
            } => {
                assert_eq!(brightness, 90);
                assert_eq!(duration_ms, 1000);
            }
            _ => panic!("Expected Fade subcommand"),
        }

        // Test preset apply
        let cli_preset = Cli::try_parse_from(["opencenter", "preset", "apply", "Focus"]).unwrap();
        match cli_preset.command.unwrap() {
            Commands::Preset { action } => match action {
                PresetCommands::Apply { name } => assert_eq!(name, "Focus"),
                _ => panic!("Expected Apply preset subcommand"),
            },
            _ => panic!("Expected Preset subcommand"),
        }
    }
}
