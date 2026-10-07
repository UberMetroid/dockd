#!/usr/bin/env bash
# dockd uninstaller
# Stops services, disables units, unregisters Omarchy plugin, and removes installed files.
set -euo pipefail

PURGE=0
FORCE=0

for arg in "$@"; do
    case "$arg" in
        --purge|-p)
            PURGE=1
            ;;
        --force|-f)
            FORCE=1
            ;;
        --help|-h)
            echo "Usage: ./uninstall.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -p, --purge    Remove configuration files (~/.config/omarchy/dockd-pinned.json)"
            echo "  -f, --force    Skip interactive confirmations"
            echo "  -h, --help     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            echo "Run ./uninstall.sh --help for usage." >&2
            exit 1
            ;;
    esac
done

if [ -t 1 ]; then
    BOLD="\033[1m" GREEN="\033[32m" BLUE="\033[34m" YELLOW="\033[33m" RESET="\033[0m"
else
    BOLD="" GREEN="" BLUE="" YELLOW="" RESET=""
fi

info() { printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"; }
ok()   { printf "${GREEN}==>${RESET} %s\n" "$1"; }
warn() { printf "${YELLOW}==>${RESET} %s\n" "$1"; }

info "Starting dockd uninstallation..."

# 1. Stop and disable systemd user units
if command -v systemctl >/dev/null 2>&1; then
    info "Stopping and disabling dockd systemd user units..."
    systemctl --user stop dockd.service dockd.socket 2>/dev/null || true
    systemctl --user disable dockd.service dockd.socket 2>/dev/null || true
fi

# 2. Remove socket file
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
if [ -S "${RUNTIME_DIR}/dockd.sock" ] || [ -f "${RUNTIME_DIR}/dockd.sock" ]; then
    info "Removing socket at ${RUNTIME_DIR}/dockd.sock..."
    rm -f "${RUNTIME_DIR}/dockd.sock"
fi
rm -f "/tmp/dockd.sock"

# 3. Disable Omarchy plugin if omarchy CLI is available
if command -v omarchy >/dev/null 2>&1; then
    info "Disabling Omarchy plugin org.ubermetroid.dockd..."
    omarchy plugin disable org.ubermetroid.dockd 2>/dev/null || true
fi

# 4. Remove installed binaries
BIN_TARGET="${HOME}/.local/bin"
if [ -f "${BIN_TARGET}/dockd" ]; then
    info "Removing binary ${BIN_TARGET}/dockd..."
    rm -f "${BIN_TARGET}/dockd"
fi
if [ -f "${BIN_TARGET}/dockd-uninstall" ]; then
    info "Removing helper ${BIN_TARGET}/dockd-uninstall..."
    rm -f "${BIN_TARGET}/dockd-uninstall"
fi

# 5. Remove Omarchy plugin directory
PLUGIN_TARGET="${HOME}/.config/omarchy/plugins/org.ubermetroid.dockd"
if [ -d "${PLUGIN_TARGET}" ]; then
    info "Removing plugin directory ${PLUGIN_TARGET}..."
    rm -rf "${PLUGIN_TARGET}"
fi

# 6. Remove systemd user unit files
SYSTEMD_TARGET="${HOME}/.config/systemd/user"
rm -f "${SYSTEMD_TARGET}/dockd.service"
rm -f "${SYSTEMD_TARGET}/dockd.socket"

# 7. Optional removal of user configuration / preferences
CONFIG_FILE="${HOME}/.config/omarchy/dockd-pinned.json"
if [ -f "${CONFIG_FILE}" ]; then
    if [ "${PURGE}" -eq 1 ]; then
        info "Purging configuration ${CONFIG_FILE}..."
        rm -f "${CONFIG_FILE}"
    elif [ "${FORCE}" -eq 0 ] && [ -t 0 ]; then
        printf "${YELLOW}==>${RESET} Remove user preferences and pinned apps (%s)? [y/N] " "${CONFIG_FILE}"
        read -r reply
        case "$reply" in
            [yY]|[yY][eE][sS])
                rm -f "${CONFIG_FILE}"
                ok "Removed configuration ${CONFIG_FILE}"
                ;;
            *)
                info "Preserving configuration ${CONFIG_FILE}"
                ;;
        esac
    else
        info "Preserving configuration ${CONFIG_FILE} (use --purge to remove)"
    fi
fi

# 8. Reload systemd user daemon
if command -v systemctl >/dev/null 2>&1; then
    info "Reloading systemd user daemon..."
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user reset-failed 2>/dev/null || true
fi

ok "dockd has been completely uninstalled from this system."
