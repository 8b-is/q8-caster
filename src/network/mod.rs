pub mod chromecast_simple;

use mdns_sd::{ServiceDaemon, ServiceInfo};
// Removed unused imports - SearchTarget and URN were just window shopping here!
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

use crate::{Result, CasterError};
use self::chromecast_simple::ChromecastManager;

pub struct NetworkReceiver {
    mdns: Option<ServiceDaemon>,
    tcp_listener: Option<TcpListener>,
    protocols: Vec<String>,
    chromecast_manager: ChromecastManager,
}

impl NetworkReceiver {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            mdns: None,
            tcp_listener: None,
            protocols: Vec::new(),
            chromecast_manager: ChromecastManager::new(),
        })
    }
    
    pub async fn start(&mut self, protocols: Vec<String>, port: u16) -> Result<()> {
        self.protocols = protocols.clone();
        
        // Start TCP listener
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await
            .map_err(|e| CasterError::Network(format!("Failed to bind to port {}: {}", port, e)))?;
        
        self.tcp_listener = Some(listener);
        
        // Start mDNS for service discovery
        let mdns = ServiceDaemon::new()
            .map_err(|e| CasterError::Network(format!("Failed to create mDNS daemon: {}", e)))?;
        
        for protocol in &protocols {
            match protocol.as_str() {
                "airplay" => self.register_airplay(&mdns, port)?,
                "upnp" => self.register_upnp(&mdns, port).await?,
                "chromecast" => self.register_chromecast(&mdns, port)?,
                _ => {}
            }
        }
        
        self.mdns = Some(mdns);
        
        info!("Network receiver started on port {} with protocols: {:?}", port, protocols);
        
        Ok(())
    }
    
    fn register_airplay(&self, mdns: &ServiceDaemon, port: u16) -> Result<()> {
        let service_info = ServiceInfo::new(
            "_airplay._tcp.local.",
            "q8-caster",
            &format!("q8-caster.local."),
            "127.0.0.1",
            port,
            None,
        ).map_err(|e| CasterError::Network(format!("Failed to create AirPlay service: {:?}", e)))?;
        
        mdns.register(service_info)
            .map_err(|e| CasterError::Network(format!("Failed to register AirPlay service: {}", e)))?;
        
        info!("Registered AirPlay service on port {}", port);
        
        Ok(())
    }
    
    async fn register_upnp(&self, mdns: &ServiceDaemon, port: u16) -> Result<()> {
        // Register UPnP/DLNA service
        let service_info = ServiceInfo::new(
            "_dlna._tcp.local.",
            "q8-caster",
            &format!("q8-caster.local."),
            "127.0.0.1",
            port,
            None,
        ).map_err(|e| CasterError::Network(format!("Failed to create UPnP service: {:?}", e)))?;
        
        mdns.register(service_info)
            .map_err(|e| CasterError::Network(format!("Failed to register UPnP service: {}", e)))?;
        
        info!("Registered UPnP/DLNA service on port {}", port);
        
        Ok(())
    }
    
    fn register_chromecast(&self, mdns: &ServiceDaemon, port: u16) -> Result<()> {
        let service_info = ServiceInfo::new(
            "_googlecast._tcp.local.",
            "q8-caster",
            &format!("q8-caster.local."),
            "127.0.0.1",
            port,
            None,
        ).map_err(|e| CasterError::Network(format!("Failed to create Chromecast service: {:?}", e)))?;
        
        mdns.register(service_info)
            .map_err(|e| CasterError::Network(format!("Failed to register Chromecast service: {}", e)))?;
        
        info!("Registered Chromecast service on port {}", port);
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mdns) = self.mdns.take() {
            mdns.shutdown()
                .map_err(|e| CasterError::Network(format!("Failed to shutdown mDNS: {}", e)))?;
        }
        
        self.tcp_listener = None;
        self.protocols.clear();
        
        Ok(())
    }
    
    // Chromecast-specific methods
    pub async fn discover_chromecasts(&mut self) -> Result<Vec<serde_json::Value>> {
        // Discovering devices - they're shy but we'll find them!
        let _devices = self.chromecast_manager.discover_devices().await?;
        Ok(self.chromecast_manager.list_devices())
    }
    
    pub async fn connect_chromecast(&mut self, device_name: &str) -> Result<()> {
        self.chromecast_manager.connect_to_device(device_name).await
    }
    
    pub async fn cast_to_chromecast(
        &mut self,
        device_name: &str,
        content_type: &crate::ContentType,
        source: &crate::ContentSource,
    ) -> Result<()> {
        self.chromecast_manager.cast_content(device_name, content_type, source).await
    }
    
    pub async fn stop_chromecast(&mut self, device_name: &str) -> Result<()> {
        self.chromecast_manager.stop_casting(device_name).await
    }
    
    pub async fn get_chromecast_status(&self, device_name: &str) -> Result<serde_json::Value> {
        self.chromecast_manager.get_device_status(device_name).await
    }
}