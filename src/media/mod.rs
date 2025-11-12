use crate::{Result, CodecInfo, AudioDevice};

pub struct MediaEngine {
    // Placeholder for media engine
}

impl MediaEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn list_codecs(&self) -> Result<Vec<CodecInfo>> {
        // Return static list of commonly supported codecs
        Ok(vec![
            CodecInfo {
                name: "h264".to_string(),
                mime_type: "video/x-h264".to_string(),
                hardware_accelerated: true,
                encode: false,
                decode: true,
            },
            CodecInfo {
                name: "h265".to_string(),
                mime_type: "video/x-h265".to_string(),
                hardware_accelerated: true,
                encode: false,
                decode: true,
            },
            CodecInfo {
                name: "vp9".to_string(),
                mime_type: "video/x-vp9".to_string(),
                hardware_accelerated: false,
                encode: false,
                decode: true,
            },
            CodecInfo {
                name: "opus".to_string(),
                mime_type: "audio/x-opus".to_string(),
                hardware_accelerated: false,
                encode: false,
                decode: true,
            },
            CodecInfo {
                name: "aac".to_string(),
                mime_type: "audio/mpeg".to_string(),
                hardware_accelerated: false,
                encode: false,
                decode: true,
            },
        ])
    }

    pub fn list_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        // Return placeholder audio devices
        Ok(vec![
            AudioDevice {
                id: "default".to_string(),
                name: "Default Audio Device".to_string(),
                is_input: false,
                is_default: true,
                channels: 2,
                sample_rate: 48000,
            },
        ])
    }
}
