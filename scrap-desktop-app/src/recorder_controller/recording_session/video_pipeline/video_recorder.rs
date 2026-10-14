use std::sync::{mpsc};

#[allow(unused_imports)]
use scap::{
    get_all_targets,
    capturer::CapturerBuildError,
    frame::{
        BGRAFrame,
        FrameType
    },
    Target::{
        self,
        Window,
        Display
    },
    capturer::{
        Capturer,
        Options,
        Area,
        Point,
        Size,
        Resolution
    },
    frame::{
        Frame,
        VideoFrame
    },
};

use std::{rc::Rc};
use crate::{
    RecordingSource,
    DeviceType,
    ModelRc,
    SharedString,
    VecModel,
    recorder_controller::recorder_supported_options::Resolution as RecorderResolution,
};

/// Check if the system supports screen capturing.
/// returns a tuple where the first element is a boolean indicating support status,
/// and the second element is an optional error message.
fn check_scap_support() -> (bool, Option<String>){
    if !scap::is_supported(){
        return (false, Some("Screen capturing is not supported on this platform.".to_string()));
    }
    (true, None)
}

/// Check if the application has permission to capture the screen.
/// If permission is not granted, it will attempt to request permission.
fn check_scap_permission() -> (bool, Option<String>){
    if !scap::has_permission(){
        //❌ Permission not granted. Requesting permission.."
         if !scap::request_permission() {
            return (false, Some("Permission denied".to_string()));
        }
    }
    (true, None)
}

/// Check if the system supports screen capturing and if the application has the necessary permissions.
/// returns a tuple where the first element is a boolean indicating support and permission status,
/// and the second element is an optional error message.
///
/// # Returns
/// A tuple containing a boolean indicating support and permission status, and an optional String error message.
pub fn has_support_and_permission() -> (bool, Option<String>){
    let (is_supported, support_error) = check_scap_support();
    if !is_supported {
        return (false, support_error);
    }
    let (has_permission, permission_error) = check_scap_permission();
    if !has_permission {
        return (false, permission_error);
    }
    (true, None)
}

/// Display available video targets (displays and windows) for screen capturing.
/// # Returns
/// A tuple where the first element is the model for the UI and the second element is an optional error message.
pub fn display_available_video_targets() -> (ModelRc<RecordingSource>, Option<String>) {
    let mut available_video_targets: Vec<RecordingSource> = vec![];

    let (has_support_and_permission, error) = has_support_and_permission();

    if has_support_and_permission{
        let displays = scap::get_all_targets();
        for (index, display) in displays.iter().enumerate(){
            match display {
                scap::Target::Display(display) => {
                    available_video_targets.push(RecordingSource{
                        title: SharedString::from(format!("Display: {}", index + 1)),
                        isActive: false,
                        deviceId: display.id as i32,
                        deviceType: DeviceType::MonitorOutput,
                    });
                    //println!("Display: {}, {}", display.title, display.id );
                },
                scap::Target::Window(window) => {
                    available_video_targets.push(RecordingSource{
                        title: SharedString::from(format!("{}", window.title)),
                        isActive: false,
                        deviceId: window.id as i32,
                        deviceType: DeviceType::WindowOutput,
                    });
                    //println!("Window: {}, {}", window.title, window.id);
                },
            }
        }
    }
    (ModelRc::from(Rc::new(VecModel::from(available_video_targets))), error)
}

/// Get video targets from a model of RecordingSource.
fn get_targets_from_model(targets: Vec<RecordingSource>) -> Vec<Target> {
        let available_targets = get_all_targets()
        .into_iter()
        .filter(|target| {
            match target {
                Display(display) => {
                    targets.iter().any(|t| t.deviceId as u32 == display.id)
                },
                Window(window) => {
                    targets.iter().any(|t| t.deviceId as u32 == window.id)
                },
            }
        })
        .collect::<Vec<Target>>();
    available_targets
}

/// Select the first target from a list of targets.
pub fn get_target_from(targets: Vec<RecordingSource>) -> Target {
    get_targets_from_model(targets).into_iter().next().unwrap()
}

pub struct VideoCapturer{
    pub recorder: Capturer,
}

impl VideoCapturer{
    #[allow(dead_code)]
    fn get_captured_frame_size(&mut self) -> [u32; 2] {
        let [width, height] = self.recorder.get_output_frame_size();
        [width, height]
    }

    /// Set the options for the video capturer based on the specified frames per second (fps) and target.
    ///
    /// # Arguments
    /// * `fps` - The frames per second for the video recording.
    /// * `target` - The target to capture (e.g., a specific window or display).
    fn set_capturer_options(fps: u32, target: scap::Target) -> Options {
        let options = Options {
            fps,
            target: Some(target),
            show_cursor: true,
            show_highlight: false,//true,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::Captured,
            crop_area: None,
            captures_audio: false,
            ..Options::default()
        };
        options
    }

    /// Initializes the video recorder with the specified frames per second (fps) and target.
    ///
    /// # Arguments
    /// * `fps` - The frames per second for the video recording.
    /// * `target` - The target to capture (e.g., a specific window or display).
    pub fn initialize_recorder(
        fps: u32,
        target: scap::Target)
        -> Result<VideoCapturer, CapturerBuildError> {
        let options = Self::set_capturer_options(fps, target);
        let recorder = Capturer::build(options)?;
        Ok(
            Self {
                recorder
            }
        )
    }

    /// Get the current frame size of the video recorder.
    pub fn get_frame_size(&mut self) -> RecorderResolution {
        let [width, height] = self.recorder.get_output_frame_size();
        RecorderResolution { width, height }
    }

    pub fn start_recorder(&mut self) {
        self.recorder.start_capture();
    }

    /// Capture a single video frame.
    /// to do:
    /// handle all possible frame types in encoder
    pub fn capture_frame(&mut self) -> Result<BGRAFrame, mpsc::RecvError> {
        match self.recorder.get_next_frame() {
            Ok(frame) =>{
                match frame{
                    Frame::Video(video_frame) => {
                        match video_frame{
                            VideoFrame::BGRA(bgra_frame) =>
                                 Ok(bgra_frame),
                                _ => Err(mpsc::RecvError),
                        }
                    },
                    _ => Err(mpsc::RecvError),
                }
            },
            Err(e) => Err(e),
        }
    }

    /// Stop the video recorder.
    pub fn stop_recorder(&mut self) {
        self.recorder.stop_capture();
    }

}