# MCP Configuration for Q8-Caster

## Installation Steps

### 1. Build the Release Binary
```bash
cd /home/hue/source/q8-caster
./scripts/manage.sh build release
```

### 2. Add to Claude Desktop Configuration

The MCP configuration needs to be added to your Claude Desktop settings. The location depends on your OS:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "q8-caster": {
      "command": "/home/hue/source/q8-caster/target/release/q8-caster",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 3. Alternative: Using the manage.sh Script

You can also configure it to use the management script:

```json
{
  "mcpServers": {
    "q8-caster": {
      "command": "/home/hue/source/q8-caster/scripts/manage.sh",
      "args": ["run"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 4. For Development (Debug Mode)

If you want to run in debug mode with more logging:

```json
{
  "mcpServers": {
    "q8-caster": {
      "command": "/usr/bin/cargo",
      "args": ["run", "--manifest-path", "/home/hue/source/q8-caster/Cargo.toml"],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

## Verifying the Installation

1. Restart Claude Desktop after updating the configuration
2. In Claude, you should see q8-caster available as an MCP tool
3. Try using commands like:
   - "List all available displays"
   - "Show me the available codecs"
   - "Discover Chromecast devices"

## Available Tools

Once configured, these tools will be available:

- `list_displays` - List all available displays
- `list_codecs` - List available audio/video codecs
- `list_audio_devices` - List audio devices
- `configure_display` - Configure display settings
- `start_receiver` - Start network receivers (UPnP, AirPlay)
- `cast_content` - Cast content to displays
- `stop_cast` - Stop casting
- `cache_content` - Cache content for later use
- `get_cast_status` - Get casting status
- `discover_chromecasts` - Find Chromecast devices
- `connect_chromecast` - Connect to a Chromecast
- `cast_to_chromecast` - Cast to a Chromecast device
- `control_chromecast` - Control Chromecast playback

## Troubleshooting

If the MCP server doesn't appear in Claude:

1. Check that the binary exists and is executable:
   ```bash
   ls -la /home/hue/source/q8-caster/target/release/q8-caster
   ```

2. Test the server manually:
   ```bash
   echo '{"jsonrpc": "2.0", "method": "initialize", "params": {}, "id": 1}' | /home/hue/source/q8-caster/target/release/q8-caster
   ```

3. Check Claude Desktop logs for any errors

4. Ensure all dependencies are installed:
   ```bash
   ./scripts/manage.sh install-deps
   ```