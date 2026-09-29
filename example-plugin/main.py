#!/usr/bin/env python3
"""
main.py — Пульт управления Роботом в NeoMir.
Поддерживает управление Роботом как через стандартные методы (Rust cdylib),
так и командами алгоритмического языка КуМир (вверх, вниз, влево, вправо, закрасить).
"""

import sys
import threading
import time

try:
    import gi
    gi.require_version("Gtk", "4.0")
    gi.require_version("Adw", "1")
    from gi.repository import Adw, GLib, Gtk
except Exception as e:
    print(f"Ошибка загрузки GTK/Libadwaita: {e}")
    sys.exit(1)

from neomir_robot import Robot


class RobotWindow:
    def __init__(self, app):
        self.app = app
        self.robot = None
        self._init_robot()

        self.window = Adw.ApplicationWindow(application=app)
        self.window.set_title("Пульт управления Роботом")
        self.window.set_default_size(380, 520)

        header = Adw.HeaderBar()
        self.window.set_content(self._build_ui(header))
        self._refresh_status()

    def _init_robot(self):
        try:
            self.robot = Robot()
        except Exception as e:
            self.robot = None
            self.init_error = str(e)

    def _build_ui(self, header):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        box.append(header)

        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        content.set_margin_start(20)
        content.set_margin_end(20)
        content.set_margin_top(8)
        content.set_margin_bottom(20)

        # Статус
        status_group = Adw.PreferencesGroup()
        self.pos_row = Adw.ActionRow()
        self.pos_row.set_title("Позиция Робота")
        self.pos_row.set_subtitle("Подключение...")
        status_group.add(self.pos_row)

        self.state_row = Adw.ActionRow()
        self.state_row.set_title("Статус клетки")
        self.state_row.set_subtitle("Чистая")
        status_group.add(self.state_row)
        content.append(status_group)

        # Пульт управления (D-Pad)
        pad_group = Adw.PreferencesGroup()
        pad_group.set_title("Пульт управления (D-Pad)")

        grid = Gtk.Grid()
        grid.set_column_spacing(8)
        grid.set_row_spacing(8)
        grid.set_halign(Gtk.Align.CENTER)

        btn_up = Gtk.Button(label="▲ Вверх")
        btn_up.add_css_class("pill")
        btn_up.connect("clicked", lambda _: self._cmd("up"))

        btn_left = Gtk.Button(label="◀ Влево")
        btn_left.add_css_class("pill")
        btn_left.connect("clicked", lambda _: self._cmd("left"))

        btn_paint = Gtk.Button(label="🖌 Закрасить")
        btn_paint.add_css_class("suggested-action")
        btn_paint.add_css_class("pill")
        btn_paint.connect("clicked", lambda _: self._cmd("paint"))

        btn_right = Gtk.Button(label="Вправо ▶")
        btn_right.add_css_class("pill")
        btn_right.connect("clicked", lambda _: self._cmd("right"))

        btn_down = Gtk.Button(label="▼ Вниз")
        btn_down.add_css_class("pill")
        btn_down.connect("clicked", lambda _: self._cmd("down"))

        grid.attach(btn_up, 1, 0, 1, 1)
        grid.attach(btn_left, 0, 1, 1, 1)
        grid.attach(btn_paint, 1, 1, 1, 1)
        grid.attach(btn_right, 2, 1, 1, 1)
        grid.attach(btn_down, 1, 2, 1, 1)

        pad_group.add(grid)
        content.append(pad_group)

        # Быстрые действия и алгоритмы
        actions_group = Adw.PreferencesGroup()
        actions_group.set_title("Действия и алгоритмы")

        btn_algo_stairs = Gtk.Button(label="📐 Лесенка (КуМир: нц 4 раз закрасить; вправо; вниз кц)")
        btn_algo_stairs.connect("clicked", lambda _: threading.Thread(target=self._run_kumir_stairs, daemon=True).start())

        btn_square = Gtk.Button(label="🌀 Закрасить квадрат 3×3 (Rust Core)")
        btn_square.connect("clicked", lambda _: threading.Thread(target=self._run_square, daemon=True).start())

        btn_reset = Gtk.Button(label="↺ Сбросить в исходную позицию")
        btn_reset.add_css_class("destructive-action")
        btn_reset.connect("clicked", lambda _: self._cmd("reset"))

        act_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        act_box.append(btn_algo_stairs)
        act_box.append(btn_square)
        act_box.append(btn_reset)
        actions_group.add(act_box)
        content.append(actions_group)

        self.info_label = Gtk.Label(label="")
        self.info_label.add_css_class("dim-label")
        content.append(self.info_label)

        box.append(content)
        return box

    def _cmd(self, name):
        if not self.robot:
            self._init_robot()
        if not self.robot:
            self.info_label.set_text("NeoMir не запущен.")
            return

        actions = {
            "up": self.robot.вверх,
            "down": self.robot.вниз,
            "left": self.robot.влево,
            "right": self.robot.вправо,
            "paint": self.robot.закрасить,
            "reset": self.robot.сброс,
        }

        func = actions.get(name, self.robot.get_state)
        res = func()
        self._update_display(res)

    def _update_display(self, res):
        if not res:
            return
        if res.get("status") == "error":
            self.info_label.set_text(f"⚠️ {res.get('message', 'Ошибка')}")
        else:
            self.info_label.set_text("Выполнено")

        x, y = res.get("x", 1), res.get("y", 1)
        self.pos_row.set_subtitle(f"X: {x}, Y: {y}")
        self.state_row.set_subtitle("Закрашена" if res.get("is_painted") else "Чистая")

    def _refresh_status(self):
        if self.robot:
            try:
                self._update_display(self.robot.get_state())
            except Exception:
                pass

    def _run_kumir_stairs(self):
        """Алгоритм лесенки на языке КуМир через Python-биндинг"""
        if not self.robot:
            return
        GLib.idle_add(lambda: self.info_label.set_text("Выполняется алгоритм лесенки..."))
        try:
            for _ in range(4):
                self.robot.закрасить()
                time.sleep(0.1)
                self.robot.вправо()
                time.sleep(0.1)
                self.robot.вниз()
                time.sleep(0.1)
            res = self.robot.закрасить()
            GLib.idle_add(lambda: self._update_display(res))
        except Exception as e:
            GLib.idle_add(lambda: self.info_label.set_text(f"Ошибка: {e}"))

    def _run_square(self):
        GLib.idle_add(lambda: self.info_label.set_text("Выполняется квадрат..."))
        try:
            res = self.robot.draw_square(size=3, delay_ms=120)
            GLib.idle_add(lambda: self._update_display(res))
        except Exception as e:
            GLib.idle_add(lambda: self.info_label.set_text(f"Ошибка: {e}"))


def main():
    app = Adw.Application(application_id="io.github.neomir.plugin.robot_controller")

    def on_activate(app):
        w = RobotWindow(app)
        app._window_ref = w
        w.window.present()

    app.connect("activate", on_activate)
    app.run(sys.argv)


if __name__ == "__main__":
    main()
