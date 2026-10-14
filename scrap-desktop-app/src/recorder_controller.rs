
use std::{
    path::PathBuf,
};


use std::{
    sync::{
        Arc,
        atomic::{
            AtomicBool,
        }
    },
    thread,
};
use crossbeam_channel::{Sender, Receiver};


use super::{
    RecordingSource,
    ModelRc,
};

mod recording_session;
use recording_session::{
    RecordingSession,
    audio_pipeline::audio_recorder,
    video_pipeline::video_recorder,
};

pub mod recorder_supported_options;
use recorder_supported_options::{Resolution, Fps, RecorderSupportedFileFormat};


#[derive(Clone)]
#[allow(dead_code)]
pub enum ControlCommand {
    StartRecording{
        audio_sources: Vec<RecordingSource>,
        video_sources: Vec<RecordingSource>,
        output_path: PathBuf,
        fps: Fps,
        resolution: Resolution,
        recording_format: RecorderSupportedFileFormat,
    },
    UpdateAudioSources{
        audio_sources: Vec<RecordingSource>,
    },
    UpdateVideoSources{
        video_sources: Vec<RecordingSource>,
    },
    StopRecording,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct RecorderController{
    transmitter: Sender<ControlCommand>,
    recording_state: Arc<AtomicBool>,
}

impl RecorderController{
    /// Creates a new instance of the RecorderController.
    /// Returns a RecorderController struct that can be used to control the recording session.
    /// This function initializes the communication channels and spawns a separate thread to handle recording commands.
    pub fn new() -> Self{
        let (tx, rx): (Sender<ControlCommand>, Receiver<ControlCommand>) = crossbeam_channel::unbounded();
        let recording_state = Arc::new(AtomicBool::new(false));

        let controller = RecorderControllerCore {
            rx,
            recording_state: recording_state.clone(),
        };

        thread::spawn(move || {
            controller.run();
        });

        Self{
            transmitter: tx,
            recording_state,
        }
    }

    /// Sends a control command to the RecorderController.
    /// Arguments
    /// * `command` - A ControlCommand enum variant representing the command to be sent.
    pub fn send(&self, command: ControlCommand){
        self.transmitter.send(command).unwrap();
    }

    /// Fetch available audio and video sources
    /// Returns a tuple of vectors: (audio_sources, video_sources)
    pub fn get_available_sources() -> (ModelRc<RecordingSource>, ModelRc<RecordingSource>){
        let audio_sources = audio_recorder::display_available_audio_devices();
        let video_sources = video_recorder::display_available_video_targets();

        (audio_sources, video_sources.0)
    }
}


struct RecorderControllerCore{
    rx: Receiver<ControlCommand>,
    recording_state: Arc<AtomicBool>,
}

impl RecorderControllerCore{
    /// Runs the main loop of the RecorderControllerCore, handling incoming control commands.
    /// This function listens for commands sent to the RecorderController and manages the recording session accordingly.
    /// It handles starting, updating, and stopping the recording session based on the received commands.
    pub fn run(self){
        let mut session: Option<RecordingSession> = None;

        while let Ok(command) = self.rx.recv(){
            match command{
                ControlCommand::StartRecording { audio_sources, video_sources, output_path, fps, resolution, recording_format }=>{
                    if session.is_none(){
                        session = Some(
                            RecordingSession::start(
                                audio_sources,
                                video_sources,
                                output_path,
                                fps,
                                resolution,
                                recording_format,
                                self.recording_state.clone()));
                    }
                },
                ControlCommand::UpdateAudioSources{audio_sources}=>{
                    if let Some(session) = session.as_mut(){
                        session.update_audio_sources(audio_sources);
                    }
                },
                ControlCommand::UpdateVideoSources{video_sources}=>{
                    if let Some(session) = session.as_mut(){
                        session.update_video_sources(video_sources);
                    }
                },
                ControlCommand::StopRecording=>{
                    if let Some(mut session) = session.take(){
                        session.stop();
                    }
                },
            }
        }

    }
}
