use std::{
    sync::{
        mpsc,
        Arc,
        Mutex,
    },
};

use ffmpeg_next::{
    Dictionary,
    Error,
    Packet,
    Rational,
    codec,
    encoder,
    format,
    threading,
    util::frame::Video
};

use scap::frame::BGRAFrame;

use crate::
        recorder_controller::{
            recorder_supported_options::{
                Fps,
                Resolution
            },
            recording_session::
                muxing_pipeline::
                    muxer::{
                        Muxer,
                        EncodedPacket,
                        StreamType,
                    },
        };



use super::frame_rescaler::FrameRescaler;

pub struct VideoEncoder {
    encoder: encoder::Video,
    packet_transmitter: mpsc::Sender<EncodedPacket>,
    scaler: FrameRescaler,
    stream_index: usize,
    pts: i64,
    target_frame: Video,
    input_frame: Video,
}

impl VideoEncoder{
    /// Creates a new VideoEncoder instance with the specified parameters.
    ///
    /// Arguments
    /// * `muxer` - The Muxer instance to use for output.
    /// * `target_resolution` - The target resolution for the encoded video.
    /// * `fps` - The frames per second for the encoded video.
    pub fn new(
        // output_context: &mut format::context::Output,
        // packet_transmitter: mpsc::Sender<Packet>,
        muxer: Arc<Mutex<Muxer>>,
        target_resolution: &Resolution,
        fps: &Fps,
    ) -> Result<Self, Error>{
        let mut muxer = muxer.lock().unwrap();
        let packet_transmitter = muxer.packet_transmitter.clone();


        let codec = encoder::find(codec::Id::H264)
                                .ok_or(ffmpeg_next::Error::EncoderNotFound)?;

        //let mut stream = output_context.add_stream(codec)?;
        let mut stream = muxer.output.add_stream(codec)?;

        let mut encoder_context = codec::Context::new_with_codec(codec).encoder().video()?;

        let time_base = Rational::new(1, fps.value as i32); //check for aligment with audio

        let gop_value = fps.value; // Set GOP size to fps for 1 second interval

        encoder_context.set_width(target_resolution.width);
        encoder_context.set_height(target_resolution.height);
        encoder_context.set_format(format::Pixel::YUV420P);
        encoder_context.set_time_base(time_base);
        encoder_context.set_bit_rate(6_000_000);
        //set b-frame eventually
        encoder_context.set_gop(gop_value);
        encoder_context.set_max_b_frames(0);
        encoder_context.set_flags(codec::Flags::GLOBAL_HEADER);

        // Threading configuration
        // threading::Type::Slice is often more efficient for Live Streaming and Online Encoding
        // encodes frame split into multiple slices processed in parallel
        // threading::Type::Frame can be better for ofline encoding of files, adds latency
        // encodes multiple frames in parallel
        encoder_context.set_threading(threading::Config {
            kind: threading::Type::Slice,
            count: 0, // let ffmpeg decide
        });

        let stream_index = stream.index();
        let mut options = Dictionary::new();
        options.set("crf", "18");
        options.set("tune", "animation"); // improves sharp edges
        options.set("preset", "ultrafast");

        let encoder = encoder_context.open_as_with(codec, options).unwrap();
        stream.set_parameters(&encoder);


        let target_frame = Video::new(
            format::Pixel::YUV420P,
            target_resolution.width,
            target_resolution.height,
        );

        let input_frame = Video::new(
            format::Pixel::BGRA,
            target_resolution.width,
            target_resolution.height,
        );

        let scaler = FrameRescaler::new(
            &target_resolution,
            input_frame.format(),
            target_frame.format(),
        )?;

        Ok(
            Self {
                encoder,
                packet_transmitter: packet_transmitter.unwrap().clone(),
                scaler,
                stream_index,
                pts: 0 as i64,
                target_frame,
                input_frame,
            }
        )
    }

    /// Encodes any available packets from the encoder and sends them to the packet transmitter.
    fn encode_packet(&mut self) -> Result<(), Error> {

        let mut packet = Packet::empty();

        while self.encoder.receive_packet(&mut packet).is_ok() {
            let encoded_packet = EncodedPacket{
                packet: packet.clone(),
                stream_index: self.stream_index,
                stream_time_base: self.encoder.time_base(),
                stream_type: StreamType::Video,
            };
            self.packet_transmitter.send(encoded_packet).unwrap();
        }
        Ok(())
    }

    /// Sets the source resolution for the encoder and updates the input frame and scaler accordingly.
    ///
    /// #Arguments
    /// * `source_resolution` - The resolution of the source frames.
    pub fn set_frame_scaling(&mut self, source_resolution: &Resolution) -> Result<(), Error> {
        let input_format = format::Pixel::BGRA;

        self.input_frame = Video::new(
            input_format,
            source_resolution.width,
            source_resolution.height,
        );

        self.scaler.change_source(
            &self.input_frame,
            &self.target_frame
        )?;
        Ok(())
    }
    /// Copies the data from a BGRA frame to the encoder's input frame, handling stride differences if necessary.
    ///
    /// # Arguments
    /// * `frame` - The BGRA frame to copy data from.
    fn copy_frame_data(&mut self, frame: &BGRAFrame) -> Result<(), Error> {
        let frame_stride = (frame.width * 4) as usize;
        let input_frame_stride = self.input_frame.stride(0);
        if input_frame_stride != frame_stride {
            copy_with_stride_aligned(
                &frame.data,
                frame_stride,
                self.input_frame.data_mut(0),
                input_frame_stride,
                frame.height as usize
            );
        }
        else{
            let buffer_len = self.input_frame.data(0).len();
            self.input_frame.data_mut(0).copy_from_slice(&frame.data[..buffer_len]);
        }
        Ok(())
    }

    /// Encodes a single frame and sends it to the encoder.
    ///
    /// Arguments
    /// * `frame` - The frame to encode.
    ///
    pub fn encode_frame(
        &mut self,
        frame: &BGRAFrame
    ) -> Result<(), Error> {
        self.copy_frame_data(frame)?;

        self.scaler.run(&self.input_frame,  &mut self.target_frame)?;

        self.target_frame.set_pts(Some(self.pts));
        self.pts += 1;

        self.encoder.send_frame(&self.target_frame)?;
        self.encode_packet()?;
        Ok(())
    }


    /// Finalizes the video encoding process by sending an end-of-file signal to the encoder and processing any remaining packets.
    pub fn finalize_video_encoding(&mut self) -> Result<(), Error> {
        self.encoder.send_eof()?;
        self.encode_packet()?;
        Ok(())
    }
}

/// Copies data from a source buffer to a target buffer with different strides, ensuring proper alignment.
///
/// # Arguments
/// * `source_data` - The source buffer containing the data to copy.
/// * `source_stride` - The stride (number of bytes per row) of the source
/// * `target_data` - The target buffer where the data will be copied to.
/// * `target_stride` - The stride (number of bytes per row) of the target
/// * `height` - The number of rows to copy.
///
fn copy_with_stride_aligned(
    source_data: &[u8],
    source_stride: usize,
    target_data: &mut [u8],
    target_stride: usize,
    height: usize,
) {
    for row in 0..height {
        let source_start = row * source_stride;
        let target_start = row * target_stride;

        let source_end = source_start + source_stride;

        let target_end = target_start + source_stride;

        target_data[target_start..target_end]
            .copy_from_slice(&source_data[source_start..source_end]);
    }
}