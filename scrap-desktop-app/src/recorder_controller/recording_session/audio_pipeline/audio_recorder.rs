use std::fmt::Debug;
#[allow(unused_imports)]
use std::{
    default,
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering
        },
        mpsc::{
            self,
            Receiver
        },
    },
    thread,
    time,
};

use slint::{
    ModelRc,
    SharedString,
    VecModel
};

use cpal::{
    BufferSize,
    Device,
    FromSample,
    Sample,
    SampleFormat,
    SizedSample,
    Stream,
    StreamConfig,
    traits::{
        DeviceTrait,
        HostTrait,
        StreamTrait
    }
};

use ringbuf::{
    HeapCons,
    HeapProd,
    HeapRb,
    traits::{*}
};

use crate::{
    RecordingSource,
    DeviceType
};

#[cfg(feature = "experimental")]
struct AudioGains {
    system: f32,
    microphone: f32,
}

/// Display available audio devices for recording.
/// Returns a model for the UI.
#[cfg(feature = "experimental")]
pub fn display_available_audio_devices() ->  ModelRc<RecordingSource>{
    let mut available_audio_devices: Vec<RecordingSource> = vec![];
    let host = cpal::default_host();
    let default_devices = vec![
        host.default_input_device().unwrap().name().unwrap(),
        host.default_output_device().unwrap().name().unwrap(),
    ];

    let devices: cpal::Devices = host.devices().unwrap();
    for (device_index, device) in devices.enumerate(){
        let device_name = device.name().unwrap();

        let recording_source = RecordingSource{
            title: SharedString::from(device.name().unwrap()),
            isActive: false,
            deviceId: device_index as i32,
            deviceType: if device.supports_input() { DeviceType::AudioInput } else { DeviceType::AudioOutput },
        };

        if default_devices.contains(&device_name){
            available_audio_devices.insert(0, recording_source);
        } else {
            available_audio_devices.push(recording_source);
        }
    }
    ModelRc::from(Rc::new(VecModel::from(available_audio_devices)))
}

/// Selects the first audio device from a list of RecordingSource.
/// Arguments
/// * `targets` - A vector of RecordingSource to select from.
/// Returns
/// A cpal::Device corresponding to the first active RecordingSource in the list.
#[cfg(not(feature = "experimental"))]
pub fn display_available_audio_devices() ->  ModelRc<RecordingSource>{
    let mut available_audio_devices: Vec<RecordingSource> = vec![];
    let host = cpal::default_host();
    if host.default_input_device().is_some(){
        available_audio_devices.push(
            RecordingSource{
                title: SharedString::from(host.default_input_device().unwrap().name().unwrap()),
                isActive: false,
                deviceId: 0,
                deviceType: DeviceType::AudioInput,
            }
        )
    }

    if host.default_output_device().is_some(){
        available_audio_devices.push(
            RecordingSource{
                title: SharedString::from(host.default_output_device().unwrap().name().unwrap()),
                isActive: false,
                deviceId: 1,
                deviceType: DeviceType::AudioOutput,
            }
        )
    }

    ModelRc::from(Rc::new(VecModel::from(available_audio_devices)))
}

pub struct AudioRecorder{
    stream: cpal::Stream,
    pub consumer: HeapCons<f32>,
    pub sample_rate: u32,
    pub channels: usize,
}

impl AudioRecorder {
    /// Gets the list of available audio devices for recording.
    /// Returns a vector of cpal::Device that are active and match the provided RecordingSources.
    pub fn get_devices_from_model(targets: Vec<RecordingSource>)  -> Vec<cpal::Device> {
        let host = cpal::default_host();

        let active_devices: Vec<(&str, u8)>  = targets.iter()
                                                                .filter(|t| t.isActive)
                                                                .map(|t| (t.title.as_str(), t.deviceType as u8)).collect();


        let devices: Vec<cpal::Device> = host.devices()
                            .unwrap()
                            .filter(|device| {
                                let device_name = device.name().unwrap();

                                let matched = active_devices.iter().any(|(name, device_type)| {
                                    *device_type == if device.supports_input() { DeviceType::AudioInput as u8 } else { DeviceType::AudioOutput as u8 } &&
                                    device_name == *name
                                });

                                matched
                            })
                            .collect();
        #[cfg(feature = "audio-thread-debug-info")]
        println!("devices found: {}", devices.len());
        devices
    }

    /// Starts audio capture for a given CPAL device.
    /// # Arguments
    /// * `device` - The CPAL device to capture audio from
    /// * `recording_state` - An Arc<AtomicBool> indicating whether recording is active
    /// Returns a tuple containing:
    /// * Receiver<Vec<f32>> - A channel receiver to get captured audio samples which are interleaved packed f32 samples [L R L R ...]
    /// * u32 - The sample rate of the captured audio
    /// * usize - The number of channels of the captured audio
    ///
    /// Spawns a CPAL audio input stream and sends captured audio samples through the channel.
    /// CPAL registers your callback with the OS audio backend (CoreAudio, WASAPI, ALSA, JACK, etc.).
    /// When calling stream.play(), the backend invokes your callback on a real-time audio thread.
    /// That thread is: Not controllable or joinable
    /// Your thread ──> build stream ──> OS audio thread ──> callback runs here
    pub fn initialize_recorder(
        device: Device,
        audio_gain: f32,
    ) -> Self {
        let config = if device.supports_input() {
            device.default_input_config().unwrap()
        } else {
            device.default_output_config().unwrap()
        };
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let ringbuffer = HeapRb::<f32>::new(sample_rate as usize * channels * 2); //2 second buffer at 48kHz
        let(producer, consumer) = ringbuffer.split();

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 =>
                build_stream::<f32>(&device, &stream_config, producer, audio_gain),
            SampleFormat::I16 =>
                build_stream::<i16>(&device, &stream_config, producer, audio_gain),
            SampleFormat::I32 =>
                build_stream::<i32>(&device, &stream_config, producer, audio_gain),
            SampleFormat::I64 =>
                build_stream::<i64>(&device, &stream_config, producer, audio_gain),
            SampleFormat::U8  =>
                build_stream::<u8>(&device, &stream_config, producer, audio_gain),
            SampleFormat::U16 =>
                build_stream::<u16>(&device, &stream_config, producer, audio_gain),
            SampleFormat::U32 =>
                build_stream::<u32>(&device, &stream_config, producer, audio_gain),
            SampleFormat::F64 =>
                build_stream::<f64>(&device, &stream_config, producer, audio_gain),
            _ => panic!("Unsupported sample format"),
        };

        Self {
            stream,
            consumer,
            sample_rate,
            channels,
        }
    }

    pub fn start(&mut self){
        self.stream.play().unwrap();
    }
}

/// Builds a CPAL audio stream for the given sample type T.
/// # Arguments
/// * `device` - The CPAL device to build the stream for
/// * `config` - The stream configuration
/// * `tx` - The channel sender to send captured samples
/// Returns a CPAL Stream.
/// This function is generic over the sample type T, which must implement the Sample, SizedSample, Copy traits,
/// and there must be a FromSample implementation to convert T to f32.
fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    mut producer: HeapProd<f32>,
    audio_gain: f32,
) -> Stream
where
    T: Sample + SizedSample + Debug, f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _info| {
                let converted: Vec<f32> =
                    data
                        .iter()
                        .map(|&s| s.to_sample::<f32>().mul_amp(audio_gain) )
                        .collect();
                #[cfg(feature = "audio-thread-debug-info")]
                if producer.is_full() {
                    println!("Ringbuffer is full");
                }
                let _ = producer.push_slice(&converted);
            },
            move |err| eprintln!("Stream error: {err:?}"),
            None,
        )
        .expect("Failed to build stream")
}
