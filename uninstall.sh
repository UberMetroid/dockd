#!/usr/bin/env bash
# ==============================================================================
# dockd Sovereign Clean Uninstaller
# Pure-standard-library Rust application dock and window manager daemon for Omarchy
#
# Usage:
#   curl -fsSL https://ubermetroid.github.io/dockd/uninstall.sh | bash
#
# Options:
#   -p, --purge    Remove configuration files (~/.config/omarchy/dockd-pinned.json)
#   -f, --force    Skip interactive confirmations
#   --dry-run      Simulate removal without disk writes
#   -h, --help     Show this help message
# ==============================================================================
set -euo pipefail

PURGE=0
FORCE=0
DRY_RUN=0

for arg in "$@"; do
    case "$arg" in
        --purge|-p)
            PURGE=1
            ;;
        --force|-f)
            FORCE=1
            ;;
        --dry-run)
            DRY_RUN=1
            ;;
        --help|-h)
            echo "Usage: ./uninstall.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -p, --purge    Remove configuration files (~/.config/omarchy/dockd-pinned.json)"
            echo "  -f, --force    Skip interactive confirmations"
            echo "  --dry-run      Simulate removal without disk writes"
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

if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    BOLD="\033[1m" GREEN="\033[32m" BLUE="\033[34m" YELLOW="\033[33m" CYAN="\033[36m" RESET="\033[0m"
else
    BOLD="" GREEN="" BLUE="" YELLOW="" CYAN="" RESET=""
fi

info() { printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"; }
ok()   { printf "${GREEN}==>${RESET} %s\n" "$1"; }
warn() { printf "${YELLOW}==>${RESET} %s\n" "$1"; }
step() { printf "\n${CYAN}${BOLD}%s${RESET}\n" "$1"; }

info "Starting dockd clean uninstallation..."

# 1. Package Managers (if installed via system package)
if command -v dpkg >/dev/null 2>&1 && dpkg -s dockd >/dev/null 2>&1; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would remove Debian package 'dockd' (apt/dpkg)"
    else
        info "Removing Debian package 'dockd'..."
        if [ "$PURGE" -eq 1 ]; then
            sudo apt-get purge -y dockd 2>/dev/null || sudo dpkg -P dockd 2>/dev/null || true
        else
            sudo apt-get remove -y dockd 2>/dev/null || sudo dpkg -r dockd 2>/dev/null || true
        fi
        ok "Removed Debian package 'dockd'"
    fi
fi

if command -v rpm >/dev/null 2>&1 && rpm -q dockd >/dev/null 2>&1; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would remove RPM package 'dockd' (dnf/rpm)"
    else
        info "Removing RPM package 'dockd'..."
        sudo dnf remove -y dockd 2>/dev/null || sudo rpm -e dockd 2>/dev/null || true
        ok "Removed RPM package 'dockd'"
    fi
fi

if command -v pacman >/dev/null 2>&1 && pacman -Q dockd >/dev/null 2>&1; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would remove Arch package 'dockd' (pacman)"
    else
        info "Removing Arch package 'dockd'..."
        sudo pacman -R --noconfirm dockd 2>/dev/null || true
        ok "Removed Arch package 'dockd'"
    fi
fi

# 2. Stop and disable systemd user units
if command -v systemctl >/dev/null 2>&1; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would stop and disable dockd systemd user units"
    else
        info "Stopping and disabling dockd systemd user units..."
        systemctl --user stop dockd.service dockd.socket 2>/dev/null || true
        systemctl --user disable dockd.service dockd.socket 2>/dev/null || true
    fi
fi

# 3. Remove socket file
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
for sock in "${RUNTIME_DIR}/dockd.sock" "/tmp/dockd.sock"; do
    if [ -S "$sock" ] || [ -f "$sock" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            info "[dry-run] Would remove socket $sock"
        else
            rm -f "$sock"
            ok "Removed socket $sock"
        fi
    fi
done

# 4. Disable Omarchy plugin if omarchy CLI is available
if command -v omarchy >/dev/null 2>&1; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would disable Omarchy plugin org.ubermetroid.dockd"
    else
        info "Disabling Omarchy plugin org.ubermetroid.dockd..."
        omarchy plugin disable org.ubermetroid.dockd 2>/dev/null || true
    fi
fi

# 5. Remove installed standalone binaries
BIN_PATHS=(
    "${HOME}/.local/bin/dockd"
    "${HOME}/.local/bin/dockd-uninstall"
    "/usr/local/bin/dockd"
    "/usr/local/bin/dockd-uninstall"
    "/usr/bin/dockd-uninstall"
)

for b in "${BIN_PATHS[@]}"; do
    if [ -f "$b" ] || [ -L "$b" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            info "[dry-run] Would remove $b"
        else
            rm -f "$b" 2>/dev/null || sudo rm -f "$b" 2>/dev/null || true
            ok "Removed $b"
        fi
    fi
done

# 6. Remove systemd user unit files
SYSTEMD_TARGET="${HOME}/.config/systemd/user"
UNIT_FILES=(
    "${SYSTEMD_TARGET}/dockd.service"
    "${SYSTEMD_TARGET}/dockd.socket"
    "${SYSTEMD_TARGET}/dockd.service.d"
    "${SYSTEMD_TARGET}/dockd.socket.d"
)

for u in "${UNIT_FILES[@]}"; do
    if [ -e "$u" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            info "[dry-run] Would remove $u"
        else
            rm -rf "$u"
            ok "Removed $u"
        fi
    fi
done

# 7. Reload systemd user daemon
if command -v systemctl >/dev/null 2>&1 && [ "$DRY_RUN" -eq 0 ]; then
    info "Reloading systemd user daemon..."
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user reset-failed 2>/dev/null || true
fi

# 8. User configuration / preferences
CONFIG_FILE="${HOME}/.config/omarchy/dockd-pinned.json"
if [ -f "${CONFIG_FILE}" ]; then
    if [ "${PURGE}" -eq 1 ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            info "[dry-run] Would purge configuration ${CONFIG_FILE}"
        else
            rm -f "${CONFIG_FILE}"
            ok "Purged configuration ${CONFIG_FILE}"
        fi
    elif [ "${FORCE}" -eq 0 ] && [ -t 0 ] && [ "$DRY_RUN" -eq 0 ]; then
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

# 9. Remove Omarchy plugin directory
PLUGIN_TARGET="${HOME}/.config/omarchy/plugins/org.ubermetroid.dockd"
if [ -d "${PLUGIN_TARGET}" ]; then
    if [ "$DRY_RUN" -eq 1 ]; then
        info "[dry-run] Would remove plugin directory ${PLUGIN_TARGET}"
    else
        info "Removing plugin directory ${PLUGIN_TARGET}..."
        rm -rf "${PLUGIN_TARGET}"
        ok "Removed plugin directory ${PLUGIN_TARGET}"
    fi
fi

if [ "$DRY_RUN" -eq 1 ]; then
    ok "Dry run complete. No modifications were made."
else
    ok "dockd has been completely uninstalled from this system."
fi
