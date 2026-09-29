#!/usr/bin/env python3
"""
neomir_robot — Python-биндинг для нативной библиотеки расширения NeoMir на Rust (libneomir_robot.so).
Позволяет управлять Роботом как через стандартные методы (move_up, paint),
так и через синтаксис школьного алгоритмического языка КуМир (вверх(), закрасить(), если сверху_свободно(): ...).
"""

import ctypes
import json
import os
import sys
from typing import Any, Dict, Optional, Tuple


def _find_rust_library() -> Optional[str]:
    """Ищет скомпилированную библиотеку Rust libneomir_robot.so."""
    dir_path = os.path.dirname(os.path.abspath(__file__))
    candidates = [
        os.path.join(dir_path, "libneomir_robot.so"),
        os.path.join(dir_path, "target", "release", "libneomir_robot.so"),
        os.path.join(dir_path, "target", "debug", "libneomir_robot.so"),
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return None


class _RustRobotFFI:
    """Низкоуровневая FFI обёртка над C ABI функциями библиотеки на Rust."""

    def __init__(self, lib_path: Optional[str] = None):
        if not lib_path:
            lib_path = _find_rust_library()

        if not lib_path:
            raise FileNotFoundError(
                "Нативная библиотека Rust libneomir_robot.so не найдена. "
                "Выполните сборку: `cargo build --release` или запустите `./build.sh`."
            )

        self._lib = ctypes.CDLL(lib_path)
        self._setup_signatures()

    def _setup_signatures(self):
        c_char_p = ctypes.c_char_p
        c_void_p = ctypes.c_void_p
        c_uint32 = ctypes.c_uint32
        c_int32 = ctypes.c_int32

        self._lib.neomir_robot_version.argtypes = []
        self._lib.neomir_robot_version.restype = c_char_p

        self._lib.neomir_robot_create.argtypes = [c_char_p]
        self._lib.neomir_robot_create.restype = c_void_p

        self._lib.neomir_robot_destroy.argtypes = [c_void_p]
        self._lib.neomir_robot_destroy.restype = None

        self._lib.neomir_robot_free_string.argtypes = [c_void_p]
        self._lib.neomir_robot_free_string.restype = None

        for fn_name in [
            "neomir_robot_move_up",
            "neomir_robot_move_down",
            "neomir_robot_move_left",
            "neomir_robot_move_right",
            "neomir_robot_paint",
            "neomir_robot_reset",
            "neomir_robot_get_state",
        ]:
            fn = getattr(self._lib, fn_name)
            fn.argtypes = [c_void_p]
            fn.restype = c_void_p

        self._lib.neomir_robot_draw_square.argtypes = [c_void_p, c_int32, c_uint32]
        self._lib.neomir_robot_draw_square.restype = c_void_p

        self._lib.neomir_robot_draw_stairs.argtypes = [c_void_p, c_int32, c_uint32]
        self._lib.neomir_robot_draw_stairs.restype = c_void_p

        self._lib.neomir_robot_draw_spiral.argtypes = [c_void_p, c_int32, c_uint32]
        self._lib.neomir_robot_draw_spiral.restype = c_void_p

    def version(self) -> str:
        res = self._lib.neomir_robot_version()
        if isinstance(res, bytes):
            return res.decode("utf-8")
        elif isinstance(res, str):
            return res
        return "Unknown"


_FFI: Optional[_RustRobotFFI] = None


def get_ffi() -> _RustRobotFFI:
    global _FFI
    if _FFI is None:
        _FFI = _RustRobotFFI()
    return _FFI


class Robot:
    """
    Высокоуровневый клиент управления Роботом на Python, использующий Rust FFI.
    Поддерживает как традиционные методы Python, так и команды языка КуМир (русскоязычные алиасы).
    """

    def __init__(self, socket_path: Optional[str] = None):
        self.ffi = get_ffi()
        sock_bytes = socket_path.encode("utf-8") if socket_path else None
        self._handle = self.ffi._lib.neomir_robot_create(sock_bytes)
        if not self._handle:
            raise ConnectionError(
                "Не удалось подключиться к NeoMir через нативный Rust-драйвер. "
                "Убедитесь, что среда NeoMir запущена."
            )

    def _call_ffi_str(self, func, *args) -> Dict[str, Any]:
        if not getattr(self, "_handle", None):
            raise RuntimeError("Клиент Робота уже закрыт.")

        ptr = func(self._handle, *args)
        if not ptr:
            return {"status": "error", "message": "Null pointer from Rust"}

        try:
            raw_bytes = ctypes.string_at(ptr)
            data = json.loads(raw_bytes.decode("utf-8"))
            return data
        finally:
            self.ffi._lib.neomir_robot_free_string(ptr)

    # ==========================================
    # Базовые методы управления (English)
    # ==========================================

    def move_up(self) -> Dict[str, Any]:
        """Переместить робота вверх на 1 клетку (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_move_up)

    def move_down(self) -> Dict[str, Any]:
        """Переместить робота вниз на 1 клетку (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_move_down)

    def move_left(self) -> Dict[str, Any]:
        """Переместить робота влево на 1 клетку (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_move_left)

    def move_right(self) -> Dict[str, Any]:
        """Переместить робота вправо на 1 клетку (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_move_right)

    def paint(self) -> Dict[str, Any]:
        """Закрасить текущую клетку поля (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_paint)

    def reset(self) -> Dict[str, Any]:
        """Сбросить робота в исходное положение (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_reset)

    def get_state(self) -> Dict[str, Any]:
        """Получить текущие координаты и состояние поля (Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_get_state)

    def get_position(self) -> Tuple[int, int]:
        """Возвращает (X, Y) координаты робота."""
        state = self.get_state()
        return (state.get("x", 1), state.get("y", 1))

    def is_painted(self) -> bool:
        """Возвращает True, если текущая клетка закрашена."""
        state = self.get_state()
        return bool(state.get("is_painted", False))

    def has_wall_up(self) -> bool:
        """Возвращает True, если сверху стена."""
        return bool(self.get_state().get("wall_up", False))

    def has_wall_down(self) -> bool:
        """Возвращает True, если снизу стена."""
        return bool(self.get_state().get("wall_down", False))

    def has_wall_left(self) -> bool:
        """Возвращает True, если слева стена."""
        return bool(self.get_state().get("wall_left", False))

    def has_wall_right(self) -> bool:
        """Возвращает True, если справа стена."""
        return bool(self.get_state().get("wall_right", False))

    def free_up(self) -> bool:
        """Возвращает True, если сверху свободно."""
        return not self.has_wall_up()

    def free_down(self) -> bool:
        """Возвращает True, если снизу свободно."""
        return not self.has_wall_down()

    def free_left(self) -> bool:
        """Возвращает True, если слева свободно."""
        return not self.has_wall_left()

    def free_right(self) -> bool:
        """Возвращает True, если справа свободно."""
        return not self.has_wall_right()

    # ==========================================
    # Команды и датчики школьного языка КуМир
    # ==========================================

    def вверх(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: вверх"""
        return self.move_up()

    def вниз(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: вниз"""
        return self.move_down()

    def влево(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: влево"""
        return self.move_left()

    def вправо(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: вправо"""
        return self.move_right()

    def закрасить(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: закрасить"""
        return self.paint()

    def сброс(self) -> Dict[str, Any]:
        """Команда алгоритмического языка: сброс"""
        return self.reset()

    def сверху_свободно(self) -> bool:
        """Датчик алгоритмического языка: сверху свободно"""
        return self.free_up()

    def снизу_свободно(self) -> bool:
        """Датчик алгоритмического языка: снизу свободно"""
        return self.free_down()

    def слева_свободно(self) -> bool:
        """Датчик алгоритмического языка: слева свободно"""
        return self.free_left()

    def справа_свободно(self) -> bool:
        """Датчик алгоритмического языка: справа свободно"""
        return self.free_right()

    def сверху_стена(self) -> bool:
        """Датчик алгоритмического языка: сверху стена"""
        return self.has_wall_up()

    def снизу_стена(self) -> bool:
        """Датчик алгоритмического языка: снизу стена"""
        return self.has_wall_down()

    def слева_стена(self) -> bool:
        """Датчик алгоритмического языка: слева стена"""
        return self.has_wall_left()

    def справа_стена(self) -> bool:
        """Датчик алгоритмического языка: справа стена"""
        return self.has_wall_right()

    def клетка_закрашена(self) -> bool:
        """Датчик алгоритмического языка: клетка закрашена"""
        return self.is_painted()

    def клетка_чистая(self) -> bool:
        """Датчик алгоритмического языка: клетка чистая"""
        return not self.is_painted()

    # ==========================================
    # Нативные алгоритмы на Rust
    # ==========================================

    def draw_square(self, size: int = 3, delay_ms: int = 120) -> Dict[str, Any]:
        """Нарисовать квадрат заданного размера (исполняется в Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_draw_square, size, delay_ms)

    def draw_stairs(self, steps: int = 4, delay_ms: int = 120) -> Dict[str, Any]:
        """Закрасить лесенкой (исполняется в Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_draw_stairs, steps, delay_ms)

    def draw_spiral(self, turns: int = 2, delay_ms: int = 120) -> Dict[str, Any]:
        """Нарисовать спираль (исполняется в Rust)."""
        return self._call_ffi_str(self.ffi._lib.neomir_robot_draw_spiral, turns, delay_ms)

    def engine_info(self) -> str:
        """Возвращает информацию о версии нативного движка Rust."""
        return self.ffi.version()

    def close(self):
        """Закрывает подключение и освобождает ресурсы Rust."""
        handle = getattr(self, "_handle", None)
        if handle:
            self._handle = None
            try:
                self.ffi._lib.neomir_robot_destroy(handle)
            except Exception:
                pass

    def __del__(self):
        try:
            self.close()
        except Exception:
            pass

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
