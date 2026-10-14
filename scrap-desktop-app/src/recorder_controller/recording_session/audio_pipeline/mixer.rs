use std::{
    cmp::max,
    collections::{VecDeque},
};

use ffmpeg_next::{
    ChannelLayout,
    frame,
    format,
};
use slint::Model;

use crate::
    recorder_controller::
        recording_session::
            audio_pipeline::
                resampler::
                    ResampledFrame;

pub struct AudioBuffer{
    samples: Vec<Vec<f32>>,
    recorder_index: usize,
    next_frame_pts_offset: VecDeque<usize>,
}

pub struct Mixer{
    buffers: Vec<AudioBuffer>,
    buffered_samples: Vec<u64>,
    mixed_frame: frame::Audio,
    required_samples: usize,
    mixed_buffer: Vec<Vec<f32>>,
}

impl Mixer{
    /// Creates a new Mixer instance with the specified parameters.
    ///
    /// Arguments
    /// * `target_sample_rate` - The sample rate of the target audio.
    /// * `target_channel_layout` - The channel layout of the target audio.
    /// * `target_format` - The sample format of the target audio.
    /// * `target_required_samples_per_channel` - The number of samples required per channel for mixing.
    /// Returns
    /// A Mixer instance.
    pub fn new(
        target_sample_rate: u32,
        target_channel_layout: ChannelLayout,
        target_format: format::Sample,
        target_required_samples_per_channel: usize,
    ) -> Self{
        let mut mixed_frame = frame::Audio::new(
                target_format,
                target_required_samples_per_channel,
                target_channel_layout,
            );
        mixed_frame.set_rate(target_sample_rate);

        Self{
            buffers: Vec::new(),
            buffered_samples: Vec::new(),
            mixed_frame,
            required_samples: target_required_samples_per_channel,
            mixed_buffer: vec![vec![0f32; target_required_samples_per_channel]; target_channel_layout.channels() as usize],
        }
    }

    /// Initializes the mixer for a new audio recorder.
    /// # Arguments
    /// * `recorder_index` - The index of the recorder to initialize the mixer for.
    /// * `encoder_channel_layout` - The channel layout of the audio encoder.
    pub fn initialize(
        &mut self,
        recorder_index: usize,
        encoder_channel_layout: ChannelLayout,
    )
    {
        self.buffered_samples.push(0);
        self.buffers.push(
            AudioBuffer{
                samples: vec![Vec::new(); encoder_channel_layout.channels() as usize],
                recorder_index,
                next_frame_pts_offset: VecDeque::new(),
            }
        );
    }

    /// Extends the buffer for the given recorder index with the resampled audio frame.
    /// # Arguments
    /// * `recorder_index` - The index of the recorder this frame is associated with
    /// * `resampled_frame` - The resampled audio frame to be added to the buffer
    pub fn extend_buffer(
        &mut self,
        recorder_index: usize,
        resampled_frame: ResampledFrame,
    ){
        let buffer = self.buffers
            .iter_mut()
            .find_map(|buffer|
                buffer
                    .recorder_index
                    .eq(&recorder_index)
                    .then_some(buffer)
                )
            .unwrap();

        self.buffered_samples[recorder_index] += resampled_frame.frame.samples() as u64;

        for plane in 0..resampled_frame.frame.planes(){
            let channel_data = resampled_frame.frame.plane::<f32>(plane);
            buffer.samples[plane].extend_from_slice(channel_data);
        }

        buffer.next_frame_pts_offset.push_back(resampled_frame.next_frame_pts_offset);
        #[cfg( feature = "debug-info")]
        for channel in 0..resampled_frame.channels() as usize{
            println!("Mixer buffer recorder index {} channel {} has {} samples.", recorder_index, channel, buffer.samples[channel].len());
        }
    }

    /// Mixes audio frames from all buffers if enough samples are available.
    /// Returns a mixed audio frame and the maximum pts offset among the input frames.
    pub fn mix(&mut self)->Option<(&mut frame::Audio, usize)>{
        let mut peak = 0.0f32;
        let mut pts_offset: usize = 0;
        #[cfg(not(feature = "deprecated"))]
        if self.buffered_samples
                .iter()
                .all(|buffered_samples|
                    *buffered_samples >= self.required_samples as u64
                )
        {
            //we have enough samples from all inputs to mix a frame
            self.buffers
                .iter_mut()
                .for_each(|buffer|{
                    for channel in 0..buffer.samples.len(){
                        let samples_to_mix = &buffer.samples[channel][0..self.required_samples];
                        self.mixed_buffer[channel]
                            .iter_mut()
                            .enumerate()
                            .for_each(|(index, sample)|{
                                *sample += samples_to_mix[index];
                                peak = peak.max(sample.abs());
                            });

                        //remove used samples from buffer
                        buffer.samples[channel].drain(0..self.required_samples);

                        //normalize mixed samples
                        if peak > 1.0{
                            #[cfg(feature = "debug-info")]
                            println!("Normalizing audio with peak {}", peak);
                            self.mixed_buffer[channel]
                                .iter_mut()
                                .for_each(|sample|{
                                    *sample *=  1.0 / peak;
                                });
                        }
                    }
                    pts_offset = max(pts_offset, buffer.next_frame_pts_offset.pop_front().unwrap());
                    peak = 0.0;
            });

            //decrease buffered samples count
            for i in 0..self.buffered_samples.len(){
                self.buffered_samples[i] -= self.required_samples as u64;
            }

            //write mixed samples to frame
            self.mixed_frame
                .planes()
                .iter()
                .enumerate()
                .for_each(|(channel_index, _)|{
                    let channel_data = self.mixed_frame.plane_mut::<f32>(channel_index);
                    channel_data.copy_from_slice(&self.mixed_buffer[channel_index]);
                    //clear mixed buffer for next frame
                    self.mixed_buffer[channel_index].iter_mut().for_each(|sample| *sample = 0.0);
                }
            );
            return Some((&mut self.mixed_frame, pts_offset));
        }
        None
    }
}