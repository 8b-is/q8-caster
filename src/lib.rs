pub mod server;
pub mod display;
pub mod media;
pub mod render;
pub mod network;
pub mod cache;
pub mod error;
pub mod secrets;

pub use error::{Result, CasterError};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastSession {
    pub id: Uuid,
    pub display_id: String,
    pub content_type: ContentType,
    pub source: ContentSource,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentType {
    Markdown { theme: Option<String> },
    Video { codec: String, container: String },
    Audio { codec: String, format: String },
    Image { format: String },
    Pdf { page: Option<u32> },
    Model3D { format: String },
    Stream { protocol: StreamProtocol },
    Presentation { format: String },
    ScreenMirror { source_display: Option<String>, quality: MirrorQuality },
    WebAssembly { module_url: String, entry_point: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirrorQuality {
    Low,      // 720p @ 30fps
    Medium,   // 1080p @ 30fps
    High,     // 1080p @ 60fps
    Ultra,    // 4K @ 60fps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol", rename_all = "snake_case")]
pub enum StreamProtocol {
    Rtsp { url: String },
    WebRtc { offer: String },
    Hls { manifest_url: String },
    Dash { manifest_url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ContentSource {
    File { path: String },
    Url { url: String },
    Memory { data: Vec<u8> },
    Cache { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub resolution: Resolution,
    pub position: Position,
    pub is_primary: bool,
    pub refresh_rate: f32,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    pub name: String,
    pub mime_type: String,
    pub hardware_accelerated: bool,
    pub encode: bool,
    pub decode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
    pub channels: u32,
    pub sample_rate: u32,
}