use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::time;
use tracing::{info, warn, error};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use rupnp::ssdp::{SearchTarget, URN};
use futures::StreamExt;

use crate::{Result, CasterError};

/// Type of discovered device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Chromecast,
    FireTv,
    AirPlay,
    Dlna,
    Upnp,
    Miracast,
    Custom(String),
}

impl DeviceType {
    pub fn from_service_type(service_type: &str) -> Self {
        match service_type {
            "_googlecast._tcp.local." => DeviceType::Chromecast,
            "_airplay._tcp.local." => DeviceType::AirPlay,
            "_dlna._tcp.local." => DeviceType::Dlna,
            "_dial._tcp.local." => DeviceType::FireTv, // FireTV uses DIAL protocol
            _ => DeviceType::Custom(service_type.to_string()),
        }
    }

    pub fn to_mdns_service(&self) -> &str {
        match self {
            DeviceType::Chromecast => "_googlecast._tcp.local.",
            DeviceType::AirPlay => "_airplay._tcp.local.",
            DeviceType::Dlna => "_dlna._tcp.local.",
            DeviceType::FireTv => "_dial._tcp.local.",
            DeviceType::Upnp => "_upnp._tcp.local.",
            DeviceType::Miracast => "_miracast._tcp.local.",
            DeviceType::Custom(s) => s.as_str(),
        }
    }
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub can_video: bool,
    pub can_audio: bool,
    pub can_image: bool,
    pub can_mirror: bool,
    pub supported_codecs: Vec<String>,
    pub max_resolution: Option<String>,
    pub protocols: Vec<String>,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            can_video: true,
            can_audio: true,
            can_image: true,
            can_mirror: false,
            supported_codecs: vec!["h264".to_string(), "aac".to_string()],
            max_resolution: Some("1080p".to_string()),
            protocols: vec![],
        }
    }
}

/// A discovered device on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub ip: IpAddr,
    pub port: u16,
    pub capabilities: DeviceCapabilities,
    pub discovered_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub metadata: JsonValue,
}

impl DiscoveredDevice {
    pub fn new(
        id: String,
        name: String,
        device_type: DeviceType,
        ip: IpAddr,
        port: u16,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            device_type,
            ip,
            port,
            capabilities: DeviceCapabilities::default(),
            discovered_at: now,
            last_seen: now,
            metadata: serde_json::json!({}),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.last_seen);
        elapsed.num_seconds() > timeout.as_secs() as i64
    }
}

/// Device discovery manager
pub struct DeviceDiscovery {
    devices: Arc<DashMap<String, DiscoveredDevice>>,
    mdns: Option<ServiceDaemon>,
    discovery_running: Arc<tokio::sync::RwLock<bool>>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl DeviceDiscovery {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(DashMap::new()),
            mdns: None,
            discovery_running: Arc::new(tokio::sync::RwLock::new(false)),
            tasks: Vec::new(),
        }
    }

    /// Start device discovery for specified device types
    pub async fn start(&mut self, device_types: Vec<DeviceType>) -> Result<()> {
        let mut running = self.discovery_running.write().await;
        if *running {
            return Ok(());
        }

        info!("Starting device discovery for: {:?}", device_types);

        // Abort any existing tasks before starting new ones
        for handle in self.tasks.drain(..) {
            handle.abort();
        }

        // Create mDNS daemon
        let mdns = ServiceDaemon::new()
            .map_err(|e| CasterError::Network(format!("Failed to create mDNS daemon: {}", e)))?;

        // Track if we need UPnP discovery
        let needs_upnp = device_types.iter().any(|dt| matches!(dt, DeviceType::Upnp | DeviceType::Dlna));

        // Start browsing for each device type
        for device_type in &device_types {
            let service_type = device_type.to_mdns_service();
            info!("Browsing for {} devices", service_type);

            let receiver = mdns.browse(service_type)
                .map_err(|e| CasterError::Network(format!("Failed to browse {}: {}", service_type, e)))?;

            // Spawn task to handle discovered devices
            let devices = Arc::clone(&self.devices);
            let dt = device_type.clone();
            let running_flag = Arc::clone(&self.discovery_running);

            let handle = tokio::spawn(async move {
                Self::handle_mdns_events(receiver, devices, dt, running_flag).await;
            });
            self.tasks.push(handle);
        }

        // Start UPnP/SSDP discovery if needed
        if needs_upnp {
            let devices = Arc::clone(&self.devices);
            let running_flag = Arc::clone(&self.discovery_running);
            let handle = tokio::spawn(async move {
                Self::discover_upnp_devices(devices, running_flag).await;
            });
            self.tasks.push(handle);
        }

        // Start cleanup task
        let devices_clone = Arc::clone(&self.devices);
        let running_flag = Arc::clone(&self.discovery_running);
        let handle = tokio::spawn(async move {
            Self::cleanup_stale_devices(devices_clone, running_flag).await;
        });
        self.tasks.push(handle);

        self.mdns = Some(mdns);
        *running = true;

        info!("Device discovery started successfully");
        Ok(())
    }

    /// Handle mDNS service events
    async fn handle_mdns_events(
        receiver: mdns_sd::Receiver<ServiceEvent>,
        devices: Arc<DashMap<String, DiscoveredDevice>>,
        device_type: DeviceType,
        running: Arc<tokio::sync::RwLock<bool>>,
    ) {
        while *running.read().await {
            match receiver.recv_async().await {
                Ok(event) => {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            info!("Discovered {} device: {}", device_type.to_mdns_service(), info.get_fullname());

                            // Extract device information
                            let name = info.get_fullname().trim_end_matches('.').to_string();
                            let id = format!("{}:{}:{}", device_type.to_mdns_service(), info.get_hostname(), info.get_port());

                            // Get IP address
                            let ip = if let Some(addr) = info.get_addresses().iter().next() {
                                *addr
                            } else {
                                warn!("No IP address found for device: {}", name);
                                continue;
                            };

                            let port = info.get_port();

                            // Create or update device
                            if let Some(mut device) = devices.get_mut(&id) {
                                device.update_last_seen();
                                info!("Updated device: {} ({}:{})", name, ip, port);
                            } else {
                                let mut device = DiscoveredDevice::new(
                                    id.clone(),
                                    name.clone(),
                                    device_type.clone(),
                                    ip,
                                    port,
                                );

                                // Parse metadata from TXT records
                                let mut metadata = serde_json::json!({});
                                for property in info.get_properties().iter() {
                                    metadata[property.key()] = serde_json::json!(property.val_str());
                                }
                                device.metadata = metadata;

                                // Set capabilities based on device type
                                device.capabilities = match device_type {
                                    DeviceType::Chromecast => DeviceCapabilities {
                                        can_video: true,
                                        can_audio: true,
                                        can_image: true,
                                        can_mirror: true,
                                        supported_codecs: vec!["h264".into(), "vp8".into(), "vp9".into(), "aac".into(), "opus".into()],
                                        max_resolution: Some("4K".to_string()),
                                        protocols: vec!["cast".into()],
                                    },
                                    DeviceType::FireTv => DeviceCapabilities {
                                        can_video: true,
                                        can_audio: true,
                                        can_image: true,
                                        can_mirror: true,
                                        supported_codecs: vec!["h264".into(), "h265".into(), "aac".into()],
                                        max_resolution: Some("4K".to_string()),
                                        protocols: vec!["dial".into(), "miracast".into()],
                                    },
                                    DeviceType::AirPlay => DeviceCapabilities {
                                        can_video: true,
                                        can_audio: true,
                                        can_image: true,
                                        can_mirror: true,
                                        supported_codecs: vec!["h264".into(), "aac".into()],
                                        max_resolution: Some("1080p".to_string()),
                                        protocols: vec!["airplay".into()],
                                    },
                                    _ => DeviceCapabilities::default(),
                                };

                                devices.insert(id.clone(), device);
                                info!("Added new device: {} ({}:{})", name, ip, port);
                            }
                        },
                        ServiceEvent::ServiceFound(_, _) => {
                            // Service found but not yet resolved, will be handled by ServiceResolved
                        },
                        ServiceEvent::ServiceRemoved(_, fullname) => {
                            info!("Device removed: {}", fullname);
                            // Note: We keep devices for a while even after removal (handled by cleanup task)
                        },
                        ServiceEvent::SearchStarted(_) => {
                            info!("mDNS search started for {}", device_type.to_mdns_service());
                        },
                        ServiceEvent::SearchStopped(_) => {
                            info!("mDNS search stopped for {}", device_type.to_mdns_service());
                        },
                    }
                },
                Err(_) => {
                    // Channel closed or error - exit the loop
                    break;
                }
            }
        }
    }

    /// Cleanup stale devices
    async fn cleanup_stale_devices(
        devices: Arc<DashMap<String, DiscoveredDevice>>,
        running: Arc<tokio::sync::RwLock<bool>>,
    ) {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if !*running.read().await {
                break;
            }

            let timeout = Duration::from_secs(300); // 5 minutes
            let mut to_remove = Vec::new();

            for entry in devices.iter() {
                if entry.value().is_stale(timeout) {
                    to_remove.push(entry.key().clone());
                }
            }

            for id in to_remove {
                if let Some((_, device)) = devices.remove(&id) {
                    info!("Removed stale device: {} ({})", device.name, device.id);
                }
            }
        }
    }

    /// Discover UPnP/DLNA devices using SSDP
    async fn discover_upnp_devices(
        devices: Arc<DashMap<String, DiscoveredDevice>>,
        running: Arc<tokio::sync::RwLock<bool>>,
    ) {
        info!("Starting UPnP/SSDP discovery");

        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if !*running.read().await {
                break;
            }

            match Self::scan_upnp_devices().await {
                Ok(discovered) => {
                    for device in discovered {
                        let id = device.id.clone();
                        if let Some(mut existing) = devices.get_mut(&id) {
                            existing.update_last_seen();
                        } else {
                            info!("Discovered UPnP device: {} ({}:{})", device.name, device.ip, device.port);
                            devices.insert(id, device);
                        }
                    }
                }
                Err(e) => {
                    error!("UPnP discovery error: {}", e);
                }
            }
        }
    }

    /// Perform a single UPnP/SSDP scan
    async fn scan_upnp_devices() -> Result<Vec<DiscoveredDevice>> {
        use rupnp::ssdp;

        let mut discovered = Vec::new();

        // Search for root devices
        let search_target = SearchTarget::RootDevice;
        let timeout = Duration::from_secs(5);

        match ssdp::search(&search_target, timeout, 2, None).await {
            Ok(mut responses) => {
                while let Some(result) = responses.next().await {
                    match result {
                        Ok(response) => {
                            // Parse device info from SSDP response
                            let location = response.location();

                            // Extract IP and port from location URL
                            if let Ok(url) = url::Url::parse(location) {
                                if let Some(host) = url.host_str() {
                                    if let Ok(ip) = host.parse::<IpAddr>() {
                                        let port = url.port().unwrap_or_else(|| {
                                            if url.scheme() == "https" { 443 } else { 80 }
                                        });
                                        let device_type = response.search_target();
                                        let name = format!("UPnP Device at {}", host);
                                        let id = format!("upnp:{}:{}", ip, port);

                                        let mut device = DiscoveredDevice::new(
                                            id,
                                            name,
                                            DeviceType::Upnp,
                                            ip,
                                            port,
                                        );

                                        // Add metadata
                                        device.metadata = serde_json::json!({
                                            "location": location,
                                            "search_target": device_type.to_string(),
                                            "server": response.server(),
                                        });

                                        discovered.push(device);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("SSDP response error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("SSDP search failed: {}", e);
            }
        }

        // Also search for MediaRenderer devices (DLNA)
        let media_renderer_urn = URN::device("schemas-upnp-org", "MediaRenderer", 1);
        let search_target = SearchTarget::URN(media_renderer_urn);

        match ssdp::search(&search_target, timeout, 2, None).await {
            Ok(mut responses) => {
                while let Some(result) = responses.next().await {
                    match result {
                        Ok(response) => {
                            let location = response.location();

                            if let Ok(url) = url::Url::parse(location) {
                                if let Some(host) = url.host_str() {
                                    if let Ok(ip) = host.parse::<IpAddr>() {
                                        let port = url.port().unwrap_or_else(|| {
                                            if url.scheme() == "https" { 443 } else { 80 }
                                        });
                                        let name = format!("DLNA Renderer at {}", host);
                                        let id = format!("dlna:{}:{}", ip, port);

                                        let mut device = DiscoveredDevice::new(
                                            id,
                                            name,
                                            DeviceType::Dlna,
                                            ip,
                                            port,
                                        );

                                        device.capabilities.can_video = true;
                                        device.capabilities.can_audio = true;
                                        device.capabilities.protocols.push("dlna".to_string());

                                        device.metadata = serde_json::json!({
                                            "location": location,
                                            "device_type": "MediaRenderer",
                                            "server": response.server(),
                                        });

                                        discovered.push(device);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("DLNA response error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("DLNA MediaRenderer search failed: {}", e);
            }
        }

        Ok(discovered)
    }

    /// Stop device discovery
    pub async fn stop(&mut self) -> Result<()> {
        let mut running = self.discovery_running.write().await;
        if !*running {
            return Ok(());
        }

        *running = false;

        // Abort all spawned tasks
        for handle in self.tasks.drain(..) {
            handle.abort();
        }

        if let Some(mdns) = self.mdns.take() {
            mdns.shutdown()
                .map_err(|e| CasterError::Network(format!("Failed to shutdown mDNS: {}", e)))?;
        }

        info!("Device discovery stopped");
        Ok(())
    }

    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<DiscoveredDevice> {
        self.devices.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: &DeviceType) -> Vec<DiscoveredDevice> {
        self.devices
            .iter()
            .filter(|entry| &entry.value().device_type == device_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get a specific device by ID
    pub fn get_device(&self, id: &str) -> Option<DiscoveredDevice> {
        self.devices.get(id).map(|entry| entry.value().clone())
    }

    /// Check if discovery is running
    pub async fn is_running(&self) -> bool {
        *self.discovery_running.read().await
    }
}

impl Default for DeviceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
