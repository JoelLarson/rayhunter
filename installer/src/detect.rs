use std::collections::HashMap;

use serde::Serialize;

pub const DASHBOARD_PORT: u16 = 8080;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedDevice {
    pub id: String,
    pub subcommand: String,
    pub display_name: String,
    pub dashboard_url: Option<String>,
    pub default_args: HashMap<String, String>,
}

pub async fn detect_devices() -> Vec<DetectedDevice> {
    let mut detected = Vec::new();

    if let Ok(devices) = nusb::list_devices() {
        for dev in devices {
            if dev.vendor_id() == 0x05c6 && dev.product_id() == 0xf601 {
                let bus = dev.bus_number();
                let addr = dev.device_address();
                detected.push(DetectedDevice {
                    id: format!("usb-orbic-{}-{}", bus, addr),
                    subcommand: "orbic-usb".to_string(),
                    display_name: "Orbic Hotspot (USB)".to_string(),
                    dashboard_url: None,
                    default_args: HashMap::new(),
                });
            }
            if dev.vendor_id() == 0x2C7C && dev.product_id() == 0x125 {
                let bus = dev.bus_number();
                let addr = dev.device_address();
                detected.push(DetectedDevice {
                    id: format!("usb-pinephone-{}-{}", bus, addr),
                    subcommand: "pinephone".to_string(),
                    display_name: "PinePhone Modem (USB)".to_string(),
                    dashboard_url: None,
                    default_args: HashMap::new(),
                });
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(600))
        .pool_max_idle_per_host(0)
        .build();

    if let Ok(client) = client {
        for ip in &["192.168.0.1", "192.168.8.1"] {
            let mut is_tplink = false;
            if let Ok(resp) = client
                .post(format!("http://{ip}/cgi-bin/qcmap_web_cgi"))
                .send()
                .await
            {
                if resp.status().is_success() {
                    is_tplink = true;
                }
            }
            if !is_tplink {
                if let Ok(resp) = client
                    .post(format!("http://{ip}/cgi-bin/web_cgi"))
                    .send()
                    .await
                {
                    if resp.status().is_success() {
                        is_tplink = true;
                    }
                }
            }
            if is_tplink {
                detected.push(DetectedDevice {
                    id: format!("net-tplink-{}", ip),
                    subcommand: "tplink".to_string(),
                    display_name: "TP-Link Hotspot (Wi-Fi)".to_string(),
                    dashboard_url: Some(format!("http://{}:{}", ip, DASHBOARD_PORT)),
                    default_args: HashMap::from([("--admin-ip".to_string(), ip.to_string())]),
                });
            }
            if let Ok(resp) = client
                .get(format!("http://{ip}/goform/GetLoginInfo"))
                .send()
                .await
            {
                if resp.status().is_success() {
                    let default_args =
                        HashMap::from([("--admin-ip".to_string(), ip.to_string())]);
                    let dashboard_url = Some(format!("http://{}:{}", ip, DASHBOARD_PORT));
                    detected.push(DetectedDevice {
                        id: format!("net-orbic-{}", ip),
                        subcommand: "orbic".to_string(),
                        display_name: "Orbic Hotspot (Wi-Fi)".to_string(),
                        dashboard_url: dashboard_url.clone(),
                        default_args: default_args.clone(),
                    });
                    detected.push(DetectedDevice {
                        id: format!("net-moxee-{}", ip),
                        subcommand: "moxee".to_string(),
                        display_name: "Moxee Hotspot (Wi-Fi)".to_string(),
                        dashboard_url,
                        default_args,
                    });
                }
            }
        }
    }

    let mut counts = std::collections::HashMap::new();
    for dev in &detected {
        *counts.entry(dev.subcommand.clone()).or_insert(0) += 1;
    }

    for dev in &mut detected {
        if let Some(&count) = counts.get(&dev.subcommand) {
            if count > 1 {
                if dev.subcommand.ends_with("-usb") || dev.subcommand == "pinephone" {
                    let parts: Vec<&str> = dev.id.split('-').collect();
                    if parts.len() >= 4 {
                        let bus = parts[parts.len() - 2];
                        let addr = parts[parts.len() - 1];
                        dev.display_name =
                            format!("{} [Bus {}, Addr {}]", dev.display_name, bus, addr);
                    }
                } else if let Some(ip) = dev.default_args.get("--admin-ip") {
                    dev.display_name = format!("{} [{}]", dev.display_name, ip);
                }
            }
        }
    }

    detected
}
