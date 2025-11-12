use std::sync::Arc;
use serde_json::{json, Value};
use tracing::info;

use crate::mcp::server::McpServer;
use crate::{ContentType, ContentSource, StreamProtocol};

pub async fn cast_content_handler(_server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let display_id = args["display_id"].as_str().map(|s| s.to_string());
    let content_type = args["content_type"].as_str().unwrap_or("");
    let source = args["source"].as_str().unwrap_or("");
    let options = &args["options"];
    
    info!("Casting {} to display {:?}", content_type, display_id);
    
    let content_type = match content_type {
        "markdown" => ContentType::Markdown { 
            theme: options["theme"].as_str().map(|s| s.to_string()) 
        },
        "video" => ContentType::Video { 
            codec: options["codec"].as_str().unwrap_or("auto").to_string(),
            container: options["container"].as_str().unwrap_or("auto").to_string()
        },
        "image" => ContentType::Image { 
            format: options["format"].as_str().unwrap_or("auto").to_string()
        },
        "model3d" => ContentType::Model3D { 
            format: options["format"].as_str().unwrap_or("gltf").to_string()
        },
        "stream" => {
            let protocol = options["protocol"].as_str().unwrap_or("rtsp");
            match protocol {
                "rtsp" => ContentType::Stream { 
                    protocol: StreamProtocol::Rtsp { url: source.to_string() }
                },
                "webrtc" => ContentType::Stream { 
                    protocol: StreamProtocol::WebRtc { offer: options["offer"].as_str().unwrap_or("").to_string() }
                },
                "hls" => ContentType::Stream { 
                    protocol: StreamProtocol::Hls { manifest_url: source.to_string() }
                },
                "dash" => ContentType::Stream { 
                    protocol: StreamProtocol::Dash { manifest_url: source.to_string() }
                },
                _ => return Ok(json!({"error": format!("Unknown stream protocol: {}", protocol)}))
            }
        },
        "presentation" => ContentType::Presentation { 
            format: options["format"].as_str().unwrap_or("auto").to_string()
        },
        _ => return Ok(json!({"error": format!("Unknown content type: {}", content_type)}))
    };
    
    let content_source = if source.starts_with("http://") || source.starts_with("https://") {
        ContentSource::Url { url: source.to_string() }
    } else if source.starts_with("cache://") {
        ContentSource::Cache { key: source.strip_prefix("cache://").unwrap_or("").to_string() }
    } else {
        ContentSource::File { path: source.to_string() }
    };
    
    // TODO: Implement actual casting logic
    let session_id = uuid::Uuid::new_v4();
    
    Ok(json!({
        "success": true,
        "session_id": session_id.to_string(),
        "display_id": display_id,
        "content_type": content_type,
        "source": content_source
    }))
}

pub async fn list_displays_handler(server: Arc<McpServer>, _args: &Value) -> jsonrpc_core::Result<Value> {
    let display_manager = server.display_manager.read().await;
    let displays = display_manager.list_displays().await.unwrap_or_default();
    
    Ok(json!({
        "displays": displays
    }))
}

pub async fn list_codecs_handler(server: Arc<McpServer>, _args: &Value) -> jsonrpc_core::Result<Value> {
    let media_engine = server.media_engine.read().await;
    let codecs = media_engine.list_codecs().unwrap_or_default();
    
    Ok(json!({
        "codecs": codecs
    }))
}

pub async fn list_audio_devices_handler(server: Arc<McpServer>, _args: &Value) -> jsonrpc_core::Result<Value> {
    let media_engine = server.media_engine.read().await;
    let devices = media_engine.list_audio_devices().unwrap_or_default();
    
    Ok(json!({
        "audio_devices": devices
    }))
}

pub async fn configure_display_handler(_server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let display_id = args["display_id"].as_str().unwrap_or("");
    let _resolution = &args["resolution"];
    let _position = &args["position"];
    let _mirror = args["mirror"].as_str();
    
    info!("Configuring display {}", display_id);
    
    // TODO: Implement display configuration
    
    Ok(json!({
        "success": true,
        "display_id": display_id
    }))
}

pub async fn start_receiver_handler(server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let protocols = args["protocols"].as_array()
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        )
        .unwrap_or_else(|| vec!["upnp".to_string(), "airplay".to_string()]);
    
    let port = args["port"].as_u64().unwrap_or(8420) as u16;
    
    info!("Starting receivers: {:?} on port {}", protocols, port);
    
    let mut network_receiver = server.network_receiver.write().await;
    network_receiver.start(protocols, port).await.unwrap();
    
    Ok(json!({
        "success": true,
        "port": port
    }))
}

pub async fn stop_cast_handler(_server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let display_id = args["display_id"].as_str();
    
    info!("Stopping cast on display {:?}", display_id);
    
    // TODO: Implement stop casting
    
    Ok(json!({
        "success": true,
        "display_id": display_id
    }))
}

pub async fn cache_content_handler(server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let key = args["key"].as_str().unwrap_or("");
    let source = args["source"].as_str().unwrap_or("");
    let ttl = args["ttl"].as_u64().map(|t| std::time::Duration::from_secs(t));
    
    info!("Caching content: {} -> {}", source, key);
    
    let mut cache = server.content_cache.write().await;
    cache.store(key.to_string(), source.to_string(), ttl).await.unwrap();
    
    Ok(json!({
        "success": true,
        "key": key
    }))
}

pub async fn get_cast_status_handler(_server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let display_id = args["display_id"].as_str();
    
    // TODO: Implement status retrieval
    
    Ok(json!({
        "display_id": display_id,
        "active": false,
        "session": null
    }))
}

pub async fn discover_chromecasts_handler(server: Arc<McpServer>, _args: &Value) -> jsonrpc_core::Result<Value> {
    info!("Discovering Chromecast devices...");
    
    let mut network_receiver = server.network_receiver.write().await;
    let devices = network_receiver.discover_chromecasts().await.unwrap_or_default();
    
    Ok(json!({
        "devices": devices
    }))
}

pub async fn connect_chromecast_handler(server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let device_name = args["device_name"].as_str().unwrap_or("");
    
    info!("Connecting to Chromecast: {}", device_name);
    
    let mut network_receiver = server.network_receiver.write().await;
    match network_receiver.connect_chromecast(device_name).await {
        Ok(_) => Ok(json!({
            "success": true,
            "device_name": device_name
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

pub async fn cast_to_chromecast_handler(server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let device_name = args["device_name"].as_str().unwrap_or("");
    let content_type = args["content_type"].as_str().unwrap_or("");
    let source = args["source"].as_str().unwrap_or("");
    let options = &args["options"];
    
    info!("Casting to Chromecast {} - type: {}", device_name, content_type);
    
    let content_type = match content_type {
        "video" => ContentType::Video { 
            codec: options["codec"].as_str().unwrap_or("auto").to_string(),
            container: options["container"].as_str().unwrap_or("auto").to_string()
        },
        "image" => ContentType::Image { 
            format: options["format"].as_str().unwrap_or("auto").to_string()
        },
        "stream" => {
            let protocol = options["protocol"].as_str().unwrap_or("hls");
            match protocol {
                "hls" => ContentType::Stream { 
                    protocol: StreamProtocol::Hls { manifest_url: source.to_string() }
                },
                "dash" => ContentType::Stream { 
                    protocol: StreamProtocol::Dash { manifest_url: source.to_string() }
                },
                _ => return Ok(json!({"error": format!("Unsupported stream protocol for Chromecast: {}", protocol)}))
            }
        },
        _ => return Ok(json!({"error": format!("Unsupported content type for Chromecast: {}", content_type)}))
    };
    
    let content_source = if source.starts_with("http://") || source.starts_with("https://") {
        ContentSource::Url { url: source.to_string() }
    } else {
        ContentSource::File { path: source.to_string() }
    };
    
    let mut network_receiver = server.network_receiver.write().await;
    match network_receiver.cast_to_chromecast(device_name, &content_type, &content_source).await {
        Ok(_) => Ok(json!({
            "success": true,
            "device_name": device_name,
            "content_type": content_type,
            "source": content_source
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

pub async fn control_chromecast_handler(server: Arc<McpServer>, args: &Value) -> jsonrpc_core::Result<Value> {
    let device_name = args["device_name"].as_str().unwrap_or("");
    let action = args["action"].as_str().unwrap_or("");
    let _value = args["value"].as_f64();
    
    info!("Controlling Chromecast {} - action: {}", device_name, action);
    
    match action {
        "stop" => {
            let mut network_receiver = server.network_receiver.write().await;
            match network_receiver.stop_chromecast(device_name).await {
                Ok(_) => Ok(json!({"success": true})),
                Err(e) => Ok(json!({"success": false, "error": e.to_string()}))
            }
        },
        "play" | "pause" | "seek" | "volume" => {
            // TODO: Implement playback controls
            Ok(json!({
                "success": false,
                "error": "Playback controls not yet implemented"
            }))
        },
        _ => Ok(json!({
            "success": false,
            "error": format!("Unknown action: {}", action)
        }))
    }
}