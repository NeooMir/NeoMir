#!/usr/bin/env bash

set -e

PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_NAME="rust-python-robot-controller.plug"

echo "============================================="
echo " Сборка расширения NeoMir: ${OUTPUT_NAME}"
echo " Ядро: Rust (cdylib) + Биндинги Python (C-FFI)"
echo "============================================="

if [ ! -f "$PLUGIN_DIR/plugin.json" ]; then
    echo "Ошибка: plugin.json не найден в $PLUGIN_DIR"
    exit 1
fi

echo "==> 1. Сборка нативной библиотеки Rust (libneomir_robot.so)..."
cd "$PLUGIN_DIR"
cargo build --release
cp "$PLUGIN_DIR/target/release/libneomir_robot.so" "$PLUGIN_DIR/libneomir_robot.so"
echo "    Скомпилирована библиотека: $(ls -lh "$PLUGIN_DIR/libneomir_robot.so")"

echo "==> 2. Проверка биндингов Python с нативным ядром Rust..."
python3 -c "import neomir_robot; ver = neomir_robot.get_ffi().version(); print(f'    Успешно! Версия ядра Rust: {ver}')"

echo "==> 3. Проверка синтаксиса интерфейса Python..."
python3 -m py_compile "$PLUGIN_DIR/main.py"
echo "    Синтаксис интерфейса корректен."

echo "==> 4. Упаковка расширения в пакет ${OUTPUT_NAME}..."
cd "$PLUGIN_DIR"
rm -f "$OUTPUT_NAME"
rm -rf __pycache__

# Упаковываем манифест, нативную библиотеку Rust .so, биндинги Python и скрипты
tar --exclude="$OUTPUT_NAME" \
    --exclude="target" \
    --exclude="build.sh" \
    --exclude="README.md" \
    --exclude="__pycache__" \
    -czf "$OUTPUT_NAME" \
    plugin.json \
    libneomir_robot.so \
    neomir_robot.py \
    main.py \
    Cargo.toml \
    src

echo "==> Расширение успешно собрано:"
ls -lh "$OUTPUT_NAME"
echo ""
echo "Установка в NeoMir:"
echo " 1. Запустите NeoMir: cargo run"
echo " 2. Откройте меню (3 полоски) -> Настройки -> Вкладка 'Расширения'."
echo " 3. Нажмите 'Выбрать файл...' и выберите '$OUTPUT_NAME'."
echo " 4. Нажмите кнопку ▶ ('Запустить окно расширения')."
