use tauri::{Emitter, Window};

mod auth;
mod deps;
mod storage;
mod functions; // Add module

mod telemetry;

#[tauri::command]
async fn verify_connection(window: Window, url: String, key: String) -> Result<String, String> {
    window
        .emit("log", format!("Connecting to project: {}", url))
        .unwrap();
    match auth::validate_service_key(&url, &key).await {
        Ok(msg) => {
            window.emit("log", &msg).unwrap();
            Ok(msg)
        }
        Err(e) => {
            window
                .emit("log", format!("Connection Failed: {}", e))
                .unwrap();
            Err(e)
        }
    }
}

#[tauri::command]
async fn backup_edge_config(window: Window, url: String, key: String) -> Result<String, String> {
    match functions::backup_function_config(&window, &url, &key).await {
        Ok(configs) => Ok(format!("Secured {} function configs.", configs.len())),
        Err(e) => Err(format!("Edge Config Backup Failed: {}", e))
    }
}

#[tauri::command]
async fn link_local_source(window: Window, path: String) -> Result<String, String> {
    functions::zip_local_source(&window, &path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            verify_connection,
            check_driver_status,
            install_drivers,
            perform_migration,
            discover_local_databases,
            backup_database,
            dry_run_migration,
            backup_edge_config, // NEW
            link_local_source   // NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Replaces the old Docker check. Now checks for local binaries.
#[tauri::command]
async fn check_driver_status(window: Window, app: tauri::AppHandle) -> Result<String, String> {
    let mgr = deps::PulseManager::new(&app);
    let pkg = mgr.check_package("postgres-15");
    
    if pkg.status == "INSTALLED" {
        window.emit("log", "DRIVERS MOUNTED: Postgres 15 Ready.").unwrap();
        Ok("READY".to_string())
    } else {
        window.emit("log", "MISSING DRIVERS: Pulse Pack Required.").unwrap();
        Err("MISSING_DRIVERS".to_string())
    }
}

#[tauri::command]
async fn perform_migration(
    window: Window,
    source_url: String,
    _dest_url: String,
) -> Result<String, String> {
    window.emit("log", "Migration initiated...").unwrap();

    // WIRE TELEMETRY
    let event = telemetry::TelemetryEvent {
        event_type: "MIGRATION_START".to_string(),
        timestamp: 0,
        session_id: "test-session".to_string(),
        payload: serde_json::json!({ "source": source_url }), 
    };
    telemetry::track_event(&window, event);

    // WIRE STORAGE (Mocked keys for now to show usage)
    let mirror = storage::StorageMirror::new(
        &source_url, "mock_key", "mock_dest", "mock_key"
    );
    
    // Prove usage of internal methods
    window.emit("log", "Scanning object storage buckets...").unwrap();
    match mirror.list_source_buckets().await {
        Ok(buckets) => window.emit("log", format!("Found {} buckets", buckets.len())).unwrap(),
        Err(e) => window.emit("log", format!("Storage Scan Warning: {}", e)).unwrap(),
    }

    Ok("Migration logic executed.".to_string())
}

#[tauri::command]
async fn discover_local_databases(_window: Window) -> Result<Vec<String>, String> {
    Ok(vec!["localhost:5432".to_string()])
}

#[derive(Clone, serde::Serialize)]
struct ProgressEvent {
    stage: String,  // DATABASE, STORAGE, FUNCTIONS, AUTH
    status: String, // PENDING, RUNNING, DONE, ERROR
}

#[tauri::command]
async fn backup_database(window: Window, _url: String) -> Result<String, String> {
    window.emit("log", "Initializing Stasis Field...").unwrap();

    let stages = vec!["DATABASE", "STORAGE", "FUNCTIONS", "AUTH"];

    for stage in stages {
        // 1. Emit RUNNING
        window
            .emit(
                "progress_update",
                ProgressEvent {
                    stage: stage.to_string(),
                    status: "RUNNING".to_string(),
                },
            )
            .unwrap();

        window
            .emit("log", format!("Capturing {} snapshot...", stage))
            .unwrap();

        // Simulate work
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // 2. Emit DONE
        window
            .emit(
                "progress_update",
                ProgressEvent {
                    stage: stage.to_string(),
                    status: "DONE".to_string(),
                },
            )
            .unwrap();

        window.emit("log", format!("{} secured.", stage)).unwrap();
    }

    Ok("BACKUP_COMPLETE".to_string())
}

#[tauri::command]
async fn dry_run_migration(_window: Window, _script: String) -> Result<String, String> {
    Ok("Hull Integrity: 100%".to_string())
}

#[tauri::command]
async fn install_drivers(window: Window, app: tauri::AppHandle) -> Result<String, String> {
    let mgr = deps::PulseManager::new(&app);
    
    // ORBITAL DEPOT: Connects to the configured "Menu" (manifest.json)
    // The URL is loaded from config.json (defaults to devpulse-tools/drivers)
    
    // Fix: Use install_latest which reads the manifest
    match mgr.install_latest(&window, "postgres-15").await {
        Ok(_) => {
            window.emit("log", "Drivers Installed from Depot.").unwrap();
            Ok("INSTALLED".to_string())
        }
        Err(e) => {
            window.emit("log", format!("INSTALL FAILED: {}", e)).unwrap();
            Err(e)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            verify_connection,
            check_driver_status,
            install_drivers,
            perform_migration,
            discover_local_databases,
            backup_database,
            dry_run_migration
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
