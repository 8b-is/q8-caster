#!/bin/bash

# Q8-Caster Management Script
# Because AI needs a way to cast spells on displays 🪄

set -e

# Colors for our magical output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# ASCII Art because we're fancy
print_banner() {
    echo -e "${CYAN}"
    cat << "EOF"
     ___    ___        ____    _    ____ _____ _____ ____  
    / _ \  ( _ )      / ___|  / \  / ___|_   _| ____|  _ \ 
   | | | | / _ \ ___ | |     / _ \ \___ \ | | |  _| | |_) |
   | |_| || (_) |___|| |___ / ___ \ ___) || | | |___|  _ < 
    \__\_\ \___/      \____/_/   \_\____/ |_| |_____|_| \_\
                                                            
    🎬 AI-Powered Display Casting MCP Server 🎬
    Port 8420 - Because that's how we roll at 8b-is
EOF
    echo -e "${NC}"
}

# Interactive mode check
INTERACTIVE=true
if [[ "$1" == "-n" || "$1" == "--non-interactive" ]]; then
    INTERACTIVE=false
    shift
fi

# Function to print with pizzazz
log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to ensure dependencies
check_dependencies() {
    log "Checking dependencies... 🔍"
    
    local missing=()
    local needs_install=false
    
    if ! command_exists cargo; then
        missing+=("cargo (Rust)")
    fi
    
    if ! command_exists gst-launch-1.0; then
        missing+=("gstreamer")
        needs_install=true
    fi
    
    if ! command_exists pkg-config; then
        missing+=("pkg-config")
        needs_install=true
    fi
    
    # Check for required system libraries
    if ! pkg-config --exists gstreamer-1.0 2>/dev/null; then
        missing+=("libgstreamer1.0-dev")
        needs_install=true
    fi
    
    if ! pkg-config --exists gio-2.0 2>/dev/null; then
        missing+=("libglib2.0-dev")
        needs_install=true
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing[*]}"
        
        if [[ "$needs_install" == true ]]; then
            warn "Installing system dependencies..."
            install_system_deps
        else
            error "Please install Rust first: https://rustup.rs"
            exit 1
        fi
    else
        log "All dependencies satisfied! ✨"
    fi
}

# Function to install system dependencies
install_system_deps() {
    info "Installing system dependencies... 📦"
    
    local deps=(
        "pkg-config"
        "libglib2.0-dev"
        "libgstreamer1.0-dev"
        "libgstreamer-plugins-base1.0-dev"
        "libgstreamer-plugins-good1.0-dev"
        "libgstreamer-plugins-bad1.0-dev"
        "gstreamer1.0-plugins-base"
        "gstreamer1.0-plugins-good"
        "gstreamer1.0-plugins-bad"
        "gstreamer1.0-plugins-ugly"
        "gstreamer1.0-libav"
        "gstreamer1.0-tools"
        "gstreamer1.0-x"
        "gstreamer1.0-alsa"
        "gstreamer1.0-gl"
        "gstreamer1.0-gtk3"
        "gstreamer1.0-pulseaudio"
        "libgtk-3-dev"
        "libx11-dev"
        "libxext-dev"
        "libxrender-dev"
        "libxi-dev"
        "libgl1-mesa-dev"
        "libegl1-mesa-dev"
    )
    
    if command_exists apt-get; then
        log "Using apt to install dependencies..."
        sudo apt-get update
        sudo apt-get install -y "${deps[@]}"
    elif command_exists dnf; then
        log "Using dnf to install dependencies..."
        sudo dnf install -y gstreamer1-devel gstreamer1-plugins-base-devel \
            gstreamer1-plugins-good gstreamer1-plugins-bad-free \
            gstreamer1-plugins-ugly-free gstreamer1-libav \
            gtk3-devel glib2-devel
    elif command_exists pacman; then
        log "Using pacman to install dependencies..."
        sudo pacman -S --noconfirm gstreamer gst-plugins-base gst-plugins-good \
            gst-plugins-bad gst-plugins-ugly gst-libav gtk3 glib2
    else
        error "Unsupported package manager! Please install GStreamer manually."
        exit 1
    fi
    
    log "System dependencies installed! 🎉"
}

# Build the project
build() {
    print_banner
    log "Building q8-caster... 🔨"
    
    if [[ "$1" == "release" ]]; then
        info "Building in RELEASE mode for maximum zoom! 🚀"
        cargo build --release
        log "Release build complete! Binary at: target/release/q8-caster"
    else
        info "Building in DEBUG mode... 🐛"
        cargo build
        log "Debug build complete! Binary at: target/debug/q8-caster"
    fi
}

# Run the server
run() {
    print_banner
    log "Starting q8-caster HTTP server... 🎭"
    
    export RUST_LOG="${RUST_LOG:-info}"
    
    local PORT="${PORT:-8420}"
    local ARGS="--port $PORT"
    
    if [[ "$1" == "elevated" ]]; then
        warn "Running in ELEVATED mode - requires sudo"
        ARGS="$ARGS --elevated"
    fi
    
    if [[ -f "target/release/q8-caster" && "$2" != "debug" ]]; then
        info "Running RELEASE build on port $PORT"
        info "Dashboard: http://localhost:$PORT"
        exec target/release/q8-caster $ARGS
    else
        info "Running DEBUG build on port $PORT"
        info "Dashboard: http://localhost:$PORT"
        exec cargo run -- $ARGS
    fi
}

# Run as daemon
daemon() {
    print_banner
    log "Starting q8-caster as daemon... 👻"
    
    local log_dir="logs"
    mkdir -p "$log_dir"
    
    local log_file="$log_dir/q8-caster-$(date +%Y%m%d-%H%M%S).log"
    
    export RUST_LOG="${RUST_LOG:-info}"
    
    if [[ -f "target/release/q8-caster" ]]; then
        nohup target/release/q8-caster > "$log_file" 2>&1 &
    else
        error "No release build found! Run './scripts/manage.sh build release' first"
        exit 1
    fi
    
    local pid=$!
    echo $pid > q8-caster.pid
    
    log "Daemon started with PID: $pid"
    log "Logs: $log_file"
    log "To stop: ./scripts/manage.sh stop"
}

# Stop daemon
stop() {
    if [[ -f "q8-caster.pid" ]]; then
        local pid=$(cat q8-caster.pid)
        if kill -0 "$pid" 2>/dev/null; then
            log "Stopping q8-caster (PID: $pid)... 🛑"
            kill "$pid"
            rm -f q8-caster.pid
            log "Stopped!"
        else
            warn "Process $pid not found. Cleaning up pid file..."
            rm -f q8-caster.pid
        fi
    else
        warn "No pid file found. Is q8-caster running?"
    fi
}

# Show status
status() {
    print_banner
    echo -e "${PURPLE}=== Q8-Caster Status ===${NC}"
    echo
    
    # Check if built
    if [[ -f "target/release/q8-caster" ]]; then
        info "Release build: ✅ Available"
    else
        info "Release build: ❌ Not found"
    fi
    
    if [[ -f "target/debug/q8-caster" ]]; then
        info "Debug build: ✅ Available"
    else
        info "Debug build: ❌ Not found"
    fi
    
    echo
    
    # Check if running
    if [[ -f "q8-caster.pid" ]]; then
        local pid=$(cat q8-caster.pid)
        if kill -0 "$pid" 2>/dev/null; then
            info "Daemon: ✅ Running (PID: $pid)"
        else
            info "Daemon: ❌ Not running (stale pid file)"
        fi
    else
        info "Daemon: ❌ Not running"
    fi
    
    echo
    
    # Check ports
    if command_exists ss; then
        if ss -tuln | grep -q ":8420"; then
            info "Port 8420: ✅ In use"
        else
            info "Port 8420: ⭕ Available"
        fi
    fi
    
    echo
}

# Run tests
test() {
    print_banner
    log "Running tests... 🧪"
    
    cargo test
    
    log "Tests complete! 🎉"
}

# Clean build artifacts
clean() {
    log "Cleaning up... 🧹"
    
    cargo clean
    rm -rf logs/
    rm -f q8-caster.pid
    
    log "Squeaky clean! ✨"
}

# Lint code
lint() {
    log "Linting code... 🔍"
    
    if command_exists cargo-clippy; then
        cargo clippy -- -D warnings
    else
        warn "cargo-clippy not installed. Installing..."
        rustup component add clippy
        cargo clippy -- -D warnings
    fi
    
    log "Code is looking sharp! 💎"
}

# Format code
fmt() {
    log "Formatting code... 💅"
    
    cargo fmt
    
    log "Code formatted! ✨"
}

# Install systemd service
install_service() {
    if [[ "$EUID" -ne 0 ]]; then
        error "Please run with sudo to install systemd service"
        exit 1
    fi
    
    log "Installing systemd service... 🔧"
    
    # Build release binary first
    log "Building release binary..."
    sudo -u $SUDO_USER bash -c "cd $(pwd) && ./scripts/manage.sh build release"
    
    # Install binary to /opt
    mkdir -p /opt/q8-caster
    cp target/release/q8-caster /opt/q8-caster/
    chmod +x /opt/q8-caster/q8-caster
    
    # Create cache directory
    mkdir -p /opt/q8-caster/cache
    mkdir -p /var/log/q8-caster
    
    # Copy service file
    cp q8-caster.service /etc/systemd/system/
    
    systemctl daemon-reload
    systemctl enable q8-caster
    
    log "Service installed! 🎉"
    log "Start with: sudo systemctl start q8-caster"
    log "View logs: sudo journalctl -u q8-caster -f"
    log "Dashboard: http://localhost:8420"
}

# Secrets management
secrets() {
    case "${1:-help}" in
        generate-key)
            log "Generating new encryption key... 🔐"
            openssl rand -base64 32
            echo
            info "Set this as Q8_CASTER_ENCRYPTION_KEY environment variable"
            ;;
        init)
            log "Initializing secrets directory... 📁"
            sudo mkdir -p /etc/q8-caster/secrets
            sudo chown $USER:$USER /etc/q8-caster/secrets
            chmod 700 /etc/q8-caster/secrets
            log "Secrets directory created at /etc/q8-caster/secrets"
            ;;
        config)
            log "Creating config.toml from template... 📝"
            if [[ -f "config.toml" ]]; then
                warn "config.toml already exists! Backing up to config.toml.bak"
                cp config.toml config.toml.bak
            fi
            cp config.toml.example config.toml
            info "Edit config.toml with your Keycloak settings"
            ;;
        *)
            echo "Secrets Management Commands:"
            echo "  $0 secrets generate-key  # Generate encryption key"
            echo "  $0 secrets init         # Initialize secrets directory"
            echo "  $0 secrets config       # Create config.toml from template"
            ;;
    esac
}

# Show help
show_help() {
    print_banner
    echo "Usage: $0 [OPTIONS] COMMAND"
    echo
    echo "OPTIONS:"
    echo "  -n, --non-interactive    Run in non-interactive mode"
    echo
    echo "COMMANDS:"
    echo "  install-deps             Install system dependencies"
    echo "  build [release]          Build the project (debug by default)"
    echo "  run [elevated] [debug]   Run the HTTP server (port 8420)"
    echo "  daemon                   Run as background daemon"
    echo "  stop                     Stop the daemon"
    echo "  status                   Show current status"
    echo "  test                     Run tests"
    echo "  clean                    Clean build artifacts"
    echo "  lint                     Run clippy linter"
    echo "  fmt                      Format code"
    echo "  install-service          Install systemd service (requires sudo)"
    echo "  secrets [cmd]            Manage secrets and encryption"
    echo "  help                     Show this help"
    echo
    echo "EXAMPLES:"
    echo "  $0 install-deps          # Install GStreamer and other deps"
    echo "  $0 build release         # Build optimized binary"
    echo "  $0 run                   # Run MCP server"
    echo "  $0 daemon                # Run as daemon"
    echo "  $0 status                # Check status"
    echo "  $0 secrets generate-key  # Generate encryption key"
    echo
    echo "Made with 💜 by 8b-is"
}

# Main command handler
main() {
    case "${1:-help}" in
        install-deps)
            install_system_deps
            ;;
        build)
            check_dependencies
            build "${2:-debug}"
            ;;
        run)
            run "${2:-normal}" "${3:-release}"
            ;;
        daemon)
            daemon
            ;;
        stop)
            stop
            ;;
        status)
            status
            ;;
        test)
            test
            ;;
        clean)
            clean
            ;;
        lint)
            lint
            ;;
        fmt)
            fmt
            ;;
        install-service)
            install_service
            ;;
        secrets)
            secrets "${2:-help}"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"