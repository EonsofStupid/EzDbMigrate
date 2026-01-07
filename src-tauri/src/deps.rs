use std::path::PathBuf;
use std::fs;
use std::io::Cursor;
use tauri::{AppHandle, Manager, Window, Emitter};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(serde::Serialize, Clone)]
pub struct PulsePackage {
    pub id: String,
    pub version: String,
    pub status: String, // "INSTALLED", "MISSING"
}

pub struct PulseManager {
    base_path: PathBuf,
    client: reqwest::Client,
}

impl PulseManager {
    pub fn new(app: &AppHandle) -> Self {
        let app_data = app.path().app_data_dir().unwrap();
        let pulse_root = app_data.join("DevPulse").join("bin");
        
        if !pulse_root.exists() {
            let _ = fs::create_dir_all(&pulse_root);
        }
        
        // GitHub requires a User-Agent
        let client = reqwest::Client::builder()
            .user_agent("DevPulse-Migrator/1.0")
            .build()
            .unwrap();

        Self {
            base_path: pulse_root,
            client,
        }
    }

    pub fn resolve(&self, package_id: &str, binary_name: &str) -> Result<PathBuf, String> {
        let pkg_root = self.base_path.join(package_id);
        
        // Search Strategy: Root -> bin -> pgsql/bin -> postgres-[ver]/bin
        let candidates = vec![
            pkg_root.join(binary_name),
            pkg_root.join("bin").join(binary_name),
            pkg_root.join("pgsql").join("bin").join(binary_name),
        ];

        for path in candidates {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(format!("Binary {} not found in package {}", binary_name, package_id))
    }

    pub fn check_package(&self, package_id: &str) -> PulsePackage {
        // We assume if we can resolve pg_dump, the package is healthy enough
        match self.resolve(package_id, "pg_dump.exe") {
            Ok(_) => PulsePackage {
                id: package_id.to_string(),
                version: "detected".to_string(),
                status: "INSTALLED".to_string(),
            },
            Err(_) => PulsePackage {
                id: package_id.to_string(),
                version: "none".to_string(),
                status: "MISSING".to_string(),
            },
        }
    }

    /// Fetches the latest release from GitHub and installs the matching asset
    pub async fn install_from_github(&self, window: &Window, package_id: &str, repo_owner: &str, repo_name: &str) -> Result<(), String> {
        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", repo_owner, repo_name);
        window.emit("log", format!("Checking updates: {}", url)).unwrap();

        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        
        if !resp.status().is_success() {
             return Err(format!("GitHub API Error: {}", resp.status()));
        }

        let release: GitHubRelease = resp.json().await.map_err(|e| e.to_string())?;
        window.emit("log", format!("Latest Release: {}", release.tag_name)).unwrap();

        // FIND ASSET: Look for "windows" and "zip"
        // In "True Pro" mode we'd be more strict, but this works for the "Smart" logic
        let asset = release.assets.iter()
            .find(|a| a.name.to_lowercase().contains("windows") && a.name.ends_with(".zip"))
            .ok_or("No windows compatible asset found in release")?;

        window.emit("log", format!("Found Asset: {} ({:.2} MB)", asset.name, asset.size as f64 / 1024.0 / 1024.0)).unwrap();

        // DOWNLOAD
        self.download_and_extract(window, package_id, &asset.browser_download_url).await
    }

    async fn download_and_extract(&self, window: &Window, package_id: &str, url: &str) -> Result<(), String> {
        let target_dir = self.base_path.join(package_id);
        
        window.emit("log", "Initiating Transfer...").unwrap();
        let response = self.client.get(url).send().await.map_err(|e| e.to_string())?;
        let content = response.bytes().await.map_err(|e| e.to_string())?;
        
        window.emit("log", "Extracting Payload...").unwrap();
        let reader = Cursor::new(content);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = target_dir.join(file.mangled_name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        window.emit("log", "Pulse Pack Installed.").unwrap();
        Ok(())
    }
}
