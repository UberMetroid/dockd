#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${ROOT_DIR}/dist"

mkdir -p "${OUT_DIR}"

echo "==> Building dockd release binary..."
cd "${ROOT_DIR}"
cargo build --release

ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64) DEB_ARCH="amd64" ;;
    aarch64|arm64) DEB_ARCH="arm64" ;;
    *) DEB_ARCH="$ARCH" ;;
esac

# 1. Build Debian (.deb) package
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "==> Building Debian package for ${DEB_ARCH}..."
    DEB_ROOT=$(mktemp -d)
    trap 'rm -rf "${DEB_ROOT}"' EXIT

    mkdir -p "${DEB_ROOT}/DEBIAN"
    mkdir -p "${DEB_ROOT}/usr/bin"
    mkdir -p "${DEB_ROOT}/usr/lib/systemd/user"
    mkdir -p "${DEB_ROOT}/usr/share/omarchy/plugins/org.ubermetroid.dockd"
    mkdir -p "${DEB_ROOT}/usr/share/doc/dockd"

    install -m 0755 "${ROOT_DIR}/target/release/dockd" "${DEB_ROOT}/usr/bin/dockd"
    install -m 0644 "${ROOT_DIR}/systemd/user/dockd.service" "${DEB_ROOT}/usr/lib/systemd/user/dockd.service"
    install -m 0644 "${ROOT_DIR}/systemd/user/dockd.socket" "${DEB_ROOT}/usr/lib/systemd/user/dockd.socket"
    cp -rf "${ROOT_DIR}/plugin/"* "${DEB_ROOT}/usr/share/omarchy/plugins/org.ubermetroid.dockd/"
    install -m 0755 "${ROOT_DIR}/uninstall.sh" "${DEB_ROOT}/usr/share/omarchy/plugins/org.ubermetroid.dockd/uninstall.sh"
    ln -sf "/usr/share/omarchy/plugins/org.ubermetroid.dockd/uninstall.sh" "${DEB_ROOT}/usr/bin/dockd-uninstall"
    install -m 0644 "${ROOT_DIR}/README.md" "${DEB_ROOT}/usr/share/doc/dockd/README.md"
    install -m 0644 "${ROOT_DIR}/LICENSE" "${DEB_ROOT}/usr/share/doc/dockd/copyright"


    cat <<EOF > "${DEB_ROOT}/DEBIAN/control"
Package: dockd
Version: 0.1.0-1
Section: x11
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: UberMetroid <ubermetroid@users.noreply.github.com>
Depends: systemd, libc6
Description: Pure-standard-library Rust application dock and window manager daemon for Omarchy
 dockd is a high-performance desktop dock and window manager daemon for Omarchy
 and Hyprland. Built strictly in pure-standard-library Rust with zero crates.io
 dependencies and native systemd user session integration.
EOF

    dpkg-deb --build --root-owner-group "${DEB_ROOT}" "${OUT_DIR}/dockd_0.1.0-1_${DEB_ARCH}.deb"
    echo "==> Created: ${OUT_DIR}/dockd_0.1.0-1_${DEB_ARCH}.deb"
fi

# 2. Build RPM (.rpm) package
if command -v rpmbuild >/dev/null 2>&1; then
    echo "==> Building RPM package..."
    RPM_TOP=$(mktemp -d)
    trap 'rm -rf "${RPM_TOP}" "${DEB_ROOT:-}"' EXIT

    mkdir -p "${RPM_TOP}/"{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Tar source directory
    TARBALL="${RPM_TOP}/SOURCES/dockd-0.1.0.tar.gz"
    tar --exclude='.git' --exclude='target' --exclude='dist' -czf "${TARBALL}" -C "${ROOT_DIR}/.." dockd --transform 's,^dockd,dockd-0.1.0,'

    cp "${SCRIPT_DIR}/rpm/dockd.spec" "${RPM_TOP}/SPECS/"

    rpmbuild --nodeps --define "_topdir ${RPM_TOP}" -bb "${RPM_TOP}/SPECS/dockd.spec"

    find "${RPM_TOP}/RPMS" -name "*.rpm" -exec cp {} "${OUT_DIR}/" \;
    echo "==> RPM packages copied to ${OUT_DIR}/"
fi

echo "==> Package build completed. Artifacts in ${OUT_DIR}:"
ls -lh "${OUT_DIR}"
