#!/bin/sh
set -e

# ====================================================================
# CONFIGURATION
# ====================================================================
REPO_USER="modelica"
REPO_NAME="fmusim"
APP_NAME="fmusim"
INSTALL_DIR="$HOME/.local/bin"

echo "Checking system compatibility..."

# 1. Detect OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin)  OS_TARGET="darwin" ;;
    linux)   OS_TARGET="linux" ;;
    *)
        echo "Error: Unsupported operating system: $OS" >&2
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)  ARCH_TARGET="x86_64" ;;
    arm64|aarch64) ARCH_TARGET="aarch64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac

# Construct the expected release asset name matching your ZIP format
# e.g., fmusim-x86_64-unknown-linux-gnu.zip
ZIP_NAME="${APP_NAME}-${ARCH_TARGET}-${OS_TARGET}.zip"
DOWNLOAD_URL="https://github.com/${REPO_USER}/${REPO_NAME}/releases/latest/download/${ZIP_NAME}"

# Ensure unzip is available
if ! command -v unzip >/dev/null 2>&1; then
    echo "Error: 'unzip' is required but not installed." >&2
    exit 1
fi

echo "Installing ${APP_NAME} to ${INSTALL_DIR}..."

# 2. Ensure installation directory exists
mkdir -p "$INSTALL_DIR"

# 3. Download and extract directly to target directory
TEMP_DIR=$(mktemp -d)
clean_up() {
    rm -rf "$TEMP_DIR"
}
trap clean_up EXIT

echo "Downloading latest release asset: ${ZIP_NAME}..."
if ! curl -LsSf "$DOWNLOAD_URL" -o "${TEMP_DIR}/${ZIP_NAME}"; then
    echo "Error: Failed to download the binary. Please verify that the release asset exists." >&2
    exit 1
fi

# Unzip into the temp directory
unzip -q "${TEMP_DIR}/${ZIP_NAME}" -d "$TEMP_DIR"

# Move the executable into place
if [ -f "${TEMP_DIR}/${APP_NAME}" ]; then
    mv "${TEMP_DIR}/${APP_NAME}" "${INSTALL_DIR}/${APP_NAME}"
else
    # Fallback if your zip folder nests everything inside a nested directory
    mv "${TEMP_DIR}"/*/"${APP_NAME}" "${INSTALL_DIR}/${APP_NAME}" 2>/dev/null || true
fi

# Ensure executable permissions are granted
chmod +x "${INSTALL_DIR}/${APP_NAME}"

# 4. Verify installation and handle PATH additions
if ! command -v "$APP_NAME" >/dev/null 2>&1; then
    case :$PATH: in
        *:"$INSTALL_DIR":*) ;;
        *)
            echo "Updating shell environment PATH..."
            if [ -f "$HOME/.zshrc" ]; then
                printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$HOME/.zshrc"
            fi
            if [ -f "$HOME/.bashrc" ]; then
                printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$HOME/.bashrc"
            fi
            export PATH="$INSTALL_DIR:$PATH"
            ;;
    esac
fi

# ====================================================================
# AUTO-COMPLETIONS SETUP
# ====================================================================
echo "Configuring shell auto-completions..."

# ZSH Setup (Default on modern macOS)
if [ -f "$HOME/.zshrc" ]; then
    ZSH_COMP_DIR="$HOME/.zshfunc"
    mkdir -p "$ZSH_COMP_DIR"
    
    "$INSTALL_DIR/$APP_NAME" completion zsh > "$ZSH_COMP_DIR/_${APP_NAME}"
    
    if ! grep -q "fpath=(.*\.zshfunc" "$HOME/.zshrc"; then
        printf '\nfpath=($HOME/.zshfunc $fpath)\nautoload -U compinit; compinit\n' >> "$HOME/.zshrc"
    fi
fi

# BASH Setup (Common on Linux distributed systems)
if [ -f "$HOME/.bashrc" ]; then
    BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$BASH_COMP_DIR"
    
    "$INSTALL_DIR/$APP_NAME" completion bash > "${BASH_COMP_DIR}/${APP_NAME}"
    
    if ! grep -q "bash-completion/completions" "$HOME/.bashrc"; then
        printf '\nif [ -f "%s/%s" ]; then\n    source "%s/%s"\nfi\n' "$BASH_COMP_DIR" "$APP_NAME" "$BASH_COMP_DIR" "$APP_NAME" >> "$HOME/.bashrc"
    fi
fi

echo ""
echo "✨ Success! ${APP_NAME} has been cleanly installed from ZIP archive."
echo "👉 Run 'source ~/.zshrc' or 'source ~/.bashrc' (or restart your terminal) to start using it."