#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

WORK_DIR="/tmp/neomir-deb-build"
PACKAGE_NAME="neomir"
VERSION="1.1"
ARCH="amd64"

echo "==> Сборка релизного бинарника NeoMir..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Подготовка файловой структуры DEB..."
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR/pkg/DEBIAN"
mkdir -p "$WORK_DIR/pkg/usr/bin"
mkdir -p "$WORK_DIR/pkg/usr/share/applications"
mkdir -p "$WORK_DIR/pkg/usr/share/metainfo"
mkdir -p "$WORK_DIR/pkg/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$WORK_DIR/pkg/usr/share/icons/hicolor/symbolic/apps"
mkdir -p "$WORK_DIR/pkg/usr/share/doc/neomir"
mkdir -p "$DIST_DIR"

install -m 755 "$ROOT_DIR/target/release/neomir" "$WORK_DIR/pkg/usr/bin/neomir"
install -m 644 "$ROOT_DIR/data/io.github.neomir.app.desktop.in" "$WORK_DIR/pkg/usr/share/applications/io.github.neomir.app.desktop"
install -m 644 "$ROOT_DIR/data/io.github.neomir.app.metainfo.xml.in" "$WORK_DIR/pkg/usr/share/metainfo/io.github.neomir.app.metainfo.xml"
install -m 644 "$ROOT_DIR/data/icons/hicolor/scalable/apps/io.github.neomir.app.svg" "$WORK_DIR/pkg/usr/share/icons/hicolor/scalable/apps/"
install -m 644 "$ROOT_DIR/data/icons/hicolor/symbolic/apps/io.github.neomir.app-symbolic.svg" "$WORK_DIR/pkg/usr/share/icons/hicolor/symbolic/apps/"
install -m 644 "$ROOT_DIR/LICENSE" "$WORK_DIR/pkg/usr/share/doc/neomir/copyright"

# Расчет размера в КБ
INSTALLED_SIZE=$(du -sk "$WORK_DIR/pkg/usr" | cut -f1)

cat << EOF > "$WORK_DIR/pkg/DEBIAN/control"
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: education
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: NeoMir Contributors <https://github.com/vptr/neomir>
Depends: libc6 (>= 2.34), libgtk-4-1 (>= 4.10), libadwaita-1-0 (>= 1.5)
Homepage: https://github.com/vptr/neomir
Description: Educational programming environment for KuMir with Robot performer
 NeoMir is a modern KuMir-compatible educational IDE written in Rust with GTK4 and Libadwaita.
EOF

echo "==> Упаковка DEB пакета..."
cd "$WORK_DIR"
echo "2.0" > debian-binary

tar -czf control.tar.gz -C "$WORK_DIR/pkg/DEBIAN" .

cd "$WORK_DIR/pkg"
tar -czf "$WORK_DIR/data.tar.gz" --exclude='./DEBIAN' .

DEB_FILE="$DIST_DIR/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
ar rcs "$DEB_FILE" "$WORK_DIR/debian-binary" "$WORK_DIR/control.tar.gz" "$WORK_DIR/data.tar.gz"

echo "==> DEB-пакет успешно создан:"
ls -lh "$DEB_FILE"
