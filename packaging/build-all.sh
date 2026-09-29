#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "============================================="
echo " Сборка установочных пакетов NeoMir"
echo "============================================="

"$SCRIPT_DIR/build-rpm.sh"
echo ""
"$SCRIPT_DIR/build-deb.sh"
echo ""

if command -v flatpak-builder &>/dev/null; then
    "$SCRIPT_DIR/build-flatpak.sh" || echo "Сборка Flatpak пропущена (требуется установленный GNOME Sdk)"
else
    echo "============================================="
    echo " Flatpak пакет:"
    echo "   Для сборки Flatpak установите: sudo dnf install flatpak-builder"
    echo "   Затем запустите: ./packaging/build-flatpak.sh"
    echo "============================================="
fi

echo ""
echo "============================================="
echo " Готовые пакеты в папке dist/:"
echo "============================================="
ls -lh "$SCRIPT_DIR/../dist"
