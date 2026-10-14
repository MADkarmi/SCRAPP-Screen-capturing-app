use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use slint::{
    ModelRc,
    VecModel,
    SharedString,
};
use std::{
    fs,
    rc::Rc,
};
use super::recorder_controller::recorder_supported_options::{RecorderSupportedFileFormat, RecorderSupportedFps, RecorderSupportedResolution};
use crate::RecorderSupportedSettings;

impl RecorderSupportedResolution {
    /// Returns a vector of strings representing the supported resolutions.
    fn to_vec() -> Vec<String>{
        vec![RecorderSupportedResolution::_720p.to_string(),
             RecorderSupportedResolution::_1080p.to_string(),
             RecorderSupportedResolution::_1440p.to_string()]
    }
}
impl RecorderSupportedFps {
    /// Returns a vector of strings representing the supported frames per second (FPS) options.
    fn to_vec() -> Vec<String>{
        vec![RecorderSupportedFps::_30.to_string(),
             RecorderSupportedFps::_60.to_string(),
             RecorderSupportedFps::_120.to_string()]
    }
}
impl RecorderSupportedFileFormat {
    /// Returns a vector of strings representing the supported file formats.
    fn to_vec() -> Vec<String>{
        vec![RecorderSupportedFileFormat::MP4.to_string()]
    }
}

/// Converts a vector of strings into a ModelRc<SharedString> for use in the UI.
fn to_model_rc_string_vec(vec: Vec<String>) -> ModelRc<SharedString>{
    let shared_string_vec: Vec<SharedString> = vec.into_iter().map(|s| SharedString::from(s)).collect();
    ModelRc::from(Rc::new(VecModel::from(shared_string_vec)))
}

/// Loads the supported settings for the recorder, including file formats, FPS options, and resolutions.
pub fn load_supported_settings() -> RecorderSupportedSettings {
    RecorderSupportedSettings{
        fileFormat: to_model_rc_string_vec(RecorderSupportedFileFormat::to_vec()),
        fps: to_model_rc_string_vec(RecorderSupportedFps::to_vec()),
        resolution: to_model_rc_string_vec(RecorderSupportedResolution::to_vec()),
    }
}

/// Defines the structure for the recorder settings, which includes the path, file format, FPS, and resolution.
/// This struct is used for serialization and deserialization of the settings to and from a JSON file.

#[derive(Serialize, Deserialize, Debug)]
pub struct RecorderSettings{
    pub path: String,
    #[serde(rename = "fileFormat")]
    pub file_format: String,
    pub fps: String,
    pub resolution: String,
}

/// Defines the path to the settings file where the recorder settings are stored in JSON format.
const SETTINGS_FILE: &str = "./appsettings/appsettings.json";

/// Reads the recorder settings from the JSON file and returns a Result containing either the RecorderSettings or an error.
fn read_from_file() -> Result<RecorderSettings, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(SETTINGS_FILE)?;
    let json_data: serde_json::Value = from_str(&data)?;
    let recorded_settings = &json_data["RecordedSettings"];
    let recorder_settings: RecorderSettings = serde_json::from_value(recorded_settings.clone())?;
    Ok(recorder_settings)
}

/// Loads the recorder settings from the JSON file. If the file does not exist or is invalid, default settings are returned.
/// Returns a RecorderSettings struct containing the loaded or default settings.
pub fn load_settings() -> RecorderSettings {
    let mut default_settings: RecorderSettings = RecorderSettings {
        path: "./recordings".to_string(),
        file_format: "mp4".to_string(),
        fps: "60".to_string(),
        resolution: "FullHD 1080p".to_string(),
    };
    match read_from_file() {
        Ok(settings) => {
            if !settings.path.is_empty(){
                default_settings.path=settings.path;
            }
            if !settings.file_format.is_empty() && RecorderSupportedFileFormat::to_vec().contains(&settings.file_format) {
                default_settings.file_format= settings.file_format;
            }
            if !settings.fps.is_empty() && RecorderSupportedFps::to_vec().contains(&settings.fps) {
                default_settings.fps = settings.fps;
            }
            if !settings.resolution.is_empty() && RecorderSupportedResolution::to_vec().contains(&settings.resolution) {
                default_settings.resolution = settings.resolution;
            }
        },
        Err(_) => return default_settings,
    };
    default_settings
}

/// Writes the recorder settings to the JSON file. If the file does not exist, it will be created.
/// Arguments
/// * `path` - The path where the recordings will be saved.
/// * `file_format` - The file format for the recordings (e.g., "mp4").
/// * `fps` - The frames per second for the recordings (e.g., "60 ").
/// * `resolution` - The resolution for the recordings (e.g., "FullHD 1080p").
/// Returns a Result indicating success or failure of the write operation.
pub fn write_to_file(path: &str, file_format: &str, fps: &str, resolution: &str) -> Result<(), Box<dyn std::error::Error>> {
    let recorder_settings = RecorderSettings {
        path: path.to_string(),
        file_format: file_format.to_string(),
        fps: fps.to_string(),
        resolution: resolution.to_string(),
    };
    let json_data = serde_json::json!({
        "RecordedSettings": recorder_settings
    });
    let json_string = to_string_pretty(&json_data)?;
    fs::write(SETTINGS_FILE, json_string)?;
    Ok(())
}