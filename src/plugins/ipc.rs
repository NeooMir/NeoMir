use crate::robot::{Direction, RobotField};
use crate::ui::console_view::ConsoleView;
use crate::ui::field_view::RobotFieldView;
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

pub struct IpcRequest {
    pub command: String,
    pub response_tx: mpsc::Sender<String>,
}

pub struct IpcServer;

impl IpcServer {
    pub fn socket_path() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("neomir.sock")
        } else {
            std::env::temp_dir().join("neomir.sock")
        }
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
                console.print_info("Python: Робот сброшен в исходное положение");
                status.set_text("Робот возвращен в исходное положение.");
                format_state(&f, "ok", None)
            }
            "get_state" | "status" => {
                let f = field.borrow();
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

fn format_state(f: &RobotField, status: &str, err_msg: Option<&str>) -> String {
    let error_part = if let Some(m) = err_msg {
        format!(",\"message\":\"{}\"", m.replace('"', "\\\""))
    } else {
        "".to_string()
    };

    format!(
        "{{\"status\":\"{}\",\"x\":{},\"y\":{},\"is_painted\":{},\"crashed\":{},\"wall_up\":{},\"wall_down\":{},\"wall_left\":{},\"wall_right\":{}{}}}",
        status,
        f.robot_x + 1,
        f.robot_y + 1,
        f.is_painted(),
        f.crashed,
        f.has_wall(&Direction::Up),
        f.has_wall(&Direction::Down),
        f.has_wall(&Direction::Left),
        f.has_wall(&Direction::Right),
        error_part
    )
}
