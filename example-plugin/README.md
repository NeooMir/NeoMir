# Расширение NeoMir: Rust + Python Пульт Робота

Полноценное расширение для среды **NeoMir**, разработанное на **Rust** (`cdylib` с C-FFI экспортом) и снабжённое **Python-биндингами** (`ctypes`) и графическим окном GNOME / Libadwaita.

---

## 🏗 Архитектура

```text
       ┌────────────────────────────────────────────────────────┐
       │                 NeoMir (Главное окно)                  │
       │         GTK4 / Libadwaita + Исполнитель «Робот»        │
       │                   IPC Unix Socket                      │
       └───────────────────────────▲────────────────────────────┘
                                   │  Unix Domain Socket
                                   │  (JSON-RPC команды)
       ┌───────────────────────────▼────────────────────────────┐
       │              libneomir_robot.so (Rust cdylib)          │
       │  • Низкоуровневый Unix-клиент сокета                   │
       │  • Нативные алгоритмы (квадрат, лесенка, спираль)      │
       │  • Экспорт C-ABI функций: neomir_robot_*               │
       └───────────────────────────▲────────────────────────────┘
                                   │  C-FFI (ctypes)
       ┌───────────────────────────▼────────────────────────────┐
       │             neomir_robot.py (Python биндинг)           │
       │  • Класс Robot с методами move_*, paint, reset...      │
       │  • Управление памятью CString (zero-leak free_string)  │
       └───────────────────────────▲────────────────────────────┘
                                   │  Python API
       ┌───────────────────────────▼────────────────────────────┐
       │            main.py (Окно пульта управления)            │
       │  • Нативное окно GTK4 / Libadwaita                     │
       │  • D-Pad пульт и кнопки запуска Rust-алгоритмов        │
       └────────────────────────────────────────────────────────┘
```

---

## 📁 Структура файлов

```text
example-plugin/
├── Cargo.toml                          # Конфигурация Rust cdylib библиотеки
├── src/
│   └── lib.rs                          # Исходный код ядра на Rust (C-FFI, IPC, алгоритмы)
├── libneomir_robot.so                  # Скомпилированная разделяемая библиотека Rust
├── neomir_robot.py                     # Python-биндинг над libneomir_robot.so (ctypes)
├── main.py                             # Графический интерфейс пульта (GTK4/Adw)
├── plugin.json                         # Манифест расширения NeoMir
├── build.sh                            # Полный цикл сборки (Rust + Python + .plug)
├── rust-python-robot-controller.plug   # Готовый установочный пакет расширения
└── README.md                           # Документация
```

---

## 🦀 Исходный код на Rust (`src/lib.rs`)

Библиотека написана без внешних зависимостей (только `std`) и экспортирует стабильный C-интерфейс:

- `neomir_robot_create(sock_path)` — создание клиента и подключение к NeoMir.
- `neomir_robot_destroy(client)` — закрытие и освобождение памяти клиента.
- `neomir_robot_move_up`, `_down`, `_left`, `_right` — шаги Робота.
- `neomir_robot_paint`, `_reset`, `_get_state` — действия и статус.
- `neomir_robot_free_string(ptr)` — освобождение выделенных Rust C-строк.
- `neomir_robot_draw_square(client, size, delay_ms)` — алгоритм квадрата на Rust.
- `neomir_robot_draw_stairs(client, steps, delay_ms)` — алгоритм лесенки на Rust.
- `neomir_robot_draw_spiral(client, turns, delay_ms)` — алгоритм спирали на Rust.

---

## 🐍 Использование из Python (`neomir_robot.py`)

```python
from neomir_robot import Robot

# Подключение к запущенному NeoMir через Rust-ядро:
robot = Robot()
print("Версия движка:", robot.engine_info())

# Базовые движения (обрабатываются в Rust):
robot.move_right()
robot.paint()
robot.move_down()
robot.paint()

# Нативные Rust алгоритмы (выполняются в отдельном потоке без зависаний UI):
robot.draw_square(size=3, delay_ms=120)
robot.draw_stairs(steps=4, delay_ms=120)
robot.draw_spiral(turns=2, delay_ms=120)

# Текущее состояние:
print("Координаты:", robot.get_position())
print("Клетка закрашена:", robot.is_painted())
```

---

## 🔨 Сборка расширения

Для автоматической компиляции Rust-библиотеки и упаковки плагина:

```bash
chmod +x build.sh
./build.sh
```

В результате создаётся файл **`rust-python-robot-controller.plug`**.

---

## 🚀 Установка и запуск

1. Запустите NeoMir:
   ```bash
   cargo run
   ```
2. Откройте меню (кнопка с 3 полосками в правом верхнем углу) ➔ **«Настройки»**.
3. Перейдите во вкладку **«Расширения»**.
4. Нажмите **«Выбрать файл...»** и выберите `rust-python-robot-controller.plug`.
5. В списке появится расширение **«Rust + Python Пульт Робота»**.
6. Нажмите кнопку **▶** («Запустить окно расширения») — откроется нативное окно пульта, управляющее Роботом через библиотеку Rust!
