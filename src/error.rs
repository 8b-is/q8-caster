use thiserror::Error;

pub type Result<T> = std::result::Result<T, CasterError>;

#[derive(Error, Debug)]
pub enum CasterError {
    #[error("Display error: {0}")]
    Display(String),
    
    #[error("Media error: {0}")]
    Media(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Render error: {0}")]
    Render(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("MCP error: {0}")]
    Mcp(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // GStreamer support disabled for now
    // #[error("GStreamer error: {0}")]
    // GStreamer(#[from] gstreamer::glib::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<String> for CasterError {
    fn from(s: String) -> Self {
        CasterError::Unknown(s)
    }
}

impl From<&str> for CasterError {
    fn from(s: &str) -> Self {
        CasterError::Unknown(s.to_string())
    }
}