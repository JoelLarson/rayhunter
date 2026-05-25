use std::sync::LazyLock;

use clap::CommandFactory;
use tauri::Emitter;

mod introspect;
mod modifiers;

static INSTALLER_COMMAND: LazyLock<clap::Command> = LazyLock::new(installer::Args::command);

async fn run_installer(app_handle: tauri::AppHandle, args: Vec<String>) -> anyhow::Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        installer::run_with_callback(
            args.iter().map(|s| s.as_str()),
            Some(Box::new(move |output| {
                app_handle
                    .emit("installer-output", output)
                    .expect("Error sending Rayhunter CLI installer output to GUI frontend");
            })),
        )
    })
    .await?
}

#[tauri::command]
async fn install_rayhunter(app_handle: tauri::AppHandle, args: Vec<String>) -> Result<(), String> {
    run_installer(app_handle, args)
        .await
        .map_err(|error| format!("{error:?}"))
}

#[tauri::command]
fn rayhunter_options() -> introspect::Command<'static> {
    introspect::Command::new(&INSTALLER_COMMAND)
}

#[derive(serde::Serialize, Clone, Debug)]
struct DetectedDevice {
    id: String,
    subcommand: String,
    display_name: String,
    admin_ip: Option<String>,
}

#[tauri::command]
async fn autodetect_device() -> Vec<DetectedDevice> {
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
                    admin_ip: None,
                });
            }
            if dev.vendor_id() == 0x2C7C && dev.product_id() == 0x125 {
                let bus = dev.bus_number();
                let addr = dev.device_address();
                detected.push(DetectedDevice {
                    id: format!("usb-pinephone-{}-{}", bus, addr),
                    subcommand: "pinephone".to_string(),
                    display_name: "PinePhone Modem (USB)".to_string(),
                    admin_ip: None,
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
            if let Ok(resp) = client.post(format!("http://{ip}/cgi-bin/qcmap_web_cgi")).send().await {
                if resp.status().is_success() {
                    is_tplink = true;
                }
            }
            if !is_tplink {
                if let Ok(resp) = client.post(format!("http://{ip}/cgi-bin/web_cgi")).send().await {
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
                    admin_ip: Some(ip.to_string()),
                });
            }
            if let Ok(resp) = client.get(format!("http://{ip}/goform/GetLoginInfo")).send().await {
                if resp.status().is_success() {
                    detected.push(DetectedDevice {
                        id: format!("net-orbic-{}", ip),
                        subcommand: "orbic".to_string(),
                        display_name: "Orbic Hotspot (Wi-Fi)".to_string(),
                        admin_ip: Some(ip.to_string()),
                    });
                    detected.push(DetectedDevice {
                        id: format!("net-moxee-{}", ip),
                        subcommand: "moxee".to_string(),
                        display_name: "Moxee Hotspot (Wi-Fi)".to_string(),
                        admin_ip: Some(ip.to_string()),
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
                        dev.display_name = format!("{} [Bus {}, Addr {}]", dev.display_name, bus, addr);
                    }
                } else if let Some(ref ip) = dev.admin_ip {
                    dev.display_name = format!("{} [{}]", dev.display_name, ip);
                }
            }
        }
    }

    detected
}

#[tauri::command]
async fn adb_forward_dashboard() -> Result<(), String> {
    let output = std::process::Command::new("adb")
        .args(["forward", "tcp:8080", "tcp:8080"])
        .output()
        .map_err(|e| format!("Failed to run adb: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "adb forward failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            install_rayhunter,
            rayhunter_options,
            autodetect_device,
            adb_forward_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
