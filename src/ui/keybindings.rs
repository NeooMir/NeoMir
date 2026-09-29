use adw::prelude::*;
use gtk::gdk;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub key: gdk::Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Shortcut {
    pub fn new(key: gdk::Key, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self { key, ctrl, shift, alt }
    }

    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        let key_name = Self::key_to_name(self.key);
        parts.push(&key_name);
        parts.join(" + ")
    }

    pub fn to_config_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        let key_name = Self::key_to_name(self.key);
        parts.push(&key_name);
        parts.join("+")
    }

    pub fn key_to_name(key: gdk::Key) -> String {
        match key {
            gdk::Key::F1 => "F1".to_string(),
            gdk::Key::F2 => "F2".to_string(),
            gdk::Key::F3 => "F3".to_string(),
            gdk::Key::F4 => "F4".to_string(),
            gdk::Key::F5 => "F5".to_string(),
            gdk::Key::F6 => "F6".to_string(),
            gdk::Key::F7 => "F7".to_string(),
            gdk::Key::F8 => "F8".to_string(),
            gdk::Key::F9 => "F9".to_string(),
            gdk::Key::F10 => "F10".to_string(),
            gdk::Key::F11 => "F11".to_string(),
            gdk::Key::F12 => "F12".to_string(),
            gdk::Key::Return => "Enter".to_string(),
            gdk::Key::space => "Space".to_string(),
            gdk::Key::BackSpace => "Backspace".to_string(),
            gdk::Key::Tab => "Tab".to_string(),
            gdk::Key::Delete => "Delete".to_string(),
            _ => {
                if let Some(name) = key.name() {
                    let s = name.as_str();
                    if s.len() == 1 {
                        s.to_uppercase()
                    } else {
                        s.to_string()
                    }
                } else {
                    format!("{:?}", key)
                }
            }
        }
    }

    pub fn from_config_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut key_str = "";

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                _ => key_str = part,
            }
        }

        if key_str.is_empty() {
            return None;
        }

        let key = match key_str.to_uppercase().as_str() {
            "F1" => gdk::Key::F1,
            "F2" => gdk::Key::F2,
            "F3" => gdk::Key::F3,
            "F4" => gdk::Key::F4,
            "F5" => gdk::Key::F5,
            "F6" => gdk::Key::F6,
            "F7" => gdk::Key::F7,
            "F8" => gdk::Key::F8,
            "F9" => gdk::Key::F9,
            "F10" => gdk::Key::F10,
            "F11" => gdk::Key::F11,
            "F12" => gdk::Key::F12,
            "ENTER" | "RETURN" => gdk::Key::Return,
            "SPACE" => gdk::Key::space,
            "BACKSPACE" => gdk::Key::BackSpace,
            "TAB" => gdk::Key::Tab,
            "DELETE" => gdk::Key::Delete,
            other => {
                if other.len() == 1 {
                    let ch = other.chars().next().unwrap();
                    gdk::Key::from_name(&ch.to_lowercase().to_string())
                        .or_else(|| gdk::Key::from_name(other))
                        .unwrap_or(gdk::Key::VoidSymbol)
                } else {
                    gdk::Key::from_name(other).unwrap_or(gdk::Key::VoidSymbol)
                }
            }
        };

        if key == gdk::Key::VoidSymbol {
            return None;
        }

        Some(Self { key, ctrl, shift, alt })
    }

    pub fn matches(&self, keyval: gdk::Key, state: gdk::ModifierType) -> bool {
        let ctrl_pressed = state.contains(gdk::ModifierType::CONTROL_MASK);
        let shift_pressed = state.contains(gdk::ModifierType::SHIFT_MASK);
        let alt_pressed = state.contains(gdk::ModifierType::ALT_MASK);

        if self.ctrl != ctrl_pressed || self.shift != shift_pressed || self.alt != alt_pressed {
            return false;
        }

        self.key.to_lower() == keyval.to_lower()
    }
}

pub fn is_modifier_key(key: gdk::Key) -> bool {
    matches!(
        key,
        gdk::Key::Shift_L
            | gdk::Key::Shift_R
            | gdk::Key::Control_L
            | gdk::Key::Control_R
            | gdk::Key::Alt_L
            | gdk::Key::Alt_R
            | gdk::Key::Super_L
            | gdk::Key::Super_R
            | gdk::Key::Meta_L
            | gdk::Key::Meta_R
            | gdk::Key::ISO_Level3_Shift
            | gdk::Key::Caps_Lock
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindingsConfig {
    pub run: Shortcut,
    pub step: Shortcut,
    pub reset: Shortcut,
    pub load: Shortcut,
    pub save: Shortcut,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            run: Shortcut::new(gdk::Key::F5, false, false, false),
            step: Shortcut::new(gdk::Key::F10, false, false, false),
            reset: Shortcut::new(gdk::Key::F8, false, false, false),
            load: Shortcut::new(gdk::Key::o, true, false, false),
            save: Shortcut::new(gdk::Key::s, true, false, false),
        }
    }
}

impl KeybindingsConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/vptr".to_string());
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("neomir");
        fs::create_dir_all(&path).ok();
        path.push("keybindings.conf");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut cfg = Self::default();
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if let Some(sc) = Shortcut::from_config_string(v) {
                        match k {
                            "run" => cfg.run = sc,
                            "step" => cfg.step = sc,
                            "reset" => cfg.reset = sc,
                            "load" => cfg.load = sc,
                            "save" => cfg.save = sc,
                            _ => {}
                        }
                    }
                }
            }
        }
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        let content = format!(
            "# Настройки горячих клавиш NeoMir\n\
             run={}\n\
             step={}\n\
             reset={}\n\
             load={}\n\
             save={}\n",
            self.run.to_config_string(),
            self.step.to_config_string(),
            self.reset.to_config_string(),
            self.load.to_config_string(),
            self.save.to_config_string(),
        );
        fs::write(path, content)
    }
}

pub fn prompt_shortcut_recording<F>(
    parent: &impl IsA<gtk::Window>,
    action_title: &str,
    current: &Shortcut,
    on_record: F,
) where
    F: Fn(Shortcut) + 'static,
{
    let dialog = adw::Window::builder()
        .title("Назначение горячей клавиши")
        .modal(true)
        .transient_for(parent)
        .default_width(380)
        .default_height(220)
        .resizable(false)
        .build();

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_valign(gtk::Align::Center);

    let title_lbl = gtk::Label::builder()
        .label(&format!("Нажмите комбинацию для:\n«{}»", action_title))
        .justify(gtk::Justification::Center)
        .css_classes(["title-3"])
        .build();

    let key_display = gtk::Label::builder()
        .label(&format!("Ожидание нажатия... (текущая: {})", current.to_display_string()))
        .justify(gtk::Justification::Center)
        .css_classes(["card", "title-2"])
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let hint_lbl = gtk::Label::builder()
        .label("Нажмите клавиши на клавиатуре (например, Shift+F10)\nНажмите Esc для отмены")
        .justify(gtk::Justification::Center)
        .css_classes(["dim-label", "caption"])
        .build();

    let btn_cancel = gtk::Button::builder()
        .label("Отмена")
        .halign(gtk::Align::Center)
        .build();

    let dlg_cancel = dialog.clone();
    btn_cancel.connect_clicked(move |_| {
        dlg_cancel.close();
    });

    content_box.append(&title_lbl);
    content_box.append(&key_display);
    content_box.append(&hint_lbl);
    content_box.append(&btn_cancel);

    dialog.set_content(Some(&content_box));

    let dlg_close = dialog.clone();
    let on_record_rc = std::rc::Rc::new(on_record);
    let key_controller = gtk::EventControllerKey::new();

    key_controller.connect_key_pressed(move |_, keyval, _code, state| {
        if is_modifier_key(keyval) {
            let mut parts = Vec::new();
            if state.contains(gdk::ModifierType::CONTROL_MASK) || keyval == gdk::Key::Control_L || keyval == gdk::Key::Control_R {
                parts.push("Ctrl");
            }
            if state.contains(gdk::ModifierType::ALT_MASK) || keyval == gdk::Key::Alt_L || keyval == gdk::Key::Alt_R {
                parts.push("Alt");
            }
            if state.contains(gdk::ModifierType::SHIFT_MASK) || keyval == gdk::Key::Shift_L || keyval == gdk::Key::Shift_R {
                parts.push("Shift");
            }
            parts.push("...");
            key_display.set_text(&parts.join(" + "));
            return glib::Propagation::Stop;
        }

        let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = state.contains(gdk::ModifierType::SHIFT_MASK);
        let alt = state.contains(gdk::ModifierType::ALT_MASK);

        // If user presses Escape without modifiers, cancel
        if keyval == gdk::Key::Escape && !ctrl && !shift && !alt {
            dlg_close.close();
            return glib::Propagation::Stop;
        }

        let new_shortcut = Shortcut::new(keyval, ctrl, shift, alt);
        on_record_rc(new_shortcut);
        dlg_close.close();
        glib::Propagation::Stop
    });

    dialog.add_controller(key_controller);
    dialog.present();
}
