use core::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct Resolution{
    pub width: u32,
    pub height: u32,
}

impl Resolution{
    /// Creates a new Resolution instance with the specified width and height.
    ///
    /// Arguments
    /// * `width` - The width of the resolution.
    /// * `height` - The height of the resolution.
    /// Returns
    /// Resolution instance.
    pub fn from (width: u32, height: u32) -> Self{
        Resolution { width, height }
    }
}

#[derive(Clone)]
pub enum RecorderSupportedResolution{
    _720p,
    _1080p,
    _1440p,
}

impl RecorderSupportedResolution{
    /// Returns the corresponding Resolution struct for the given RecorderSupportedResolution variant.
    pub fn value(&self) -> Resolution {
        match *self {
            RecorderSupportedResolution::_720p => Resolution { width: 1280, height: 720 },
            RecorderSupportedResolution::_1080p => Resolution { width: 1920, height: 1080 },
            RecorderSupportedResolution::_1440p => Resolution { width: 2560, height: 1440 },
        }
    }
}

impl fmt::Display for RecorderSupportedResolution{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            RecorderSupportedResolution::_720p => write!(f, "HD 720p"),
            RecorderSupportedResolution::_1080p => write!(f, "FullHD 1080p"),
            RecorderSupportedResolution::_1440p => write!(f, "QHD 1440p"),
        }
    }
}

#[derive(Clone)]
pub struct Fps{
    pub value: u32,
}

#[derive(Clone)]
pub enum RecorderSupportedFps{
    _30,
    _60,
    _120,
}
impl RecorderSupportedFps {
    /// Returns the corresponding Fps struct for the given RecorderSupportedFps variant.
    pub fn value(&self) -> Fps {
        match *self {
            RecorderSupportedFps::_30 => Fps{ value: 30 },
            RecorderSupportedFps::_60 => Fps{ value: 60 },
            RecorderSupportedFps::_120 => Fps{ value: 120 },
        }
    }
}
impl fmt::Display for RecorderSupportedFps{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            RecorderSupportedFps::_30 => write!(f, "30"),
            RecorderSupportedFps::_60 => write!(f, "60"),
            RecorderSupportedFps::_120 => write!(f, "120"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RecorderSupportedFileFormat{
    MP4,
}
impl fmt::Display for RecorderSupportedFileFormat{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            RecorderSupportedFileFormat::MP4 => write!(f, "mp4"),
        }
    }
}
