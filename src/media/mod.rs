use gstreamer::prelude::*;
use gstreamer as gst;

use crate::{Result, CasterError, CodecInfo, AudioDevice};

pub struct MediaEngine {
    pipeline: Option<gst::Pipeline>,
}

impl MediaEngine {
    pub fn new() -> Result<Self> {
        gst::init().map_err(|e| CasterError::Media(e.to_string()))?;
        
        Ok(Self {
            pipeline: None,
        })
    }
    
    pub fn list_codecs(&self) -> Result<Vec<CodecInfo>> {
        let mut codecs = Vec::new();
        
        // Video codecs
        let video_codecs = vec![
            ("h264", "video/x-h264", true),
            ("h265", "video/x-h265", true),
            ("vp8", "video/x-vp8", false),
            ("vp9", "video/x-vp9", false),
            ("av1", "video/x-av1", false),
        ];
        
        for (name, mime, hw) in video_codecs {
            // Check if encoder exists
            let encode = gst::ElementFactory::find(&format!("{}enc", name)).is_some();
            let decode = gst::ElementFactory::find(&format!("{}dec", name)).is_some();
            
            codecs.push(CodecInfo {
                name: name.to_string(),
                mime_type: mime.to_string(),
                hardware_accelerated: hw,
                encode,
                decode,
            });
        }
        
        // Audio codecs
        let audio_codecs = vec![
            ("opus", "audio/x-opus", false),
            ("aac", "audio/mpeg", false),
            ("mp3", "audio/mpeg", false),
            ("flac", "audio/x-flac", false),
        ];
        
        for (name, mime, hw) in audio_codecs {
            let encode = gst::ElementFactory::find(&format!("{}enc", name)).is_some();
            let decode = gst::ElementFactory::find(&format!("{}dec", name)).is_some();
            
            codecs.push(CodecInfo {
                name: name.to_string(),
                mime_type: mime.to_string(),
                hardware_accelerated: hw,
                encode,
                decode,
            });
        }
        
        Ok(codecs)
    }
    
    pub fn list_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();
        
        // List PulseAudio devices
        if let Some(_pulsesrc) = gst::ElementFactory::make("pulsesrc").build().ok() {
            // Default input
            devices.push(AudioDevice {
                id: "pulse_input_default".to_string(),
                name: "Default Input".to_string(),
                is_input: true,
                is_default: true,
                channels: 2,
                sample_rate: 48000,
            });
        }
        
        if let Some(_pulsesink) = gst::ElementFactory::make("pulsesink").build().ok() {
            // Default output
            devices.push(AudioDevice {
                id: "pulse_output_default".to_string(),
                name: "Default Output".to_string(),
                is_input: false,
                is_default: true,
                channels: 2,
                sample_rate: 48000,
            });
        }
        
        Ok(devices)
    }
    
    pub async fn play_video(&mut self, path: &str, _display_window: &str) -> Result<()> {
        // Create pipeline for video playback
        let pipeline = gst::parse::launch(&format!(
            "filesrc location={} ! decodebin ! videoconvert ! autovideosink",
            path
        )).map_err(|e| CasterError::Media(e.to_string()))?;
        
        let pipeline = pipeline.downcast::<gst::Pipeline>()
            .map_err(|_| CasterError::Media("Failed to create pipeline".into()))?;
        
        pipeline.set_state(gst::State::Playing)
            .map_err(|_| CasterError::Media("Failed to start playback".into()))?;
        
        self.pipeline = Some(pipeline);
        
        Ok(())
    }
    
    pub async fn stream_rtsp(&mut self, url: &str, _display_window: &str) -> Result<()> {
        // Create RTSP streaming pipeline
        let pipeline = gst::parse::launch(&format!(
            "rtspsrc location={} ! decodebin ! videoconvert ! autovideosink",
            url
        )).map_err(|e| CasterError::Media(e.to_string()))?;
        
        let pipeline = pipeline.downcast::<gst::Pipeline>()
            .map_err(|_| CasterError::Media("Failed to create pipeline".into()))?;
        
        pipeline.set_state(gst::State::Playing)
            .map_err(|_| CasterError::Media("Failed to start streaming".into()))?;
        
        self.pipeline = Some(pipeline);
        
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)
                .map_err(|_| CasterError::Media("Failed to stop pipeline".into()))?;
        }
        self.pipeline = None;
        Ok(())
    }
}