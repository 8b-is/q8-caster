// use rust_cast::{CastDevice, ChannelMessage};
// use rust_cast::channels::heartbeat::HeartbeatChannel;
// use rust_cast::channels::connection::ConnectionChannel;
// use rust_cast::channels::media::MediaChannel;
// use rust_cast::channels::receiver::{ReceiverChannel, CastDeviceApp};
// use rust_cast::message_manager::MessageManager;
use std::net::IpAddr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, debug};
use serde_json::json;

use crate::{Result, CasterError, ContentType, ContentSource};

pub struct ChromecastManager {
    devices: Vec<ChromecastDevice>,
}

pub struct ChromecastDevice {
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    // Store connection info instead of the device directly
    connected: bool,
}

impl ChromecastManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
    
    pub async fn discover_devices(&mut self) -> Result<Vec<ChromecastDevice>> {
        info!("Discovering Chromecast devices...");
        
        // Use mdns-sd to discover Chromecast devices
        let mdns = mdns_sd::ServiceDaemon::new()
            .map_err(|e| CasterError::Network(format!("Failed to create mDNS daemon: {}", e)))?;
        
        let receiver = mdns.browse("_googlecast._tcp.local.")
            .map_err(|e| CasterError::Network(format!("Failed to browse for Chromecast: {}", e)))?;
        
        // Collect devices for 3 seconds
        let start = std::time::Instant::now();
        let mut devices = Vec::new();
        
        while start.elapsed() < Duration::from_secs(3) {
            if let Ok(event) = receiver.try_recv() {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        if let Some(addr) = info.get_addresses().iter().next() {
                            devices.push(ChromecastDevice {
                                name: info.get_fullname().to_string(),
                                ip: *addr,
                                port: info.get_port(),
                                cast_device: None,
                            });
                            info!("Found Chromecast: {} at {}:{}", info.get_fullname(), addr, info.get_port());
                        }
                    }
                    _ => {}
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        
        self.devices = devices.clone();
        Ok(devices)
    }
    
    pub async fn connect_to_device(&mut self, device_name: &str) -> Result<()> {
        let device = self.devices.iter_mut()
            .find(|d| d.name == device_name)
            .ok_or_else(|| CasterError::Network(format!("Device {} not found", device_name)))?;
        
        info!("Connecting to Chromecast {} at {}:{}", device.name, device.ip, device.port);
        
        let cast_device = CastDevice::connect_without_host_verification(
            device.ip.to_string(),
            device.port,
        ).map_err(|e| CasterError::Network(format!("Failed to connect to Chromecast: {}", e)))?;
        
        // Start heartbeat
        cast_device.heartbeat.set_handler(|_| {
            debug!("Received heartbeat");
        });
        
        device.cast_device = Some(cast_device);
        
        Ok(())
    }
    
    pub async fn cast_content(
        &mut self,
        device_name: &str,
        content_type: &ContentType,
        source: &ContentSource,
    ) -> Result<()> {
        let device = self.devices.iter_mut()
            .find(|d| d.name == device_name)
            .ok_or_else(|| CasterError::Network(format!("Device {} not found", device_name)))?;
        
        let cast_device = device.cast_device.as_ref()
            .ok_or_else(|| CasterError::Network("Device not connected".into()))?;
        
        // Launch default media receiver app
        let app = cast_device.receiver
            .launch_app("CC1AD845") // Default Media Receiver App ID
            .map_err(|e| CasterError::Network(format!("Failed to launch app: {}", e)))?;
        
        info!("Launched media receiver app: {:?}", app);
        
        // Determine media URL and type
        let (media_url, media_type) = match (content_type, source) {
            (ContentType::Video { .. }, ContentSource::Url { url }) => {
                (url.clone(), "video/mp4")
            }
            (ContentType::Video { .. }, ContentSource::File { path }) => {
                // For local files, we need to serve them via HTTP
                let server_url = self.start_media_server(path).await?;
                (server_url, "video/mp4")
            }
            (ContentType::Image { format }, ContentSource::Url { url }) => {
                let media_type = match format.as_str() {
                    "png" => "image/png",
                    "gif" => "image/gif",
                    _ => "image/jpeg",
                };
                (url.clone(), media_type)
            }
            (ContentType::Stream { protocol }, _) => {
                match protocol {
                    crate::StreamProtocol::Hls { manifest_url } => {
                        (manifest_url.clone(), "application/x-mpegURL")
                    }
                    crate::StreamProtocol::Dash { manifest_url } => {
                        (manifest_url.clone(), "application/dash+xml")
                    }
                    _ => return Err(CasterError::Network("Unsupported stream protocol for Chromecast".into())),
                }
            }
            _ => return Err(CasterError::Network("Unsupported content type for Chromecast".into())),
        };
        
        // Load media
        let media_info = json!({
            "contentId": media_url,
            "contentType": media_type,
            "streamType": "BUFFERED",
            "metadata": {
                "type": 0,
                "metadataType": 0,
                "title": "Q8-Caster Media",
            }
        });
        
        cast_device.media
            .load(
                app.transport_id.as_str(),
                &app.session_id,
                &media_info,
            )
            .map_err(|e| CasterError::Network(format!("Failed to load media: {}", e)))?;
        
        info!("Successfully cast content to {}", device_name);
        
        Ok(())
    }
    
    pub async fn stop_casting(&mut self, device_name: &str) -> Result<()> {
        let device = self.devices.iter_mut()
            .find(|d| d.name == device_name)
            .ok_or_else(|| CasterError::Network(format!("Device {} not found", device_name)))?;
        
        if let Some(cast_device) = &device.cast_device {
            cast_device.receiver.stop_app(&cast_device.receiver.get_current_app()?.unwrap().session_id)
                .map_err(|e| CasterError::Network(format!("Failed to stop app: {}", e)))?;
            
            info!("Stopped casting on {}", device_name);
        }
        
        Ok(())
    }
    
    pub async fn get_device_status(&self, device_name: &str) -> Result<serde_json::Value> {
        let device = self.devices.iter()
            .find(|d| d.name == device_name)
            .ok_or_else(|| CasterError::Network(format!("Device {} not found", device_name)))?;
        
        if let Some(cast_device) = &device.cast_device {
            let status = cast_device.receiver.get_status()
                .map_err(|e| CasterError::Network(format!("Failed to get status: {}", e)))?;
            
            Ok(json!({
                "connected": true,
                "volume": status.volume,
                "app": status.applications.first().map(|app| json!({
                    "id": app.app_id,
                    "name": app.display_name,
                    "status": app.status_text,
                })),
            }))
        } else {
            Ok(json!({
                "connected": false,
            }))
        }
    }
    
    pub fn list_devices(&self) -> Vec<serde_json::Value> {
        self.devices.iter().map(|device| {
            json!({
                "name": device.name,
                "ip": device.ip.to_string(),
                "port": device.port,
                "connected": device.cast_device.is_some(),
            })
        }).collect()
    }
    
    async fn start_media_server(&self, file_path: &str) -> Result<String> {
        // TODO: Implement a simple HTTP server to serve local files
        // For now, return a placeholder
        error!("Local file serving not yet implemented");
        Err(CasterError::Network("Local file serving not yet implemented".into()))
    }
}

// Additional tools for Chromecast control
pub struct ChromecastController {
    device: CastDevice,
    session_id: String,
    transport_id: String,
}

impl ChromecastController {
    pub fn play(&self) -> Result<()> {
        self.device.media
            .play(&self.transport_id, &self.session_id)
            .map_err(|e| CasterError::Network(format!("Failed to play: {}", e)))?;
        Ok(())
    }
    
    pub fn pause(&self) -> Result<()> {
        self.device.media
            .pause(&self.transport_id, &self.session_id)
            .map_err(|e| CasterError::Network(format!("Failed to pause: {}", e)))?;
        Ok(())
    }
    
    pub fn seek(&self, position: f64) -> Result<()> {
        self.device.media
            .seek(&self.transport_id, &self.session_id, Some(position), None)
            .map_err(|e| CasterError::Network(format!("Failed to seek: {}", e)))?;
        Ok(())
    }
    
    pub fn set_volume(&self, level: f32) -> Result<()> {
        self.device.receiver
            .set_volume(level)
            .map_err(|e| CasterError::Network(format!("Failed to set volume: {}", e)))?;
        Ok(())
    }
}