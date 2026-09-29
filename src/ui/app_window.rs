use crate::lang::{compile_source, vm::StepResult, vm::VirtualMachine};
use crate::plugins::ipc::IpcServer;
use crate::robot::RobotField;
use crate::ui::console_view::ConsoleView;
use crate::ui::editor_view::EditorView;
use crate::ui::field_view::RobotFieldView;
use crate::ui::keybindings::KeybindingsConfig;
use crate::ui::settings_dialog::SettingsDialog;
use adw::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct AppWindow {
    pub window: adw::ApplicationWindow,
}

impl AppWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("NeoMir")
            .default_width(1150)
            .default_height(780)
            .build();

        let field = Rc::new(RefCell::new(RobotField::new(10, 10)));
        let editor_view = Rc::new(EditorView::new());
        let field_view = Rc::new(RobotFieldView::new(field.clone()));
        let console_view = Rc::new(ConsoleView::new());

        editor_view.buffer.set_text("использовать Робот\nалг\nнач\n    \nкон\n");

        let vm: Rc<RefCell<Option<VirtualMachine>>> = Rc::new(RefCell::new(None));
        let is_running = Rc::new(RefCell::new(false));
        let keybindings = Rc::new(RefCell::new(KeybindingsConfig::load()));

        // Status Label & IPC Server
        let status_label = gtk::Label::new(Some("Готов к работе"));
        status_label.add_css_class("dim-label");

        IpcServer::start(
            field.clone(),
            field_view.clone(),
            console_view.clone(),
            status_label.clone(),
        );

        // HeaderBar Buttons
        let btn_run = gtk::Button::new();
        Self::set_btn_content(&btn_run, "media-playback-start-symbolic", "Выполнить");
        btn_run.add_css_class("suggested-action");

        let btn_step = gtk::Button::new();
        Self::set_btn_content(&btn_step, "media-seek-forward-symbolic", "Шаг");

        let btn_reset = gtk::Button::new();
        Self::set_btn_content(&btn_reset, "view-refresh-symbolic", "Сброс");

        let btn_load = gtk::Button::new();
        Self::set_btn_content(&btn_load, "document-open-symbolic", "Загрузить");

        let btn_save = gtk::Button::new();
        Self::set_btn_content(&btn_save, "document-save-symbolic", "Сохранить");

        // Tooltips with shortcut keys
        let update_tooltips = {
            let kb = keybindings.clone();
            let b_run = btn_run.clone();
            let b_step = btn_step.clone();
            let b_reset = btn_reset.clone();
            let b_load = btn_load.clone();
            let b_save = btn_save.clone();
            Rc::new(move || {
                let cfg = kb.borrow();
                b_run.set_tooltip_text(Some(&format!(
                    "Запустить выполнение программы ({})",
                    cfg.run.to_display_string()
                )));
                b_step.set_tooltip_text(Some(&format!(
                    "Выполнить один шаг программы ({})",
                    cfg.step.to_display_string()
                )));
                b_reset.set_tooltip_text(Some(&format!(
                    "Сбросить робота в исходное состояние ({})",
                    cfg.reset.to_display_string()
                )));
                b_load.set_tooltip_text(Some(&format!(
                    "Загрузить файл алгоритма ({})",
                    cfg.load.to_display_string()
                )));
                b_save.set_tooltip_text(Some(&format!(
                    "Сохранить файл алгоритма ({})",
                    cfg.save.to_display_string()
                )));
            })
        };
        update_tooltips();

        // Hamburger Menu (3 тире) Popover
        let menu_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);

        let btn_menu_settings = gtk::Button::new();
        Self::set_btn_content(&btn_menu_settings, "preferences-system-symbolic", "Настройки");
        btn_menu_settings.add_css_class("flat");
        menu_box.append(&btn_menu_settings);

        let btn_menu_extensions = gtk::Button::new();
        Self::set_btn_content(&btn_menu_extensions, "application-x-addon-symbolic", "Расширения");
        btn_menu_extensions.add_css_class("flat");
        menu_box.append(&btn_menu_extensions);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        menu_box.append(&sep);

        let btn_menu_about = gtk::Button::new();
        Self::set_btn_content(&btn_menu_about, "help-about-symbolic", "О программе");
        btn_menu_about.add_css_class("flat");
        menu_box.append(&btn_menu_about);

        let menu_popover = gtk::Popover::new();
        menu_popover.set_child(Some(&menu_box));

        let btn_menu = gtk::MenuButton::new();
        btn_menu.set_icon_name("open-menu-symbolic");
        btn_menu.set_tooltip_text(Some("Главное меню"));
        btn_menu.set_popover(Some(&menu_popover));

        // Connect menu actions
        {
            let win = window.clone();
            let ev = editor_view.clone();
            let kb = keybindings.clone();
            let ut = update_tooltips.clone();
            let pop = menu_popover.clone();
            btn_menu_settings.connect_clicked(move |_| {
                pop.popdown();
                SettingsDialog::show(&win, &ev, &kb, ut.clone());
            });
        }
        {
            let win = window.clone();
            let ev = editor_view.clone();
            let kb = keybindings.clone();
            let ut = update_tooltips.clone();
            let pop = menu_popover.clone();
            btn_menu_extensions.connect_clicked(move |_| {
                pop.popdown();
                SettingsDialog::show_extensions(&win, &ev, &kb, ut.clone());
            });
        }
        {
            let win = window.clone();
            let pop = menu_popover.clone();
            btn_menu_about.connect_clicked(move |_| {
                pop.popdown();
                let about = adw::AboutDialog::builder()
                    .application_name("NeoMir")
                    .developer_name("Разработчики NeoMir")
                    .version(env!("CARGO_PKG_VERSION"))
                    .comments("Современная среда учебного программирования на алгоритмическом языке КуМир с Исполнителем «Робот».\nРазработано на Rust, GTK4 и Libadwaita.")
                    .website("https://github.com/neoomir/neomir")
                    .issue_url("https://github.com/neoomir/neomir/issues")
                    .license_type(gtk::License::Gpl20)
                    .application_icon("io.neoomir.neomir.app")
                    .copyright("© 2026 NeoMir")
                    .build();
                about.present(Some(&win));
            });
        }

        // HeaderBar assembly
        let header = adw::HeaderBar::new();
        let title_widget = adw::WindowTitle::new("NeoMir", "Исполнитель: Робот | Поле: 10×10");
        header.set_title_widget(Some(&title_widget));

        let left_controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        left_controls.append(&btn_run);
        left_controls.append(&btn_step);
        left_controls.append(&btn_reset);
        left_controls.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        left_controls.append(&btn_load);
        left_controls.append(&btn_save);
        header.pack_start(&left_controls);

        header.pack_end(&btn_menu);

        // Load / Save Handlers
        {
            let win = window.clone();
            let ev = editor_view.clone();
            let con = console_view.clone();
            btn_load.connect_clicked(move |_| {
                let dialog = gtk::FileChooserNative::new(
                    Some("Загрузить алгоритм"),
                    Some(&win),
                    gtk::FileChooserAction::Open,
                    Some("Открыть"),
                    Some("Отмена"),
                );
                let filter_kum = gtk::FileFilter::new();
                filter_kum.set_name(Some("КуМир программы (*.kum)"));
                filter_kum.add_pattern("*.kum");
                dialog.add_filter(&filter_kum);

                let filter_all = gtk::FileFilter::new();
                filter_all.set_name(Some("Все файлы (*.*)"));
                filter_all.add_pattern("*");
                dialog.add_filter(&filter_all);

                let ev_clone = ev.clone();
                let con_clone = con.clone();
                dialog.connect_response(move |d, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                match std::fs::read_to_string(&path) {
                                    Ok(code) => {
                                        ev_clone.buffer.set_text(&code);
                                        ev_clone.update_gutter();
                                        con_clone.print_info(&format!(
                                            "Загружен файл: {}",
                                            path.display()
                                        ));
                                    }
                                    Err(e) => {
                                        con_clone.print_error(&format!(
                                            "Ошибка при чтении файла: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            });
        }

        {
            let win = window.clone();
            let ev = editor_view.clone();
            let con = console_view.clone();
            btn_save.connect_clicked(move |_| {
                let dialog = gtk::FileChooserNative::new(
                    Some("Сохранить алгоритм"),
                    Some(&win),
                    gtk::FileChooserAction::Save,
                    Some("Сохранить"),
                    Some("Отмена"),
                );
                let filter_kum = gtk::FileFilter::new();
                filter_kum.set_name(Some("КуМир программы (*.kum)"));
                filter_kum.add_pattern("*.kum");
                dialog.add_filter(&filter_kum);

                let filter_all = gtk::FileFilter::new();
                filter_all.set_name(Some("Все файлы (*.*)"));
                filter_all.add_pattern("*");
                dialog.add_filter(&filter_all);

                let ev_clone = ev.clone();
                let con_clone = con.clone();
                dialog.connect_response(move |d, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                let start = ev_clone.buffer.start_iter();
                                let end = ev_clone.buffer.end_iter();
                                let text = ev_clone.buffer.text(&start, &end, false);
                                match std::fs::write(&path, text.as_str()) {
                                    Ok(_) => {
                                        con_clone.print_success(&format!(
                                            "Файл успешно сохранён: {}",
                                            path.display()
                                        ));
                                    }
                                    Err(e) => {
                                        con_clone.print_error(&format!(
                                            "Ошибка при сохранении: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            });
        }

        // Field Tools Bar
        let field_tools_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        field_tools_box.set_margin_start(8);
        field_tools_box.set_margin_end(8);
        field_tools_box.set_margin_top(6);
        field_tools_box.set_margin_bottom(6);

        let lbl_field = gtk::Label::new(Some("Поле:"));
        lbl_field.add_css_class("heading");
        field_tools_box.append(&lbl_field);

        let btn_clear_walls = gtk::Button::with_label("Стереть стены");
        btn_clear_walls.add_css_class("flat");
        field_tools_box.append(&btn_clear_walls);

        let btn_clear_paint = gtk::Button::with_label("Стереть краску");
        btn_clear_paint.add_css_class("flat");
        field_tools_box.append(&btn_clear_paint);

        field_tools_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        let btn_size_10 = gtk::Button::with_label("10×10");
        btn_size_10.add_css_class("flat");
        field_tools_box.append(&btn_size_10);

        let btn_size_15 = gtk::Button::with_label("15×15");
        btn_size_15.add_css_class("flat");
        field_tools_box.append(&btn_size_15);

        let btn_size_20 = gtk::Button::with_label("20×20");
        btn_size_20.add_css_class("flat");
        field_tools_box.append(&btn_size_20);

        Self::bind_field_tools(
            &field,
            &field_view,
            &title_widget,
            &btn_clear_walls,
            &btn_clear_paint,
            &btn_size_10,
            &btn_size_15,
            &btn_size_20,
        );

        // Right side: Field + Field Tools + Console View
        let right_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        right_box.append(&field_tools_box);
        right_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        right_box.append(&field_view.widget);
        right_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        right_box.append(&console_view.container);

        // Split Paned: Editor on left, Field + Console on right
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_start_child(Some(&editor_view.container));
        paned.set_end_child(Some(&right_box));
        paned.set_position(550);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        // Bottom status bar
        let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        status_box.set_margin_start(10);
        status_box.set_margin_end(10);
        status_box.set_margin_top(4);
        status_box.set_margin_bottom(4);
        status_box.append(&status_label);

        // Window root container
        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root_box.append(&header);
        root_box.append(&paned);
        root_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        root_box.append(&status_box);

        window.set_content(Some(&root_box));

        // Step function closure
        let do_step_fn = {
            let vm_clone = vm.clone();
            let f_clone = field.clone();
            let ev_clone = editor_view.clone();
            let fv_clone = field_view.clone();
            let con_clone = console_view.clone();
            let stat_clone = status_label.clone();
            Rc::new(move || -> StepResult {
                Self::do_step(
                    &vm_clone,
                    &f_clone,
                    &ev_clone,
                    &fv_clone,
                    &con_clone,
                    &stat_clone,
                )
            })
        };

        // Reset function closure
        let do_reset_fn = {
            let vm_clone = vm.clone();
            let f_clone = field.clone();
            let ev_clone = editor_view.clone();
            let fv_clone = field_view.clone();
            let con_clone = console_view.clone();
            let stat_clone = status_label.clone();
            let is_r = is_running.clone();
            let b_run = btn_run.clone();
            Rc::new(move || {
                *is_r.borrow_mut() = false;
                Self::set_btn_content(&b_run, "media-playback-start-symbolic", "Выполнить");
                *vm_clone.borrow_mut() = None;
                f_clone.borrow_mut().reset_execution();
                fv_clone.widget.queue_draw();
                ev_clone.highlight_line(None);
                stat_clone.set_text("Готов к работе");
                con_clone.print_info("Робот сброшен в исходную позицию.");
            })
        };

        // Connect Reset Button
        {
            let r_fn = do_reset_fn.clone();
            btn_reset.connect_clicked(move |_| {
                r_fn();
            });
        }

        // Connect Step Button
        {
            let s_fn = do_step_fn.clone();
            btn_step.connect_clicked(move |_| {
                let _ = s_fn();
            });
        }

        // Connect Run / Pause Button
        {
            let is_r = is_running.clone();
            let b_run = btn_run.clone();
            let s_fn = do_step_fn.clone();
            let con = console_view.clone();
            let vm_ref = vm.clone();
            let ev_ref = editor_view.clone();

            btn_run.connect_clicked(move |_| {
                let mut running = is_r.borrow_mut();
                if *running {
                    *running = false;
                    Self::set_btn_content(&b_run, "media-playback-start-symbolic", "Выполнить");
                    return;
                }

                // If starting fresh
                if vm_ref.borrow().is_none() {
                    let start = ev_ref.buffer.start_iter();
                    let end = ev_ref.buffer.end_iter();
                    let code = ev_ref.buffer.text(&start, &end, false);
                    con.print_info("Компиляция...");
                    match compile_source(code.as_str()) {
                        Ok(compiled_vm) => {
                            *vm_ref.borrow_mut() = Some(compiled_vm);
                        }
                        Err(e) => {
                            con.print_error(&format!("Ошибка компиляции: {}", e));
                            return;
                        }
                    }
                }

                *running = true;
                Self::set_btn_content(&b_run, "media-playback-pause-symbolic", "Пауза");
                drop(running);

                let is_r_loop = is_r.clone();
                let b_run_loop = b_run.clone();
                let s_fn_loop = s_fn.clone();

                glib::timeout_add_local(Duration::from_millis(60), move || {
                    if !*is_r_loop.borrow() {
                        return glib::ControlFlow::Break;
                    }
                    let res = s_fn_loop();
                    match res {
                        StepResult::Finished | StepResult::Error { .. } => {
                            *is_r_loop.borrow_mut() = false;
                            Self::set_btn_content(
                                &b_run_loop,
                                "media-playback-start-symbolic",
                                "Выполнить",
                            );
                            glib::ControlFlow::Break
                        }
                        _ => glib::ControlFlow::Continue,
                    }
                });
            });
        }

        // Global Keybindings Controller
        let key_controller = gtk::EventControllerKey::new();
        {
            let kb_cfg = keybindings.clone();
            let b_run = btn_run.clone();
            let b_step = btn_step.clone();
            let b_load = btn_load.clone();
            let b_save = btn_save.clone();
            let r_fn = do_reset_fn.clone();

            key_controller.connect_key_pressed(move |_, keyval, _keycode, state| {
                let cfg = kb_cfg.borrow();
                if cfg.run.matches(keyval, state) {
                    b_run.emit_clicked();
                    return glib::Propagation::Stop;
                }
                if cfg.step.matches(keyval, state) {
                    b_step.emit_clicked();
                    return glib::Propagation::Stop;
                }
                if cfg.reset.matches(keyval, state) {
                    r_fn();
                    return glib::Propagation::Stop;
                }
                if cfg.load.matches(keyval, state) {
                    b_load.emit_clicked();
                    return glib::Propagation::Stop;
                }
                if cfg.save.matches(keyval, state) {
                    b_save.emit_clicked();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
        }
        window.add_controller(key_controller);

        Self { window }
    }

    fn set_btn_content(btn: &gtk::Button, icon: &str, label: &str) {
        let content = adw::ButtonContent::new();
        content.set_icon_name(icon);
        content.set_label(label);
        btn.set_child(Some(&content));
    }

    fn bind_field_tools(
        field: &Rc<RefCell<RobotField>>,
        field_view: &Rc<RobotFieldView>,
        title_widget: &adw::WindowTitle,
        btn_clear_walls: &gtk::Button,
        btn_clear_paint: &gtk::Button,
        btn_size_10: &gtk::Button,
        btn_size_15: &gtk::Button,
        btn_size_20: &gtk::Button,
    ) {
        {
            let f = field.clone();
            let fv = field_view.clone();
            btn_clear_walls.connect_clicked(move |_| {
                let mut field_mut = f.borrow_mut();
                for row in field_mut.h_walls.iter_mut() {
                    row.fill(false);
                }
                for row in field_mut.v_walls.iter_mut() {
                    row.fill(false);
                }
                fv.widget.queue_draw();
            });
        }

        {
            let f = field.clone();
            let fv = field_view.clone();
            btn_clear_paint.connect_clicked(move |_| {
                let mut field_mut = f.borrow_mut();
                for row in field_mut.painted.iter_mut() {
                    row.fill(false);
                }
                for row in field_mut.initial_painted.iter_mut() {
                    row.fill(false);
                }
                fv.widget.queue_draw();
            });
        }

        {
            let f = field.clone();
            let fv = field_view.clone();
            let tw = title_widget.clone();
            btn_size_10.connect_clicked(move |_| {
                *f.borrow_mut() = RobotField::new(10, 10);
                tw.set_subtitle("Исполнитель: Робот | Поле: 10×10");
                fv.widget.queue_draw();
            });
        }
        {
            let f = field.clone();
            let fv = field_view.clone();
            let tw = title_widget.clone();
            btn_size_15.connect_clicked(move |_| {
                *f.borrow_mut() = RobotField::new(15, 15);
                tw.set_subtitle("Исполнитель: Робот | Поле: 15×15");
                fv.widget.queue_draw();
            });
        }
        {
            let f = field.clone();
            let fv = field_view.clone();
            let tw = title_widget.clone();
            btn_size_20.connect_clicked(move |_| {
                *f.borrow_mut() = RobotField::new(20, 20);
                tw.set_subtitle("Исполнитель: Робот | Поле: 20×20");
                fv.widget.queue_draw();
            });
        }
    }

    fn do_step(
        vm: &Rc<RefCell<Option<VirtualMachine>>>,
        field: &Rc<RefCell<RobotField>>,
        editor_view: &Rc<EditorView>,
        field_view: &Rc<RobotFieldView>,
        console_view: &Rc<ConsoleView>,
        status_label: &gtk::Label,
    ) -> StepResult {
        if vm.borrow().is_none() {
            let start = editor_view.buffer.start_iter();
            let end = editor_view.buffer.end_iter();
            let code = editor_view.buffer.text(&start, &end, false);
            console_view.print_info("Компиляция для пошагового выполнения...");
            match compile_source(code.as_str()) {
                Ok(compiled_vm) => {
                    *vm.borrow_mut() = Some(compiled_vm);
                }
                Err(e) => {
                    console_view.print_error(&format!("Ошибка компиляции: {}", e));
                    return StepResult::Error {
                        line: 1,
                        message: e,
                    };
                }
            }
        }

        let mut vm_mut = vm.borrow_mut();
        let current_vm = match vm_mut.as_mut() {
            Some(v) => v,
            None => return StepResult::Finished,
        };

        let result = current_vm.step(&mut field.borrow_mut());
        match &result {
            StepResult::Stepped { line } => {
                editor_view.highlight_line(Some(*line));
                status_label.set_text(&format!("Строка {}", line));
            }
            StepResult::RobotMoved { line } => {
                editor_view.highlight_line(Some(*line));
                field_view.widget.queue_draw();
                status_label.set_text(&format!("Робот переместился (строка {})", line));
            }
            StepResult::Output(msg) => {
                console_view.print_output(msg);
            }
            StepResult::Finished => {
                editor_view.highlight_line(None);
                status_label.set_text("Выполнение завершено");
                console_view.print_success("Программа успешно завершена.");
            }
            StepResult::Error { line, message } => {
                editor_view.highlight_line(Some(*line));
                field_view.widget.queue_draw();
                status_label.set_text(&format!("Ошибка на строке {}", line));
                console_view.print_error(&format!("Строка {}: {}", line, message));
            }
        }
        result
    }
}
