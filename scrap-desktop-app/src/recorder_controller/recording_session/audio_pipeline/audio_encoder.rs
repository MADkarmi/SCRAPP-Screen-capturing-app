use std::{
    sync::{
        mpsc,
        Arc,
        Mutex,
    },
};

use ffmpeg_next::{
    ChannelLayout,
    Error,
    Packet,
    Rational,
    codec,
    encoder,
    format::{
        self,
        sample::Type
    },
    frame,
};

use crate::
    recorder_controller::
        recording_session::{
            muxing_pipeline::
                muxer::{
                    EncodedPacket,
                    StreamType,
                    Muxer
                }
            };

const AUDIO_SAMPLES_PER_FRAME: usize = 1024; // AAC frame size
const AUDIO_BIT_RATE: usize = 128_000; // 128 kbps
pub struct AudioEncoder {
    encoder: encoder::Audio,
    packet_transmitter: mpsc::Sender<EncodedPacket>,
    pts: i64,
    audio_samples_per_frame: usize,  //1024 for AAC
    stream_index: usize,
}

impl AudioEncoder {
    /// Creates a new AudioEncoder for AAC encoding
    /// # Arguments
    /// * `output` - The output format context where the audio stream will be added, taken from VideoEncoder
    /// * `sample_rate` - The sample rate of the input audio
    /// * `channels` - The number of channels of the input audio
    pub fn new(
        // output: &mut format::context::Output,
        // packet_transmitter: mpsc::Sender<Packet>,
        muxer: Arc<Mutex<Muxer>>,
        sample_rate: i32,
        channels: i32
    ) -> Result<Self, Error> {
        let codec = encoder::find(codec::Id::AAC)
            .ok_or(ffmpeg_next::Error::EncoderNotFound)?;

        let mut muxer = muxer.lock().unwrap();
        let packet_transmitter = muxer.packet_transmitter.clone();

        let mut stream = muxer.output.add_stream(codec)?;

        let mut encoder_context: encoder::audio::Audio = codec::Context::new_with_codec(codec).encoder().audio()?;

        let time_base = Rational::new(1, sample_rate);

        encoder_context.set_rate(sample_rate);
        match channels {
            1 => encoder_context.set_channel_layout(ChannelLayout::MONO),
            2 => encoder_context.set_channel_layout(ChannelLayout::STEREO),
            _ => encoder_context.set_channel_layout(ChannelLayout::SURROUND),
        }
        //AAC requires planar f32 input
        encoder_context.set_format(format::Sample::F32(Type::Planar));
        encoder_context.set_time_base(time_base);
        encoder_context.set_bit_rate(AUDIO_BIT_RATE);
        encoder_context.set_flags(codec::Flags::GLOBAL_HEADER);



        let encoder = encoder_context.open_as(codec)?;
        stream.set_parameters(&encoder);

        Ok(Self {
            encoder,
            packet_transmitter: packet_transmitter.unwrap().clone(),
            stream_index: stream.index(),
            pts: 0,
            audio_samples_per_frame: AUDIO_SAMPLES_PER_FRAME,
        })
    }


    /// flushes encoded packets to output
    fn encode_packet(&mut self) -> Result<(), Error> {
        let mut packet = Packet::empty();

        while self.encoder.receive_packet(&mut packet).is_ok() {
            let encoded_packet = EncodedPacket{
                packet: packet.clone(),
                stream_index: self.stream_index,
                stream_time_base: self.encoder.time_base(),
                stream_type: StreamType::Audio,
            };
            self.packet_transmitter.send(encoded_packet).unwrap();
        }
        Ok(())
    }

    /// Encodes a mixed audio frame and sends the encoded packets to the muxer.
    /// # Arguments
    /// * `mixed_frame` - The mixed audio frame to be encoded
    /// * `next_frame_pts_offset` - The number of samples from the next frame that were not included in this frame
    pub fn encode_mixed_frame(
        &mut self,
        mixed_frame: &mut frame::Audio,
        next_frame_pts_offset: usize
    ) -> Result<(), Error> {
        //mixed_frame is planar f32 with required samples per channel
        mixed_frame.set_pts(Some(self.pts));
        let next_frame_pts_offset = next_frame_pts_offset.clamp(0, self.audio_samples_per_frame - 1);

        self.pts += (self.audio_samples_per_frame - next_frame_pts_offset) as i64;

        self.encoder.send_frame(&mixed_frame)?;
        self.encode_packet()?;

        Ok(())
    }

    /// Finalizes the audio encoding process by sending an end-of-stream signal to the encoder and flushing any remaining packets.
    pub fn finalize_audio_encoding(&mut self) -> Result<(), Error> {
        self.encoder.send_eof()?;
        self.encode_packet()?;
        Ok(())
    }

    pub fn sample_rate(&self) -> u32 {
        self.encoder.rate()
    }

    pub fn channel_layout(&self) -> ChannelLayout {
        self.encoder.channel_layout()
    }

    pub fn format(&self) -> format::Sample {
        self.encoder.format()
    }

    pub fn required_samples_per_channel(&self) -> usize {
        AUDIO_SAMPLES_PER_FRAME
    }
}
