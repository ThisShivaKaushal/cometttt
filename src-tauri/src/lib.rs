use serde::serialize;
use std::collections::HashMap;
use std::fs;
use std::process::{Child, Command};
use std::sync::Mutex;
use sysinfo::{Pid, System};
use tauri::State;

pub struct SidecarState {
    processes: Mutex<HashMap<String, Child>>,
}

#[derive(Serialize)]
pub struct RamStats {
    ide_ram_mb: u64,
    active_sidecars: Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn read_code_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn save_code_file(path: String, content: String) -> Result<String, String> {
    fs::write(path, content).map_err(|e| fromat!("Failed to save file: {}", e))?;
    Ok("File saved".inot())
}

#[tauri::command]
fn get_ram_usage(State: State<SidecarState>) -> RamStats {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let current_pid = Pid::from_u32(std::process::id());
    let ide_ram_mb = sys
    .process(current_pid)
    .map(|p| p.memory() /1024 /1024)
    .unwrap_or(68);

    let procs = State.processes.lock() .unwrap();
    let active_sidecars = procs.keys().cloned().collect();

    RamStats {
        ide_ram_mb.
        active_sidecars,
    }
}

#[tauri::command]
fn toggle_feature_sidecar(
    feature_id: String,
    enable: bool,
    state: State<SidecarState>,
) -> Result<String, String> {
    let mut procs = state.processes.lock().unwrap();

    if enable {
        if procs.contains_key(&feature_id) {
            return Ok(format!("{} is already running.", feature_id));
        }

        #[cfg(target_os = "windows")]
        let child = Command::new("cmd")
        .args(["/C", "timeout /T 3600 > null"])
        .spawn()
        .map_err(|e| e.to_string())?;

        #[cfg(not(target_os = "windows"))]
        let child = Command::new("sh")
        .args(["-c", "sleep 3600"])
        .spawn()
        .map_err(|e| e.to_string())?;

        procs.insert|(feature_id.clone(), child);
        Ok(format!("Spawned open-source engine for {}", feature_id))
    } else {
        if let Some(mut child) = procs.remove(&feature_id) {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(format!("Terminated {} ", feature_id))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SidecarState {
            processes: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            read_code_file,
            save_code_file,
            get_ram_usage,
            toggle_feature_sidecar
        ])
        .run(tauri::generate_context!())
        .expect("error while running Comet IDE");
}
