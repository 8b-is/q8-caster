# Q8-Caster 🎬

AI-powered display casting MCP server with comprehensive media support. Cast anything from markdown presentations to 3D models, videos, and live streams to any display.

## Features

- **MCP Integration**: Full Model Context Protocol support for AI-driven display control
- **Multi-Display Support**: Control multiple displays, mirror screens, configure resolutions
- **Content Types**:
  - Markdown with live rendering (dark/light themes)
  - Video playback with hardware acceleration
  - Image display
  - 3D model visualization (GLTF)
  - Live streaming (RTSP, WebRTC, HLS, DASH)
  - Presentations
- **Network Receivers**: UPnP/DLNA, AirPlay, Chromecast support
- **Content Caching**: Smart caching with TTL support
- **Codec Detection**: Reports all available audio/video codecs
- **Audio Routing**: Control audio devices and routing

## Quick Start

```bash
# Build the project
./scripts/manage.sh build release

# Run as MCP server (stdio)
./scripts/manage.sh run

# Or run as daemon
./scripts/manage.sh daemon

# Check status
./scripts/manage.sh status
```

## MCP Tools

### cast_content
Cast content to a display.

```json
{
  "display_id": "display_0",
  "content_type": "markdown",
  "source": "/path/to/presentation.md",
  "options": {
    "theme": "dark"
  }
}
```

### list_displays
List all available displays with their properties.

### list_codecs
List all available audio/video codecs with hardware acceleration info.

### configure_display
Configure display settings (resolution, position, mirroring).

### start_receiver
Start network receivers (UPnP, AirPlay, Chromecast).

### cache_content
Cache content for faster access.

### discover_chromecasts
Discover available Chromecast devices on the network.

### connect_chromecast
Connect to a specific Chromecast device by name.

### cast_to_chromecast
Cast content directly to a Chromecast device.

```json
{
  "device_name": "Living Room TV",
  "content_type": "video",
  "source": "https://example.com/video.mp4"
}
```

### control_chromecast
Control Chromecast playback (play, pause, stop, seek, volume).

## Examples

### Cast a Markdown Presentation
```json
{
  "tool": "cast_content",
  "arguments": {
    "content_type": "markdown",
    "source": "presentation.md",
    "options": {
      "theme": "dark"
    }
  }
}
```

### Stream RTSP Camera
```json
{
  "tool": "cast_content",
  "arguments": {
    "content_type": "stream",
    "source": "rtsp://camera.local:554/stream",
    "options": {
      "protocol": "rtsp"
    }
  }
}
```

### Start AirPlay Receiver
```json
{
  "tool": "start_receiver",
  "arguments": {
    "protocols": ["airplay", "upnp"],
    "port": 8420
  }
}
```

### Cast to Chromecast
```json
// First discover devices
{
  "tool": "discover_chromecasts"
}

// Connect to a device
{
  "tool": "connect_chromecast",
  "arguments": {
    "device_name": "Living Room TV"
  }
}

// Cast content
{
  "tool": "cast_to_chromecast",
  "arguments": {
    "device_name": "Living Room TV",
    "content_type": "video",
    "source": "https://example.com/movie.mp4"
  }
}
```

## Architecture

- **Display Manager**: Handles display enumeration and window creation
- **Media Engine**: GStreamer-based media playback with codec detection
- **Render Engine**: GPU-accelerated rendering for markdown and 3D content
- **Network Receiver**: Multi-protocol network discovery and streaming
- **Content Cache**: LRU memory cache with disk persistence

## Requirements

- Rust 1.70+
- GStreamer 1.0
- X11/Wayland (for display control)

## License

MIT - Made with 💜 by 8b-is