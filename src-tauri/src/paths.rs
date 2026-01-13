use std::path::PathBuf;
use std::fs;
use tauri::{AppHandle, Manager};

/// Determines the app root directory.
/// - Portable Mode: If `userdata/` exists next to the exe, use exe directory.
/// - Installed Mode: Use standard AppData directory.
pub fn get_app_root(app: &AppHandle) -> PathBuf {
    // Get the directory containing the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_marker = exe_dir.join("userdata");
            if portable_marker.exists() {
                return exe_dir.to_path_buf();
            }
        }
    }
    
    // Fallback to standard AppData location
    app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Gets the userdata directory (for config, profiles, backups)
pub fn get_userdata_dir(app: &AppHandle) -> PathBuf {
    get_app_root(app).join("userdata")
}

/// Gets the drivers directory (for pg_dump, psql, etc.)
pub fn get_drivers_dir(app: &AppHandle) -> PathBuf {
    get_app_root(app).join("drivers")
}

/// Gets the logs directory
pub fn get_logs_dir(app: &AppHandle) -> PathBuf {
    get_app_root(app).join("logs")
}

/// Ensures all required directories exist
pub fn ensure_directories(app: &AppHandle) -> Result<(), String> {
    let dirs = vec![
        get_userdata_dir(app),
        get_userdata_dir(app).join("backups"),
        get_drivers_dir(app),
        get_logs_dir(app),
    ];
    
    for dir in dirs {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create {:?}: {}", dir, e))?;
    }
    
    Ok(())
}

/// Gets the config file path
pub fn get_config_path(app: &AppHandle) -> PathBuf {
    get_userdata_dir(app).join("config.json")
}

/// Gets the profiles file path
pub fn get_profiles_path(app: &AppHandle) -> PathBuf {
    get_userdata_dir(app).join("profiles.json")
}
