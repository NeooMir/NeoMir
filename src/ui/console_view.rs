use gtk::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;

use crate::ui::i18n::Language;

pub struct ConsoleView {
    pub container: gtk::Box,
    pub notebook: gtk::Notebook,
    pub py_entry: gtk::Entry,
    pub output_buffer: gtk::TextBuffer,
    pub log_buffer: gtk::TextBuffer,
    pub py_buffer: gtk::TextBuffer,
    pub header_title: gtk::Label,
    pub btn_terminal: gtk::Button,
    pub btn_clear: gtk::Button,
    pub lbl_tab_output: gtk::Label,
    pub lbl_tab_log: gtk::Label,
    pub lbl_tab_py: gtk::Label,
    pub py_btn_run: gtk::Button,
    pub lang: Rc<RefCell<Language>>,
    tag_error: gtk::TextTag,
    tag_success: gtk::TextTag,
    tag_info: gtk::TextTag,
}

impl ConsoleView {
    pub fn new(initial_lang: Language) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let lang = Rc::new(RefCell::new(initial_lang));

        // Header with title and control buttons
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_start(8);
        header.set_margin_end(8);
        header.set_margin_top(4);
        header.set_margin_bottom(4);

        let initial_title = if initial_lang.is_en() {
            "Output & Log"
        } else {
            "Вывод и Журнал"
        };
        let header_title = gtk::Label::new(Some(initial_title));
        header_title.add_css_class("heading");
        header_title.set_hexpand(true);
        header_title.set_halign(gtk::Align::Start);
        header.append(&header_title);

        let btn_terminal = gtk::Button::from_icon_name("utilities-terminal-symbolic");
        let initial_term_tip = if initial_lang.is_en() {
            "Launch Python in system terminal"
        } else {
            "Запустить Python в системном терминале"
        };
        btn_terminal.set_tooltip_text(Some(initial_term_tip));
        btn_terminal.add_css_class("flat");
        btn_terminal.connect_clicked(|_| {
            Self::launch_external_python();
        });
        header.append(&btn_terminal);

        let btn_clear = gtk::Button::from_icon_name("edit-clear-symbolic");
        let initial_clear_tip = if initial_lang.is_en() {
            "Clear console"
        } else {
            "Очистить консоль"
        };
        btn_clear.set_tooltip_text(Some(initial_clear_tip));
        btn_clear.add_css_class("flat");
        header.append(&btn_clear);

        container.append(&header);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        container.append(&sep);

        // Notebook with Output, Robot Log, and Python Console
        let notebook = gtk::Notebook::new();
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);

        // 1. Output tab
        let output_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let tag_error = output_buffer.create_tag(Some("err"), &[
            ("foreground", &"#e01b24"),
            ("weight", &700),
        ]).expect("Failed to create error tag");

        let tag_success = output_buffer.create_tag(Some("ok"), &[
            ("foreground", &"#26a269"),
            ("weight", &600),
        ]).expect("Failed to create success tag");

        let tag_info = output_buffer.create_tag(Some("info"), &[
            ("foreground", &"#1c71d8"),
        ]).expect("Failed to create info tag");

        let output_tv = gtk::TextView::builder()
            .buffer(&output_buffer)
            .editable(false)
            .monospace(true)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .top_margin(6)
            .bottom_margin(6)
            .left_margin(8)
            .right_margin(8)
            .build();

        let output_scroll = gtk::ScrolledWindow::builder()
            .child(&output_tv)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        let initial_tab_out = if initial_lang.is_en() { "Output" } else { "Вывод" };
        let lbl_tab_output = gtk::Label::new(Some(initial_tab_out));
        notebook.append_page(&output_scroll, Some(&lbl_tab_output));

        // 2. Log tab (step-by-step history)
        let log_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let log_tv = gtk::TextView::builder()
            .buffer(&log_buffer)
            .editable(false)
            .monospace(true)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .top_margin(6)
            .bottom_margin(6)
            .left_margin(8)
            .right_margin(8)
            .build();

        let log_scroll = gtk::ScrolledWindow::builder()
            .child(&log_tv)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        let initial_tab_log = if initial_lang.is_en() { "Log" } else { "Журнал" };
        let lbl_tab_log = gtk::Label::new(Some(initial_tab_log));
        notebook.append_page(&log_scroll, Some(&lbl_tab_log));

        // 3. Interactive Python tab
        let py_container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let py_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let py_tv = gtk::TextView::builder()
            .buffer(&py_buffer)
            .editable(false)
            .monospace(true)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .top_margin(6)
            .bottom_margin(6)
            .left_margin(8)
            .right_margin(8)
            .build();

        let mut init_iter = py_buffer.end_iter();
        if initial_lang.is_en() {
            py_buffer.insert(&mut init_iter, "NeoMir Python 3 REPL (connected to Robot and Turtle)\n");
            py_buffer.insert(&mut init_iter, "Robot commands: up(), down(), left(), right(), paint(), reset()\n");
            py_buffer.insert(&mut init_iter, "Turtle commands: forward(n), backward(n), turn_left(deg), turn_right(deg), pen_down(), pen_up()\n");
            py_buffer.insert(&mut init_iter, "Switch performer: robot(), turtle()\n>>> ");
        } else {
            py_buffer.insert(&mut init_iter, "NeoMir Python 3 REPL (подключен к Роботу и Черепахе)\n");
            py_buffer.insert(&mut init_iter, "Команды Робота: вверх(), вниз(), влево(), вправо(), закрасить(), сброс()\n");
            py_buffer.insert(&mut init_iter, "Команды Черепахи: вперед(n), назад(n), налево(град), направо(град), опустить_хвост(), поднять_хвост()\n");
            py_buffer.insert(&mut init_iter, "Смена исполнителя: робот(), черепаха()\n>>> ");
        }

        let py_scroll = gtk::ScrolledWindow::builder()
            .child(&py_tv)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();
        py_container.append(&py_scroll);

        let py_input_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        py_input_box.set_margin_start(6);
        py_input_box.set_margin_end(6);
        py_input_box.set_margin_bottom(6);

        let py_entry = gtk::Entry::new();
        py_entry.set_hexpand(true);
        let initial_placeholder = if initial_lang.is_en() {
            "Enter Python code (e.g. forward(3); paint(); turtle(); robot())..."
        } else {
            "Введите код Python (например: вперед(3); закрасить(); черепаха(); робот())..."
        };
        py_entry.set_placeholder_text(Some(initial_placeholder));

        let initial_py_run = if initial_lang.is_en() { "Run" } else { "Выполнить" };
        let py_btn_run = gtk::Button::with_label(initial_py_run);
        py_btn_run.add_css_class("suggested-action");

        py_input_box.append(&py_entry);
        py_input_box.append(&py_btn_run);
        py_container.append(&py_input_box);

        let initial_tab_py = if initial_lang.is_en() { "Python Console" } else { "Консоль Python" };
        let lbl_tab_py = gtk::Label::new(Some(initial_tab_py));
        notebook.append_page(&py_container, Some(&lbl_tab_py));

        container.append(&notebook);

        // Connect built-in Python execution handler via mpsc channel
        let py_buf_exec = py_buffer.clone();
        let py_tv_exec = py_tv.clone();
        let execute_py_command = move |entry: &gtk::Entry| {
            let code = entry.text().trim().to_string();
            if code.is_empty() {
                return;
            }
            entry.set_text("");

            let mut end = py_buf_exec.end_iter();
            py_buf_exec.insert(&mut end, &format!(">>> {}\n", code));

            let (sender, receiver) = mpsc::channel::<(Option<String>, Option<String>)>();

            std::thread::spawn(move || {
                let py_script = format!(r#"
import os, sys, socket, json

sock_path = os.environ.get("XDG_RUNTIME_DIR", "/tmp") + "/neomir.sock"
client_sock = None
try:
    client_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client_sock.connect(sock_path)
except Exception:
    client_sock = None

def _send(payload):
    if not client_sock:
        return "NeoMir socket not connected"
    if isinstance(payload, str):
        data = json.dumps({{"cmd": payload}})
    else:
        data = json.dumps(payload)
    client_sock.sendall((data + "\n").encode())
    return client_sock.recv(4096).decode().strip()

# Robot commands (Russian & English)
вверх = up = lambda: _send("up")
вниз = down = lambda: _send("down")
влево = left = lambda: _send("left")
вправо = right = lambda: _send("right")
закрасить = paint = lambda: _send("paint")
сброс = reset = lambda: _send("reset")

# Turtle commands (Russian & English)
вперед = forward = lambda d=1: _send({{"cmd": "turtle_forward", "dist": float(d)}})
назад = backward = lambda d=1: _send({{"cmd": "turtle_backward", "dist": float(d)}})
налево = turn_left = lambda a=90: _send({{"cmd": "turtle_turn_left", "angle": float(a)}})
направо = turn_right = lambda a=90: _send({{"cmd": "turtle_turn_right", "angle": float(a)}})
опустить_хвост = pen_down = lambda: _send("turtle_pen_down")
поднять_хвост = pen_up = lambda: _send("turtle_pen_up")
опустить_перо = pen_down
поднять_перо = pen_up
tail_down = pen_down
tail_up = pen_up

# Performer switcher
черепаха = turtle = lambda: _send({{"cmd": "set_performer", "performer": "turtle"}})
робот = robot = lambda: _send({{"cmd": "set_performer", "performer": "robot"}})

code_input = {}
try:
    res = eval(compile(code_input, "<console>", "eval"))
    if res is not None:
        print(repr(res))
except SyntaxError:
    exec(compile(code_input, "<console>", "exec"))
"#, serde_json::to_string(&code).unwrap_or_else(|_| "\"\"".to_string()));

                let output = Command::new("python3")
                    .arg("-c")
                    .arg(py_script)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout_str = if !out.stdout.is_empty() {
                            Some(String::from_utf8_lossy(&out.stdout).to_string())
                        } else {
                            None
                        };
                        let stderr_str = if !out.stderr.is_empty() {
                            Some(String::from_utf8_lossy(&out.stderr).to_string())
                        } else {
                            None
                        };
                        let _ = sender.send((stdout_str, stderr_str));
                    }
                    Err(e) => {
                        let _ = sender.send((None, Some(format!("Error launching Python: {}\n", e))));
                    }
                }
            });

            let buf_cb = py_buf_exec.clone();
            let tv_cb = py_tv_exec.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
                match receiver.try_recv() {
                    Ok((stdout_opt, stderr_opt)) => {
                        let mut end_it = buf_cb.end_iter();
                        if let Some(stdout) = stdout_opt {
                            buf_cb.insert(&mut end_it, &stdout);
                        }
                        if let Some(stderr) = stderr_opt {
                            buf_cb.insert(&mut end_it, &stderr);
                        }
                        let mark = buf_cb.create_mark(None, &buf_cb.end_iter(), false);
                        tv_cb.scroll_to_mark(&mark, 0.05, false, 0.0, 0.0);
                        glib::ControlFlow::Break
                    }
                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        };

        {
            let exec_fn = execute_py_command.clone();
            py_entry.connect_activate(move |entry| {
                exec_fn(entry);
            });
        }
        {
            let exec_fn = execute_py_command;
            let entry_clone = py_entry.clone();
            py_btn_run.connect_clicked(move |_| {
                exec_fn(&entry_clone);
            });
        }

        // Connect clear button
        let out_buf_clear = output_buffer.clone();
        let log_buf_clear = log_buffer.clone();
        let py_buf_clear = py_buffer.clone();
        btn_clear.connect_clicked(move |_| {
            out_buf_clear.set_text("");
            log_buf_clear.set_text("");
            py_buf_clear.set_text("");
        });

        Self {
            container,
            notebook,
            py_entry,
            output_buffer,
            log_buffer,
            py_buffer,
            header_title,
            btn_terminal,
            btn_clear,
            lbl_tab_output,
            lbl_tab_log,
            lbl_tab_py,
            py_btn_run,
            lang,
            tag_error,
            tag_success,
            tag_info,
        }
    }

    pub fn set_language(&self, lang: Language) {
        *self.lang.borrow_mut() = lang;
        if lang.is_en() {
            self.header_title.set_text("Output & Log");
            self.lbl_tab_output.set_text("Output");
            self.lbl_tab_log.set_text("Log");
            self.lbl_tab_py.set_text("Python Console");
            self.btn_terminal.set_tooltip_text(Some("Launch Python in system terminal"));
            self.btn_clear.set_tooltip_text(Some("Clear console"));
            self.py_entry.set_placeholder_text(Some("Enter Python code (e.g. forward(3); paint(); turtle(); robot())..."));
            self.py_btn_run.set_label("Run");
        } else {
            self.header_title.set_text("Вывод и Журнал");
            self.lbl_tab_output.set_text("Вывод");
            self.lbl_tab_log.set_text("Журнал");
            self.lbl_tab_py.set_text("Консоль Python");
            self.btn_terminal.set_tooltip_text(Some("Запустить Python в системном терминале"));
            self.btn_clear.set_tooltip_text(Some("Очистить консоль"));
            self.py_entry.set_placeholder_text(Some("Введите код Python (например: вперед(3); закрасить(); черепаха(); робот())..."));
            self.py_btn_run.set_label("Выполнить");
        }
    }

    pub fn switch_to_python(&self) {
        self.notebook.set_current_page(Some(2));
        self.py_entry.grab_focus();
    }

    pub fn launch_external_python() {
        if std::env::var("FLATPAK_ID").is_ok() {
            let terminals = ["ptyxis", "gnome-terminal", "kgx", "xterm"];
            for term in &terminals {
                if let Ok(_) = Command::new("flatpak-spawn")
                    .args(["--host", term, "-e", "python3"])
                    .spawn()
                {
                    return;
                }
            }
        }

        let terminals = [
            "ptyxis",
            "gnome-terminal",
            "kgx",
            "alacritty",
            "konsole",
            "xfce4-terminal",
            "xterm",
        ];
        for term in &terminals {
            if let Ok(_) = Command::new(term).arg("-e").arg("python3").spawn() {
                return;
            }
        }
        let _ = Command::new("xdg-terminal-exec").arg("python3").spawn();
    }

    pub fn print_output(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        self.output_buffer.insert(&mut end, text);
    }

    pub fn print_error(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        let prefix = if self.lang.borrow().is_en() { "[Error] " } else { "[Ошибка] " };
        self.output_buffer.insert(&mut end, &format!("{}{}\n", prefix, text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_error, &mut start_iter, &end_iter);
    }

    pub fn print_success(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        self.output_buffer.insert(&mut end, &format!("[OK] {}\n", text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_success, &mut start_iter, &end_iter);
    }

    pub fn print_info(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        let prefix = if self.lang.borrow().is_en() { "[Info] " } else { "[Инфо] " };
        self.output_buffer.insert(&mut end, &format!("{}{}\n", prefix, text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_info, &mut start_iter, &end_iter);
    }

    pub fn log_step(&self, step_num: usize, description: &str) {
        let mut end = self.log_buffer.end_iter();
        let prefix = if self.lang.borrow().is_en() { "Step" } else { "Шаг" };
        self.log_buffer.insert(&mut end, &format!("[{} {:03}] {}\n", prefix, step_num, description));
    }

    pub fn log_action(&self, action: &str) {
        let mut end = self.log_buffer.end_iter();
        self.log_buffer.insert(&mut end, &format!("{}\n", action));
    }

    pub fn clear(&self) {
        self.output_buffer.set_text("");
        self.log_buffer.set_text("");
        self.py_buffer.set_text("");
    }
}
