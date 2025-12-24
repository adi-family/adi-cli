#!/bin/sh
# ADI CLI Installer
# Usage: curl -fsSL https://adi.the-ihor.com/install.sh | sh
#
# Environment variables:
#   ADI_INSTALL_DIR  - Installation directory (default: ~/.local/bin)
#   ADI_VERSION      - Specific version to install (default: latest)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

REPO="adi-family/adi-cli"
BINARY_NAME="adi"

info() {
    printf "${CYAN}info${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}done${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC} %s\n" "$1"
}

error() {
    printf "${RED}error${NC} %s\n" "$1" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin)
            echo "darwin"
            ;;
        Linux)
            echo "linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            error "Windows detected. Please use: winget install adi-cli"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        arm64|aarch64)
            echo "aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            ;;
    esac
}

# Get target triple
get_target() {
    local os="$1"
    local arch="$2"

    case "$os" in
        darwin)
            echo "${arch}-apple-darwin"
            ;;
        linux)
            echo "${arch}-unknown-linux-gnu"
            ;;
    esac
}

# Fetch latest version from GitHub API
fetch_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Download file
download() {
    local url="$1"
    local output="$2"

    info "Downloading from $url"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found"
    fi
}

# Verify checksum
verify_checksum() {
    local file="$1"
    local expected="$2"

    if [ -z "$expected" ]; then
        warn "Skipping checksum verification (checksum not available)"
        return 0
    fi

    local actual=""
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$file" | cut -d' ' -f1)
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$file" | cut -d' ' -f1)
    else
        warn "Skipping checksum verification (sha256sum/shasum not found)"
        return 0
    fi

    if [ "$actual" != "$expected" ]; then
        error "Checksum verification failed!\nExpected: $expected\nActual: $actual"
    fi

    success "Checksum verified"
}

# Extract archive
extract() {
    local archive="$1"
    local dest="$2"

    info "Extracting archive"

    case "$archive" in
        *.tar.gz|*.tgz)
            tar -xzf "$archive" -C "$dest"
            ;;
        *.zip)
            unzip -q "$archive" -d "$dest"
            ;;
        *)
            error "Unknown archive format: $archive"
            ;;
    esac
}

# Add to PATH
setup_path() {
    local install_dir="$1"
    local shell_name=""
    local rc_file=""

    # Detect shell
    if [ -n "$SHELL" ]; then
        shell_name=$(basename "$SHELL")
    fi

    case "$shell_name" in
        zsh)
            rc_file="$HOME/.zshrc"
            ;;
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                rc_file="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                rc_file="$HOME/.bash_profile"
            fi
            ;;
        fish)
            rc_file="$HOME/.config/fish/config.fish"
            ;;
        *)
            rc_file="$HOME/.profile"
            ;;
    esac

    # Check if already in PATH
    case ":$PATH:" in
        *":$install_dir:"*)
            return 0
            ;;
    esac

    echo ""
    warn "$install_dir is not in your PATH"
    echo ""
    echo "Add it by running:"
    echo ""

    case "$shell_name" in
        fish)
            printf "  ${CYAN}fish_add_path %s${NC}\n" "$install_dir"
            ;;
        *)
            printf "  ${CYAN}echo 'export PATH=\"%s:\$PATH\"' >> %s${NC}\n" "$install_dir" "$rc_file"
            ;;
    esac

    echo ""
    echo "Then restart your shell or run:"
    case "$shell_name" in
        fish)
            printf "  ${CYAN}source %s${NC}\n" "$rc_file"
            ;;
        *)
            printf "  ${CYAN}source %s${NC}\n" "$rc_file"
            ;;
    esac
}

main() {
    echo ""
    printf "${BLUE}ADI CLI Installer${NC}\n"
    echo ""

    # Detect platform
    local os=$(detect_os)
    local arch=$(detect_arch)
    local target=$(get_target "$os" "$arch")

    info "Detected platform: $target"

    # Determine version
    local version="${ADI_VERSION:-}"
    if [ -z "$version" ]; then
        info "Fetching latest version"
        version=$(fetch_latest_version)
        if [ -z "$version" ]; then
            error "Failed to fetch latest version"
        fi
    fi

    # Normalize version (remove 'v' prefix if present for asset naming)
    local version_num=$(echo "$version" | sed 's/^v//')

    info "Installing version: $version"

    # Determine install directory
    local install_dir="${ADI_INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "$install_dir"

    info "Install directory: $install_dir"

    # Determine archive extension
    local archive_ext="tar.gz"
    if [ "$os" = "windows" ]; then
        archive_ext="zip"
    fi

    # Construct download URL
    local archive_name="adi-${version}-${target}.${archive_ext}"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"
    local checksums_url="https://github.com/${REPO}/releases/download/${version}/SHA256SUMS"

    # Create temp directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" EXIT

    # Download archive
    local archive_path="$temp_dir/$archive_name"
    download "$download_url" "$archive_path"

    # Download and verify checksum
    local checksums_path="$temp_dir/SHA256SUMS"
    if download "$checksums_url" "$checksums_path" 2>/dev/null; then
        local expected_checksum=$(grep "$archive_name" "$checksums_path" | cut -d' ' -f1)
        verify_checksum "$archive_path" "$expected_checksum"
    else
        warn "Checksums file not available, skipping verification"
    fi

    # Extract
    extract "$archive_path" "$temp_dir"

    # Install binary
    local binary_path="$temp_dir/$BINARY_NAME"
    if [ ! -f "$binary_path" ]; then
        error "Binary not found in archive"
    fi

    chmod +x "$binary_path"
    mv "$binary_path" "$install_dir/$BINARY_NAME"

    success "Installed $BINARY_NAME to $install_dir/$BINARY_NAME"

    # Setup PATH
    setup_path "$install_dir"

    # Verify installation
    echo ""
    if command -v adi >/dev/null 2>&1 || [ -x "$install_dir/$BINARY_NAME" ]; then
        local installed_version=$("$install_dir/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        success "ADI CLI installed successfully!"
        echo ""
        printf "  Version: ${CYAN}%s${NC}\n" "$installed_version"
        printf "  Path:    ${CYAN}%s${NC}\n" "$install_dir/$BINARY_NAME"
        echo ""
        echo "Get started:"
        printf "  ${CYAN}adi --help${NC}\n"
        printf "  ${CYAN}adi plugin list${NC}\n"
    else
        warn "Installation completed but binary verification failed"
    fi
}

main "$@"
