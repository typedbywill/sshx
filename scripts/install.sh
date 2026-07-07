#!/bin/bash
set -e

# Repository configuration
REPO="typedbywill/sshx"

# Determine OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_NAME="linux"
        ;;
    Darwin)
        OS_NAME="mac"
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="amd64"
        ;;
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="sshx-${OS_NAME}-${ARCH_NAME}"

# Fetch latest release version from GitHub API
echo "Checking latest release of sshx..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    # Fallback to v0.1.0 if API call fails
    LATEST_RELEASE="v0.1.0"
    echo "Could not fetch latest release tag, using fallback: $LATEST_RELEASE"
else
    echo "Found release $LATEST_RELEASE"
fi

DOWNLOAD_URL="https://github.com/typedbywill/sshx/releases/download/${LATEST_RELEASE}/${BINARY_NAME}"

# Temporary download location
TEMP_DIR=$(mktemp -d)
TEMP_BIN="${TEMP_DIR}/sshx"

echo "Downloading ${BINARY_NAME} from ${DOWNLOAD_URL}..."
if ! curl -L -f -o "$TEMP_BIN" "$DOWNLOAD_URL"; then
    echo "Error: Failed to download the binary. Please ensure release ${LATEST_RELEASE} exists and contains ${BINARY_NAME}."
    rm -rf "$TEMP_DIR"
    exit 1
fi

chmod +x "$TEMP_BIN"

# Determine install location
INSTALL_DIR="/usr/local/bin"
USE_SUDO=false

if [ ! -w "$INSTALL_DIR" ]; then
    if [ "$OS_NAME" = "mac" ] && [ -w "/opt/homebrew/bin" ]; then
        INSTALL_DIR="/opt/homebrew/bin"
    elif [ -d "$HOME/.local/bin" ] && [ -w "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        # Prompt or check if sudo is available
        if command -v sudo >/dev/null 2>&1; then
            USE_SUDO=true
        else
            INSTALL_DIR="$HOME/.local/bin"
            mkdir -p "$INSTALL_DIR"
        fi
    fi
fi

echo "Installing to ${INSTALL_DIR}/sshx..."
if [ "$USE_SUDO" = true ]; then
    sudo mv "$TEMP_BIN" "${INSTALL_DIR}/sshx"
else
    mv "$TEMP_BIN" "${INSTALL_DIR}/sshx"
fi

rm -rf "$TEMP_DIR"

echo "Installation complete!"
echo "Verify installation by running: sshx --help"
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo "WARNING: ${INSTALL_DIR} is not in your PATH. You might need to add it to your shell configuration."
fi
