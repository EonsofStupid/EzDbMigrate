use tauri::{Window, Emitter};
use reqwest::Client;
use regex::Regex;

// "The Pivot": Since we can't download code, we backup config and zip local source.

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FunctionConfig {
    pub name: String,
    pub slug: String,
    pub version: i32,
    pub status: String,
    pub entrypoint: String,
    pub verify_jwt: bool,
}

/// Extracts project reference from Supabase URL
/// Example: "https://dcmgooupmorhqjbdaxtm.supabase.co" -> "dcmgooupmorhqjbdaxtm"
pub fn extract_project_ref(url: &str) -> Result<String, String> {
    let re = Regex::new(r"https://([a-z0-9]+)\.supabase\.co")
        .map_err(|e| format!("Regex error: {}", e))?;
    
    re.captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| format!("Invalid Supabase URL format: {}", url))
}

pub async fn backup_function_config(
    window: &Window, 
    project_url: &str, 
    service_key: &str
) -> Result<Vec<FunctionConfig>, String> {
    window.emit("log", "Fetching Edge Function configurations...").unwrap();
    
    // Extract project ref for Management API
    let project_ref = extract_project_ref(project_url)?;
    window.emit("log", format!("Project Ref: {}", project_ref)).unwrap();
    
    let client = Client::new();
    
    // Supabase Management API endpoint for functions
    let management_url = format!(
        "https://api.supabase.com/v1/projects/{}/functions", 
        project_ref
    );
    
    let response = client
        .get(&management_url)
        .header("Authorization", format!("Bearer {}", service_key))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        // Management API may require org-level token, not project service key
        window.emit("log", format!(
            "Management API returned {}. This API may require an organization access token.", 
            status
        )).unwrap();
        
        // Return empty - user must link local source
        window.emit("log", "Falling back to local source linking mode.").unwrap();
        return Ok(vec![]);
    }
    
    let configs: Vec<FunctionConfig> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    window.emit("log", format!("Found {} function configurations.", configs.len())).unwrap();
    Ok(configs)
}

pub fn zip_local_source(window: &Window, local_path: &str) -> Result<String, String> {
    use std::fs::File;
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;
    
    let source_path = std::path::Path::new(local_path);
    if !source_path.exists() {
        return Err(format!("Path does not exist: {}", local_path));
    }

    window.emit("log", format!("Zipping source from: {}", local_path)).unwrap();
    
    // Create output zip file in temp directory
    let output_path = std::env::temp_dir().join("devpulse_functions_backup.zip");
    let file = File::create(&output_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;
    
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    // If it's a directory, walk and add all files
    if source_path.is_dir() {
        for entry in walkdir(source_path)? {
            let entry_path = entry;
            let name = entry_path.strip_prefix(source_path)
                .map_err(|e| e.to_string())?
                .to_string_lossy();
            
            if entry_path.is_file() {
                zip.start_file(name.to_string(), options)
                    .map_err(|e| e.to_string())?;
                let content = std::fs::read(&entry_path)
                    .map_err(|e| e.to_string())?;
                zip.write_all(&content).map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Single file
        let name = source_path.file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy();
        zip.start_file(name.to_string(), options)
            .map_err(|e| e.to_string())?;
        let content = std::fs::read(source_path)
            .map_err(|e| e.to_string())?;
        zip.write_all(&content).map_err(|e| e.to_string())?;
    }
    
    zip.finish().map_err(|e| e.to_string())?;
    
    let output_str = output_path.to_string_lossy().to_string();
    window.emit("log", format!("Source archived: {}", output_str)).unwrap();
    Ok(output_str)
}

/// Simple directory walker
fn walkdir(path: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut results = vec![];
    
    fn visit(dir: &std::path::Path, results: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            results.push(path.clone());
            if path.is_dir() {
                visit(&path, results)?;
            }
        }
        Ok(())
    }
    
    visit(path, &mut results)?;
    Ok(results)
}
