#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompressionParams {
    pub quality: u32,
    pub mode: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompressedInfo {
    pub sha256: String,
    pub size: u64,
    pub library: String,
    pub version: String,
    pub parameters: CompressionParams,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlobDescriptor {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub uploaded: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed: Option<CompressedInfo>,
}

#[derive(Clone, Debug)]
pub struct BlossomClient {
    url: String,
}

impl BlossomClient {
    pub fn new(url: &str) -> Self {
        BlossomClient {
            url: url.to_string(),
        }
    }

    pub async fn upload(
        &self,
        file: Vec<u8>,
    ) -> Result<BlobDescriptor, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .put(format!("{}/upload", self.url))
            .body(file)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Upload failed with status: {}", response.status()).into());
        }

        let blob_descriptor: BlobDescriptor = response.json().await?;
        Ok(blob_descriptor)
    }

    pub async fn download(
        &self,
        url: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
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

    #[tokio::test]
    async fn test_upload() {
        let client = BlossomClient::new("http://localhost:3000");

        // Generate random bytes for testing
        let random_bytes: Vec<u8> = uuid::Uuid::new_v4().as_bytes().to_vec();

        // First upload the file
        let blob_descriptor = client
            .upload(random_bytes.clone())
            .await
            .expect("Failed to upload file");

        println!("Uploaded file descriptor: {:?}", blob_descriptor);

        // Now download the file and verify contents
        let downloaded_bytes = client
            .download(&blob_descriptor.url)
            .await
            .expect("Failed to download file");

        // Assert that we got the same bytes back
        assert_eq!(
            downloaded_bytes, random_bytes,
            "Downloaded file contents don't match original"
        );
    }
}
