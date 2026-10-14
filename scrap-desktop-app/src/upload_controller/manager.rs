use std::{
    env,
    path::Path
};

use futures::{
    stream,
    StreamExt
};

use crate::upload_controller::UploadResult;

use super:: {
    super::{
        OAuthConfig,
        OAuthSession
    },
    client::UploadClient,
    models::{
        CreateUploadRequest,
        UploadMetadata,
        UploadMethod,
        UploadError,
        UploadChunkResponse,
    }
};

pub struct UploadManager{
    client: UploadClient,
    oauth_config: OAuthConfig,
    oauth_session: OAuthSession,
    //chunk_size: Option<i64>,
}

impl UploadManager{
    /// Creates a new instance of `UploadManager`.
    /// Manager is responsible for coordinating the upload process and managing the upload client.
    pub fn new(
        oauth_config: OAuthConfig,
        oauth_session: OAuthSession,
        upload_client: UploadClient
    ) -> Self {
        UploadManager {
            client: upload_client,
            oauth_config,
            oauth_session,
        }
    }

    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `file_path` - The path to the file to upload.
    /// * `metadata` - The metadata for the upload.
    /// * `upload_method` - The method to use for the upload (Stream, Multipart
    /// or Chunked).
    ///
    /// Additionally times the upload process and returns the elapsed time in the UploadResult.
    pub async fn upload_file(
        &self,
        file_path: &str,
        metadata: UploadMetadata,
        upload_method: super::models::UploadMethod
    ) -> Result<UploadResult, UploadError> {
        match upload_method {
            UploadMethod::Stream => {
                self.upload_stream(file_path, metadata, upload_method).await
            },
            UploadMethod::Multipart => {
                self.upload_multipart(file_path, metadata, upload_method).await
            },
            UploadMethod::Chunked => {
                let default_chunk_size : i64 =
                    env::var("DEFAULT_CHUNK_SIZE")
                    .unwrap_or_else(|_| "5242880".into())
                    .parse()
                    .unwrap_or(5242880);


                let max_concurrent_chunks : u32 =
                    env::var("MAX_CONCURRENT_CHUNKS")
                    .unwrap_or_else(|_| "4".into())
                    .parse()
                    .unwrap_or(4);

                self.upload_chunked(file_path, metadata, upload_method, default_chunk_size, max_concurrent_chunks).await
            },
        }
    }

    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `file_path` - The path to the file to upload.
    /// * `metadata` - The metadata for the upload.
    /// * `upload_method` - The method to use for the upload (Stream, Multipart
    /// or Chunked).
    ///
    /// Additionally times the upload process and returns the elapsed time in the UploadResult.
    async fn upload_stream(
        &self,
        file_path: &str,
        metadata: UploadMetadata,
        upload_method: UploadMethod
    ) -> Result<UploadResult, UploadError> {
        let upload_request = self
            .create_upload_request(
                metadata,
                file_path,
                upload_method,
                None)?;

        let file_size = upload_request.file_size as u64;

        let created = self.client
            .create_upload(
                &self.oauth_config,
                &self.oauth_session,
                upload_request)
            .await?;

        let timer = std::time::Instant::now();

        let completed = self.client
            .upload_raw(
                &self.oauth_config,
                &self.oauth_session,
                created.upload_id,
                file_size,
                file_path.as_ref()
            ).await?;

        let elapsed_time = timer.elapsed();

        Ok(UploadResult {
            upload_id: completed.upload_id,
            video_id: completed.video_id,
            file_size: completed.file_size as u64,
            elapsed_time,
        })
    }

    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `file_path` - The path to the file to upload.
    /// * `metadata` - The metadata for the upload.
    /// * `upload_method` - The method to use for the upload (Stream, Multipart
    /// or Chunked).
    ///
    /// Additionally times the upload process and returns the elapsed time in the UploadResult.
    async fn upload_multipart(
        &self,
        file_path: &str,
        metadata: UploadMetadata,
        upload_method: UploadMethod
    ) -> Result<UploadResult, UploadError> {
        let upload_request = self
            .create_upload_request(
                metadata,
                file_path,
                upload_method,
                None)?;

        let created = self.client
            .create_upload(
                &self.oauth_config,
                &self.oauth_session,
                upload_request).await?;

        let timer = std::time::Instant::now();

        let completed = self.client
            .upload_multipart(
                &self.oauth_config,
                &self.oauth_session,
                created.upload_id,
                file_path.as_ref()
            ).await?;

        let elapsed_time = timer.elapsed();

        Ok(UploadResult {
            upload_id: completed.upload_id,
            video_id: completed.video_id,
            file_size: completed.file_size as u64,
            elapsed_time,
        })
    }


    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `file_path` - The path to the file to upload.
    /// * `metadata` - The metadata for the upload.
    /// * `upload_method` - The method to use for the upload (Stream, Multipart
    /// or Chunked).
    /// * `chunk_size` - Size for each chunk for chunked uploads.
    ///
    /// Additionally times the upload process and returns the elapsed time in the UploadResult.
    async fn upload_chunked(
        &self,
        file_path: &str,
        metadata: UploadMetadata,
        upload_method: UploadMethod,
        chunk_size: i64,
        max_concurrent_chunks: u32
    ) -> Result<UploadResult, UploadError> {
        let upload_request = self
            .create_upload_request(
                metadata,
                file_path,
                upload_method,
                Some(chunk_size))?;

        let file_size = upload_request.file_size as u64;

        let created = self.client
            .create_upload(
                &self.oauth_config,
                &self.oauth_session,
                upload_request).await?;

        let chunk_size = created.chunk_size
            .ok_or_else(|| UploadError::InvalidResponse(
                "Server did not provide a chunk size for chunked upload".to_string()))? as u64;

        let total_chunks = created.total_chunks
            .ok_or_else(|| UploadError::InvalidResponse(
                "Server did not provide total chunks for chunked upload".to_string()))? as u64;

        let upload_id = created.upload_id;
        let file_path = file_path.to_string();

        let timer  = std::time::Instant::now();

        let chunk_results =
            stream::iter(0..total_chunks)
            .map(|chunk_number| {
                let file_path = file_path.clone();
                let oauth_config = &self.oauth_config;
                let oauth_session = &self.oauth_session;
                let client = &self.client;

                let offset = chunk_number * chunk_size;
                let remaining_bytes = file_size.saturating_sub(offset);
                let length = remaining_bytes.min(chunk_size);

                async move{
                    client.upload_chunk(
                        oauth_config,
                        oauth_session,
                        upload_id,
                        chunk_number as u32,
                        file_path.as_ref(),
                        offset,
                        length)
                    .await
                }
            })
            .buffer_unordered(max_concurrent_chunks as usize)
            .collect::<Vec<Result<UploadChunkResponse, UploadError>>>()
            .await;

        for result in chunk_results {
            result.map_err(|e| {
                UploadError::Network(format!("Failed to upload chunk: {}", e))
            })?;
        }

        let completed = self
            .client
                .complete_upload(
                    &self.oauth_config,
                    &self.oauth_session,
                    created.upload_id)
                .await?;

        let elapsed_time = timer.elapsed();

        Ok(UploadResult {
            upload_id: completed.upload_id,
            video_id: completed.video_id,
            file_size: completed.file_size as u64,
            elapsed_time,
        })
    }

    /// Validates the file path and checks if the file exists.
    ///
    /// # Arguments
    /// * `file_path` - The path to the file to validate.
    ///
    fn validate_file_path(&self, file_path: &Path) -> Result<(), UploadError> {
        if !file_path.exists() {
            return Err(
                UploadError::FileNotFound(
                    format!("File path does not exist: {}", file_path.display())));
        }
        if !file_path.is_file() {
            return Err(
                UploadError::FileNotFound(
                    format!("Path is not a file: {}", file_path.display())));
        }
        Ok(())
    }

    /// Gets the metadata of the file at the specified path.
    ///
    /// # Arguments
    /// * `file_path` - The path to the file.
    ///
    fn get_file_metadata(&self, file_path: &str) -> Result<std::fs::Metadata, UploadError> {
        let path = std::path::Path::new(file_path);

        self.validate_file_path(path)?;

        std::fs::metadata(path)
            .map_err(|e|
                UploadError::FileRead(
                    format!("Failed to get file metadata for {}: {}", path.display(), e)))
    }

    /// Creates a `CreateUploadRequest` from the provided metadata and file path.
    ///
    /// # Arguments
    /// * `metadata` - The metadata for the upload.
    /// * `file_path` - The path to the file to upload.
    /// * `upload_method` - The method to use for the upload (Stream, Multipart, or Chunked).
    /// * `chunk_size` - Optional size for each chunk for chunked uploads.
    ///
    fn create_upload_request(
        &self,
        metadata: UploadMetadata,
        file_path: &str,
        upload_method: UploadMethod,
        chunk_size: Option<i64>
    ) -> Result<CreateUploadRequest, UploadError> {
        let file_metadata = self.get_file_metadata(file_path)?;
        let file_size = file_metadata.len() as i64;

        Ok(CreateUploadRequest {
            title: metadata.title,
            description: metadata.description,
            file_name: metadata.file_name,
            content_type: "application/octet-stream".to_string(),
            file_size,
            method: upload_method,
            chunk_size,
        })
    }

}
