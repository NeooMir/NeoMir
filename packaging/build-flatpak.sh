#!/usr/bin/env bash

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
FLATPAK_BUILD_DIR="$PROJECT_ROOT/_flatpak_build"
FLATPAK_REPO_DIR="$PROJECT_ROOT/_flatpak_repo"
APP_ID="io.github.neomir.app"
MANIFEST="$PROJECT_ROOT/$APP_ID.json"

mkdir -p "$DIST_DIR"

echo "============================================="
echo " Сборка Flatpak пакета NeoMir"
echo "============================================="

# 1. Update cargo-sources.json
echo "==> Генерация / обновление источников cargo-sources.json..."
python3 "$PROJECT_ROOT/build-aux/generate-cargo-sources.py" "$PROJECT_ROOT/Cargo.lock" "$PROJECT_ROOT/cargo-sources.json"

# 2. Check for flatpak-builder
if ! command -v flatpak-builder &>/dev/null; then
    echo "Предупреждение: flatpak-builder не установлен."
    echo "Для сборки Flatpak установите его:"
    echo "  sudo dnf install -y flatpak-builder   # Fedora / RHEL"
    echo "  sudo apt install -y flatpak-builder   # Ubuntu / Debian"
    exit 1
fi

# 3. Check for Flathub remote and GNOME SDK
echo "==> Проверка окружения Flatpak..."
if ! flatpak remotes | grep -q "flathub"; then
    echo "Добавление репозитория Flathub..."
    flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
fi

echo "==> Проверка наличия GNOME 50 Sdk / Platform..."
echo "Если компоненты не установлены, выполните:"
echo "  flatpak install --user -y flathub org.gnome.Platform//50 org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//26.08"
echo ""

# 4. Build with flatpak-builder
echo "==> Сборка Flatpak через flatpak-builder..."
flatpak-builder \
    --user \
    --force-clean \
    --repo="$FLATPAK_REPO_DIR" \
    "$FLATPAK_BUILD_DIR" \
    "$MANIFEST"

# 5. Create single-file bundle (.flatpak) in dist/
BUNDLE_FILE="$DIST_DIR/neomir-0.1.0.flatpak"
echo "==> Создание автономного Flatpak-бандла: $BUNDLE_FILE..."
flatpak build-bundle "$FLATPAK_REPO_DIR" "$BUNDLE_FILE" "$APP_ID"

echo ""
echo "============================================="
echo " Flatpak успешно собран!"
echo " Бандл для установки на любую систему:"
echo "   $BUNDLE_FILE"
echo ""
echo " Команда для установки бандла пользователем:"
echo "   flatpak install --user $BUNDLE_FILE"
echo " Команда для запуска:"
echo "   flatpak run $APP_ID"
echo "============================================="
