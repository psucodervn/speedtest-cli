#!/bin/sh
set -e  # Exit immediately if a command fails

# Set repository details
REPO="psucodervn/speedtest-cli"

# Detect OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Define binary name based on OS
if [ "$OS" = "linux" ]; then
    FILE_PATTERN="speedtest-linux-x86_64.tar.gz"
elif [ "$OS" = "darwin" ]; then
    FILE_PATTERN="speedtest-macos-x86_64.tar.gz"
elif [ "$OS" = "windows_nt" ]; then
    FILE_PATTERN="speedtest-windows-x86_64.zip"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

# Fetch latest release from GitHub API
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url" | grep "$FILE_PATTERN" | cut -d '"' -f 4)

if [ -z "$LATEST_RELEASE" ]; then
    echo "No release found for $OS"
    exit 1
fi

echo "Downloading latest release: $LATEST_RELEASE"

# Download the binary
curl -L -o "$FILE_PATTERN" "$LATEST_RELEASE" || wget -O "$FILE_PATTERN" "$LATEST_RELEASE"

# Extract the archive
if [ "$OS" = "linux" ] || [ "$OS" = "darwin" ]; then
    tar -xzf "$FILE_PATTERN"
    BIN_NAME=$(tar -tzf "$FILE_PATTERN" | head -n 1)
elif [ "$OS" = "windows_nt" ]; then
    unzip "$FILE_PATTERN"
    BIN_NAME=$(unzip -l "$FILE_PATTERN" | awk '{print $4}' | grep -v '/$' | head -n 1)
fi

# Move binary to /usr/local/bin (Linux/macOS)
if [ "$OS" = "linux" ] || [ "$OS" = "darwin" ]; then
    # Check for sudo/root permissions
    if [ "$(id -u)" -ne 0 ]; then
        echo "⚠️  This script should be run with sudo or as root."
        # echo "Try running: sudo sh install.sh"
        # exit 1
    fi
    chmod +x "$BIN_NAME"
    mv "$BIN_NAME" /usr/local/bin/speedtest
    echo "✅ Installed to /usr/local/bin/speedtest"
elif [ "$OS" = "windows_nt" ]; then
    echo "Binary extracted. Please move '$BIN_NAME' to a directory in your PATH manually."
fi

# Cleanup
rm -f "$FILE_PATTERN"
