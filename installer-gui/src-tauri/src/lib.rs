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

#[tauri::command]
async fn autodetect_device() -> Vec<String> {
    let mut detected = Vec::new();

    if let Ok(devices) = nusb::list_devices() {
        for dev in devices {
            if dev.vendor_id() == 0x05c6 && dev.product_id() == 0xf601 {
                detected.push("orbic-usb".to_string());
            }
            if dev.vendor_id() == 0x2C7C && dev.product_id() == 0x125 {
                detected.push("pinephone".to_string());
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
                detected.push("tplink".to_string());
            }
            if let Ok(resp) = client.get(format!("http://{ip}/goform/GetLoginInfo")).send().await {
                if resp.status().is_success() {
                    // Orbic and Moxee hotspots run identical firmware web backends.
                    detected.push("orbic".to_string());
                    detected.push("moxee".to_string());
                }
            }
        }
    }

    detected.sort_unstable();
    detected.dedup();
    detected
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            install_rayhunter,
            rayhunter_options,
            autodetect_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
