use ffmpeg_next::{
    Error,
    format,
    software::{
        scaling::Context,
        scaling::flag::Flags,
    },
    util::frame::Video
};

use crate::
        recorder_controller::
            recorder_supported_options::
                Resolution;

pub struct FrameRescaler {
    pub scaler: Context
}

impl FrameRescaler {
    /// Creates a new FrameRescaler instance with the specified parameters.
    ///
    /// Arguments
    /// * `target_resolution` - The target resolution for the encoded video.
    /// * `source_format` - The pixel format of the source video frames.
    /// * `target_format` - The pixel format of the target video frames.
    /// Returns
    /// A Result containing the FrameRescaler instance or an ffmpeg_next::Error.
    pub fn new(
        target_resolution: &Resolution,
        source_format: format::Pixel,
        target_format: format::Pixel,
    ) -> Result<Self, Error> {

        let scaler = Context::get(
            source_format,
            target_resolution.width,
            target_resolution.height,
            target_format,
            target_resolution.width,
            target_resolution.height,
            Flags::BILINEAR,
        )?;
        Ok(
            Self {
                scaler,
            }
        )
    }

    /// Changes the source format and resolution for the FrameRescaler.
    ///
    /// Arguments
    /// * `source` - A reference to the source Video frame.
    /// * `target` - A reference to the target Video frame.
    /// Returns
    /// A Result indicating success or an ffmpeg_next::Error.
    pub fn change_source(&mut self, source: &Video, target: &Video) -> Result<(), Error> {
        self.scaler = Context::get(
            source.format(),
            source.width(),
            source.height(),
            target.format(),
            target.width(),
            target.height(),
            Flags::FAST_BILINEAR //BILINEAR
        )?;
        Ok(())
    }

    /// Runs the frame rescaling process from the source frame to the target frame.
    ///
    /// Arguments
    /// * `source_frame` - A reference to the source Video frame.
    /// * `target_frame` - A mutable reference to the target Video frame.
    /// Returns
    /// A Result indicating success or an ffmpeg_next::Error.
    pub fn run(&mut self, source_frame: &Video, target_frame: &mut Video) -> Result<(), Error> {
        self.scaler.run(source_frame, target_frame)?;
        Ok(())
    }
}