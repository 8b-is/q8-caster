use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_stdio_server::ServerBuilder;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::Result;
use crate::display::DisplayManager;
use crate::media::MediaEngine;
use crate::render::RenderEngine;
use crate::network::NetworkReceiver;
use crate::cache::ContentCache;

use super::handlers::*;

pub struct McpServer {
    pub display_manager: Arc<RwLock<DisplayManager>>,
    pub media_engine: Arc<RwLock<MediaEngine>>,
    pub render_engine: Arc<RwLock<RenderEngine>>,
    pub network_receiver: Arc<RwLock<NetworkReceiver>>,
    pub content_cache: Arc<RwLock<ContentCache>>,
}

impl McpServer {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            display_manager: Arc::new(RwLock::new(DisplayManager::new().await?)),
            media_engine: Arc::new(RwLock::new(MediaEngine::new()?)),
            render_engine: Arc::new(RwLock::new(RenderEngine::new().await?)),
            network_receiver: Arc::new(RwLock::new(NetworkReceiver::new().await?)),
            content_cache: Arc::new(RwLock::new(ContentCache::new()?)),
        })
    }

    pub async fn run(self) -> Result<()> {
        let mut io = IoHandler::new();
        
        // Initialize MCP
        io.add_method("initialize", |_params: Params| async {
            debug!("Received initialize request");
            Ok(json!({
                "protocolVersion": "0.3.0",
                "serverInfo": {
                    "name": "q8-caster",
                    "version": env!("CARGO_PKG_VERSION"),
                    "description": "AI-powered display casting MCP server"
                },
                "capabilities": {
                    "tools": {
                        "listTools": {},
                        "describeTool": {}
                    },
                    "resources": {
                        "listResources": {},
                        "describeResource": {},
                        "readResource": {}
                    }
                }
            }))
        });

        // List available tools
        io.add_method("tools/list", |_params: Params| async {
            Ok(json!({
                "tools": [
                    {
                        "name": "cast_content",
                        "description": "Cast content to a display",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "display_id": {"type": "string", "description": "Target display ID (use list_displays to get IDs)"},
                                "content_type": {"type": "string", "enum": ["markdown", "video", "image", "model3d", "stream", "presentation"]},
                                "source": {"type": "string", "description": "File path, URL, or cache key"},
                                "options": {"type": "object", "description": "Type-specific options"}
                            },
                            "required": ["content_type", "source"]
                        }
                    },
                    {
                        "name": "list_displays",
                        "description": "List all available displays",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "list_codecs",
                        "description": "List all available codecs",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "list_audio_devices",
                        "description": "List all audio devices",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "configure_display",
                        "description": "Configure display settings",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "display_id": {"type": "string"},
                                "resolution": {"type": "object", "properties": {"width": {"type": "number"}, "height": {"type": "number"}}},
                                "position": {"type": "object", "properties": {"x": {"type": "number"}, "y": {"type": "number"}}},
                                "mirror": {"type": "string", "description": "Display ID to mirror"}
                            },
                            "required": ["display_id"]
                        }
                    },
                    {
                        "name": "start_receiver",
                        "description": "Start UPnP/AirPlay receiver",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "protocols": {"type": "array", "items": {"type": "string", "enum": ["upnp", "airplay", "chromecast"]}},
                                "port": {"type": "number"}
                            }
                        }
                    },
                    {
                        "name": "stop_cast",
                        "description": "Stop casting on a display",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "display_id": {"type": "string"}
                            }
                        }
                    },
                    {
                        "name": "cache_content",
                        "description": "Cache content for later use",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "key": {"type": "string"},
                                "source": {"type": "string"},
                                "ttl": {"type": "number", "description": "Time to live in seconds"}
                            },
                            "required": ["key", "source"]
                        }
                    },
                    {
                        "name": "get_cast_status",
                        "description": "Get current casting status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "display_id": {"type": "string"}
                            }
                        }
                    },
                    {
                        "name": "discover_chromecasts",
                        "description": "Discover available Chromecast devices",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "connect_chromecast",
                        "description": "Connect to a Chromecast device",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "device_name": {"type": "string", "description": "Name of the Chromecast device"}
                            },
                            "required": ["device_name"]
                        }
                    },
                    {
                        "name": "cast_to_chromecast",
                        "description": "Cast content to a Chromecast device",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "device_name": {"type": "string", "description": "Name of the Chromecast device"},
                                "content_type": {"type": "string", "enum": ["video", "image", "stream"]},
                                "source": {"type": "string", "description": "URL or file path of content"},
                                "options": {"type": "object", "description": "Type-specific options"}
                            },
                            "required": ["device_name", "content_type", "source"]
                        }
                    },
                    {
                        "name": "control_chromecast",
                        "description": "Control Chromecast playback",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "device_name": {"type": "string"},
                                "action": {"type": "string", "enum": ["play", "pause", "stop", "seek", "volume"]},
                                "value": {"type": "number", "description": "Value for seek (seconds) or volume (0-1)"}
                            },
                            "required": ["device_name", "action"]
                        }
                    },
                    {
                        "name": "discover_devices",
                        "description": "Discover all network devices (Chromecast, FireTV, AirPlay, DLNA, UPnP) via mDNS, Bonjour, and SSDP",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "device_type": {
                                    "type": "string",
                                    "enum": ["chromecast", "firetv", "fire_tv", "airplay", "dlna", "upnp", "miracast"],
                                    "description": "Optional: Filter by device type. If not specified, returns all discovered devices."
                                }
                            }
                        }
                    },
                    {
                        "name": "get_device",
                        "description": "Get detailed information about a specific discovered device",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "device_id": {
                                    "type": "string",
                                    "description": "The ID of the device to retrieve"
                                }
                            },
                            "required": ["device_id"]
                        }
                    },
                    {
                        "name": "discovery_status",
                        "description": "Get the current status of device discovery",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                ]
            }))
        });

        // Tool execution
        let server = Arc::new(self);
        let server_clone = Arc::clone(&server);
        
        io.add_method("tools/call", move |params: Params| {
            let server = Arc::clone(&server_clone);
            async move {
                let params = params.parse::<Value>().unwrap_or_default();
                let tool_name = params["name"].as_str().unwrap_or("");
                let arguments = &params["arguments"];
                
                match tool_name {
                    "cast_content" => cast_content_handler(server, arguments).await,
                    "list_displays" => list_displays_handler(server, arguments).await,
                    "list_codecs" => list_codecs_handler(server, arguments).await,
                    "list_audio_devices" => list_audio_devices_handler(server, arguments).await,
                    "configure_display" => configure_display_handler(server, arguments).await,
                    "start_receiver" => start_receiver_handler(server, arguments).await,
                    "stop_cast" => stop_cast_handler(server, arguments).await,
                    "cache_content" => cache_content_handler(server, arguments).await,
                    "get_cast_status" => get_cast_status_handler(server, arguments).await,
                    "discover_chromecasts" => discover_chromecasts_handler(server, arguments).await,
                    "connect_chromecast" => connect_chromecast_handler(server, arguments).await,
                    "cast_to_chromecast" => cast_to_chromecast_handler(server, arguments).await,
                    "control_chromecast" => control_chromecast_handler(server, arguments).await,
                    "discover_devices" => discover_devices_handler(server, arguments).await,
                    "get_device" => get_device_handler(server, arguments).await,
                    "discovery_status" => discovery_status_handler(server, arguments).await,
                    _ => Ok(json!({"error": format!("Unknown tool: {}", tool_name)}))
                }
            }
        });

        // Resources
        io.add_method("resources/list", |_params: Params| async {
            Ok(json!({
                "resources": [
                    {
                        "uri": "display://status",
                        "name": "Display Status",
                        "description": "Current status of all displays",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "cache://list",
                        "name": "Cache Contents",
                        "description": "List of cached content",
                        "mimeType": "application/json"
                    }
                ]
            }))
        });

        info!("Starting q8-caster MCP server on stdio");
        debug!("Server is ready to accept JSON-RPC requests on stdin");
        
        // Build and run the server - it will handle stdio until EOF
        ServerBuilder::new(io)
            .build()
            .await;
        
        info!("MCP server has shut down");
        Ok(())
    }
}