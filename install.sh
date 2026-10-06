#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_TARGET="${HOME}/.local/bin"
PLUGIN_TARGET="${HOME}/.config/omarchy/plugins/org.ubermetroid.dockd"
SYSTEMD_TARGET="${HOME}/.config/systemd/user"

echo "==> Building dockd release binary (zero crates, pure std)..."
cd "${SCRIPT_DIR}"
cargo build --release

echo "==> Installing dockd binary to ${BIN_TARGET}..."
mkdir -p "${BIN_TARGET}"
cp -f "${SCRIPT_DIR}/target/release/dockd" "${BIN_TARGET}/dockd"
chmod +x "${BIN_TARGET}/dockd"

echo "==> Installing Omarchy plugin to ${PLUGIN_TARGET}..."
mkdir -p "${PLUGIN_TARGET}"
cp -rf "${SCRIPT_DIR}/plugin/"* "${PLUGIN_TARGET}/"

echo "==> Installing systemd user units to ${SYSTEMD_TARGET}..."
mkdir -p "${SYSTEMD_TARGET}"
cp -f "${SCRIPT_DIR}/systemd/user/dockd.service" "${SYSTEMD_TARGET}/"
cp -f "${SCRIPT_DIR}/systemd/user/dockd.socket" "${SYSTEMD_TARGET}/"

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload || true
fi

echo "==> dockd installation complete."
echo "    Enable and start socket activation:"
echo "      systemctl --user enable --now dockd.socket"
echo "    Enable plugin in Omarchy:"
echo "      omarchy plugin enable org.ubermetroid.dockd"
