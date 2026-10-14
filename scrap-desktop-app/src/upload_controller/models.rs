use serde::{
    Deserialize,
    Serialize
};

use uuid::Uuid;

use std::fmt::Display;

/// Represents the method to use for uploading a file.
///
/// Arguments
/// * `Stream` - Upload the file as a single stream.
/// * `Multipart` - Upload the file using multipart form data.
/// * `Chunked` - Upload the file in chunks.

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UploadMethod {
    #[serde(rename = "Stream")]
    Stream,
    #[serde(rename = "Multipart")]
    Multipart,
    #[serde(rename = "Chunked")]
    Chunked,
}


/// Represents the metadata for an upload.
///
/// Arguments
/// * `title` - The title of the upload.
/// * `description` - The description of the upload.
/// * `file_name` - The name of the file to upload.
///
#[derive(Debug, Clone)]
pub struct UploadMetadata {
    pub title: String,
    pub description: Option<String>,
    pub file_name: String,
}

/// Represents the request to create a new upload.
///
/// Arguments
/// * `title` - The title of the upload.
/// * `description` - The description of the upload.
/// * `file_name` - The name of the file to upload.
/// * `content_type` - The MIME type of the file.
/// * `file_size` - The size of the file in bytes.
/// * `method` - The method to use for the upload.
/// * `chunk_size` - The size of each chunk for chunked uploads.
///
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUploadRequest {
    pub title: String,
    pub description: Option<String>,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub method: UploadMethod,
    pub chunk_size: Option<i64>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUploadResponse {
    pub upload_id: Uuid,
    pub video_id: Uuid,
    pub status: String,
    pub method: UploadMethod,
    pub total_bytes: i64,
    pub chunk_size: Option<i32>,
    pub total_chunks: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteUploadResponse {
    pub upload_id: Uuid,
    pub video_id: Uuid,
    pub status: String,
    pub video_status: String,
    pub file_size: i64,
    pub completed_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadChunkResponse {
    pub upload_id: Uuid,
    pub chunk_number: i32,
    pub status: String,
    pub total_bytes: i64,
    pub uploaded_bytes: i64,
    pub total_chunks: Option<i32>,
    pub uploaded_chunks: i32,
}

/// Represents the result of an upload.
///
/// Arguments
/// * `upload_id` - The ID of the upload.
/// * `video_id` - The ID of the video.
/// * `file_size` - The size of the uploaded file in bytes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub upload_id: Uuid,
    pub video_id: Uuid,
    pub file_size: u64,
    pub elapsed_time: std::time::Duration,
}

#[derive(Debug)]
pub enum UploadError {
    FileNotFound(String),
    FileRead(String),
    Authentication(String),
    Network(String),
    Server {
        status: u16,
        message: String,
    },
    InvalidResponse(String),
}

impl Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            UploadError::FileRead(msg) => write!(f, "File read error: {}", msg),
            UploadError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            UploadError::Network(msg) => write!(f, "Network error: {}", msg),
            UploadError::Server { status, message } => write!(f, "Server error ({}): {}", status, message),
            UploadError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}