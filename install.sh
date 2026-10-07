#!/usr/bin/env bash
# ==============================================================================
# dockd Universal Web Installer
# Pure-standard-library Rust application dock and window manager daemon for Omarchy
#
# Usage:
#   curl -fsSL https://ubermetroid.github.io/dockd/install.sh | bash
#
# Options:
#   --prefix <dir>       Target installation directory for standalone binary
#   --deb, --apt         Install Debian/Ubuntu package (.deb) via apt/dpkg
#   --dnf, --rpm         Install Fedora/RHEL package (.rpm) via dnf
#   --pkgbuild, --arch   Build and install Arch Linux package via PKGBUILD
#   --dry-run            Simulate installation without disk writes
#   --uninstall          Cleanly remove dockd binary, packages, and units
#   --purge              When used with --uninstall, also remove pinned preferences
#   -h, --help           Show this help message
# ==============================================================================
set -euo pipefail

REPO="UberMetroid/dockd"
BRANCH="main"
CANONICAL_URL="https://ubermetroid.github.io/dockd"
GITHUB_URL="https://github.com/${REPO}"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/${BRANCH}"
VERSION_PIN="v0.1.4"
RAW_VERSION="0.1.4"
RELEASE_BASE="${GITHUB_URL}/releases/download/${VERSION_PIN}"

if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    CYAN="\033[38;5;51m"
    GREEN="\033[38;5;82m"
    YELLOW="\033[38;5;220m"
    BLUE="\033[38;5;39m"
    DIM="\033[38;5;242m"
    BOLD="\033[1m"
    RESET="\033[0m"
else
    CYAN="" GREEN="" YELLOW="" BLUE="" DIM="" BOLD="" RESET=""
fi

say()  { printf '%b\n' "$*"; }
info() { printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"; }
ok()   { printf "${GREEN}==>${RESET} %s\n" "$1"; }
warn() { printf "${YELLOW}==>${RESET} %s\n" "$1"; }
err()  { printf "${YELLOW}ERROR:${RESET} %s\n" "$1" >&2; }
step() { printf "\n${CYAN}${BOLD}%s${RESET}\n" "$1"; }

PREFIX=""
PURGE=0
FORCE=0
DRY_RUN=0
UNINSTALL=0
INSTALL_DEB=0
INSTALL_DNF=0
INSTALL_ARCH=0
EXTRA_ARGS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --apt|--deb)
            INSTALL_DEB=1
            shift
            ;;
        --dnf|--rpm)
            INSTALL_DNF=1
            shift
            ;;
        --pkgbuild|--arch)
            INSTALL_ARCH=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --purge|-p)
            PURGE=1
            shift
            ;;
        --force|-f)
            FORCE=1
            shift
            ;;
        --uninstall)
            UNINSTALL=1
            shift
            ;;
        -h|--help)
            say "${BOLD}dockd Universal Installer${RESET}"
            say ""
            say "Usage: ./install.sh [options]"
            say ""
            say "Options:"
            say "  --prefix <dir>       Target installation directory for standalone binary"
            say "  --deb, --apt         Install Debian/Ubuntu package (.deb) via apt/dpkg"
            say "  --dnf, --rpm         Install Fedora/RHEL package (.rpm) via dnf"
            say "  --pkgbuild, --arch   Build and install Arch Linux package via PKGBUILD"
            say "  --dry-run            Simulate installation without disk writes"
            say "  --uninstall          Cleanly remove dockd binary, packages, and units"
            say "  --purge              When used with --uninstall, also remove configuration"
            say "  -f, --force          Skip interactive confirmation"
            say "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            err "Unknown option: $1"
            say "Run with --help for valid options."
            exit 2
            ;;
    esac
done

# --- 1. Handle Uninstallation ---
if [ "$UNINSTALL" -eq 1 ]; then
    UNINSTALL_FLAGS=()
    [ "$PURGE" -eq 1 ] && UNINSTALL_FLAGS+=("--purge")
    [ "$FORCE" -eq 1 ] && UNINSTALL_FLAGS+=("--force")
    [ "$DRY_RUN" -eq 1 ] && UNINSTALL_FLAGS+=("--dry-run")

    SCRIPT_DIR_CANDIDATE="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd 2>/dev/null || echo ".")"
    if [ -f "${SCRIPT_DIR_CANDIDATE}/uninstall.sh" ]; then
        exec "${SCRIPT_DIR_CANDIDATE}/uninstall.sh" "${UNINSTALL_FLAGS[@]}"
    else
        TMP_UNINSTALL="$(mktemp)"
        if curl -fsSL "${CANONICAL_URL}/uninstall.sh" -o "$TMP_UNINSTALL" 2>/dev/null || \
           curl -fsSL "${RAW_BASE}/uninstall.sh" -o "$TMP_UNINSTALL" 2>/dev/null; then
            bash "$TMP_UNINSTALL" "${UNINSTALL_FLAGS[@]}"
            rm -f "$TMP_UNINSTALL"
            exit 0
        else
            err "Could not retrieve uninstall.sh from ${CANONICAL_URL}"
            exit 1
        fi
    fi
fi

# Determine whether we are in a local checkout
IS_LOCAL=0
SCRIPT_DIR=""
if [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]:-}" ]; then
    CANDIDATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd 2>/dev/null || echo ".")"
    if [ -f "${CANDIDATE_DIR}/Cargo.toml" ] && [ -d "${CANDIDATE_DIR}/plugin" ]; then
        IS_LOCAL=1
        SCRIPT_DIR="${CANDIDATE_DIR}"
    fi
fi

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

# --- 2. DEB / APT Installation ---
if [ "$INSTALL_DEB" -eq 1 ]; then
    step "Installing dockd via DEB/apt (${VERSION_PIN})"
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64) DEB_ARCH="amd64" ;;
        aarch64|arm64) DEB_ARCH="arm64" ;;
        *) DEB_ARCH="$ARCH" ;;
    esac

    DEB_NAME="dockd_${RAW_VERSION}-1_${DEB_ARCH}.deb"
    DEB_FILE=""

    if [ "$IS_LOCAL" -eq 1 ] && [ -f "${SCRIPT_DIR}/dist/${DEB_NAME}" ]; then
        DEB_FILE="${SCRIPT_DIR}/dist/${DEB_NAME}"
    elif [ -f "./dist/${DEB_NAME}" ]; then
        DEB_FILE="./dist/${DEB_NAME}"
    fi

    if [ -z "$DEB_FILE" ]; then
        DEB_URL="${RELEASE_BASE}/${DEB_NAME}"
        info "Downloading Debian package from ${DEB_URL}..."
        if curl -fsSL "$DEB_URL" -o "${TMP_DIR}/${DEB_NAME}" 2>/dev/null; then
            DEB_FILE="${TMP_DIR}/${DEB_NAME}"
        elif [ "$IS_LOCAL" -eq 1 ] && [ -f "${SCRIPT_DIR}/packaging/build-packages.sh" ]; then
            info "Prebuilt package not found online; building locally with packaging script..."
            (cd "${SCRIPT_DIR}" && bash packaging/build-packages.sh)
            DEB_FILE="${SCRIPT_DIR}/dist/${DEB_NAME}"
        else
            err "Unable to find or download Debian package ${DEB_NAME}"
            exit 1
        fi
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        ok "[dry-run] Would install $DEB_FILE via apt/dpkg"
        exit 0
    fi

    info "Installing ${DEB_FILE} with root privileges..."
    sudo apt-get update -qq || true
    sudo apt-get install -y "${DEB_FILE}" || (sudo dpkg -i "${DEB_FILE}" && sudo apt-get install -f -y)
    ok "Installed dockd Debian package successfully."
    exit 0
fi

# --- 3. DNF / RPM Installation ---
if [ "$INSTALL_DNF" -eq 1 ]; then
    step "Installing dockd via DNF/rpm (${VERSION_PIN})"
    RPM_FILE=""
    if [ "$IS_LOCAL" -eq 1 ]; then
        FOUND_RPM=$(find "${SCRIPT_DIR}/dist" -name "dockd-${RAW_VERSION}-1*.rpm" ! -name "*debug*" 2>/dev/null | head -n 1)
        if [ -n "$FOUND_RPM" ]; then
            RPM_FILE="$FOUND_RPM"
        fi
    fi

    if [ -z "$RPM_FILE" ]; then
        FOUND_RPM=$(find "./dist" -name "dockd-${RAW_VERSION}-1*.rpm" ! -name "*debug*" 2>/dev/null | head -n 1)
        if [ -n "$FOUND_RPM" ]; then
            RPM_FILE="$FOUND_RPM"
        fi
    fi

    if [ -z "$RPM_FILE" ]; then
        RPM_NAME="dockd-${RAW_VERSION}-1.fc44.x86_64.rpm"
        RPM_URL="${RELEASE_BASE}/${RPM_NAME}"
        info "Downloading RPM package from ${RPM_URL}..."
        if curl -fsSL "$RPM_URL" -o "${TMP_DIR}/${RPM_NAME}" 2>/dev/null; then
            RPM_FILE="${TMP_DIR}/${RPM_NAME}"
        elif [ "$IS_LOCAL" -eq 1 ] && [ -f "${SCRIPT_DIR}/packaging/build-packages.sh" ]; then
            info "Prebuilt RPM not found online; building locally with packaging script..."
            (cd "${SCRIPT_DIR}" && bash packaging/build-packages.sh)
            RPM_FILE=$(find "${SCRIPT_DIR}/dist" -name "dockd-${RAW_VERSION}-1*.rpm" ! -name "*debug*" 2>/dev/null | head -n 1)
        else
            err "Unable to find or download RPM package ${RPM_NAME}"
            exit 1
        fi
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        ok "[dry-run] Would install $RPM_FILE via dnf"
        exit 0
    fi

    info "Installing ${RPM_FILE} with root privileges..."
    sudo dnf install -y "${RPM_FILE}"
    ok "Installed dockd RPM package successfully."
    exit 0
fi

# --- 4. Arch Linux / PKGBUILD Installation ---
if [ "$INSTALL_ARCH" -eq 1 ]; then
    step "Installing dockd via PKGBUILD (${VERSION_PIN})"
    if ! command -v makepkg >/dev/null 2>&1; then
        err "makepkg not found. Install base-devel on Arch Linux or install the standalone binary."
        exit 1
    fi

    BUILD_DIR="${TMP_DIR}/pkgbuild"
    mkdir -p "${BUILD_DIR}"

    if [ "$IS_LOCAL" -eq 1 ] && [ -f "${SCRIPT_DIR}/packaging/arch/PKGBUILD" ]; then
        cp "${SCRIPT_DIR}/packaging/arch/PKGBUILD" "${BUILD_DIR}/PKGBUILD"
    elif [ -f "./packaging/arch/PKGBUILD" ]; then
        cp "./packaging/arch/PKGBUILD" "${BUILD_DIR}/PKGBUILD"
    else
        PKGBUILD_URL="${RAW_BASE}/packaging/arch/PKGBUILD"
        info "Fetching PKGBUILD from ${PKGBUILD_URL}..."
        curl -fsSL "$PKGBUILD_URL" -o "${BUILD_DIR}/PKGBUILD"
    fi

    if [ "$DRY_RUN" -eq 1 ]; then
        ok "[dry-run] Would execute makepkg -si in ${BUILD_DIR}"
        exit 0
    fi

    info "Building and installing Arch Linux package via makepkg..."
    (cd "${BUILD_DIR}" && makepkg -si --noconfirm)
    ok "Installed dockd via PKGBUILD successfully."
    exit 0
fi

# --- 5. Standalone Binary Installation (Default) ---
step "Installing dockd standalone binary & Omarchy plugin (${VERSION_PIN})"

BIN_TARGET="${PREFIX:-$HOME/.local/bin}"
PLUGIN_TARGET="${HOME}/.config/omarchy/plugins/org.ubermetroid.dockd"
SYSTEMD_TARGET="${HOME}/.config/systemd/user"

say "  Binary destination:  ${BOLD}${BIN_TARGET}/dockd${RESET}"
say "  Plugin destination:  ${BOLD}${PLUGIN_TARGET}${RESET}"
say "  Systemd user units:  ${BOLD}${SYSTEMD_TARGET}${RESET}"

if [ "$DRY_RUN" -eq 1 ]; then
    ok "[dry-run] Simulating standalone installation complete."
    exit 0
fi

mkdir -p "${BIN_TARGET}"
mkdir -p "${PLUGIN_TARGET}/components"
mkdir -p "${SYSTEMD_TARGET}"

if [ "$IS_LOCAL" -eq 1 ]; then
    info "Detected local source checkout at ${SCRIPT_DIR}"
    info "Building release binary via cargo..."
    (cd "${SCRIPT_DIR}" && cargo build --release --quiet)
    cp -f "${SCRIPT_DIR}/target/release/dockd" "${BIN_TARGET}/dockd"

    info "Installing Omarchy plugin and components..."
    cp -rf "${SCRIPT_DIR}/plugin/"* "${PLUGIN_TARGET}/"

    info "Installing systemd user service and socket units..."
    cp -f "${SCRIPT_DIR}/systemd/user/dockd.service" "${SYSTEMD_TARGET}/"
    cp -f "${SCRIPT_DIR}/systemd/user/dockd.socket" "${SYSTEMD_TARGET}/"

    info "Installing clean uninstaller..."
    cp -f "${SCRIPT_DIR}/uninstall.sh" "${PLUGIN_TARGET}/uninstall.sh"
    chmod +x "${PLUGIN_TARGET}/uninstall.sh"
    cp -f "${SCRIPT_DIR}/uninstall.sh" "${BIN_TARGET}/dockd-uninstall"
    chmod +x "${BIN_TARGET}/dockd-uninstall"
else
    info "Fetching standalone components from ${CANONICAL_URL}..."
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64) TARGET_ARCH="x86_64" ;;
        aarch64|arm64) TARGET_ARCH="aarch64" ;;
        *) TARGET_ARCH="$ARCH" ;;
    esac

    PREBUILT_URL="${RELEASE_BASE}/dockd-linux-${TARGET_ARCH}"
    if curl -fsSL -o "${TMP_DIR}/dockd" "${PREBUILT_URL}" 2>/dev/null; then
        cp -f "${TMP_DIR}/dockd" "${BIN_TARGET}/dockd"
    elif command -v cargo >/dev/null 2>&1; then
        info "Prebuilt release binary not found; building from git repository..."
        git clone --depth 1 -q "${GITHUB_URL}.git" "${TMP_DIR}/src"
        (cd "${TMP_DIR}/src" && cargo build --release --quiet)
        cp -f "${TMP_DIR}/src/target/release/dockd" "${BIN_TARGET}/dockd"
    else
        err "Could not download prebuilt release from ${PREBUILT_URL} and cargo is not installed."
        exit 1
    fi

    info "Fetching Omarchy plugin manifests and QML assets..."
    for asset in manifest.json DockPanel.qml BarWidget.qml; do
        curl -fsSL -o "${PLUGIN_TARGET}/${asset}" "${RAW_BASE}/plugin/${asset}"
    done
    for comp in AppMenu.qml DockIcon.qml SettingsPopup.qml; do
        curl -fsSL -o "${PLUGIN_TARGET}/components/${comp}" "${RAW_BASE}/plugin/components/${comp}"
    done

    info "Fetching systemd user units..."
    curl -fsSL -o "${SYSTEMD_TARGET}/dockd.service" "${RAW_BASE}/systemd/user/dockd.service"
    curl -fsSL -o "${SYSTEMD_TARGET}/dockd.socket" "${RAW_BASE}/systemd/user/dockd.socket"

    info "Fetching clean uninstaller..."
    curl -fsSL -o "${PLUGIN_TARGET}/uninstall.sh" "${RAW_BASE}/uninstall.sh"
    chmod +x "${PLUGIN_TARGET}/uninstall.sh"
    cp -f "${PLUGIN_TARGET}/uninstall.sh" "${BIN_TARGET}/dockd-uninstall"
    chmod +x "${BIN_TARGET}/dockd-uninstall"
fi

chmod +x "${BIN_TARGET}/dockd"

# Reload and enable systemd user socket activation
if command -v systemctl >/dev/null 2>&1; then
    info "Activating systemd user socket for dockd..."
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user enable --now dockd.socket 2>/dev/null || true
fi

ok "dockd successfully installed to ${BIN_TARGET}/dockd"
ok "Omarchy plugin installed to ${PLUGIN_TARGET}"
ok "systemd units installed to ${SYSTEMD_TARGET}"
ok "Uninstaller installed to ${BIN_TARGET}/dockd-uninstall"

# Auto-enable in Omarchy and restart shell
if command -v omarchy >/dev/null 2>&1; then
    info "Refreshing Omarchy plugin registry..."
    command -v omarchy-shell >/dev/null 2>&1 && omarchy-shell shell rescanPlugins 2>/dev/null || true
    info "Enabling dockd plugin in Omarchy..."
    omarchy plugin enable org.ubermetroid.dockd || true
    info "Restarting Omarchy shell..."
    omarchy restart shell || true
    ok "Omarchy dock panel is now active at the bottom of your screen!"
else
    say ""
    say "${BOLD}Next Steps:${RESET}"
    say "  1. Start socket activation:    ${CYAN}systemctl --user enable --now dockd.socket${RESET}"
    say "  2. Rescan plugins:             ${CYAN}omarchy-shell shell rescanPlugins${RESET}"
    say "  3. Enable Omarchy dock plugin: ${CYAN}omarchy plugin enable org.ubermetroid.dockd${RESET}"
    say "  4. Restart Omarchy shell:      ${CYAN}omarchy restart shell${RESET}"
fi
say "  To uninstall cleanly:          ${CYAN}dockd-uninstall${RESET} (or: ${CYAN}dockd-uninstall --purge${RESET})"
