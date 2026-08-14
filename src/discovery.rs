use crate::api::ElgatoClient;
use crate::models::DeviceConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::info;

pub struct DiscoveryManager;

impl DiscoveryManager {
    pub async fn discover_mdns(timeout_secs: u64) -> Vec<DeviceConfig> {
        let mut discovered = Vec::new();
        let service_type = "_elg._tcp.local.";

        let mdns = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(_) => return Self::scan_local_subnet().await,
        };

        let receiver = match mdns.browse(service_type) {
            Ok(r) => r,
            Err(_) => return Self::scan_local_subnet().await,
        };

        let mut seen_ips = HashSet::new();
        let start = std::time::Instant::now();
        let client = ElgatoClient::new();

        while start.elapsed() < Duration::from_secs(timeout_secs) {
            if let Ok(ServiceEvent::ServiceResolved(info)) =
                receiver.recv_timeout(Duration::from_millis(200))
            {
                for ip in info.get_addresses() {
                    let ip_str = ip.to_string();
                    if !seen_ips.contains(&ip_str) {
                        seen_ips.insert(ip_str.clone());
                        if let Ok(acc) = client.get_accessory_info(&ip_str).await {
                            discovered.push(DeviceConfig {
                                ip: ip_str,
                                name: acc.display_name.unwrap_or(acc.product_name.clone()),
                                serial: acc.serial_number,
                                model: Some(acc.product_name),
                                enabled: true,
                            });
                        }
                    }
                }
            }
        }

        let _ = mdns.shutdown();

        if discovered.is_empty() {
            return Self::scan_local_subnet().await;
        }

        discovered
    }

    pub async fn scan_local_subnet() -> Vec<DeviceConfig> {
        let mut discovered = Vec::new();
        let client = ElgatoClient::new();

        let base_ips = match get_local_ip_prefix() {
            Some(prefix) => prefix,
            None => vec!["192.168.1".to_string(), "192.168.0".to_string()],
        };

        // Semaphore limit of 32 concurrent requests to avoid router socket congestion
        let semaphore = Arc::new(Semaphore::new(32));
        let mut tasks = Vec::new();

        for prefix in base_ips {
            for i in 1..=254 {
                let ip_str = format!("{}.{}", prefix, i);
                let client_clone = client.clone();
                let sem = semaphore.clone();
                tasks.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.ok();
                    if let Ok(Ok(acc)) = timeout(
                        Duration::from_millis(300),
                        client_clone.get_accessory_info(&ip_str),
                    )
                    .await
                    {
                        Some(DeviceConfig {
                            ip: ip_str,
                            name: acc.display_name.unwrap_or(acc.product_name.clone()),
                            serial: acc.serial_number,
                            model: Some(acc.product_name),
                            enabled: true,
                        })
                    } else {
                        None
                    }
                }));
            }
        }

        for task in tasks {
            if let Ok(Some(dev)) = task.await {
                info!(
                    "Discovered Elgato device via IP scan: {} at {}",
                    dev.name, dev.ip
                );
                discovered.push(dev);
            }
        }

        discovered
    }
}

fn get_local_ip_prefix() -> Option<Vec<String>> {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                if let IpAddr::V4(ipv4) = local_addr.ip() {
                    let octets = ipv4.octets();
                    return Some(vec![format!("{}.{}.{}", octets[0], octets[1], octets[2])]);
                }
            }
        }
    }
    None
}
