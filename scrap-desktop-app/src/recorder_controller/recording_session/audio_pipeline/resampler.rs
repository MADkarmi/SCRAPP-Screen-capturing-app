use ffmpeg_next::{
    ChannelLayout,
    Error,
    format::{
        self,
        sample::{Type}
    },
    frame,
    software
};

pub struct ResampledFrame<'a>{
    pub frame: &'a frame::Audio,
    pub next_frame_pts_offset: usize,
}

pub struct SamplesBuffer{
    pub samples: Vec<f32>,
    pub buffer_size: usize,
    pub samples_offset_ptr: usize,
}

struct IndexedSamplesBuffer{
    recorder_index: usize,
    samples_buffer: SamplesBuffer,
}

pub struct AudioResamplingPipelines{
    pipelines: Vec<AudioResamplingPipeline>,
    sample_buffers: Vec<IndexedSamplesBuffer>,
}

impl AudioResamplingPipelines{
    pub fn new() -> Self{
        Self{
            pipelines: Vec::new(),
            sample_buffers: Vec::new(),
        }
    }

    /// Creates and initializes a new AudioResamplingPipeline for the given recorder index
    /// # Arguments
    /// * `recorder_index` - The index of the recorder this pipeline is associated with
    /// * `recorder_sample_rate` - The sample rate of the captured audio
    /// * `recorder_channels_nb` - The number of channels of the captured audio
    /// * `target_sample_rate` - The target sample rate for resampling
    /// * `target_channel_layout` - The target channel layout for resampling
    /// * `target_format` - The target sample format for resampling
    /// * `samples` - The captured audio samples which are interleaved packed f32 samples [L R L R ...]
    /// Returns a reference to the resampled audio frame
    /// Resamples interleaved packed f32 samples from input to target format, sample rate and channel layout.
    pub fn initialize_resampling_pipeline_for(
        &mut self,
        recorder_index: usize,
        recorder_sample_rate: u32,
        recorder_channels_nb: usize,
        target_required_samples_per_channel: usize,
        target_sample_rate: u32,
        target_channel_layout: ChannelLayout,
        target_format: format::Sample,
    ) -> Result<(), Error>{
        let pipeline = AudioResamplingPipeline::new(
            recorder_index,
            target_sample_rate,
            target_format,
            target_channel_layout,
            recorder_sample_rate,
            recorder_channels_nb,
            target_required_samples_per_channel,
        )?;

        let sample_buffer = IndexedSamplesBuffer{
            recorder_index,
            samples_buffer: SamplesBuffer{
                samples: vec![0f32; recorder_channels_nb * target_required_samples_per_channel],
                buffer_size: recorder_channels_nb * target_required_samples_per_channel,
                samples_offset_ptr: 0,
            },
        };

        self.pipelines.push(pipeline);
        self.sample_buffers.push(sample_buffer);
        Ok(())
    }

    /// Resamples interleaved packed f32 samples from input to target format, sample rate and channel layout.
    /// # Arguments
    /// * `recorder_index` - The index of the recorder this pipeline is associated with
    pub fn resample(
        &mut self,
        recorder_index: usize,
    ) -> ResampledFrame<'_> { // &frame::Audio{
        let pipeline = self.pipelines
            .iter_mut()
            .find_map(|pipeiline|
                pipeiline
                    .recorder_index()
                    .eq(&recorder_index)
                    .then_some(pipeiline)
                )
            .unwrap();

        #[cfg(feature = "deprecated")]
        let samples = self.sample_buffers
            .iter()
            .find_map(|buffer|
                buffer
                    .recorder_index
                    .eq(&recorder_index)
                    .then_some(&buffer.samples)
                )
            .unwrap();
        #[cfg(feature = "resampler-debug-info")]
        println!("Resampling {} samples for recorder {}, samples: {:?}", samples.len(), recorder_index, samples.iter().take(16).collect::<Vec<&f32>>());

        let (sample_buffer, offset) = self.sample_buffers
            .iter_mut()
            .find_map(|indexed_buffer|
                indexed_buffer
                    .recorder_index
                    .eq(&recorder_index)
                    .then_some((&indexed_buffer.samples_buffer.samples, indexed_buffer.samples_buffer.samples_offset_ptr))
                )
            .unwrap();

        ResampledFrame{
            frame: pipeline.resample(sample_buffer.as_slice()).unwrap(),
            next_frame_pts_offset: offset,
        }
    }

    /// Provides mutable access to the samples buffer for the given recorder index
    /// # Arguments
    /// * `recorder_index` - The index of the recorder this pipeline is associated with
    /// Returns a mutable reference to the SamplesBuffer
    pub fn mutable_buffer_for(
        &mut self,
        recorder_index: usize
    ) -> &mut SamplesBuffer{
        self.sample_buffers
            .iter_mut()
            .find_map(|indexed_buffer|
                indexed_buffer
                    .recorder_index
                    .eq(&recorder_index)
                    .then_some(&mut indexed_buffer.samples_buffer)
                )
            .unwrap()
    }
}

pub struct AudioResamplingPipeline{
    recorder_index: usize,
    needs_resampling: bool,
    resampled_frame: frame::Audio,
    input_frame: Option<frame::Audio>,
    resampler: Option<software::resampling::Context>,
    input_frame_pts: i64,
}

#[allow(dead_code)]
impl AudioResamplingPipeline{
    /// Creates a new AudioResamplingPipeline
    /// # Arguments
    /// * `recorder_index` - The index of the recorder this pipeline is associated with
    /// * `target_sample_rate` - The target sample rate for resampling
    /// * `target_format` - The target sample format for resampling
    /// * `target_channel_layout` - The target channel layout for resampling
    /// * `input_sample_rate` - The input sample rate of the captured audio
    /// * `input_channels_nb` - The number of channels of the captured audio
    /// * `required_samples_per_channel` - The length of the input samples slice
    /// Returns an AudioResamplingPipeline
    /// Resamples interleaved packed f32 samples from input to target format, sample rate and channel layout.
    pub fn new(
        recorder_index: usize,
        target_sample_rate: u32,
        target_format: format::Sample,
        target_channel_layout: ChannelLayout,
        input_sample_rate: u32,
        input_channels_nb: usize,
        required_samples_per_channel: usize,
    ) -> Result<Self, Error>{
        let mut input_frame = None;
        let mut resampler = None;

        let mut resampled_frame = frame::Audio::new(
            target_format,
            required_samples_per_channel,
            target_channel_layout
        );
        resampled_frame.set_rate(target_sample_rate);

        let needs_resampling =
            input_sample_rate != target_sample_rate as u32
            || input_channels_nb != target_channel_layout.channels() as usize
            || format::Sample::F32(Type::Packed) != target_format;

        if needs_resampling {
            let channel_layout = match input_channels_nb {
                1 => ChannelLayout::MONO,
                2 => ChannelLayout::STEREO,
                _ => ChannelLayout::SURROUND,
            };

            input_frame = Some(frame::Audio::new(
                format::Sample::F32(Type::Packed),
                required_samples_per_channel,
                channel_layout
            ));
            input_frame.as_mut().unwrap().set_rate(input_sample_rate);

            let input_resampler =
                input_frame.as_ref()
                .unwrap()
                .resampler(
                    target_format,
                    target_channel_layout,
                    target_sample_rate
                )?;

            resampler = Some(input_resampler);
        }

        Ok( Self{
            recorder_index,
            needs_resampling,
            resampled_frame,
            input_frame,
            resampler,
            input_frame_pts: 0,
        })
    }

    pub fn recorder_index(&self) -> usize {
        self.recorder_index
    }

    fn write(&mut self, samples: &[f32]) {
        let source_frame = if self.needs_resampling {
            self.input_frame.as_mut().unwrap()
        } else {
            &mut self.resampled_frame
        };
        #[cfg(feature = "resampler-debug-info")]
        println!("Writing {} samples to source frame", samples.len());

        if source_frame.is_packed(){
            source_frame
                .plane_mut::<f32>(0)
                .copy_from_slice(samples);
        } else {
            source_frame
                .data_mut(0)
                .copy_from_slice(
                    samples
                        .into_iter()
                        .map(|sample|Self::audio_f32_to_u8_byte(*sample))
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice()
                    );
        }

        let input_frame_pts = &mut self.input_frame_pts;
        source_frame.set_pts(Some(*input_frame_pts));
        *input_frame_pts += source_frame.samples() as i64;
    }

    /// Resamples interleaved packed f32 samples from input to target format, sample rate and channel layout.
    /// # Arguments
    /// * `samples` - The captured audio samples which are interleaved packed f32 samples [L R L R ...]
    /// Returns a reference to the resampled audio frame
    pub fn resample(
        &mut self,
        samples: &[f32]
    ) -> Result<&frame::Audio, Error>{

        self.write(samples);

        #[cfg(feature = "resampler-debug-info")]
        println!("data for channel {0} length: {} before resampling \n first 16 samples {}",
            self.input_frame.as_ref().unwrap().data(0).len()/4,
            self.input_frame.as_ref().unwrap().data(0)[..16].iter().map(|b| format!("{:?}", b)).collect::<Vec<String>>().join(", ")
        );

        if self.needs_resampling{
            let resampler = self.resampler.as_mut().unwrap();

            let _ = resampler.run(
                self.input_frame.as_ref().unwrap(),
                &mut self.resampled_frame,
            )?;
        }
        #[cfg(feature = "resampler-debug-info")]
        for channel in 0..self.resampled_frame.channels() as usize {
            println!("data per channel length: {} \n first 8 samples {}",
                self.resampled_frame.data(channel).len(),
                self.resampled_frame.data(channel)[..8].iter().map(|b| format!("{:?}", b)).collect::<Vec<String>>().join(", ")
            );
        }
        Ok(&self.resampled_frame)
    }

    /// Finalizes the resampling pipeline by flushing any remaining samples in the resampler.
    /// Returns a Result indicating success or failure.
    pub fn finalize(&mut self) -> Result<(), Error>{
        if self.needs_resampling{
            let resampler = self.resampler.as_mut().unwrap();
            let _ = resampler.flush(&mut self.resampled_frame)?;
        }
        Ok(())
    }

    fn audio_f32_to_u8_byte(sample: f32) -> [u8;4] {
        sample.to_ne_bytes()
    }
}
