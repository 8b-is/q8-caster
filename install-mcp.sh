#!/bin/bash

# Q8-Caster MCP Installation Script

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BINARY_PATH="$SCRIPT_DIR/target/release/q8-caster"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Q8-Caster MCP Configuration Helper${NC}"
echo "===================================="

# Detect OS and config path
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/Claude"
    CONFIG_PATH="$CONFIG_DIR/claude_desktop_config.json"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    CONFIG_DIR="$APPDATA/Claude"
    CONFIG_PATH="$CONFIG_DIR/claude_desktop_config.json"
else
    CONFIG_DIR="$HOME/.config/Claude"
    CONFIG_PATH="$CONFIG_DIR/claude_desktop_config.json"
fi

echo -e "Detected OS: ${YELLOW}$OSTYPE${NC}"
echo -e "Config path: ${YELLOW}$CONFIG_PATH${NC}"
echo

# Check if release binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Release binary not found. Building...${NC}"
    cd "$SCRIPT_DIR"
    ./scripts/manage.sh build release
fi

# Create config directory if it doesn't exist
mkdir -p "$CONFIG_DIR"

# Generate the MCP server configuration
MCP_CONFIG='{
  "q8-caster": {
    "command": "'$BINARY_PATH'",
    "args": [],
    "env": {
      "RUST_LOG": "info"
    }
  }
}'

# Check if config file exists
if [ -f "$CONFIG_PATH" ]; then
    echo -e "${YELLOW}Existing Claude config found.${NC}"
    
    # Backup existing config
    cp "$CONFIG_PATH" "$CONFIG_PATH.backup"
    echo -e "Backup created: ${GREEN}$CONFIG_PATH.backup${NC}"
    
    # Check if mcpServers section exists
    if grep -q '"mcpServers"' "$CONFIG_PATH"; then
        echo -e "\n${YELLOW}Manual step required:${NC}"
        echo "Add this to the 'mcpServers' section in $CONFIG_PATH:"
        echo
        echo "$MCP_CONFIG"
        echo
        echo "The file has been opened in your editor..."
        ${EDITOR:-nano} "$CONFIG_PATH"
    else
        # Add mcpServers section
        echo -e "${GREEN}Adding mcpServers section...${NC}"
        # This is tricky with JSON, so we'll just show the user what to add
        echo -e "\n${YELLOW}Please add this to your config file:${NC}"
        echo
        cat << EOF
{
  "mcpServers": $MCP_CONFIG
}
EOF
        echo
        echo "Opening config file..."
        ${EDITOR:-nano} "$CONFIG_PATH"
    fi
else
    # Create new config
    echo -e "${GREEN}Creating new Claude config...${NC}"
    cat > "$CONFIG_PATH" << EOF
{
  "mcpServers": $MCP_CONFIG
}
EOF
    echo -e "${GREEN}✅ Configuration created!${NC}"
fi

echo
echo -e "${GREEN}Installation complete!${NC}"
echo
echo "Next steps:"
echo "1. Restart Claude Desktop"
echo "2. Look for 'q8-caster' in the MCP tools"
echo "3. Try commands like 'List all displays' or 'Discover Chromecast devices'"
echo
echo -e "${YELLOW}Testing the MCP server...${NC}"
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {}, "id": 1}' | "$BINARY_PATH" | jq '.result.serverInfo' 2>/dev/null || {
    echo -e "${RED}Server test failed. Please check the installation.${NC}"
    exit 1
}
echo -e "${GREEN}✅ Server is working!${NC}"