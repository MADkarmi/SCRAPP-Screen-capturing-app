use std::{
    path::Path,
    sync::{
        mpsc::{
            self,
            Sender,
            Receiver,
        },
    },
};

use ffmpeg_next::{
    Dictionary, codec::packet::packet::Packet, format::{
        self, context::Output
    }, log
};

#[derive(Debug)]
pub enum StreamType{
    Video,
    Audio,
}

#[allow(dead_code)]
pub struct EncodedPacket{
    pub packet: Packet,
    pub stream_index: usize,
    pub stream_time_base: ffmpeg_next::Rational,
    pub stream_type: StreamType,
}

pub struct Muxer {
    pub output: Output,
    packet_receiver: Receiver<EncodedPacket>,
    pub packet_transmitter: Option<Sender<EncodedPacket>>,
}

impl Muxer {
    /// Creates a new Muxer instance with the specified output path.
    ///
    /// Arguments
    /// * `path` - The path to the output file.
    /// Returns
    /// A Result containing the Muxer instance or an ffmpeg_next::Error.
    pub fn new_context<P: AsRef<Path>+?Sized>(path: &P) -> Result<Self, ffmpeg_next::Error> {
        ffmpeg_next::init()?;
        log::set_level(log::Level::Quiet);
        let mut output = format::output(path).unwrap();

        let mut metadata = Dictionary::new();
        metadata.set("title", "Screen Capture");
        output.set_metadata(metadata);

        let (packet_transmitter, packet_receiver) = mpsc::channel::<EncodedPacket>();
        Ok(
            Self {
                output,
                packet_receiver: packet_receiver,
                packet_transmitter: Some(packet_transmitter)
            }
        )
    }

    /// Initializes the muxer by writing the header to the output file.
    pub fn initialize(&mut self) -> Result<(), ffmpeg_next::Error> {
        self.output.write_header()
    }

    /// Writes the encoded packets received from the packet_receiver to the output file.
    /// This function runs in a loop until the packet_receiver is closed.
    pub fn write_packets(&mut self) -> Result<(), ffmpeg_next::Error> {
        drop(self.packet_transmitter.take());
        while let Ok(mut encoded_packet) = self.packet_receiver.recv() {
            encoded_packet.packet.set_stream(encoded_packet.stream_index);
            encoded_packet.packet.rescale_ts(
            encoded_packet.stream_time_base,
            self.output.stream(encoded_packet.stream_index).unwrap().time_base()
            );

            #[cfg(feature = "muxer-debug-info")]
            println!("Muxer: writing packet pts {}, dts {} from stream {:?} size {} data length {}", encoded_packet.packet.pts().unwrap(), encoded_packet.packet.dts().unwrap(), encoded_packet.stream_type, encoded_packet.packet.size(), encoded_packet.packet.data().unwrap().len());

            encoded_packet.packet.write_interleaved(&mut self.output)?;
        }
        Ok(())
    }

    /// Finalizes the muxer by writing the trailer to the output file.
    pub fn finalize(&mut self) -> Result<(), ffmpeg_next::Error> {
        #[cfg(feature = "muxer-debug-info")]
        println!("Video saved to output.mp4 successfully");
        self.output.write_trailer()
    }
}