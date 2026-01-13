use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Window};

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PulseChannel {
    pub version: String,
    pub required: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PulseRollout {
    pub id: String,
    pub r#type: String, // "toast", "popup", "modal"
    pub title: String,
    pub message: String,
    pub media_url: Option<String>,
    pub action_url: Option<String>,
    pub min_app_version: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PulseManifest {
    pub tool: String,
    // "Intelligent Channels" - Stable vs Insider
    pub channels: Option<std::collections::HashMap<String, PulseChannel>>,
    // "Unwrap Experience" - Media/Toasts
    pub pulse_rollout: Option<PulseRollout>,
    pub packages: std::collections::HashMap<String, PulsePackageSpec>,
    pub message_of_the_day: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PulsePackageSpec {
    pub url: String,
    pub checksum: String, // sha256
    pub size_mb: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PulseConfig {
    pub channel: String, // "stable", "insider"
    pub supabase_url: String,
    pub supabase_key: String, // Public Anon Key
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct PulsePackage {
    pub id: String,
    pub version: String,
    pub status: String,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            channel: "stable".to_string(),
            // Pre-configured for DevPulse - User can override in config.json
            supabase_url: "https://dcmgooupmorhqjbdaxtm.supabase.co".to_string(),
            supabase_key: "".to_string(), // TODO: Must be provided by user or build arg
        }
    }
}

pub struct PulseManager {
    base_path: PathBuf,
    client: reqwest::Client,
    config: PulseConfig,
}

impl PulseManager {
    pub fn new(app: &AppHandle) -> Self {
        let app_data = app.path().app_data_dir().unwrap();
        let pulse_root = app_data.join("DevPulse").join("bin");
        let config_path = app_data.join("DevPulse").join("config.json");

        if !pulse_root.exists() {
            let _ = fs::create_dir_all(&pulse_root);
        }

        // Load Config or Create Default
        let config = if config_path.exists() {
            let data = fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            let def = PulseConfig::default();
            let _ = fs::write(&config_path, serde_json::to_string_pretty(&def).unwrap());
            def
        };

        let client = reqwest::Client::builder()
            .user_agent("DevPulse-Migrator/1.0")
            .build()
            .unwrap();

        Self {
            base_path: pulse_root,
            client,
            config,
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

        Err(format!(
            "Binary {} not found in package {}",
            binary_name, package_id
        ))
    }

    pub fn check_package(&self, package_id: &str) -> PulsePackage {
        // We assume if we can resolve pg_dump, the package is healthy enough
        match self.resolve(package_id, "pg_dump.exe") {
            Ok(_) => PulsePackage {
                id: package_id.to_string(),
                version: "detected".to_string(), // In future: read local manifest
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
    /// FALLBACK MODE: Direct GitHub access if Supabase is offline
    pub async fn install_from_github(
        &self,
        window: &Window,
        package_id: &str,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<(), String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            repo_owner, repo_name
        );
        window
            .emit("log", format!("Checking updates (fallback): {}", url))
            .unwrap();

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("GitHub API Error: {}", resp.status()));
        }

        let release: GitHubRelease = resp.json().await.map_err(|e| e.to_string())?;
        window
            .emit("log", format!("Latest Release: {}", release.tag_name))
            .unwrap();

        let asset = release
            .assets
            .iter()
            .find(|a| a.name.to_lowercase().contains("windows") && a.name.ends_with(".zip"))
            .ok_or("No windows compatible asset found in release")?;

        window
            .emit(
                "log",
                format!(
                    "Found Asset: {} ({:.2} MB)",
                    asset.name,
                    asset.size as f64 / 1024.0 / 1024.0
                ),
            )
            .unwrap();

        self.download_and_extract(window, package_id, &asset.browser_download_url)
            .await
    }

    /// STEP 1: RESOLVE - Ask Supabase "Brain" for the correct Manifest
    async fn resolve_active_release(&self, window: &Window) -> Result<String, String> {
        window
            .emit(
                "log",
                format!(
                    "Pulse Protocol: Syncing with Channel '{}'...",
                    self.config.channel
                ),
            )
            .unwrap();

        // 1. Construct Query: Select * from pulse_releases where channel_slug = $1 and is_active = true limit 1
        let query_url = format!(
            "{}/rest/v1/pulse_releases?channel_slug=eq.{}&is_active=eq.true&select=manifest_url,version,rollout_message&limit=1",
            self.config.supabase_url, self.config.channel
        );

        // 2. Execute Query
        let resp = self
            .client
            .get(&query_url)
            .header("apikey", &self.config.supabase_key)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.supabase_key),
            )
            .send()
            .await
            .map_err(|e| format!("Network Error: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Pulse Brain Unreachable ({}). Is Anon Key valid?",
                resp.status()
            ));
        }

        // 3. Parse Response
        let releases: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;
        let active = releases
            .first()
            .ok_or("No active release found for this channel.")?;

        let manifest_url = active["manifest_url"]
            .as_str()
            .ok_or("Invalid Manifest URL")?
            .to_string();
        let version = active["version"].as_str().unwrap_or("unknown");

        window
            .emit(
                "log",
                format!("Resolved Release v{} [{}]", version, manifest_url),
            )
            .unwrap();

        Ok(manifest_url)
    }

    /// ORBITAL DEPOT LOGIC: Fetch the "Menu" (Manifest)
    /// Now accepts a specific URL (resolved from Supabase)
    pub async fn fetch_manifest(&self, url: &str) -> Result<PulseManifest, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Manifest Unreachable ({})", resp.status()));
        }

        resp.json().await.map_err(|e| e.to_string())
    }

    /// Installs the package defined in the manifest for the current OS
    pub async fn install_latest(&self, window: &Window, package_id: &str) -> Result<(), String> {
        // STEP 1: Resolve (Supabase)
        let manifest_url = match self.resolve_active_release(window).await {
            Ok(url) => url,
            Err(e) => {
                window
                    .emit("log", format!("Pulse Protocol Sync Failed: {}", e))
                    .unwrap();
                window
                    .emit("log", "Falling back to hardcoded Depot default...")
                    .unwrap();
                "https://raw.githubusercontent.com/devpulse-tools/dptools-deps/main/deps/apps/ezdb/manifest.json".to_string()
            }
        };

        // STEP 2: Hydrate (GitHub Manifest)
        window.emit("log", "Acquiring Manifest...").unwrap();
        let manifest = self.fetch_manifest(&manifest_url).await?;

        // Intelligent Version Resolution
        let version = if let Some(channels) = &manifest.channels {
            channels
                .get("stable")
                .map(|c| c.version.clone())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "legacy".to_string()
        };

        window
            .emit(
                "log",
                format!("Manifest Acquired: {} v{}", manifest.tool, version),
            )
            .unwrap();

        // Intelligent Unwrap (Rollouts)
        if let Some(rollout) = &manifest.pulse_rollout {
            window
                .emit(
                    "log",
                    format!(
                        "PULSE ROLLOUT: [{}] {}",
                        rollout.r#type.to_uppercase(),
                        rollout.title
                    ),
                )
                .unwrap();
            // TODO: Emit "pulse_rollout" event to frontend
        }

        // STEP 3: Download Binary (GitHub Assets)
        // OS Detection (Hardcoded to win32-x64 for this Windows-only tool)
        let target_os = "win32-x64";
        let pkg_spec = manifest
            .packages
            .get(target_os)
            .ok_or("No package found for this OS in manifest")?;

        window
            .emit(
                "log",
                format!("Acquiring Ordnance: {:.2} MB", pkg_spec.size_mb),
            )
            .unwrap();

        self.download_and_extract(window, package_id, &pkg_spec.url)
            .await
    }

    async fn download_and_extract(
        &self,
        window: &Window,
        package_id: &str,
        url: &str,
    ) -> Result<(), String> {
        let target_dir = self.base_path.join(package_id);

        window.emit("log", "Initiating Transfer...").unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
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
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        window.emit("log", "Pulse Pack Installed.").unwrap();
        Ok(())
    }
}
