#!/bin/bash

# Test script for q8-caster MCP server

echo "Testing q8-caster MCP server..."

# Function to send request and pretty print response
send_request() {
    local method=$1
    local params=$2
    local id=$3
    
    echo -e "\n=== Testing: $method ==="
    echo "{\"jsonrpc\": \"2.0\", \"method\": \"$method\", \"params\": $params, \"id\": $id}" | \
        cargo run --quiet 2>/dev/null | \
        jq '.'
}

# Initialize
send_request "initialize" "{}" 1

# List tools
send_request "tools/list" "{}" 2

# List displays
send_request "tools/call" '{"name": "list_displays", "arguments": {}}' 3

# List codecs
send_request "tools/call" '{"name": "list_codecs", "arguments": {}}' 4

# Discover Chromecasts
send_request "tools/call" '{"name": "discover_chromecasts", "arguments": {}}' 5

echo -e "\n✅ MCP server test complete!"