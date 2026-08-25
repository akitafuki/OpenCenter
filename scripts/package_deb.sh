#!/usr/bin/env bash
set -e

VERSION="${1:-1.0.0}"
VERSION="${VERSION#v}"
OUT_DIR="${2:-dist}"

mkdir -p "${OUT_DIR}"
DEB_DIR="deb_pkg/opencenter_${VERSION}_amd64"
rm -rf "deb_pkg"

mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/applications"
mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${DEB_DIR}/usr/lib/systemd/user"

cp target/release/opencenter "${DEB_DIR}/usr/bin/opencenter"
chmod 755 "${DEB_DIR}/usr/bin/opencenter"

cat << 'DESK' > "${DEB_DIR}/usr/share/applications/opencenter.desktop"
[Desktop Entry]
Name=OpenCenter
Comment=Control Center & System Tray for Elgato Key Lights
Exec=opencenter gui
Icon=opencenter
Terminal=false
Type=Application
Categories=Utility;HardwareSettings;
DESK

cp crates/opencenter-opendeck/assets/icon@2x.png "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps/opencenter.png"

cat << 'SERV' > "${DEB_DIR}/usr/lib/systemd/user/opencenter.service"
[Unit]
Description=OpenCenter - Linux Control Center & System Tray for Elgato Key Lights
After=network.target

[Service]
ExecStart=/usr/bin/opencenter gui
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
SERV

cat << CTRL > "${DEB_DIR}/DEBIAN/control"
Package: opencenter
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Jimmie Maggard Jr <https://github.com/akitafuki/OpenCenter>
Depends: libdbus-1-3, libx11-6, libxkbcommon0
Description: Control Elgato Key Lights locally on Linux with OpenCenter
 OpenCenter is a fast, lightweight Linux control center for Elgato Key Lights,
 Key Light Air, Key Light Mini, and Light Strips. Provides a native desktop
 GUI, DBus system tray applet, CLI automation, and OpenDeck integration.
CTRL

dpkg-deb --build "${DEB_DIR}" "${OUT_DIR}/opencenter_${VERSION}_amd64.deb"
rm -rf "deb_pkg"
