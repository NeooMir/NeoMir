mod lang;
mod plugins;
mod robot;
mod ui;

use gtk::gio;
use gtk::prelude::*;

const APP_ID: &str = "io.github.neomir.app";

fn main() -> glib::ExitCode {
    adw::init().expect("Failed to initialize Libadwaita");

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        let app_window = ui::AppWindow::new(app);
        app_window.window.present();
    });

    app.run()
}

#[cfg(test)]
mod tests {
    use crate::lang::compile_source;
    use crate::lang::vm::StepResult;
    use crate::plugins::PluginMetadata;
    use crate::robot::RobotField;
    use crate::ui::keybindings::{KeybindingsConfig, Shortcut};
    use gtk::gdk;

    #[test]
    fn test_robot_loop_execution() {
        let code = r#"использовать Робот
алг
нач
    нц 3 раз
        вправо
        закрасить
    кц
кон
"#;
        let mut vm = compile_source(code).expect("Compilation must succeed");
        let mut field = RobotField::new(10, 10);
        assert_eq!(field.robot_x, 0);

        let mut steps = 0;
        loop {
            let res = vm.step(&mut field);
            match res {
                StepResult::Finished => break,
                StepResult::Error { message, .. } => panic!("VM error: {}", message),
                _ => {}
            }
            steps += 1;
            assert!(steps < 50, "Infinite loop detected");
        }

        assert_eq!(field.robot_x, 3);
        assert!(field.painted[0][1]);
        assert!(field.painted[0][2]);
        assert!(field.painted[0][3]);
    }

    #[test]
    fn test_plugin_metadata_json() {
        let json = r#"{
            "id": "test.plugin",
            "name": "Test Plugin",
            "version": "1.2.3",
            "author": "Alice",
            "description": "A demo plugin",
            "enabled": true,
            "entry": "main.py"
        }"#;

        let meta = PluginMetadata::from_json(json).expect("Must parse plugin json");
        assert_eq!(meta.id, "test.plugin");
        assert_eq!(meta.name, "Test Plugin");
        assert_eq!(meta.version, "1.2.3");
        assert_eq!(meta.author, "Alice");
        assert!(meta.enabled);
        assert_eq!(meta.entry, Some("main.py".to_string()));

        let serialized = meta.to_json();
        let meta2 = PluginMetadata::from_json(&serialized).expect("Must re-parse serialized json");
        assert_eq!(meta2.id, meta.id);
        assert_eq!(meta2.name, meta.name);
        assert_eq!(meta2.entry, Some("main.py".to_string()));
    }

    #[test]
    fn test_keybindings_parsing_and_format() {
        let sc = Shortcut::new(gdk::Key::F10, false, true, false);
        assert_eq!(sc.to_display_string(), "Shift + F10");
        assert_eq!(sc.to_config_string(), "Shift+F10");

        let parsed = Shortcut::from_config_string("Shift+F10").expect("Must parse Shift+F10");
        assert_eq!(parsed.key, gdk::Key::F10);
        assert!(parsed.shift);
        assert!(!parsed.ctrl);
        assert!(!parsed.alt);

        let parsed_ctrl_s = Shortcut::from_config_string("Ctrl+S").expect("Must parse Ctrl+S");
        assert!(parsed_ctrl_s.ctrl);
        assert_eq!(parsed_ctrl_s.to_display_string(), "Ctrl + S");

        let default_cfg = KeybindingsConfig::default();
        assert_eq!(default_cfg.run.to_display_string(), "F5");
        assert_eq!(default_cfg.step.to_display_string(), "F10");
        assert_eq!(default_cfg.reset.to_display_string(), "F8");
    }
}
