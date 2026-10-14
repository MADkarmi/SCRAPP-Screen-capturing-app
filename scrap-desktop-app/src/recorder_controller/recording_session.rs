
use chrono::{
    Local,
    Datelike,
    Timelike
};

use std::{
    path::PathBuf,
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering
        },
        mpsc::{
            self,
            Sender,
        }
    }, thread
};

use super::{
    RecordingSource,
    recorder_supported_options::{
        RecorderSupportedFileFormat,
        Fps,
        Resolution},
};

pub mod audio_pipeline;
use audio_pipeline::audio_thread;

pub mod video_pipeline;
use video_pipeline::video_thread::{self, VideoRecorderCommand};

pub mod muxing_pipeline;
use muxing_pipeline::{muxer_thread, muxer::Muxer};

#[derive(Debug)]
pub enum EncoderStatus{
    Initialized(EncoderType),
}
#[derive(Debug)]
pub enum EncoderType{
    Audio,
    Video,
}

#[allow(dead_code)]
const AUDIO_SAMPLE_RATE: u32 = 48000;
#[allow(dead_code)]
const AUDIO_CHANNELS: u16 = 2;

pub struct RecordingSession{
    muxer: Arc<Mutex<Muxer>>,
    active_threads: Vec<thread::JoinHandle<()>>,
    recording_state: Arc<AtomicBool>,
    video_command_transmitter: Sender<VideoRecorderCommand>,
}

impl RecordingSession{
    /// Starts a new recording session with the specified parameters.
    ///
    /// Arguments
    /// * `audio_sources` - A vector of audio sources to capture.
    /// * `video_sources` - A vector of video sources to capture.
    /// * `output_folder_path` - The path to the folder where the recording will be saved.
    /// * `fps` - The frames per second for the video recording.
    /// * `resolution` - The target resolution for the encoded video.
    /// * `recording_format` - The file format for the recording which is instantiated from the RecorderSupportedFileFormat enum.
    /// * `recording_state` - An Arc<AtomicBool> to control the recording state.
    pub fn start(
        audio_sources: Vec<RecordingSource>,
        video_sources: Vec<RecordingSource>,
        output_folder_path: PathBuf,
        fps: Fps,
        resolution: Resolution,
        recording_format: RecorderSupportedFileFormat,
        recording_state: Arc<AtomicBool>) -> Self{
            recording_state.store(true, Ordering::SeqCst);

            let muxer = Muxer::new_context(
                output_folder_path.join(
                    RecordingSession::generate_file_name(recording_format)
                ).as_path())
                .unwrap();
            let (encoder_status_transmitter,
                encoder_status_receiver) = mpsc::sync_channel::<EncoderStatus>(1);

            let muxer = Arc::new(Mutex::new(muxer));

            let mut active_threads: Vec<thread::JoinHandle<()>> = vec![];

            if !audio_sources.is_empty(){
                let audio_recording_thread = audio_thread::spawn(
                    audio_sources,
                    muxer.clone(),
                    recording_state.clone(),
                    encoder_status_transmitter.clone()
                );
                active_threads.push(audio_recording_thread);
            }

            let (video_commands_tx,
                video_commands_rx) = mpsc::channel::<VideoRecorderCommand>();

            let video_recording_thread = video_thread::spawn(
                fps,
                resolution,
                video_sources,
                muxer.clone(),
                recording_state.clone(),
                encoder_status_transmitter.clone(),
                video_commands_rx
            );



            active_threads.push(video_recording_thread);

            let mut encoders_ready: Vec<EncoderType> = Vec::new();
            while encoders_ready.len() < active_threads.len() {
                while let Ok(status) = encoder_status_receiver.try_recv() {
                    match status {
                        EncoderStatus::Initialized(encoder_type) => {
                            encoders_ready.push(encoder_type);
                        }
                    }
                }
            }
            {
                #[cfg(feature = "recording-session-debug-info")]
                println!("All encoders initialized, starting muxer...");
                let _ = muxer_thread::spawn(
                    muxer.clone(),
                );
                //active_threads.push(muxer_thread);
                //muxer.lock().unwrap().initialize().unwrap();
            }
        Self {
            muxer: muxer.clone(),
            active_threads,
            recording_state,
            video_command_transmitter: video_commands_tx,
        }
    }

    pub fn update_audio_sources(&mut self, _audio_sources: Vec<RecordingSource>){
        //update audio sources in the pipelines
    }

    /// Updates the video sources for the recording session.
    ///
    /// Arguments
    /// * `video_sources` - A vector of new video sources to capture.
    pub fn update_video_sources(&mut self, video_sources: Vec<RecordingSource>){
        self.video_command_transmitter.send(
            VideoRecorderCommand::UpdateVideoSources(video_sources)
        ).unwrap();
    }

    /// Stops the recording session and finalizes the muxer.
    pub fn stop(&mut self){
        self.recording_state.store(false, Ordering::SeqCst);

        for recording_thread in self.active_threads.drain(..){
            let _ = recording_thread.join();
        }
        self.muxer.lock().unwrap().finalize().unwrap();
    }

    /// Generates a file name for the recording based on the current date and time.
    ///
    /// Arguments
    /// * `format` - The file format for the recording.
    /// Returns
    /// A string representing the generated file name.
    fn generate_file_name(format: RecorderSupportedFileFormat) -> String {
        let now = Local::now();
        format!("{}-{:02}-{:02}-{:02}-{:02}-{:02}.{}",
                now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second(), format)
    }
}