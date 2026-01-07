use reqwest::Client;


pub async fn validate_service_key(project_url: &str, service_key: &str) -> Result<String, String> {
    let client = Client::new();
    // We check /storage/v1/bucket because we specifically need Storage Admin rights
    // and it's a good proxy for "Service Role" validity.
    let url = format!("{}/storage/v1/bucket", project_url);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", service_key))
        .header("apikey", service_key)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if res.status().is_success() {
        // We could verify we get a list back, but 200 OK is sufficient proof of auth
        Ok("Key Validated: Storage Admin Access Confirmed".to_string())
    } else {
        // Try to parse error message
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!("Validation Failed (Status {}): {}", status, body))
    }
}
