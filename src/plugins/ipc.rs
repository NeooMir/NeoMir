use crate::robot::{Direction, PerformerMode, RobotField};
use crate::ui::console_view::ConsoleView;
use crate::ui::field_view::RobotFieldView;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct IpcServer;

struct IpcRequest {
    command: String,
    response_tx: Sender<String>,
}

impl IpcServer {
    pub fn socket_path() -> PathBuf {
        let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(base).join("neomir.sock")
    }

    pub fn start(
        field: Rc<RefCell<RobotField>>,
        field_view: Rc<RobotFieldView>,
        console: Rc<ConsoleView>,
        status_label: gtk::Label,
    ) {
        let sock_path = Self::socket_path();
        if sock_path.exists() {
            let _ = std::fs::remove_file(&sock_path);
        }

        let listener = match UnixListener::bind(&sock_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[IPC] Не удалось запустить Unix socket listener: {}", e);
                return;
            }
        };

        let (ipc_tx, ipc_rx) = mpsc::channel::<IpcRequest>();
        let ipc_rx = Arc::new(Mutex::new(ipc_rx));

        // Main thread polling via GLib timeout (15ms ~ 60 FPS)
        let rx_for_main = ipc_rx.clone();
        let f = field.clone();
        let fv = field_view.clone();
        let con = console.clone();
        let status = status_label.clone();

        glib::timeout_add_local(Duration::from_millis(15), move || {
            if let Ok(rx) = rx_for_main.try_lock() {
                while let Ok(req) = rx.try_recv() {
                    let resp = Self::handle_command(&req.command, &f, &fv, &con, &status);
                    let _ = req.response_tx.send(resp);
                }
            }
            glib::ControlFlow::Continue
        });

        // Background worker thread for socket handling
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(client) => {
                        let tx_clone = ipc_tx.clone();
                        thread::spawn(move || {
                            Self::handle_client(client, tx_clone);
                        });
                    }
                    Err(e) => {
                        eprintln!("[IPC] Ошибка подключения клиента: {}", e);
                        break;
                    }
                }
            }
        });
    }

    fn handle_client(stream: UnixStream, ipc_tx: Sender<IpcRequest>) {
        let mut writer = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let reader = BufReader::new(stream);

        for line in reader.lines().flatten() {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            let (tx, rx) = mpsc::channel();
            let req = IpcRequest {
                command: trimmed,
                response_tx: tx,
            };

            if ipc_tx.send(req).is_ok() {
                if let Ok(resp) = rx.recv() {
                    let _ = writer.write_all(resp.as_bytes());
                    let _ = writer.write_all(b"\n");
                    let _ = writer.flush();
                }
            }
        }
    }

    fn handle_command(
        cmd: &str,
        field: &Rc<RefCell<RobotField>>,
        field_view: &Rc<RobotFieldView>,
        console: &Rc<ConsoleView>,
        status: &gtk::Label,
    ) -> String {
        let action = if cmd.contains("\"cmd\":") {
            extract_cmd_val(cmd, "cmd").unwrap_or_else(|| cmd.to_string())
        } else {
            cmd.trim_matches('"').to_string()
        };

        match action.as_str() {
            // Robot Movement
            "move_up" | "up" | "вверх" => {
                let mut f = field.borrow_mut();
                match f.move_robot(&Direction::Up) {
                    Ok(_) => {
                        field_view.widget.queue_draw();
                        let msg = format!("Python: Робот переместился вверх ({}, {})", f.robot_x + 1, f.robot_y + 1);
                        console.log_action(&msg);
                        status.set_text(&msg);
                        format_state(&f, "ok", None)
                    }
                    Err(err) => {
                        field_view.widget.queue_draw();
                        console.print_error(&format!("Python IPC ошибка: {}", err));
                        status.set_text(&err);
                        format_state(&f, "error", Some(&err))
                    }
                }
            }
            "move_down" | "down" | "вниз" => {
                let mut f = field.borrow_mut();
                match f.move_robot(&Direction::Down) {
                    Ok(_) => {
                        field_view.widget.queue_draw();
                        let msg = format!("Python: Робот переместился вниз ({}, {})", f.robot_x + 1, f.robot_y + 1);
                        console.log_action(&msg);
                        status.set_text(&msg);
                        format_state(&f, "ok", None)
                    }
                    Err(err) => {
                        field_view.widget.queue_draw();
                        console.print_error(&format!("Python IPC ошибка: {}", err));
                        status.set_text(&err);
                        format_state(&f, "error", Some(&err))
                    }
                }
            }
            "move_left" | "left" | "влево" => {
                let mut f = field.borrow_mut();
                match f.move_robot(&Direction::Left) {
                    Ok(_) => {
                        field_view.widget.queue_draw();
                        let msg = format!("Python: Робот переместился влево ({}, {})", f.robot_x + 1, f.robot_y + 1);
                        console.log_action(&msg);
                        status.set_text(&msg);
                        format_state(&f, "ok", None)
                    }
                    Err(err) => {
                        field_view.widget.queue_draw();
                        console.print_error(&format!("Python IPC ошибка: {}", err));
                        status.set_text(&err);
                        format_state(&f, "error", Some(&err))
                    }
                }
            }
            "move_right" | "right" | "вправо" => {
                let mut f = field.borrow_mut();
                match f.move_robot(&Direction::Right) {
                    Ok(_) => {
                        field_view.widget.queue_draw();
                        let msg = format!("Python: Робот переместился вправо ({}, {})", f.robot_x + 1, f.robot_y + 1);
                        console.log_action(&msg);
                        status.set_text(&msg);
                        format_state(&f, "ok", None)
                    }
                    Err(err) => {
                        field_view.widget.queue_draw();
                        console.print_error(&format!("Python IPC ошибка: {}", err));
                        status.set_text(&err);
                        format_state(&f, "error", Some(&err))
                    }
                }
            }
            "paint" | "закрасить" => {
                let mut f = field.borrow_mut();
                f.paint_cell();
                field_view.widget.queue_draw();
                let msg = format!("Python: Клетка ({}, {}) закрашена", f.robot_x + 1, f.robot_y + 1);
                console.log_action(&msg);
                status.set_text(&msg);
                format_state(&f, "ok", None)
            }
            "reset" | "сброс" => {
                let mut f = field.borrow_mut();
                f.reset_execution();
                field_view.widget.queue_draw();
                console.print_info("Python: Поле сброшено в исходное положение");
                status.set_text("Исполнитель возвращен в исходное положение.");
                format_state(&f, "ok", None)
            }
            "get_state" | "status" => {
                let f = field.borrow();
                format_state(&f, "ok", None)
            }

            // ==========================================
            // Console Interaction
            // ==========================================
            "set_console_input" | "console_set_input" => {
                let text = extract_cmd_val(cmd, "text").unwrap_or_default();
                console.py_entry.set_text(&text);
                console.switch_to_python();
                format!("{{\"status\":\"ok\",\"input\":\"{}\"}}", text.replace('"', "\\\""))
            }
            "get_console_input" | "console_get_input" => {
                let text = console.py_entry.text().to_string();
                format!("{{\"status\":\"ok\",\"input\":\"{}\"}}", text.replace('"', "\\\""))
            }
            "exec_console" | "console_exec" | "run_console" => {
                if let Some(text) = extract_cmd_val(cmd, "text") {
                    console.py_entry.set_text(&text);
                }
                console.switch_to_python();
                console.py_entry.emit_activate();
                "{\"status\":\"ok\"}".to_string()
            }
            "console_print" | "print" => {
                let text = extract_cmd_val(cmd, "text").unwrap_or_default();
                console.print_output(&text);
                "{\"status\":\"ok\"}".to_string()
            }
            "console_clear" | "clear_console" => {
                console.clear();
                "{\"status\":\"ok\"}".to_string()
            }

            // ==========================================
            // Performer Switcher (Робот ↔ Черепаха)
            // ==========================================
            "get_performer" => {
                let f = field.borrow();
                let p = match f.performer {
                    PerformerMode::Robot => "robot",
                    PerformerMode::Turtle => "turtle",
                };
                format!("{{\"status\":\"ok\",\"performer\":\"{}\"}}", p)
            }
            "set_performer" => {
                let perf_arg = extract_cmd_val(cmd, "performer").unwrap_or_default().to_lowercase();
                let mut f = field.borrow_mut();
                if perf_arg == "turtle" || perf_arg == "черепаха" {
                    f.performer = PerformerMode::Turtle;
                    status.set_text("Активен Исполнитель: Черепаха");
                } else {
                    f.performer = PerformerMode::Robot;
                    status.set_text("Активен Исполнитель: Робот");
                }
                field_view.widget.queue_draw();
                let p = match f.performer {
                    PerformerMode::Robot => "robot",
                    PerformerMode::Turtle => "turtle",
                };
                format!("{{\"status\":\"ok\",\"performer\":\"{}\"}}", p)
            }

            // ==========================================
            // Turtle Performer Commands
            // ==========================================
            "turtle_forward" | "вперед" => {
                let dist = extract_cmd_f64(cmd, "dist").unwrap_or(1.0);
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_forward(dist);
                field_view.widget.queue_draw();
                let msg = format!("Черепаха: шаг вперед на {}", dist);
                console.log_action(&msg);
                status.set_text(&msg);
                format_state(&f, "ok", None)
            }
            "turtle_backward" | "назад" => {
                let dist = extract_cmd_f64(cmd, "dist").unwrap_or(1.0);
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_backward(dist);
                field_view.widget.queue_draw();
                let msg = format!("Черепаха: шаг назад на {}", dist);
                console.log_action(&msg);
                status.set_text(&msg);
                format_state(&f, "ok", None)
            }
            "turtle_turn_left" | "влево_угол" => {
                let angle = extract_cmd_f64(cmd, "angle").unwrap_or(90.0);
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_turn_left(angle);
                field_view.widget.queue_draw();
                let msg = format!("Черепаха: поворот влево на {}°", angle);
                console.log_action(&msg);
                status.set_text(&msg);
                format_state(&f, "ok", None)
            }
            "turtle_turn_right" | "вправо_угол" => {
                let angle = extract_cmd_f64(cmd, "angle").unwrap_or(90.0);
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_turn_right(angle);
                field_view.widget.queue_draw();
                let msg = format!("Черепаха: поворот вправо на {}°", angle);
                console.log_action(&msg);
                status.set_text(&msg);
                format_state(&f, "ok", None)
            }
            "turtle_pen_down" | "опустить_хвост" => {
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_pen_down();
                field_view.widget.queue_draw();
                let msg = "Черепаха: хвост опущен (рисование включено)";
                console.log_action(msg);
                status.set_text(msg);
                format_state(&f, "ok", None)
            }
            "turtle_pen_up" | "поднять_хвост" => {
                let mut f = field.borrow_mut();
                f.performer = PerformerMode::Turtle;
                f.turtle_pen_up();
                field_view.widget.queue_draw();
                let msg = "Черепаха: хвост поднят (рисование отключено)";
                console.log_action(msg);
                status.set_text(msg);
                format_state(&f, "ok", None)
            }

            other => {
                format!("{{\"status\":\"error\",\"message\":\"Неизвестная команда: {}\"}}", other)
            }
        }
    }
}

fn extract_cmd_val(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let key_pos = json.find(&key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();

    if !after_colon.starts_with('"') {
        return None;
    }

    let end_quote = after_colon[1..].find('"')?;
    Some(after_colon[1..1 + end_quote].to_string())
}

fn extract_cmd_f64(json: &str, field: &str) -> Option<f64> {
    let key = format!("\"{}\"", field);
    let key_pos = json.find(&key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    let num_str: String = after_colon
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    num_str.parse::<f64>().ok()
}

fn format_state(f: &RobotField, status: &str, err_msg: Option<&str>) -> String {
    let error_part = if let Some(m) = err_msg {
        format!(",\"message\":\"{}\"", m.replace('"', "\\\""))
    } else {
        "".to_string()
    };

    let perf_str = match f.performer {
        PerformerMode::Robot => "robot",
        PerformerMode::Turtle => "turtle",
    };

    format!(
        "{{\"status\":\"{}\",\"performer\":\"{}\",\"x\":{},\"y\":{},\"turtle_angle\":{},\"pen_down\":{},\"is_painted\":{},\"crashed\":{},\"wall_up\":{},\"wall_down\":{},\"wall_left\":{},\"wall_right\":{}{}}}",
        status,
        perf_str,
        f.robot_x + 1,
        f.robot_y + 1,
        f.turtle_angle,
        f.turtle_pen_down,
        f.is_painted(),
        f.crashed,
        f.has_wall(&Direction::Up),
        f.has_wall(&Direction::Down),
        f.has_wall(&Direction::Left),
        f.has_wall(&Direction::Right),
        error_part
    )
}
