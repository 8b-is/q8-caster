# Q8 Caster - Enhanced Casting Features

## Overview

This document outlines the comprehensive casting features that have been implemented in Q8 Caster, including multiple content types, rendering engines, and a modern egui-based display interface.

## New Content Types Added

### 1. **Audio Casting** (`ContentType::Audio`)
- Supports multiple audio codecs (opus, aac, mp3, flac)
- Format-aware playback
- Audio visualization with waveform and spectrum rendering
- Volume control and playback controls

**Usage:**
```json
{
  "content_type": "audio",
  "source": "path/to/audio.mp3",
  "options": {
    "codec": "mp3",
    "format": "mp3"
  }
}
```

### 2. **PDF Casting** (`ContentType::Pdf`)
- Page-by-page PDF rendering
- High-quality rendering at 2x resolution
- Page navigation support
- Powered by pdfium-render

**Usage:**
```json
{
  "content_type": "pdf",
  "source": "path/to/document.pdf",
  "options": {
    "page": 1
  }
}
```

### 3. **Screen Mirroring** (`ContentType::ScreenMirror`)
- Real-time screen capture
- Multiple quality levels: Low (720p@30fps), Medium (1080p@30fps), High (1080p@60fps), Ultra (4K@60fps)
- Source display selection
- Cross-platform support via xcap

**Usage:**
```json
{
  "content_type": "screen_mirror",
  "source": "screen://primary",
  "options": {
    "quality": "high",
    "source_display": "display_0"
  }
}
```

### 4. **WebAssembly Casting** (`ContentType::WebAssembly`)
- Execute WebAssembly modules
- WASI support for file system access
- Custom entry point selection
- Module URL support

**Usage:**
```json
{
  "content_type": "webassembly",
  "source": "https://example.com/module.wasm",
  "options": {
    "entry_point": "main"
  }
}
```

## Existing Content Types (Enhanced)

### 5. **Markdown** (with themes)
- Dark and light theme support
- CommonMark extensions
- Syntax highlighting ready

### 6. **Video**
- Multiple codec support (h264, h265, vp8, vp9, av1)
- Container format detection

### 7. **Image**
- Format-aware rendering
- Multiple format support

### 8. **3D Models**
- GLTF format support (planned)

### 9. **Streaming**
- RTSP, WebRTC, HLS, DASH protocol support
- Live streaming capabilities

### 10. **Presentations**
- Multiple presentation format support

## egui-based Display Window

### Features:
- **Modern UI Framework**: Built with egui for cross-platform compatibility
- **GPU-accelerated Rendering**: Using wgpu for efficient rendering
- **Playback Controls**:
  - Play/Pause/Stop buttons
  - Previous/Next track
  - Volume slider with visual indicator
  - Seek bar with time display
- **Content-specific Rendering**:
  - Markdown: Scrollable text view
  - Audio: Waveform visualization
  - Video: Video playback surface
  - PDF: Page rendering
  - Screen Mirror: Live screen feed
  - WebAssembly: Module execution output

### Window Features:
- Resizable window (default: 1280x720)
- Real-time content updates
- Smooth animations
- Low-latency input handling

## Rendering Engines

### 1. PDF Renderer (`src/render/pdf.rs`)
- Pdfium-based PDF rendering
- Page count detection
- High-quality bitmap generation
- Configurable DPI/resolution

### 2. Audio Renderer (`src/render/audio.rs`)
- **Waveform Visualization**: Time-domain representation
- **Spectrum Analyzer**: Frequency-domain visualization with gradient colors
- **Level Meter**: Circular audio level indicator with color coding (green/yellow/red)
- Customizable dimensions

### 3. Screen Mirror (`src/render/mirror.rs`)
- Cross-platform screen capture using xcap
- Monitor enumeration and selection
- Real-time frame capture
- Primary display detection

### 4. WebAssembly Runner (`src/render/wasm.rs`)
- WebAssembly module execution (placeholder - ready for wasmer/wasmtime integration)
- WASI environment support
- Memory inspection
- Custom function invocation

## Content Cache System (`src/cache/mod.rs`)

### Features:
- **LRU Eviction**: Automatic cache management with Least Recently Used eviction
- **Dual Storage**: In-memory cache for fast access + disk persistence
- **Configurable Size**: Default 500MB, customizable
- **Smart Eviction**: Frees space automatically when needed
- **Cache Statistics**: Track memory items, disk items, and total size

### Usage:
```rust
let cache = ContentCache::new()?; // Uses default config
let cache = ContentCache::with_config(cache_dir, 1000)?; // 1GB custom

let id = cache.store(content_type, source, data, mime_type).await?;
let content = cache.get(&id).await?;
cache.remove(&id).await?;
let stats = cache.stats();
```

## MCP Handler Updates

All MCP handlers (`src/mcp/handlers.rs`) have been updated to support the new content types:

### New Content Type Parsers:
- Audio: codec and format options
- PDF: page number option
- Screen Mirror: quality and source display options
- WebAssembly: entry point option

### Quality Levels for Screen Mirroring:
```rust
pub enum MirrorQuality {
    Low,      // 720p @ 30fps
    Medium,   // 1080p @ 30fps
    High,     // 1080p @ 60fps
    Ultra,    // 4K @ 60fps
}
```

## Architecture Improvements

### Module Structure:
```
src/
├── cache/          # Content caching with LRU eviction
├── display/        # Display management + egui window
│   └── window.rs   # Full egui implementation
├── mcp/            # MCP server and handlers
├── media/          # Media engine (codec detection)
├── network/        # Network receivers (Chromecast, UPnP, AirPlay)
├── render/         # Content rendering engines
│   ├── pdf.rs      # PDF rendering
│   ├── audio.rs    # Audio visualization
│   ├── mirror.rs   # Screen capture
│   ├── wasm.rs     # WebAssembly execution
│   └── themes/     # CSS themes for markdown
└── server/         # HTTP REST API

```

### Dependencies Added:
- **UI**: `egui`, `egui-wgpu`, `egui-winit`, `wgpu`, `winit`, `pollster`
- **PDF**: `pdf`, `pdfium-render`
- **Screen Capture**: `xcap`, `scrap`
- **Caching**: Already had `lru`, `dashmap`

### Dependencies Commented (Pending Integration):
- **WebAssembly**: `wasmer`, `wasmer-wasi`, `wat` (needs API update)
- **Media**: `gstreamer` family (requires system libraries)
- **3D Rendering**: `bevy` (deferred for future)

## Example MCP Tool Calls

### Cast Audio File:
```json
{
  "name": "cast_content",
  "arguments": {
    "display_id": "display_0",
    "content_type": "audio",
    "source": "/music/song.mp3",
    "options": {
      "codec": "mp3"
    }
  }
}
```

### Cast PDF Document:
```json
{
  "name": "cast_content",
  "arguments": {
    "content_type": "pdf",
    "source": "/documents/report.pdf",
    "options": {
      "page": 5
    }
  }
}
```

### Start Screen Mirroring:
```json
{
  "name": "cast_content",
  "arguments": {
    "content_type": "screen_mirror",
    "source": "screen://primary",
    "options": {
      "quality": "high"
    }
  }
}
```

### Cast WebAssembly Module:
```json
{
  "name": "cast_content",
  "arguments": {
    "content_type": "wasm",
    "source": "https://example.com/app.wasm",
    "options": {
      "entry_point": "run"
    }
  }
}
```

## Future Enhancements

### Planned:
1. **Full WebAssembly Support**: Complete wasmer/wasmtime integration
2. **GStreamer Integration**: Full media playback when system libraries available
3. **3D Model Rendering**: Bevy-based GLTF viewer
4. **Session Management**: Track and manage active casting sessions
5. **Chromecast Playback Controls**: Play/pause/seek/volume
6. **Enhanced Caching**: TTL support, bandwidth-aware fetching
7. **Audio Analysis**: Real-time FFT for spectrum analysis
8. **Multi-display Support**: Simultaneous casting to multiple displays

## Building

```bash
# Build the project
cargo build --release

# Run MCP server
cargo run --release

# Run with HTTP server
cargo run --release -- --http

# Run egui display window
cargo run --release -- --window
```

## Notes

- The egui implementation provides a cross-platform UI that works on Linux, macOS, and Windows
- All rendering is GPU-accelerated via wgpu
- The architecture is extensible for adding new content types
- Modular design allows independent development of rendering engines
- Ready for WebAssembly target compilation (egui supports WASM)

## WebAssembly Compatibility

The egui framework used for the display window is fully compatible with WebAssembly, meaning the Q8 Caster UI could be compiled to run in a web browser. This aligns perfectly with the WebAssembly casting feature!

```bash
# Future: Compile to WASM
# cargo build --target wasm32-unknown-unknown
```

---

**Project:** q8-caster
**Author:** 8b-is
**License:** MIT
**Repository:** https://github.com/8b-is/q8-caster
