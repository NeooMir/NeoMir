// NeoMir - Educational Programming Environment (KuMir language)
// Main entry point

mod lang;
mod plugins;
mod robot;
mod ui;

use crate::ui::AppWindow;
use adw::prelude::*;

const APP_ID: &str = "io.github.neomir.app";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let app_window = AppWindow::new(app);
    app_window.window.present();
}

#[cfg(test)]
mod tests {
    use crate::lang::compile_source;
    use crate::lang::vm::StepResult;
    use crate::plugins::PluginMetadata;
    use crate::robot::{PerformerMode, RobotField};
    use crate::ui::keybindings::{KeybindingsConfig, Shortcut};
    use crate::ui::settings_dialog::AppSettings;
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
    fn test_multiple_runs_reexecution() {
        let code = r#"использовать Робот
алг
нач
    вправо
    закрасить
кон
"#;
        let mut field = RobotField::new(10, 10);

        // Run 1
        let mut vm1 = compile_source(code).expect("Compilation must succeed");
        loop {
            match vm1.step(&mut field) {
                StepResult::Finished => break,
                _ => {}
            }
        }
        assert_eq!(field.robot_x, 1);
        assert!(field.painted[0][1]);

        // Re-execute: reset field execution and compile fresh VM
        field.reset_execution();
        assert_eq!(field.robot_x, 0);
        assert!(!field.painted[0][1]);

        // Run 2 must succeed identically
        let mut vm2 = compile_source(code).expect("Compilation must succeed");
        loop {
            match vm2.step(&mut field) {
                StepResult::Finished => break,
                _ => {}
            }
        }
        assert_eq!(field.robot_x, 1);
        assert!(field.painted[0][1]);
    }

    #[test]
    fn test_turtle_movement_and_pen() {
        let mut field = RobotField::new(10, 10);
        field.performer = PerformerMode::Turtle;
        assert_eq!(field.turtle_angle, 0.0);
        assert!(field.turtle_pen_down);
        assert!(field.turtle_lines.is_empty());

        field.turtle_forward(2.0);
        assert_eq!(field.turtle_lines.len(), 1);
        assert_eq!(field.robot_x, 2);

        field.turtle_turn_right(90.0);
        assert_eq!(field.turtle_angle, 270.0);

        field.turtle_pen_up();
        assert!(!field.turtle_pen_down);

        field.turtle_forward(1.0);
        assert_eq!(field.turtle_lines.len(), 1); // No new line when pen is up
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

    #[test]
    fn test_app_settings_persistence() {
        let default_settings = AppSettings::default();
        assert_eq!(default_settings.performer, "robot");
        assert_eq!(default_settings.language, "ru");
        assert!(!default_settings.vim_mode);

        let json = serde_json::to_string(&default_settings).expect("Serialization must succeed");
        let loaded: AppSettings = serde_json::from_str(&json).expect("Deserialization must succeed");
        assert_eq!(loaded.performer, "robot");
        assert_eq!(loaded.language, "ru");
        assert_eq!(loaded.theme, default_settings.theme);
        assert!(!loaded.vim_mode);

        let mut custom = default_settings.clone();
        custom.performer = "turtle".to_string();
        custom.language = "en".to_string();
        custom.vim_mode = true;
        let json_custom = serde_json::to_string(&custom).unwrap();
        let loaded_custom: AppSettings = serde_json::from_str(&json_custom).unwrap();
        assert_eq!(loaded_custom.performer, "turtle");
        assert_eq!(loaded_custom.language, "en");
        assert!(loaded_custom.vim_mode);
    }

    #[test]
    fn test_english_kumir_program_execution() {
        let code = "alg test_en\nbegin\n  right\n  down\n  paint\nend\n";
        let mut vm = compile_source(code).expect("Should compile English KuMir");
        let mut field = RobotField::new(10, 10);
        loop {
            match vm.step(&mut field) {
                StepResult::Finished => break,
                StepResult::Error { message, .. } => panic!("Execution error: {}", message),
                _ => {}
            }
        }
        assert_eq!(field.robot_x, 1);
        assert_eq!(field.robot_y, 1);
        assert!(field.painted[1][1]);
    }

    #[test]
    fn test_i18n_dictionary() {
        use crate::ui::i18n::Language;
        let en = Language::En;
        let ru = Language::Ru;
        assert_eq!(en.tr("run"), "Run");
        assert_eq!(ru.tr("run"), "Выполнить");
        assert_eq!(en.tr("settings_title"), "NeoMir Settings");
        assert_eq!(ru.tr("settings_title"), "Настройки NeoMir");
        assert_eq!(en.tr("active_performer"), "Active Performer");
        assert_eq!(ru.tr("active_performer"), "Активный исполнитель");
        assert!(en.is_en());
        assert!(!ru.is_en());
        assert_eq!(Language::from_code("en"), Language::En);
        assert_eq!(Language::from_code("ru"), Language::Ru);
    }

    #[test]
    fn test_robot_step_back() {
        let mut field = RobotField::new(10, 10);
        assert_eq!(field.robot_x, 0);
        assert_eq!(field.robot_y, 0);

        // Move right then down
        let _ = field.move_robot(&crate::robot::Direction::Right);
        assert_eq!(field.robot_x, 1);
        assert_eq!(field.robot_y, 0);

        let _ = field.move_robot(&crate::robot::Direction::Down);
        assert_eq!(field.robot_x, 1);
        assert_eq!(field.robot_y, 1);

        // Step back should return to (1, 0)
        let stepped = field.step_back();
        assert!(stepped);
        assert_eq!(field.robot_x, 1);
        assert_eq!(field.robot_y, 0);

        // Step back again should return to (0, 0)
        let stepped2 = field.step_back();
        assert!(stepped2);
        assert_eq!(field.robot_x, 0);
        assert_eq!(field.robot_y, 0);

        // Third step back at origin should return false and keep at start
        let stepped3 = field.step_back();
        assert!(!stepped3);
        assert_eq!(field.robot_x, 0);
        assert_eq!(field.robot_y, 0);
    }

    #[test]
    fn test_syntax_mode_toggle() {
        use crate::ui::editor_view::SyntaxMode;
        assert_eq!(SyntaxMode::from_str("kumir"), SyntaxMode::Kumir);
        assert_eq!(SyntaxMode::from_str("python"), SyntaxMode::Python);
        assert_eq!(SyntaxMode::from_str("py"), SyntaxMode::Python);
        assert_eq!(SyntaxMode::from_str("unknown"), SyntaxMode::Kumir);
        assert_eq!(SyntaxMode::Kumir.as_str(), "kumir");
        assert_eq!(SyntaxMode::Python.as_str(), "python");
    }
}
