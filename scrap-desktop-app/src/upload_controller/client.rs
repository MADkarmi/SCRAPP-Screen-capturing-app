use std::{
    path::Path,
};

use reqwest::{
    Client,
    header,
    Body
};

use reqwest::multipart::{Part, Form};
use uuid::Uuid;

use crate::oauth::{
    OAuthConfig,
    session::OAuthSession
};

use tokio::{
    fs::File,
    io::{
        AsyncSeekExt,
        AsyncReadExt,
        SeekFrom,
    },
};

use tokio_util::io::ReaderStream;

use std::env;

use super::models::{
    CreateUploadRequest,
    CreateUploadResponse,
    CompleteUploadResponse,
    UploadError,
    UploadChunkResponse,
};
pub struct UploadClient {
    client: Client,
    uploads_endpoint: String,
}

impl UploadClient {
    /// Creates a new instance of `UploadClient`.
    /// Client is responsible for handling server communication for file uploads.
    pub fn new(uploads_endpoint: &str) -> Self {
        UploadClient {
            client: Client::new(),
            uploads_endpoint: uploads_endpoint.to_string(),
        }
    }

    /// Creates a new upload on the server.
    ///
    /// # Arguments
    /// * `oauth_config` - The OAuth configuration.
    /// * `oauth_session` - The OAuth session.
    /// * `request` - The upload request.
    ///
    pub async fn create_upload(
        &self,
        oauth_config: &OAuthConfig,
        oauth_session: &OAuthSession,
        request: CreateUploadRequest,
    ) -> Result<CreateUploadResponse, UploadError> {

        let access_token = oauth_session
            .get_access_token(oauth_config)
            .map_err(|e|
                UploadError::Authentication(
                    format!("Failed to get access token: {}", e)))?;

        let url = self.uploads_endpoint.clone();

        let response = self.client
            .post(&url)
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await
            .map_err(|e|
                UploadError::Network(
                    format!("Failed to send create upload request: {}", e)))?;

        let response_status = response.status();
        if !response.status().is_success(){
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(
                UploadError::Server {
                    status: response_status.as_u16(),
                    message: format!("Failed to create upload: {}", error_text)});
        }

        response
            .json::<CreateUploadResponse>()
            .await
            .map_err(|e|
                UploadError::InvalidResponse(
                    format!("Failed to parse create upload response: {}", e)))
    }

    /// Uploads a file to the server using the specified upload method.
    ///
    /// # Arguments
    /// * `oauth_config` - The OAuth configuration.
    /// * `oauth_session` - The OAuth session.
    /// * `upload_id` - The ID of the upload.
    /// * `file_size` - The size of the file to upload.
    /// * `file_path` - The path to the file to upload.
    ///
    pub async fn upload_raw(
        &self,
        oauth_config: &OAuthConfig,
        oauth_session: &OAuthSession,
        upload_id: Uuid,
        file_size: u64,
        file_path: &Path,
    ) -> Result<CompleteUploadResponse, UploadError>
    {

        let access_token = oauth_session
            .get_access_token(oauth_config)
            .map_err(|e|
                UploadError::Authentication(
                    format!("Failed to get access token: {}", e)))?;

        let file_path = file_path;

        let file = File::open(file_path)
            .await
            .map_err(|e|
                UploadError::FileRead(
                    format!("Failed to open file {}: {}", file_path.display(), e)))?;

        let stream = ReaderStream::new(file);

        //fix if error with cargo - failed to select a version for the requirement `wasm-streams = "^0.5"` then cargo update -p reqwest

        let body = reqwest::Body::wrap_stream(stream);

        let url =
            format!("{}/{}/{}",
                self.uploads_endpoint,
                upload_id,
                env::var("UPLOAD_RAW_API_ENDPOINT").unwrap_or_else(|_| "stream".into()));

        let response = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, file_size)
            .body(body)
            .send()
            .await
            .map_err(|e|
                UploadError::Network(
                    format!("Failed to upload video: {}", e)
            ))?;

        let status = response.status();
        if !status.is_success(){
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(
                UploadError::Server {
                    status: status.as_u16(),
                    message: format!("Failed to upload video with status {}: {}",
                        status, error)});
        }

        response
            .json::<CompleteUploadResponse>()
            .await
            .map_err(|e|
                UploadError::InvalidResponse(
                    format!("Failed to parse complete upload response: {}", e)))


    }


    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `oauth_config` - The OAuth configuration.
    /// * `oauth_session` - The OAuth session.
    /// * `file_path` - The path to the file to upload.
    /// * `upload_id` - The ID of the upload.
    ///
    ///
    pub async fn upload_multipart(
        &self,
        oauth_config: &OAuthConfig,
        oauth_session: &OAuthSession,
        upload_id: Uuid,
        file_path: &Path
    ) -> Result<CompleteUploadResponse, UploadError>
    {
        let access_token = oauth_session
            .get_access_token(oauth_config)
            .map_err(|e|
                UploadError::Authentication(
                    format!("Failed to get access token: {}", e)))?;

        let file_path = file_path;

        let file = File::open(file_path)
            .await
            .map_err(|e|
                UploadError::FileRead(
                    format!("Failed to open file {}: {}", file_path.display(), e)))?;

        let stream = ReaderStream::new(file);

        let part = Part::stream(Body::wrap_stream(stream))
            .file_name(
                file_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "upload".to_string())
            )
            .mime_str("application/octet-stream")
            .map_err(|e| UploadError::FileRead(
                format!("Failed to create multipart part: {}", e))
            )?;

        let form = Form::new()
            .part("file", part);

        let url =
            format!("{}/{}/{}",
                self.uploads_endpoint,
                upload_id,
                env::var("UPLOAD_MULTIPART_API_ENDPOINT").unwrap_or_else(|_| "multipart".into()));

        let response = self.
            client
            .post(url)
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|e|
                UploadError::Network(
                    format!("Failed to upload video: {}", e)
            ))?;

        let status = response.status();
        if !status.is_success(){
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(
                UploadError::Server {
                    status: status.as_u16(),
                    message: format!("Failed to upload video with status {}: {}",
                        status, error)});
        }

        response
            .json::<CompleteUploadResponse>()
            .await
            .map_err(|e|
                UploadError::InvalidResponse(
                    format!("Failed to parse complete upload response: {}", e)))
    }


    /// Uploads a single chunk of a file to the server.
    ///
    /// # Arguments
    /// * `oauth_config` - The OAuth configuration.
    /// * `oauth_session` - The OAuth session.
    /// * `upload_id` - The ID of the upload.
    /// * `chunk_number` - The number of the chunk to upload.
    /// * `chunk` - The chunk data to upload.
    pub async fn upload_chunk(
        &self,
        oauth_config: &OAuthConfig,
        oauth_session: &OAuthSession,
        upload_id: Uuid,
        chunk_number: u32,
        file_path: &Path,
        offset: u64,
        length: u64,
    ) -> Result<UploadChunkResponse, UploadError>
    {
        let access_token = oauth_session
            .get_access_token(oauth_config)
            .map_err(|e|
                UploadError::Authentication(
                    format!("Failed to get access token: {}", e)))?;

        let upload_id = upload_id.simple().to_string();
        let url =
            format!("{}/{}/{}/{}",
                self.uploads_endpoint,
                upload_id,
                env::var("UPLOAD_CHUNKED_API_ENDPOINT")
                    .unwrap_or_else(|_| "chunks".into()),
                chunk_number);

        let mut file = File::open(file_path)
            .await
            .map_err(|e|
                UploadError::FileRead(
                    format!("Failed to open file {}: {}",
                        file_path.display(), e)))?;

        file.seek(SeekFrom::Start(offset))
            .await
            .map_err(|e|
                UploadError::FileRead(
                    format!("Failed to seek in file {}: {}",
                        file_path.display(), e)))?;

        #[cfg(not(feature = "debug-upload-file-read"))]
        let chunk_reader = file.take(length);
        let stream = ReaderStream::new(chunk_reader);
        let body = reqwest::Body::wrap_stream(stream);

        #[cfg(feature = "debug-upload-file-read")]
        {
            let mut chunk_reader = vec![0u8; length as usize];
            file.read_exact(&mut chunk_reader)
                .await
                .map_err(|e|
                    UploadError::FileRead(
                        format!("Failed to read chunk from file {}: {}",
                            file_path.display(), e)))?;

            println!(
                "Sending chunk {}: offset={}, length={}",
                chunk_number,
                offset,
                length
            );

            let body = chunk_reader;
        }

        let response = self
            .client
            .put(url)
            .bearer_auth(access_token)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, length)
            .body(body)
            .send()
            .await
            .map_err(|e|
                UploadError::Network(
                    format!("Failed to upload chunk {}: {}",
                        chunk_number, e)))?;

        let status = response.status();

        if !status.is_success(){
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(
                UploadError::Server {
                    status: status.as_u16(),
                    message: format!("Failed to upload chunk {} with status {}: {}",
                        chunk_number, status, error)});
        }

        response
            .json::<UploadChunkResponse>()
            .await
            .map_err(|e| {
                UploadError::InvalidResponse(
                    format!(
                        "Failed to parse chunk {} response: {}",
                        chunk_number,
                        e
                    )
                )
            })
    }

    pub async fn complete_upload(
        &self,
        oauth_config: &OAuthConfig,
        oauth_session: &OAuthSession,
        upload_id: Uuid
    ) -> Result<CompleteUploadResponse, UploadError> {

        let access_token = oauth_session
            .get_access_token(oauth_config)
            .map_err(|e|
                UploadError::Authentication(
                    format!("Failed to get access token: {}", e)))?;

        let url =
            format!("{}/{}/{}",
                self.uploads_endpoint,
                upload_id,
                env::var("UPLOAD_COMPLETE_API_ENDPOINT").unwrap_or_else(|_| "complete".into()));

        let response = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e|
                UploadError::Network(
                    format!("Failed to send complete upload request: {}", e)))?;

        let status = response.status();
        if !status.is_success(){
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(
                UploadError::Server {
                    status: status.as_u16(),
                    message: format!("Failed to complete upload: {}, with status : {}",
                        error, status)});
        }

        response
            .json::<CompleteUploadResponse>()
            .await
            .map_err(|e|
                UploadError::InvalidResponse(
                    format!("Failed to parse complete upload response: {}", e)
            ))
    }



}