use image::DynamicImage;
use xcap::Monitor;

use crate::{CasterError, Result};

pub struct ScreenMirror {
    monitor: Monitor,
}

impl ScreenMirror {
    pub fn new(display_id: Option<String>) -> Result<Self> {
        let mut monitors = Monitor::all()
            .map_err(|e| CasterError::Display(format!("Failed to get monitors: {}", e)))?;

        let monitor = if let Some(id) = display_id {
            // Find monitor by ID
            monitors
                .into_iter()
                .find(|m| m.id().to_string() == id || m.name() == id)
                .ok_or_else(|| CasterError::Display(format!("Monitor '{}' not found", id)))?
        } else {
            // Use primary monitor or first available
            let primary = monitors.iter().position(|m| m.is_primary());
            if let Some(idx) = primary {
                monitors.remove(idx)
            } else {
                monitors.into_iter().next()
                    .ok_or_else(|| CasterError::Display("No monitors found".into()))?
            }
        };

        Ok(Self { monitor })
    }

    pub fn capture_frame(&mut self) -> Result<Option<DynamicImage>> {
        let image = self
            .monitor
            .capture_image()
            .map_err(|e| CasterError::Display(format!("Failed to capture screen: {}", e)))?;

        // Convert xcap::Image to image::DynamicImage
        let width = image.width();
        let height = image.height();
        let rgba = image.as_rgba();

        let rgba_image = image::RgbaImage::from_raw(width, height, rgba.to_vec())
            .ok_or_else(|| CasterError::Display("Failed to create image from capture".into()))?;

        Ok(Some(DynamicImage::ImageRgba8(rgba_image)))
    }

    pub fn get_monitor_info(&self) -> MonitorInfo {
        MonitorInfo {
            id: self.monitor.id().to_string(),
            name: self.monitor.name().to_string(),
            width: self.monitor.width(),
            height: self.monitor.height(),
            x: self.monitor.x(),
            y: self.monitor.y(),
            is_primary: self.monitor.is_primary(),
        }
    }

    pub fn list_monitors() -> Result<Vec<MonitorInfo>> {
        let monitors = Monitor::all()
            .map_err(|e| CasterError::Display(format!("Failed to get monitors: {}", e)))?;

        Ok(monitors
            .into_iter()
            .map(|m| MonitorInfo {
                id: m.id().to_string(),
                name: m.name().to_string(),
                width: m.width(),
                height: m.height(),
                x: m.x(),
                y: m.y(),
                is_primary: m.is_primary(),
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}
