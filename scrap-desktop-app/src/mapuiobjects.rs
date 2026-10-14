use super::{
    recorder_controller::recorder_supported_options::{
        RecorderSupportedFileFormat,
        RecorderSupportedFps,
        Fps,
        RecorderSupportedResolution,
        Resolution
    },
    Model,
    ModelRc,
    RecordingSource,
    VecModel
};

/// Maps a string representation of a recording resolution to the corresponding Resolution enum value.
/// Arguments
/// * `resolution` - A string representing the desired recording resolution (e.g., "HD 720p", "FullHD 1080p", "QHD 1440p").
/// Returns
/// Resolution - The corresponding Resolution enum value for the given string representation.
pub fn map_recording_resolution(resolution: &str) -> Resolution{
    let resolution = match resolution{
        "HD 720p" => RecorderSupportedResolution::_720p,
        "FullHD 1080p" => RecorderSupportedResolution::_1080p,
        "QHD 1440p" => RecorderSupportedResolution::_1440p,
        _ => RecorderSupportedResolution::_1080p, //default
    };
    resolution.value()
}

/// Maps a string representation of a recording FPS (frames per second) to the corresponding Fps enum value.
/// Arguments
/// * `fps` - A string representing the desired recording FPS (e.g., "30", "60", "120").
/// Returns
/// Fps - The corresponding Fps enum value for the given string representation.
pub fn map_recording_fps(fps: &str) -> Fps{
    let fps = match fps{
        #[cfg(feature = "experimental")]
        "30" => RecorderSupportedFps::_30,
        "60" => RecorderSupportedFps::_60,
        #[cfg(feature = "experimental")]
        "120" => RecorderSupportedFps::_120,
        _ => RecorderSupportedFps::_60, //default
    };
    fps.value()
}

/// Maps a string representation of a recording file format to the corresponding RecorderSupportedFileFormat enum value.
/// Arguments
/// * `file_format` - A string representing the desired recording file format (e.g., "mp4").
/// Returns
/// RecorderSupportedFileFormat - The corresponding RecorderSupportedFileFormat enum value for the given string representation.
pub fn map_recording_file_format(file_format: &str) -> RecorderSupportedFileFormat{
    match file_format{
        "mp4" => RecorderSupportedFileFormat::MP4,
        _ => RecorderSupportedFileFormat::MP4, //default
    }
}

/// Retrieves the active recording targets from a ModelRc<RecordingSource>.
/// Arguments
/// * `targets` - A ModelRc<RecordingSource> containing the recording targets.
/// Returns
/// Vec<RecordingSource> - A vector of active RecordingSource instances.
/// This function filters the provided ModelRc<RecordingSource> to return only those targets that are marked as active (isActive == true).
pub fn get_recording_targets(targets: ModelRc<RecordingSource>) -> Vec<RecordingSource> {
    let target_list = targets.as_any().downcast_ref::<VecModel<RecordingSource>>().expect("failed to downcast to vecmodel");
    let mut active_targets: Vec<RecordingSource> = vec![];
    for target in target_list.iter(){
        if target.isActive {
            active_targets.push(target);
        }
    }
    active_targets
}