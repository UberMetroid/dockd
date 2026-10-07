#!/usr/bin/env bash
# dockd installer
# Supports both:
#   1. Web installation:  curl -fsSL https://raw.githubusercontent.com/UberMetroid/dockd/main/install.sh | bash
#   2. Local repository:  ./install.sh
#   3. Direct uninstallation: ./install.sh --uninstall
set -euo pipefail

REPO="UberMetroid/dockd"
BRANCH="main"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/${BRANCH}"
RELEASE_BASE="https://github.com/${REPO}/releases/latest/download"

# Support --uninstall flag directly
for arg in "$@"; do
    if [ "$arg" = "--uninstall" ]; then
        shift
        if [ -f "$(dirname "${BASH_SOURCE[0]:-$0}")/uninstall.sh" ]; then
            exec "$(dirname "${BASH_SOURCE[0]:-$0}")/uninstall.sh" "$@"
        else
            echo "Fetching uninstaller from ${REPO}..."
            curl -fsSL "${RAW_BASE}/uninstall.sh" | bash -s -- "$@"
            exit $?
        fi
    fi
done

BIN_TARGET="${HOME}/.local/bin"
PLUGIN_TARGET="${HOME}/.config/omarchy/plugins/org.ubermetroid.dockd"
SYSTEMD_TARGET="${HOME}/.config/systemd/user"

if [ -t 1 ]; then
    BOLD="\033[1m" GREEN="\033[32m" BLUE="\033[34m" RESET="\033[0m"
else
    BOLD="" GREEN="" BLUE="" RESET=""
fi

info() { printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"; }
ok()   { printf "${GREEN}==>${RESET} %s\n" "$1"; }

info "Starting dockd installation..."

# Check if we are inside a local checkout of the dockd repo
IS_LOCAL=0
SCRIPT_DIR=""
if [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]:-}" ]; then
    CANDIDATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -f "${CANDIDATE_DIR}/Cargo.toml" ] && [ -d "${CANDIDATE_DIR}/plugin" ]; then
        IS_LOCAL=1
        SCRIPT_DIR="${CANDIDATE_DIR}"
    fi
fi

TMP_DIR=$(mktemp -d)
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mkdir -p "${BIN_TARGET}"
mkdir -p "${PLUGIN_TARGET}/components"
mkdir -p "${SYSTEMD_TARGET}"

if [ "${IS_LOCAL}" -eq 1 ]; then
    info "Detected local source checkout at ${SCRIPT_DIR}"
    info "Building release binary via cargo..."
    (cd "${SCRIPT_DIR}" && cargo build --release)
    cp -f "${SCRIPT_DIR}/target/release/dockd" "${BIN_TARGET}/dockd"

    info "Installing Omarchy plugin and components..."
    cp -rf "${SCRIPT_DIR}/plugin/"* "${PLUGIN_TARGET}/"

    info "Installing systemd user service and socket units..."
    cp -f "${SCRIPT_DIR}/systemd/user/dockd.service" "${SYSTEMD_TARGET}/"
    cp -f "${SCRIPT_DIR}/systemd/user/dockd.socket" "${SYSTEMD_TARGET}/"

    info "Installing clean uninstaller..."
    cp -f "${SCRIPT_DIR}/uninstall.sh" "${PLUGIN_TARGET}/uninstall.sh"
    chmod +x "${PLUGIN_TARGET}/uninstall.sh"
    ln -sf "${PLUGIN_TARGET}/uninstall.sh" "${BIN_TARGET}/dockd-uninstall"
else
    info "Running web installer from ${REPO}..."
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64) TARGET_ARCH="x86_64" ;;
        aarch64|arm64) TARGET_ARCH="aarch64" ;;
        *) TARGET_ARCH="$ARCH" ;;
    esac

    # Attempt downloading prebuilt release binary
    PREBUILT_URL="${RELEASE_BASE}/dockd-linux-${TARGET_ARCH}"
    info "Attempting to download prebuilt binary: ${PREBUILT_URL}"
    if curl -fsSL -o "${TMP_DIR}/dockd" "${PREBUILT_URL}" 2>/dev/null; then
        cp -f "${TMP_DIR}/dockd" "${BIN_TARGET}/dockd"
    elif command -v cargo >/dev/null 2>&1; then
        info "Prebuilt binary not found; building from git source..."
        git clone --depth 1 "https://github.com/${REPO}.git" "${TMP_DIR}/src"
        (cd "${TMP_DIR}/src" && cargo build --release)
        cp -f "${TMP_DIR}/src/target/release/dockd" "${BIN_TARGET}/dockd"
    else
        echo "Error: Could not download prebuilt release and 'cargo' is not installed." >&2
        exit 1
    fi

    info "Fetching Omarchy plugin manifests and QML assets..."
    curl -fsSL -o "${PLUGIN_TARGET}/manifest.json" "${RAW_BASE}/plugin/manifest.json"
    curl -fsSL -o "${PLUGIN_TARGET}/DockPanel.qml" "${RAW_BASE}/plugin/DockPanel.qml"
    curl -fsSL -o "${PLUGIN_TARGET}/BarWidget.qml" "${RAW_BASE}/plugin/BarWidget.qml"
    curl -fsSL -o "${PLUGIN_TARGET}/components/AppMenu.qml" "${RAW_BASE}/plugin/components/AppMenu.qml"
    curl -fsSL -o "${PLUGIN_TARGET}/components/DockIcon.qml" "${RAW_BASE}/plugin/components/DockIcon.qml"
    curl -fsSL -o "${PLUGIN_TARGET}/components/SettingsPopup.qml" "${RAW_BASE}/plugin/components/SettingsPopup.qml"

    info "Fetching systemd user units..."
    curl -fsSL -o "${SYSTEMD_TARGET}/dockd.service" "${RAW_BASE}/systemd/user/dockd.service"
    curl -fsSL -o "${SYSTEMD_TARGET}/dockd.socket" "${RAW_BASE}/systemd/user/dockd.socket"

    info "Fetching clean uninstaller..."
    curl -fsSL -o "${PLUGIN_TARGET}/uninstall.sh" "${RAW_BASE}/uninstall.sh"
    chmod +x "${PLUGIN_TARGET}/uninstall.sh"
    ln -sf "${PLUGIN_TARGET}/uninstall.sh" "${BIN_TARGET}/dockd-uninstall"
fi

chmod +x "${BIN_TARGET}/dockd"

# Reload systemd user daemon
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload 2>/dev/null || true
fi

ok "dockd successfully installed to ${BIN_TARGET}/dockd"
ok "Omarchy plugin installed to ${PLUGIN_TARGET}"
ok "systemd units installed to ${SYSTEMD_TARGET}"
ok "Uninstaller installed to ${BIN_TARGET}/dockd-uninstall"
echo ""
echo "To start dockd via socket activation:"
echo "  systemctl --user enable --now dockd.socket"
echo ""
echo "To enable the dock in Omarchy:"
echo "  omarchy plugin enable org.ubermetroid.dockd"
