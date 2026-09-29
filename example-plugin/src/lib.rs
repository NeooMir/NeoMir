use std::ffi::{CStr, CString};
use std::io::{BufRead, BufReader, Write};
use std::os::raw::c_char;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub struct RobotClient {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

impl RobotClient {
    pub fn connect(custom_path: Option<&str>) -> Result<Self, String> {
        let sock_path = if let Some(p) = custom_path {
            PathBuf::from(p)
        } else if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("neomir.sock")
        } else {
            std::env::temp_dir().join("neomir.sock")
        };

        if !sock_path.exists() {
            return Err(format!("Сокет NeoMir не найден: {}", sock_path.display()));
        }

        let stream = UnixStream::connect(&sock_path)
            .map_err(|e| format!("Не удалось подключиться к NeoMir: {}", e))?;

        let read_stream = stream.try_clone()
            .map_err(|e| format!("Ошибка клонирования сокета: {}", e))?;

        Ok(Self {
            writer: stream,
            reader: BufReader::new(read_stream),
        })
    }

    pub fn send_cmd(&mut self, cmd: &str) -> Result<String, String> {
        let payload = format!("{{\"cmd\":\"{}\"}}\n", cmd);
        self.writer.write_all(payload.as_bytes())
            .map_err(|e| format!("Ошибка отправки команды: {}", e))?;
        self.writer.flush()
            .map_err(|e| format!("Ошибка flush сокета: {}", e))?;

        let mut line = String::new();
        self.reader.read_line(&mut line)
            .map_err(|e| format!("Ошибка чтения ответа: {}", e))?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err("Получен пустой ответ от NeoMir".to_string());
        }

        Ok(trimmed.to_string())
    }

    pub fn step_with_delay(&mut self, cmd: &str, delay_ms: u32) -> Result<String, String> {
        let resp = self.send_cmd(cmd)?;
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms as u64));
        }
        Ok(resp)
    }
}

// ==========================================
// C-FFI Экспортируемые функции для Python биндинга
// ==========================================

fn to_c_string(s: Result<String, String>) -> *mut c_char {
    let text = match s {
        Ok(t) => t,
        Err(e) => format!("{{\"status\":\"error\",\"message\":\"{}\"}}", e.replace('"', "\\\"")),
    };
    CString::new(text).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn neomir_robot_create(custom_sock_path: *const c_char) -> *mut RobotClient {
    let path_opt = if custom_sock_path.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(custom_sock_path).to_str().ok()
        }
    };

    match RobotClient::connect(path_opt) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn neomir_robot_destroy(client: *mut RobotClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

#[no_mangle]
pub extern "C" fn neomir_robot_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub extern "C" fn neomir_robot_version() -> *const c_char {
    static VERSION: &[u8] = b"NeoMir Robot Rust Engine 1.0.0 (cdylib)\0";
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn neomir_robot_move_up(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("move_up"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_move_down(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("move_down"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_move_left(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("move_left"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_move_right(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("move_right"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_paint(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("paint"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_reset(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("reset"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_get_state(client: *mut RobotClient) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    to_c_string(c.send_cmd("get_state"))
}

// ==========================================
// Высокоуровневые алгоритмы, исполняемые на Rust
// ==========================================

#[no_mangle]
pub extern "C" fn neomir_robot_draw_square(client: *mut RobotClient, size: i32, delay_ms: u32) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    let n = size.max(1);

    for _ in 0..n {
        if let Err(e) = c.step_with_delay("move_right", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("paint", delay_ms) { return to_c_string(Err(e)); }
    }
    for _ in 0..n {
        if let Err(e) = c.step_with_delay("move_down", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("paint", delay_ms) { return to_c_string(Err(e)); }
    }
    for _ in 0..n {
        if let Err(e) = c.step_with_delay("move_left", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("paint", delay_ms) { return to_c_string(Err(e)); }
    }
    for _ in 0..n {
        if let Err(e) = c.step_with_delay("move_up", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("paint", delay_ms) { return to_c_string(Err(e)); }
    }

    to_c_string(c.send_cmd("get_state"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_draw_stairs(client: *mut RobotClient, steps: i32, delay_ms: u32) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    let n = steps.max(1);

    for _ in 0..n {
        if let Err(e) = c.step_with_delay("paint", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("move_right", delay_ms) { return to_c_string(Err(e)); }
        if let Err(e) = c.step_with_delay("move_down", delay_ms) { return to_c_string(Err(e)); }
    }
    let _ = c.step_with_delay("paint", delay_ms);

    to_c_string(c.send_cmd("get_state"))
}

#[no_mangle]
pub extern "C" fn neomir_robot_draw_spiral(client: *mut RobotClient, turns: i32, delay_ms: u32) -> *mut c_char {
    if client.is_null() {
        return to_c_string(Err("Null client pointer".to_string()));
    }
    let c = unsafe { &mut *client };
    let total_turns = turns.max(1);

    let mut step = 1;
    for _ in 0..total_turns {
        for _ in 0..step {
            if let Err(e) = c.step_with_delay("move_right", delay_ms) { return to_c_string(Err(e)); }
            let _ = c.send_cmd("paint");
        }
        for _ in 0..step {
            if let Err(e) = c.step_with_delay("move_down", delay_ms) { return to_c_string(Err(e)); }
            let _ = c.send_cmd("paint");
        }
        step += 1;
        for _ in 0..step {
            if let Err(e) = c.step_with_delay("move_left", delay_ms) { return to_c_string(Err(e)); }
            let _ = c.send_cmd("paint");
        }
        for _ in 0..step {
            if let Err(e) = c.step_with_delay("move_up", delay_ms) { return to_c_string(Err(e)); }
            let _ = c.send_cmd("paint");
        }
        step += 1;
    }

    to_c_string(c.send_cmd("get_state"))
}
