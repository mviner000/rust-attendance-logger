// src/lib.rs

mod setup;
mod database;

use database::{Database, User, CreateUserRequest, RegisterRequest, LoginRequest, AuthError};
use setup::SystemSetup;
use tauri::{Manager, Emitter};
use anyhow::Result;

#[tauri::command]
fn create_user(
    database: tauri::State<Database>, 
    request: CreateUserRequest
) -> Result<User, String> {
    database
        .create_user(request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_users(database: tauri::State<Database>) -> Result<Vec<User>, String> {
    database
        .get_all_users()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_system_requirements(app: tauri::AppHandle) -> Result<String, String> {
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

#[tauri::command]
fn register_user(database: tauri::State<Database>, request: RegisterRequest) -> Result<User, String> {
    database
        .register_user(request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn login_user(database: tauri::State<Database>, request: LoginRequest) -> Result<User, String> {
    database
        .login_user(request)
        .map_err(|e| e.to_string())
}

pub fn run() {
    let database = Database::new().expect("Failed to create database");
    database.init().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(database)  // Make database state available
        .invoke_handler(tauri::generate_handler![
            check_system_requirements,
            register_user,
            login_user,
            get_users,
            create_user
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            
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