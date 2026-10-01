#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

RPM_TOPDIR="/tmp/neomir-rpm-topdir"
BUILD_ROOT="/tmp/neomir-rpm-buildroot"

echo "==> Сборка релизного бинарника NeoMir..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Подготовка файловой структуры пакета..."
rm -rf "$RPM_TOPDIR" "$BUILD_ROOT"
mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
mkdir -p "$DIST_DIR"

# Создание spec-файла
cat << EOF > "$RPM_TOPDIR/SPECS/neomir.spec"
Name:           neomir
Version:        1.1
Release:        1%{?dist}
Summary:        Среда учебного программирования NeoMir (КуМир / Робот)
License:        GPL-2.0-or-later
URL:            https://github.com/vptr/neomir

AutoReqProv:    yes

%description
NeoMir — современный легковесный аналог среды КуМир с Исполнителем «Робот»,
написанный на Rust с графическим интерфейсом на GTK4 и Libadwaita.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/metainfo
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
mkdir -p %{buildroot}/usr/share/icons/hicolor/symbolic/apps
mkdir -p %{buildroot}/usr/share/licenses/neomir

install -m 755 "$ROOT_DIR/target/release/neomir" %{buildroot}/usr/bin/neomir
install -m 644 "$ROOT_DIR/data/io.github.neomir.app.desktop.in" %{buildroot}/usr/share/applications/io.github.neomir.app.desktop
install -m 644 "$ROOT_DIR/data/io.github.neomir.app.metainfo.xml.in" %{buildroot}/usr/share/metainfo/io.github.neomir.app.metainfo.xml
install -m 644 "$ROOT_DIR/data/icons/hicolor/scalable/apps/io.github.neomir.app.svg" %{buildroot}/usr/share/icons/hicolor/scalable/apps/
install -m 644 "$ROOT_DIR/data/icons/hicolor/symbolic/apps/io.github.neomir.app-symbolic.svg" %{buildroot}/usr/share/icons/hicolor/symbolic/apps/
install -m 644 "$ROOT_DIR/LICENSE" %{buildroot}/usr/share/licenses/neomir/LICENSE

%files
/usr/bin/neomir
/usr/share/applications/io.github.neomir.app.desktop
/usr/share/metainfo/io.github.neomir.app.metainfo.xml
/usr/share/icons/hicolor/scalable/apps/io.github.neomir.app.svg
/usr/share/icons/hicolor/symbolic/apps/io.github.neomir.app-symbolic.svg
/usr/share/licenses/neomir/LICENSE

%changelog
EOF

echo "==> Запуск rpmbuild..."
rpmbuild --define "_topdir $RPM_TOPDIR" \
         --buildroot "$BUILD_ROOT" \
         -bb "$RPM_TOPDIR/SPECS/neomir.spec"

RPM_FILE=$(find "$RPM_TOPDIR/RPMS" -name "*.rpm" | head -n 1)
cp "$RPM_FILE" "$DIST_DIR/"

echo "==> RPM-пакет успешно создан:"
ls -lh "$DIST_DIR/$(basename "$RPM_FILE")"
