// src/lib.rs

mod setup;
mod database;

use database::{Database, User, RegisterRequest, LoginRequest};
use setup::SystemSetup;
use tauri::{Manager, Emitter};

#[tauri::command]
async fn check_system_requirements(app: tauri::AppHandle) -> Result<String, String> {
    // First check Windows version
    if let Ok(is_compatible) = SystemSetup::check_windows_version(&app).await {
        if !is_compatible {
            return Err("Docker Desktop requires Windows 10/11 Pro, Enterprise, or Education".to_string());
        }
    }

    match SystemSetup::setup_system(&app).await {
        Ok(_) => Ok("System setup completed successfully".to_string()),
        Err(e) => Err(format!("Setup failed: {}", e)),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_system_requirements,
        ])
        .setup(|app| {
            // Create a fully owned handle by cloning
            let app_handle = app.handle().clone();
            
            // Now we can safely move app_handle into the async block
            tauri::async_runtime::spawn(async move {
                match SystemSetup::setup_system(&app_handle).await {
                    Ok(_) => println!("System setup completed successfully"),
                    Err(e) => {
                        eprintln!("System setup failed: {}", e);
                        let _ = app_handle.emit("setup-error", format!("System setup failed: {}", e));
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}