use std::{
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering
        },
        mpsc::{
            SyncSender,
            Receiver
        }
    },
    thread
};

use crate::{
    RecordingSource,
    recorder_controller::{
        recorder_supported_options::{
            Fps,
            Resolution
        },
        recording_session::{
            EncoderStatus,
            EncoderType,
            muxing_pipeline::
                muxer::Muxer
        }
    }
};

use super::{
    video_encoder::VideoEncoder,
    video_recorder::{
        self,
        VideoCapturer
    },
};

#[allow(dead_code)]
pub enum VideoRecorderCommand {
    Stop,
    UpdateVideoSources(Vec<RecordingSource>),
}

/// Spawns a new thread for video recording and encoding.
///
/// Arguments
/// * `fps` - The frames per second for the video recording.
/// * `target_resolution` - The target resolution for the encoded video.
/// * `video_sources` - A vector of video sources to capture.
/// * `muxer` - An Arc<Mutex<Muxer>> for thread-safe access to the muxer.
/// * `recording_state` - An Arc<AtomicBool> to control the recording state
/// * `initialization_transmitter` - A SyncSender<EncoderStatus> to send the initialization status of the encoder.
/// * `commands_receiver` - A Receiver<VideoRecorderCommand> to receive commands for the video recorder.
pub fn spawn(
    fps: Fps,
    target_resolution: Resolution,
    video_sources: Vec<RecordingSource>,
    muxer: Arc<Mutex<Muxer>>,
    recording_state: Arc<AtomicBool>,
    initialization_transmitter: SyncSender<EncoderStatus>,
    commands_receiver: Receiver<VideoRecorderCommand>,
) -> thread::JoinHandle<()> {

    let video_recorder_thread = thread::Builder::new()
        .name("video capture".to_string())
        .spawn(move || {
            let mut video_encoder
                = VideoEncoder::new(
                    muxer.clone(),
                    &target_resolution,
                    &fps
                ).unwrap();

            #[cfg(feature = "debug-info")]
            println!("Video encoder initialized.");

            initialization_transmitter.send(EncoderStatus::Initialized(EncoderType::Video)).unwrap();
            drop(initialization_transmitter);

            let (mut video_recorder, mut recorder_resolution) =
                start_recording(&mut video_encoder, video_sources, &fps);

            while recording_state.load(Ordering::SeqCst) {
                while let Ok(command) = commands_receiver.try_recv() {
                    match command {
                        VideoRecorderCommand::Stop => {
                            break;
                        }
                        VideoRecorderCommand::UpdateVideoSources(new_video_sources) => {
                            video_recorder.stop_recorder();
                            (video_recorder, recorder_resolution) =
                                start_recording(&mut video_encoder, new_video_sources, &fps);
                        }
                    }
                }

                match video_recorder.capture_frame() {
                    Ok(frame) => {
                        if captured_window_size_changed(frame.width, frame.height, &recorder_resolution){
                            recorder_resolution = Resolution::from(frame.width as u32, frame.height as u32);

                            #[cfg(feature = "debug-info")]
                            println!("updating frame scaling to: {}x{}", recorder_resolution.width, recorder_resolution.height);

                            video_encoder.set_frame_scaling(&recorder_resolution).unwrap();
                        }

                        video_encoder.encode_frame(
                            &frame
                        ).unwrap();

                    }
                    Err(e) => {
                        eprintln!("Error capturing video frame: {}", e);
                    }
                }
            }

            video_recorder.stop_recorder();
            video_encoder.finalize_video_encoding().unwrap();

        }).unwrap();


    video_recorder_thread
}

/// Starts the video recording process by initializing the video recorder and setting up the encoder.
///
/// Arguments
/// * `video_encoder` - A mutable reference to the VideoEncoder.
/// * `targets` - A vector of RecordingSource to capture.
/// * `fps` - A reference to the Fps struct representing the frames per second for the recording.
/// Returns
/// A tuple containing the initialized VideoCapturer and the current Resolution of the recorder.
fn start_recording(
    video_encoder: &mut VideoEncoder,
    targets: Vec<RecordingSource>,
    fps: &Fps
) -> (VideoCapturer, Resolution){
    let target = video_recorder::get_target_from(targets);
    let mut video_recorder = video_recorder::VideoCapturer::initialize_recorder(fps.value, target).unwrap();
    let recorder_resolution  = video_recorder.get_frame_size();
    video_encoder.set_frame_scaling(&recorder_resolution).unwrap();
    video_recorder.start_recorder();
    (video_recorder, recorder_resolution)
}

/// Checks if the captured window size has changed compared to the current resolution.
///
/// Arguments
/// * `width` - The current width of the captured window.
/// * `height` - The current height of the captured window.
/// * `resolution` - A reference to the current Resolution of the recorder.
/// Returns
/// true if the captured window size has changed, false otherwise.
fn captured_window_size_changed(width: i32, height: i32, resolution: &Resolution) -> bool {
    width as u32 != resolution.width || height as u32 != resolution.height
}