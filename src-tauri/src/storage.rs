use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct StorageObject {
    pub name: String,
    pub id: String,
    // Add other fields as needed (metadata, etc.)
}

pub struct StorageMirror {
    client: Client,
    source_url: String,
    source_key: String,
    dest_url: String,
    dest_key: String,
}

impl StorageMirror {
    pub fn new(source_url: &str, source_key: &str, dest_url: &str, dest_key: &str) -> Self {
        Self {
            client: Client::new(),
            source_url: source_url.to_string(),
            source_key: source_key.to_string(),
            dest_url: dest_url.to_string(),
            dest_key: dest_key.to_string(),
        }
    }

    pub async fn list_source_buckets(&self) -> Result<Vec<Bucket>, String> {
        let url = format!("{}/storage/v1/bucket", self.source_url);
        let res = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.source_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Failed to list buckets: {}", res.status()));
        }

        res.json::<Vec<Bucket>>().await.map_err(|e| e.to_string())
    }

    pub async fn list_objects(&self, bucket_id: &str) -> Result<Vec<StorageObject>, String> {
        let url = format!("{}/storage/v1/object/list/{}", self.source_url, bucket_id);

        // Supabase list objects is a POST with prefix/limit/offset
        let body = serde_json::json!({
            "prefix": "",
            "limit": 100,
            "offset": 0,
            "sortBy": {
                "column": "name",
                "order": "asc"
            }
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.source_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!(
                "Failed to list objects in {}: {}",
                bucket_id,
                res.status()
            ));
        }

        res.json::<Vec<StorageObject>>()
            .await
            .map_err(|e| e.to_string())
    }
    /// Upload object to destination bucket
    pub async fn upload_object(
        &self,
        bucket_id: &str,
        object_name: &str,
        data: Vec<u8>,
    ) -> Result<(), String> {
        let url = format!(
            "{}/storage/v1/object/{}/{}", 
            self.dest_url, bucket_id, object_name
        );
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.dest_key))
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await
            .map_err(|e| format!("Upload failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Upload failed with status: {}", response.status()));
        }
        
        Ok(())
    }
}
