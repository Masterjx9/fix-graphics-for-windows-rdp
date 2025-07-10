// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use reset_graphics_for_rdp_lib::run;
use serde::{Deserialize, Serialize};  
use wmi::{COMLibrary, WMIConnection};
use std::{process::Command, thread::sleep, time::Duration};


#[derive(Debug, Serialize, Deserialize)]
struct PnpEntity {
    Name:         Option<String>,
    PNPDeviceID:  Option<String>,   // <- InstanceId for PowerShell
}


#[derive(Debug, Serialize, Deserialize)] 
struct VideoController {
    Name: Option<String>,
    DriverVersion: Option<String>,
    Status: Option<String>,
}


#[tauri::command]
async fn get_gpu_info() -> Result<Vec<VideoController>, String> {
    // run on a fresh thread → COM can initialise as MTA safely
    tauri::async_runtime::spawn_blocking(|| {
        let com = COMLibrary::new().map_err(|e| e.to_string())?;          // MTA
        let wmi = WMIConnection::new(com.into()).map_err(|e| e.to_string())?;
        let data: Vec<VideoController> =
            wmi.raw_query("SELECT Name, DriverVersion, Status FROM Win32_VideoController")
               .map_err(|e| e.to_string())?;
        Ok(data)
    })
    .await
    .map_err(|e| e.to_string())?        // join-error → String
}

// powershell method to reset the GPU
// try {
//     Disable-PnpDevice -InstanceId $device.InstanceId -Confirm:$false -ErrorAction Stop
// } catch {}
// Start-Sleep -Seconds 3
// try {
//     Enable-PnpDevice -InstanceId $device.InstanceId -Confirm:$false -ErrorAction Stop
// } catch {}

// Tauri command to reset the GPU

#[tauri::command]
async fn reset_gpu(gpu_name: String) -> Result<&'static str, String> {
    tauri::async_runtime::spawn_blocking(move || reset_gpu_inner(&gpu_name)).await
        .map_err(|e| e.to_string())??;
    Ok("OK")
}

fn reset_gpu_inner(gpu_name: &str) -> Result<(), String> {
    // ── 1. find the device’s InstanceId ────────────────────────────────
    let com = COMLibrary::new().map_err(|e| e.to_string())?;
    let wmi = WMIConnection::new(com.into()).map_err(|e| e.to_string())?;

    // escape single quotes inside the name for the WMI query
    let escaped = gpu_name.replace('\'', "''");
    let q = format!(
        "SELECT Name, PNPDeviceID FROM Win32_PnPEntity WHERE Name = '{}'",
        escaped
    );

    let list: Vec<PnpEntity> = wmi.raw_query(&q).map_err(|e| e.to_string())?;
    let device = list
        .into_iter()
        .next()
        .ok_or_else(|| "GPU not found".to_owned())?;
    let id = device
        .PNPDeviceID
        .ok_or_else(|| "DeviceID missing".to_owned())?;

    // ── 2. disable, wait, enable via PowerShell ────────────────────────
    run_pnp_cmd(&id, true)?;              // Disable-PnpDevice
    sleep(Duration::from_secs(3));
    run_pnp_cmd(&id, false)?;             // Enable-PnpDevice

    Ok(())
}

fn run_pnp_cmd(instance_id: &str, disable: bool) -> Result<(), String> {
    let cmdlet = if disable { "Disable-PnpDevice" } else { "Enable-PnpDevice" };
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "{cmdlet} -InstanceId \"{instance_id}\" -Confirm:$false -ErrorAction Stop"
            ),
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmdlet} failed: {status}"))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_gpu_info, reset_gpu])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
