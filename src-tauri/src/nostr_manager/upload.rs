pub struct BlossomClient {
    url: String,
}

impl BlossomClient {
    pub fn new(url: &str) -> Self {
        BlossomClient {
            url: url.to_string(),
        }
    }

    pub async fn upload(&self, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let file = tokio::fs::read(file_path).await?;

        let client = reqwest::Client::new();
        let response = client
            .put(format!("{}/upload", self.url))
            .body(file)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Upload failed with status: {}", response.status()).into());
        }

        let json: serde_json::Value = response.json().await?;
        Ok(json["url"].as_str().unwrap_or_default().to_string())
    }

    pub async fn download(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()).into());
        }

        Ok(response.bytes().await?.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_upload() {
        let client = BlossomClient::new("http://localhost:3000");

        // Path to the test image
        // FIXME: generate a random image every time ...
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pic.jpg");

        // First upload the file
        let url = client
            .upload(file_path.to_str().unwrap())
            .await
            .expect("Failed to upload file");

        println!("Uploaded file URL: {}", url);

        // Now download the file and verify contents
        let downloaded_bytes = client
            .download(&url)
            .await
            .expect("Failed to download file");

        // Read original file for comparison
        let original_bytes = tokio::fs::read(&file_path)
            .await
            .expect("Failed to read original file");

        assert_eq!(
            downloaded_bytes.len(),
            original_bytes.len(),
            "Downloaded file size doesn't match original"
        );
        assert_eq!(
            downloaded_bytes, original_bytes,
            "Downloaded file contents don't match original"
        );
    }
}
