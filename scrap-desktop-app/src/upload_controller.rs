use std::sync::Arc;
use slint::Weak;
use tokio::runtime::Runtime;
use crate::AppWindow;

pub mod client;
pub mod models;
pub mod manager;

pub use manager::UploadManager;
pub use client::UploadClient;
pub use models::{
    UploadMetadata,
    UploadResult,
    UploadMethod,
};

#[derive(Clone)]
pub struct UploadController {
    manager: Arc<UploadManager>,
}

impl UploadController{
    /// Creates a new instance of `UploadController`.
    /// Controller is responsible for managing the upload process and interacting with the UI.
    pub fn new(manager: Arc<UploadManager>) -> Self {
        UploadController {
            manager,
        }
    }

    /// Uploads a file to the server using the specified upload method.
    ///
    /// Arguments
    /// * `file_path` - The path to the file to upload.
    /// * `title` - The title of the upload.
    /// * `description` - The description of the upload.
    /// * `method` - The method to use for the upload (Stream, Multipart, or Chunked).
    /// * `ui` - A weak reference to the application window for UI updates.
    pub fn upload(
        &self,
        file_path: String,
        title: String,
        description: String,
        method: UploadMethod,
        ui: Weak<AppWindow>,
    ) {
        let title = title
            .trim()
            .chars()
            .take(255)
            .collect::<String>();

        if title.is_empty(){
            Self::show_message(ui, "Title cannot be empty".to_string());
            return;
        }

        if file_path.trim().is_empty(){
            Self::show_message(ui, "File path cannot be empty".to_string());
            return;
        }

        let description = description
            .trim()
            .chars()
            .take(5000)
            .collect::<String>();

        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let metadata = UploadMetadata {
            title,
            description: if description.is_empty() { None } else { Some(description) },
            file_name
        };

        let manager = self.manager.clone();
        std::thread::spawn(move ||{
            let runtime = match Runtime::new(){
                Ok(runtime) => runtime,
                Err(e) => {
                    Self::show_message(ui, format!("Failed to create runtime: {}", e));
                    return;
                }

            };

            let result = runtime.block_on(
                manager.upload_file(
                    &file_path,
                    metadata,
                    method
                ));

            match result {
                Ok(upload_result) => {
                    Self::show_message(ui,
                        format!(
                            "Upload successful! \n
                            Video ID: {}, File Size: {} bytes \n
                            Execution time: {:.2?} seconds
                            Average speed: {:.2?} MB/sec",
                            upload_result.video_id,
                            upload_result.file_size,
                            upload_result.elapsed_time.as_secs_f64(),
                            upload_result.file_size as f64 / upload_result.elapsed_time.as_secs_f64() / 1_000_000.0
                    ));
                },
                Err(e) => {
                    Self::show_message(ui, format!("Upload failed: {}", e));
                }
            }


        });

    }

    fn show_message(
        ui: Weak<AppWindow>,
        message: String
    ){
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade(){
                ui.set_uploadMessage(message.into());
                ui.invoke_show_upload_message();
            }
        });
    }
}
