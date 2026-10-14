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
        }
    },
    thread,
    time,
};

use cpal::traits::DeviceTrait;
#[cfg(feature = "audio-thread-debug-info")]
use cpal::traits::DeviceTrait;
//use ffmpeg_next::format::sample;

use crate::{
    RecordingSource,
    recorder_controller::recording_session::{
        EncoderStatus,
        EncoderType,
        muxing_pipeline::{
            muxer::Muxer
        }
    }
};

use ringbuf::{
    HeapCons, traits::{Consumer, Observer},
};

use super::{
    audio_encoder::AudioEncoder,
    audio_recorder::AudioRecorder,
    resampler::{
        AudioResamplingPipelines,
        SamplesBuffer,
    },
    mixer::Mixer,
};

const TICK_NS: u64 = 21_333_333; //approx 1024 samples at 48kHz

#[allow(dead_code)]
fn drain_late_samples(
    consumer: &mut HeapCons<f32>,
    offset_ptr: usize,
){
    #[cfg(feature = "audio-thread-debug-info")]{
        println!("Draining outdated samples...");
        println!("Draining {} late samples.", offset_ptr);

    println!("Draining {} late samples, consumer length: {}", offset_ptr, consumer.occupied_len());
    }
    for _ in 0..offset_ptr {
        consumer.try_pop();

    }
}
/// Pull samples from the consumer into the buffer, filling with silence if needed
/// # Arguments
/// * `consumer` - The audio samples consumer
/// * `buffer` - The buffer to fill with samples
/// Returns nothing, the buffer is filled in place
/// `buffer.samples_offset_ptr` is updated to reflect the number of samples that were not filled
/// with actual samples (i.e., filled with silence)
/// This function first drains any late samples from the consumer based on the offset pointer.
/// Then, it attempts to pull samples from the consumer into the buffer. If there are not enough samples,
/// it fills the remaining space in the buffer with silence (0.0f32).
fn pull_audio_samples(
    consumer: &mut HeapCons<f32>,
    buffer: &mut SamplesBuffer,
){
    #[cfg(feature = "audio-thread-debug-info")]{
        println!("audio buffer is empty? {}", consumer.is_empty());
        println!("number of audio samples in ring buffer {:?}", consumer.occupied_len());
    }

    //drain_late_samples(consumer, buffer.samples_offset_ptr);
    if consumer.is_empty() {
        buffer.samples.fill(0f32);
        buffer.samples_offset_ptr = 0;
    } else {
        #[cfg(feature = "deprecated")]{
            let mut pulled = 0;
            for i in 0..buffer.buffer_size {
                match consumer.try_pop(){
                    Some(sample) => {
                        buffer.samples[i] = sample;
                        pulled += 1;
                    },
                    None => {
                        buffer.samples[i] = 0f32; // fill silence
                    },
                }
            }
            buffer.samples_offset_ptr = buffer.buffer_size - pulled;
        }

        let pulled = consumer.pop_slice(&mut buffer.samples);
        buffer.samples_offset_ptr = buffer.buffer_size - pulled;
        buffer.samples[pulled..].fill(0f32);

        #[cfg(feature = "audio-thread-debug-info")]
        println!("Pulled {} samples (total {}/{})", pulled, total_pulled, expected_samples);
    }
}

#[cfg(feature = "deprecated")]
/// Waits until the next audio clock tick
/// # Arguments
/// * `next_tick` - The next tick instant to wait for
/// This function simply parks the thread until the next tick instant is reached.
/// This is a basic implementation and may not be as accurate as desired for audio timing.
fn await_for_audio_clock(next_tick: &mut time::Instant) {
    *next_tick += time::Duration::from_nanos(TICK_NS);

    let now = time::Instant::now();
    if *next_tick > now {
        thread::park_timeout(*next_tick - now);
    }
}

const SPIN_TRESHOLD_US: time::Duration = time::Duration::from_micros(200);
const SLEEP_EROR_US: time::Duration = time::Duration::from_micros(100);

/// Advances audio_thread timeline
/// Waits until the next audio clock tick
/// # Arguments
/// * `next_tick` - The next tick instant to wait for
///
/// This function uses a combination of sleeping and spin-waiting to efficiently wait until the next tick.
/// It sleeps if the wait time is longer than SPIN_TRESHOLD_US, otherwise it spin-waits.
/// This approach helps to reduce CPU usage while maintaining accurate timing.
fn await_for_audio_clock(next_tick: &mut time::Instant) {
    *next_tick += time::Duration::from_nanos(TICK_NS);

    loop{
        let now = time::Instant::now();
        if now >= *next_tick {
            break;
        }
        let time_to_wait = *next_tick - now;
        if time_to_wait > SPIN_TRESHOLD_US {
            thread::sleep(time_to_wait - SLEEP_EROR_US);
        } else {
            std::hint::spin_loop();
        }
    }
}

/// Spawns a new thread for audio capture, encoding, and mixing.
///
/// Arguments
/// * `audio_sources` - A vector of RecordingSource to capture audio from.
/// * `muxer` - An Arc<Mutex<Muxer>> for thread-safe access to the muxer.
/// * `recording_state` - An Arc<AtomicBool> to control the recording state
/// * `initialization_transmitter` - A SyncSender<EncoderStatus> to send the initialization status of the encoder.
pub fn spawn(
    audio_sources: Vec<RecordingSource>,
    muxer: Arc<Mutex<Muxer>>,
    recording_state: Arc<AtomicBool>,
    initialization_transmitter: SyncSender<EncoderStatus>,
) -> thread::JoinHandle<()> {

    let audio_capture_thread = thread::Builder::new()
        .name("audio capture".to_string())
        .spawn(move || {
            let active_targets = AudioRecorder::get_devices_from_model(audio_sources);
            let mut audio_gains = Vec::new();
            active_targets.iter().for_each(|device| {
                if device.supports_output(){
                    audio_gains.push(0.3f32); //reduce gain for output devices
                }
                audio_gains.push(1.0f32); //default unity gain
            });

            let target_encoder_sample_rate: u32 = 48_000; //was 44100
            let encoder_channels_nb: usize = 2;
            let mut audio_encoder
                = AudioEncoder::new(
                    muxer.clone(),
                    target_encoder_sample_rate as i32,
                    encoder_channels_nb as i32
                ).unwrap();

            #[cfg(feature = "audio-thread-debug-info")]
            println!("Audio encoder initialized.");

            initialization_transmitter.send(EncoderStatus::Initialized(EncoderType::Audio)).unwrap();
            drop(initialization_transmitter);

            let mut audio_recorders: Vec<AudioRecorder> = vec![];
            let mut audio_resampling_pipelines = AudioResamplingPipelines::new();
            let mut mixer = Mixer::new(
                audio_encoder.sample_rate(),
                audio_encoder.channel_layout(),
                audio_encoder.format(),
                audio_encoder.required_samples_per_channel(),
            );

            active_targets.into_iter().enumerate().for_each(|(index, device)| {
                #[cfg(feature = "audio-thread-debug-info")]
                println!("Using audio device: {:?}", device.name());

                let mut recorder = AudioRecorder::initialize_recorder(device, audio_gains[index]);
                audio_resampling_pipelines.initialize_resampling_pipeline_for(
                    audio_recorders.len(),
                    recorder.sample_rate,
                    recorder.channels,
                    audio_encoder.required_samples_per_channel(),
                    audio_encoder.sample_rate(),
                    audio_encoder.channel_layout(),
                    audio_encoder.format(),
                ).unwrap();
                mixer.initialize(
                    audio_recorders.len(),
                    audio_encoder.channel_layout(),
                );
                recorder.start();
                audio_recorders.push(recorder);

            });

            let mut next_tick = time::Instant::now();

            while recording_state.load(Ordering::SeqCst) {
                await_for_audio_clock(&mut next_tick);

                audio_recorders.iter_mut().enumerate().for_each(|(recorder_index, recorder)|{
                    pull_audio_samples(
                        &mut recorder.consumer,
                        audio_resampling_pipelines.mutable_buffer_for(recorder_index)
                    );

                    mixer.extend_buffer(
                        recorder_index,
                        audio_resampling_pipelines.resample(
                            recorder_index,
                        ),
                    );

                });
                while let Some((mixed_frame, next_frame_pts_offset)) = mixer.mix(){
                    audio_encoder.encode_mixed_frame(mixed_frame, next_frame_pts_offset).unwrap();
                }

            }
            #[cfg(feature = "audio-thread-debug-info")]
            print!("Stopping audio capturer...");

            audio_encoder.finalize_audio_encoding().unwrap();
            drop(audio_recorders);

            #[cfg(feature = "audio-thread-debug-info")]
            println!(" Audio capturer stopped.");
        }).unwrap();
    audio_capture_thread
    }
