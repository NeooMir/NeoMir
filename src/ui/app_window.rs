use crate::lang::{compile_source, vm::StepResult, vm::VirtualMachine};
use crate::plugins::ipc::IpcServer;
use crate::robot::{PerformerMode, RobotField};
use crate::ui::console_view::ConsoleView;
use crate::ui::editor_view::{EditorView, SyntaxMode};
use crate::ui::field_view::RobotFieldView;
use crate::ui::i18n::Language;
use crate::ui::keybindings::KeybindingsConfig;
use crate::ui::settings_dialog::{AppSettings, SettingsDialog};
use crate::ui::theme::ThemeId;
use adw::prelude::*;
use gtk::{gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

pub struct AppWindow {
    pub window: adw::ApplicationWindow,
}

impl AppWindow {
    fn set_btn_content(btn: &gtk::Button, icon: &str, label: &str) {
        let content = adw::ButtonContent::new();
        content.set_icon_name(icon);
        content.set_label(label);
        btn.set_child(Some(&content));
    }

    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("NeoMir")
            .default_width(1150)
            .default_height(780)
            .build();

        let app_settings = Rc::new(RefCell::new(AppSettings::load()));
        let current_lang = Language::from_code(&app_settings.borrow().language);
        let lang_ref = Rc::new(RefCell::new(current_lang));

        let field = Rc::new(RefCell::new(RobotField::new(10, 10)));
        let editor_view = Rc::new(EditorView::new());
        let field_view = Rc::new(RobotFieldView::new(field.clone()));
        let console_view = Rc::new(ConsoleView::new(current_lang));

        let vm: Rc<RefCell<Option<VirtualMachine>>> = Rc::new(RefCell::new(None));
        let is_running = Rc::new(RefCell::new(false));
        let keybindings = Rc::new(RefCell::new(KeybindingsConfig::load()));

        // Apply saved performer
        let initial_performer = if app_settings.borrow().performer == "turtle" {
            PerformerMode::Turtle
        } else {
            PerformerMode::Robot
        };
        field.borrow_mut().performer = initial_performer;

        // Apply saved theme
        let saved_theme = app_settings.borrow().theme.clone();
        if let Some(t) = ThemeId::all().iter().find(|t| t.name() == saved_theme) {
            editor_view.apply_theme(*t);
        }

        // Apply saved Vim mode
        editor_view.vim.set_enabled(app_settings.borrow().vim_mode);

        // Apply saved syntax mode
        let initial_syntax = SyntaxMode::from_str(&app_settings.borrow().syntax_mode);
        editor_view.set_syntax_mode(initial_syntax);

        // Status Label & IPC Server
        let initial_status_text = if initial_performer == PerformerMode::Turtle {
            current_lang.tr("status_active_turtle")
        } else {
            current_lang.tr("status_ready")
        };
        let status_label = gtk::Label::new(Some(initial_status_text));
        status_label.add_css_class("dim-label");

        IpcServer::start(
            field.clone(),
            field_view.clone(),
            console_view.clone(),
            status_label.clone(),
        );

        // Top HeaderBar Buttons: Primary Execution Only
        let btn_run = gtk::Button::new();
        Self::set_btn_content(&btn_run, "media-playback-start-symbolic", current_lang.tr("run"));
        btn_run.add_css_class("suggested-action");

        let btn_step = gtk::Button::new();
        Self::set_btn_content(&btn_step, "media-seek-forward-symbolic", current_lang.tr("step"));

        // Field Tools Bar
        let field_tools_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        field_tools_box.set_margin_start(8);
        field_tools_box.set_margin_end(8);
        field_tools_box.set_margin_top(6);
        field_tools_box.set_margin_bottom(6);

        let lbl_field = gtk::Label::new(Some(current_lang.tr("field_label")));
        lbl_field.add_css_class("heading");
        field_tools_box.append(&lbl_field);

        let btn_clear_walls = gtk::Button::with_label(current_lang.tr("walls"));
        btn_clear_walls.add_css_class("flat");
        btn_clear_walls.set_tooltip_text(Some(current_lang.tr("tip_clear_walls")));
        field_tools_box.append(&btn_clear_walls);

        let btn_clear_paint = gtk::Button::with_label(current_lang.tr("clear_paint"));
        btn_clear_paint.add_css_class("flat");
        btn_clear_paint.set_tooltip_text(Some(current_lang.tr("tip_clear_paint")));
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

        // Hamburger Menu Popover (All operations consolidated here)
        let menu_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);

        // Section 1: File Operations
        let btn_menu_load = gtk::Button::new();
        Self::set_btn_content(&btn_menu_load, "document-open-symbolic", current_lang.tr("menu_load"));
        btn_menu_load.add_css_class("flat");
        menu_box.append(&btn_menu_load);

        let btn_menu_save = gtk::Button::new();
        Self::set_btn_content(&btn_menu_save, "document-save-symbolic", current_lang.tr("menu_save"));
        btn_menu_save.add_css_class("flat");
        menu_box.append(&btn_menu_save);

        menu_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Section 2: Execution & Field Control
        let btn_menu_reset = gtk::Button::new();
        Self::set_btn_content(&btn_menu_reset, "view-refresh-symbolic", current_lang.tr("menu_reset"));
        btn_menu_reset.add_css_class("flat");
        menu_box.append(&btn_menu_reset);

        let btn_menu_step_back = gtk::Button::new();
        Self::set_btn_content(&btn_menu_step_back, "edit-undo-symbolic", current_lang.tr("menu_step_back"));
        btn_menu_step_back.add_css_class("flat");
        btn_menu_step_back.set_tooltip_text(Some(current_lang.tr("tip_step_back")));
        menu_box.append(&btn_menu_step_back);

        menu_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Section 3: Syntax Mode & Python Environment
        let btn_menu_syntax = gtk::Button::new();
        let initial_syn_label = match initial_syntax {
            SyntaxMode::Kumir => current_lang.tr("syntax_kumir"),
            SyntaxMode::Python => current_lang.tr("syntax_python"),
        };
        Self::set_btn_content(&btn_menu_syntax, "format-text-code-symbolic", initial_syn_label);
        btn_menu_syntax.add_css_class("flat");
        btn_menu_syntax.set_tooltip_text(Some(current_lang.tr("tip_syntax_toggle")));
        menu_box.append(&btn_menu_syntax);

        let btn_menu_python = gtk::Button::new();
        Self::set_btn_content(&btn_menu_python, "utilities-terminal-symbolic", current_lang.tr("py_console"));
        btn_menu_python.add_css_class("flat");
        menu_box.append(&btn_menu_python);

        menu_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // Section 4: Application & Help
        let btn_menu_settings = gtk::Button::new();
        Self::set_btn_content(&btn_menu_settings, "preferences-system-symbolic", current_lang.tr("settings"));
        btn_menu_settings.add_css_class("flat");
        menu_box.append(&btn_menu_settings);

        let btn_menu_extensions = gtk::Button::new();
        Self::set_btn_content(&btn_menu_extensions, "application-x-addon-symbolic", current_lang.tr("extensions"));
        btn_menu_extensions.add_css_class("flat");
        menu_box.append(&btn_menu_extensions);

        let btn_menu_about = gtk::Button::new();
        Self::set_btn_content(&btn_menu_about, "help-about-symbolic", current_lang.tr("about"));
        btn_menu_about.add_css_class("flat");
        menu_box.append(&btn_menu_about);

        let menu_popover = gtk::Popover::new();
        menu_popover.set_child(Some(&menu_box));

        let btn_menu = gtk::MenuButton::new();
        btn_menu.set_icon_name("open-menu-symbolic");
        btn_menu.set_tooltip_text(Some(current_lang.tr("main_menu")));
        btn_menu.set_popover(Some(&menu_popover));

        // Title widget
        let initial_subtitle = if initial_performer == PerformerMode::Turtle {
            let tail = if field.borrow().turtle_pen_down { current_lang.tr("tail_down") } else { current_lang.tr("tail_up") };
            format!("Performer: Turtle | Angle: {}° | Tail: {}", field.borrow().turtle_angle.round() as i32, tail)
        } else {
            format!("Performer: Robot | Field: {}×{}", field.borrow().width, field.borrow().height)
        };
        let title_widget = adw::WindowTitle::new("NeoMir", &initial_subtitle);

        // Tooltips with shortcut keys
        let update_tooltips = {
            let kb = keybindings.clone();
            let lr = lang_ref.clone();
            let b_run = btn_run.clone();
            let b_step = btn_step.clone();
            let b_m_reset = btn_menu_reset.clone();
            let b_m_load = btn_menu_load.clone();
            let b_m_save = btn_menu_save.clone();
            let b_m_back = btn_menu_step_back.clone();
            let b_m_syn = btn_menu_syntax.clone();
            Rc::new(move || {
                let cfg = kb.borrow();
                let lang = *lr.borrow();
                let tip_run = lang.tr("tip_run").replace("{}", &cfg.run.to_display_string());
                let tip_step = lang.tr("tip_step").replace("{}", &cfg.step.to_display_string());
                let tip_reset = lang.tr("tip_reset").replace("{}", &cfg.reset.to_display_string());
                let tip_open = lang.tr("tip_open").replace("{}", &cfg.load.to_display_string());
                let tip_save = lang.tr("tip_save").replace("{}", &cfg.save.to_display_string());

                b_run.set_tooltip_text(Some(&tip_run));
                b_step.set_tooltip_text(Some(&tip_step));
                b_m_reset.set_tooltip_text(Some(&tip_reset));
                b_m_load.set_tooltip_text(Some(&tip_open));
                b_m_save.set_tooltip_text(Some(&tip_save));
                b_m_back.set_tooltip_text(Some(lang.tr("tip_step_back")));
                b_m_syn.set_tooltip_text(Some(lang.tr("tip_syntax_toggle")));
            })
        };
        update_tooltips();

        // Central Performer synchronization closure (updates title & status, NO EMOJIS)
        let sync_performer_ui = {
            let f = field.clone();
            let fv = field_view.clone();
            let tw = title_widget.clone();
            let stat = status_label.clone();
            let lr = lang_ref.clone();
            Rc::new(move || {
                let f_borrow = f.borrow();
                let lang = *lr.borrow();
                if f_borrow.performer == PerformerMode::Turtle {
                    let tail = if f_borrow.turtle_pen_down { lang.tr("tail_down") } else { lang.tr("tail_up") };
                    let angle = f_borrow.turtle_angle.round() as i32;
                    let template = lang.tr("title_turtle");
                    let sub = template
                        .replacen("{}", &angle.to_string(), 1)
                        .replacen("{}", tail, 1);
                    tw.set_subtitle(&sub);
                    stat.set_text(lang.tr("status_active_turtle"));
                } else {
                    let template = lang.tr("title_robot");
                    let sub = template
                        .replacen("{}", &f_borrow.width.to_string(), 1)
                        .replacen("{}", &f_borrow.height.to_string(), 1);
                    tw.set_subtitle(&sub);
                    stat.set_text(lang.tr("status_active_robot"));
                }
                fv.widget.queue_draw();
            })
        };
        sync_performer_ui();

        // Dynamic language switch updater closure
        let update_ui_language = {
            let lr = lang_ref.clone();
            let b_run = btn_run.clone();
            let b_step = btn_step.clone();
            let b_m_load = btn_menu_load.clone();
            let b_m_save = btn_menu_save.clone();
            let b_m_reset = btn_menu_reset.clone();
            let b_m_back = btn_menu_step_back.clone();
            let b_m_syntax = btn_menu_syntax.clone();
            let b_menu = btn_menu.clone();
            let b_m_settings = btn_menu_settings.clone();
            let b_m_ext = btn_menu_extensions.clone();
            let b_m_py = btn_menu_python.clone();
            let b_m_about = btn_menu_about.clone();
            let l_field = lbl_field.clone();
            let b_walls = btn_clear_walls.clone();
            let b_paint = btn_clear_paint.clone();
            let con = console_view.clone();
            let is_r = is_running.clone();
            let ut = update_tooltips.clone();
            let sync = sync_performer_ui.clone();
            let ev = editor_view.clone();

            Rc::new(move || {
                let lang = *lr.borrow();
                let run_label = if *is_r.borrow() { lang.tr("pause") } else { lang.tr("run") };
                let run_icon = if *is_r.borrow() { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" };
                Self::set_btn_content(&b_run, run_icon, run_label);
                Self::set_btn_content(&b_step, "media-seek-forward-symbolic", lang.tr("step"));

                Self::set_btn_content(&b_m_load, "document-open-symbolic", lang.tr("menu_load"));
                Self::set_btn_content(&b_m_save, "document-save-symbolic", lang.tr("menu_save"));
                Self::set_btn_content(&b_m_reset, "view-refresh-symbolic", lang.tr("menu_reset"));
                Self::set_btn_content(&b_m_back, "edit-undo-symbolic", lang.tr("menu_step_back"));

                let syn_label = match ev.syntax_mode() {
                    SyntaxMode::Kumir => lang.tr("syntax_kumir"),
                    SyntaxMode::Python => lang.tr("syntax_python"),
                };
                Self::set_btn_content(&b_m_syntax, "format-text-code-symbolic", syn_label);

                b_menu.set_tooltip_text(Some(lang.tr("main_menu")));
                Self::set_btn_content(&b_m_settings, "preferences-system-symbolic", lang.tr("settings"));
                Self::set_btn_content(&b_m_ext, "application-x-addon-symbolic", lang.tr("extensions"));
                Self::set_btn_content(&b_m_py, "utilities-terminal-symbolic", lang.tr("py_console"));
                Self::set_btn_content(&b_m_about, "help-about-symbolic", lang.tr("about"));

                l_field.set_text(lang.tr("field_label"));
                b_walls.set_label(lang.tr("walls"));
                b_walls.set_tooltip_text(Some(lang.tr("tip_clear_walls")));
                b_paint.set_label(lang.tr("clear_paint"));
                b_paint.set_tooltip_text(Some(lang.tr("tip_clear_paint")));

                con.set_language(lang);
                ut();
                sync();
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
            let lr_clone = lang_ref.clone();
            Rc::new(move || {
                let lang = *lr_clone.borrow();
                *is_r.borrow_mut() = false;
                Self::set_btn_content(&b_run, "media-playback-start-symbolic", lang.tr("run"));
                *vm_clone.borrow_mut() = None;
                f_clone.borrow_mut().reset_execution();
                fv_clone.widget.queue_draw();
                ev_clone.highlight_line(None);
                stat_clone.set_text(lang.tr("status_ready"));
                con_clone.print_info(lang.tr("field_reset_msg"));
            })
        };

        // Return Robot / Step Back function closure
        let do_step_back_fn = {
            let f_clone = field.clone();
            let fv_clone = field_view.clone();
            let con_clone = console_view.clone();
            let stat_clone = status_label.clone();
            let lr_clone = lang_ref.clone();
            let sync_clone = sync_performer_ui.clone();
            Rc::new(move || {
                let lang = *lr_clone.borrow();
                let mut f = f_clone.borrow_mut();
                let was_turtle = f.performer == PerformerMode::Turtle;
                let ok = f.step_back();
                let (rx, ry) = (f.robot_x + 1, f.robot_y + 1);
                let (sx, sy) = (f.start_x + 1, f.start_y + 1);
                drop(f);
                fv_clone.widget.queue_draw();
                sync_clone();

                if ok {
                    let template = if was_turtle { lang.tr("status_turtle_backed") } else { lang.tr("status_robot_backed") };
                    let msg = template.replacen("{}", &rx.to_string(), 1).replacen("{}", &ry.to_string(), 1);
                    con_clone.log_action(&msg);
                    stat_clone.set_text(&msg);
                } else {
                    let template = lang.tr("status_already_at_start");
                    let msg = template.replacen("{}", &sx.to_string(), 1).replacen("{}", &sy.to_string(), 1);
                    con_clone.log_action(&msg);
                    stat_clone.set_text(&msg);
                }
            })
        };

        // File Open / Load closure
        let do_load_fn = {
            let win = window.clone();
            let ev = editor_view.clone();
            let con = console_view.clone();
            let lr = lang_ref.clone();
            let st = app_settings.clone();
            let b_m_syn = btn_menu_syntax.clone();
            Rc::new(move || {
                let lang = *lr.borrow();
                let dialog = gtk::FileChooserNative::new(
                    Some(lang.tr("dialog_open_title")),
                    Some(&win),
                    gtk::FileChooserAction::Open,
                    Some(lang.tr("btn_open")),
                    Some(lang.tr("cancel")),
                );
                let filter_kum = gtk::FileFilter::new();
                filter_kum.set_name(Some(lang.tr("filter_kumir")));
                filter_kum.add_pattern("*.kum");
                dialog.add_filter(&filter_kum);

                let filter_py = gtk::FileFilter::new();
                filter_py.set_name(Some("Python (*.py)"));
                filter_py.add_pattern("*.py");
                dialog.add_filter(&filter_py);

                let filter_all = gtk::FileFilter::new();
                filter_all.set_name(Some(lang.tr("filter_all")));
                filter_all.add_pattern("*");
                dialog.add_filter(&filter_all);

                let ev_clone = ev.clone();
                let con_clone = con.clone();
                let lr_clone = lr.clone();
                let st_clone = st.clone();
                let b_syn_clone = b_m_syn.clone();

                dialog.connect_response(move |d, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                let l = *lr_clone.borrow();
                                match std::fs::read_to_string(&path) {
                                    Ok(code) => {
                                        ev_clone.buffer.set_text(&code);
                                        ev_clone.update_gutter();

                                        // Auto-detect syntax mode from extension
                                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                            if ext.eq_ignore_ascii_case("py") {
                                                ev_clone.set_syntax_mode(SyntaxMode::Python);
                                                st_clone.borrow_mut().syntax_mode = "python".to_string();
                                                let _ = st_clone.borrow().save();
                                                Self::set_btn_content(&b_syn_clone, "format-text-code-symbolic", l.tr("syntax_python"));
                                            } else if ext.eq_ignore_ascii_case("kum") {
                                                ev_clone.set_syntax_mode(SyntaxMode::Kumir);
                                                st_clone.borrow_mut().syntax_mode = "kumir".to_string();
                                                let _ = st_clone.borrow().save();
                                                Self::set_btn_content(&b_syn_clone, "format-text-code-symbolic", l.tr("syntax_kumir"));
                                            }
                                        }

                                        let msg = l.tr("file_loaded").replace("{}", &path.display().to_string());
                                        con_clone.print_info(&msg);
                                    }
                                    Err(e) => {
                                        con_clone.print_error(&format!("Error reading file: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            })
        };

        // File Save closure
        let do_save_fn = {
            let win = window.clone();
            let ev = editor_view.clone();
            let con = console_view.clone();
            let lr = lang_ref.clone();
            Rc::new(move || {
                let lang = *lr.borrow();
                let dialog = gtk::FileChooserNative::new(
                    Some(lang.tr("dialog_save_title")),
                    Some(&win),
                    gtk::FileChooserAction::Save,
                    Some(lang.tr("save")),
                    Some(lang.tr("cancel")),
                );
                let filter_kum = gtk::FileFilter::new();
                filter_kum.set_name(Some(lang.tr("filter_kumir")));
                filter_kum.add_pattern("*.kum");
                dialog.add_filter(&filter_kum);

                let filter_py = gtk::FileFilter::new();
                filter_py.set_name(Some("Python (*.py)"));
                filter_py.add_pattern("*.py");
                dialog.add_filter(&filter_py);

                let filter_all = gtk::FileFilter::new();
                filter_all.set_name(Some(lang.tr("filter_all")));
                filter_all.add_pattern("*");
                dialog.add_filter(&filter_all);

                let ev_clone = ev.clone();
                let con_clone = con.clone();
                let lr_clone = lr.clone();
                dialog.connect_response(move |d, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                let l = *lr_clone.borrow();
                                let start = ev_clone.buffer.start_iter();
                                let end = ev_clone.buffer.end_iter();
                                let text = ev_clone.buffer.text(&start, &end, false);
                                match std::fs::write(&path, text.as_str()) {
                                    Ok(_) => {
                                        let msg = l.tr("file_saved").replace("{}", &path.display().to_string());
                                        con_clone.print_success(&msg);
                                    }
                                    Err(e) => {
                                        con_clone.print_error(&format!("Error saving file: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            })
        };

        // Connect menu actions
        {
            let load_fn = do_load_fn.clone();
            let pop = menu_popover.clone();
            btn_menu_load.connect_clicked(move |_| {
                pop.popdown();
                load_fn();
            });
        }
        {
            let save_fn = do_save_fn.clone();
            let pop = menu_popover.clone();
            btn_menu_save.connect_clicked(move |_| {
                pop.popdown();
                save_fn();
            });
        }
        {
            let reset_fn = do_reset_fn.clone();
            let pop = menu_popover.clone();
            btn_menu_reset.connect_clicked(move |_| {
                pop.popdown();
                reset_fn();
            });
        }
        {
            let step_back_fn = do_step_back_fn.clone();
            let pop = menu_popover.clone();
            btn_menu_step_back.connect_clicked(move |_| {
                pop.popdown();
                step_back_fn();
            });
        }
        {
            let ev = editor_view.clone();
            let b_syn = btn_menu_syntax.clone();
            let lr = lang_ref.clone();
            let st = app_settings.clone();
            let stat = status_label.clone();
            let con = console_view.clone();
            let pop = menu_popover.clone();
            btn_menu_syntax.connect_clicked(move |_| {
                pop.popdown();
                let cur = ev.syntax_mode();
                let next = match cur {
                    SyntaxMode::Kumir => SyntaxMode::Python,
                    SyntaxMode::Python => SyntaxMode::Kumir,
                };
                ev.set_syntax_mode(next);
                st.borrow_mut().syntax_mode = next.as_str().to_string();
                let _ = st.borrow().save();

                let lang = *lr.borrow();
                let label = match next {
                    SyntaxMode::Kumir => lang.tr("syntax_kumir"),
                    SyntaxMode::Python => lang.tr("syntax_python"),
                };
                Self::set_btn_content(&b_syn, "format-text-code-symbolic", label);

                let notice = match next {
                    SyntaxMode::Kumir => if lang.is_en() { "Switched editor syntax to KuMir" } else { "Синтаксис редактора переключён на КуМир" },
                    SyntaxMode::Python => if lang.is_en() { "Switched editor syntax to Python" } else { "Синтаксис редактора переключён на Python" },
                };
                stat.set_text(notice);
                con.print_info(notice);
            });
        }
        {
            let con = console_view.clone();
            let pop = menu_popover.clone();
            btn_menu_python.connect_clicked(move |_| {
                pop.popdown();
                con.switch_to_python();
            });
        }
        {
            let win = window.clone();
            let ev = editor_view.clone();
            let kb = keybindings.clone();
            let ut = update_tooltips.clone();
            let fld = field.clone();
            let fv = field_view.clone();
            let sync = sync_performer_ui.clone();
            let st = app_settings.clone();
            let pop = menu_popover.clone();
            let lr = lang_ref.clone();
            let upd_lang = update_ui_language.clone();

            btn_menu_settings.connect_clicked(move |_| {
                pop.popdown();
                let lr_inner = lr.clone();
                let upd_inner = upd_lang.clone();
                let st_inner = st.clone();
                let on_lang_changed = Rc::new(move || {
                    *lr_inner.borrow_mut() = Language::from_code(&st_inner.borrow().language);
                    upd_inner();
                });
                SettingsDialog::show(&win, &ev, &kb, ut.clone(), &fld, &fv, sync.clone(), &st, on_lang_changed);
            });
        }
        {
            let win = window.clone();
            let ev = editor_view.clone();
            let kb = keybindings.clone();
            let ut = update_tooltips.clone();
            let fld = field.clone();
            let fv = field_view.clone();
            let sync = sync_performer_ui.clone();
            let st = app_settings.clone();
            let pop = menu_popover.clone();
            let lr = lang_ref.clone();
            let upd_lang = update_ui_language.clone();

            btn_menu_extensions.connect_clicked(move |_| {
                pop.popdown();
                let lr_inner = lr.clone();
                let upd_inner = upd_lang.clone();
                let st_inner = st.clone();
                let on_lang_changed = Rc::new(move || {
                    *lr_inner.borrow_mut() = Language::from_code(&st_inner.borrow().language);
                    upd_inner();
                });
                SettingsDialog::show_extensions(&win, &ev, &kb, ut.clone(), &fld, &fv, sync.clone(), &st, on_lang_changed);
            });
        }
        {
            let win = window.clone();
            let pop = menu_popover.clone();
            let lr = lang_ref.clone();
            btn_menu_about.connect_clicked(move |_| {
                pop.popdown();
                let lang = *lr.borrow();
                let about = adw::AboutDialog::builder()
                    .application_name("NeoMir")
                    .developer_name(lang.tr("about_devs"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .comments(lang.tr("about_comments"))
                    .website("https://github.com/neoomir/neomir")
                    .issue_url("https://github.com/neoomir/neomir/issues")
                    .license_type(gtk::License::Gpl20)
                    .application_icon("io.github.neomir.app")
                    .copyright("© 2026 NeoMir")
                    .build();
                about.present(Some(&win));
            });
        }

        // HeaderBar assembly: Primary controls only
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&title_widget));

        let left_controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        left_controls.append(&btn_run);
        left_controls.append(&btn_step);
        header.pack_start(&left_controls);

        header.pack_end(&btn_menu);

        // Field size button handlers
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
            let sync = sync_performer_ui.clone();
            btn_size_10.connect_clicked(move |_| {
                f.borrow_mut().resize(10, 10);
                fv.widget.queue_draw();
                sync();
            });
        }

        {
            let f = field.clone();
            let fv = field_view.clone();
            let sync = sync_performer_ui.clone();
            btn_size_15.connect_clicked(move |_| {
                f.borrow_mut().resize(15, 15);
                fv.widget.queue_draw();
                sync();
            });
        }

        {
            let f = field.clone();
            let fv = field_view.clone();
            let sync = sync_performer_ui.clone();
            btn_size_20.connect_clicked(move |_| {
                f.borrow_mut().resize(20, 20);
                fv.widget.queue_draw();
                sync();
            });
        }

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
            let lr_clone = lang_ref.clone();
            Rc::new(move || -> StepResult {
                Self::do_step(
                    &vm_clone,
                    &f_clone,
                    &ev_clone,
                    &fv_clone,
                    &con_clone,
                    &stat_clone,
                    *lr_clone.borrow(),
                )
            })
        };

        // Connect Step Button
        {
            let s_fn = do_step_fn.clone();
            btn_step.connect_clicked(move |_| {
                let _ = s_fn();
            });
        }

        // Connect Run / Pause Button (supports both KuMir & Python execution)
        {
            let is_r = is_running.clone();
            let b_run = btn_run.clone();
            let s_fn = do_step_fn.clone();
            let con = console_view.clone();
            let vm_ref = vm.clone();
            let ev_ref = editor_view.clone();
            let lr = lang_ref.clone();
            let stat = status_label.clone();
            let fv = field_view.clone();

            btn_run.connect_clicked(move |_| {
                let lang = *lr.borrow();
                let cur_syntax = ev_ref.syntax_mode();

                // -----------------------------------------------------------
                // Python Execution Mode
                // -----------------------------------------------------------
                if cur_syntax == SyntaxMode::Python {
                    let mut running = is_r.borrow_mut();
                    if *running {
                        *running = false;
                        Self::set_btn_content(&b_run, "media-playback-start-symbolic", lang.tr("run"));
                        stat.set_text(lang.tr("status_ready"));
                        return;
                    }

                    *running = true;
                    Self::set_btn_content(&b_run, "media-playback-pause-symbolic", lang.tr("pause"));
                    stat.set_text(lang.tr("python_run_started"));
                    con.print_info(lang.tr("python_run_started"));

                    let start = ev_ref.buffer.start_iter();
                    let end = ev_ref.buffer.end_iter();
                    let py_code = ev_ref.buffer.text(&start, &end, false).to_string();

                    let (sender, receiver) = std::sync::mpsc::channel();
                    let full_script = format!(r#"
import socket, json, sys

client_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
try:
    client_sock.connect("/tmp/neomir.sock")
except Exception:
    client_sock = None

def _send(payload):
    if not client_sock:
        return "Socket not connected"
    data = json.dumps({{"cmd": payload}}) if isinstance(payload, str) else json.dumps(payload)
    client_sock.sendall((data + "\n").encode())
    return client_sock.recv(4096).decode().strip()

# Robot commands
up = вверх = lambda: _send("up")
down = вниз = lambda: _send("down")
left = влево = lambda: _send("left")
right = вправо = lambda: _send("right")
paint = закрасить = lambda: _send("paint")
step_back = назад_робот = lambda: _send("step_back")
reset = сброс = lambda: _send("reset")

# Turtle commands
forward = вперед = lambda d=1: _send({{"cmd": "turtle_forward", "dist": float(d)}})
backward = назад = lambda d=1: _send({{"cmd": "turtle_backward", "dist": float(d)}})
turn_left = налево = lambda a=90: _send({{"cmd": "turtle_turn_left", "angle": float(a)}})
turn_right = направо = lambda a=90: _send({{"cmd": "turtle_turn_right", "angle": float(a)}})
pen_down = tail_down = опустить_хвост = опустить_перо = lambda: _send("turtle_pen_down")
pen_up = tail_up = поднять_хвост = поднять_перо = lambda: _send("turtle_pen_up")
turtle = черепаха = lambda: _send({{"cmd": "set_performer", "performer": "turtle"}})
robot = робот = lambda: _send({{"cmd": "set_performer", "performer": "robot"}})

{}
"#, py_code);

                    thread::spawn(move || {
                        let output = std::process::Command::new("python3")
                            .arg("-c")
                            .arg(&full_script)
                            .output();

                        let res = match output {
                            Ok(out) => {
                                let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
                                let stderr_str = String::from_utf8_lossy(&out.stderr).to_string();
                                Ok((stdout_str, stderr_str))
                            }
                            Err(e) => Err(e.to_string()),
                        };
                        let _ = sender.send(res);
                    });

                    let is_r_loop = is_r.clone();
                    let b_run_loop = b_run.clone();
                    let con_loop = con.clone();
                    let stat_loop = stat.clone();
                    let lr_loop = lr.clone();
                    let fv_loop = fv.clone();

                    glib::timeout_add_local(Duration::from_millis(50), move || {
                        match receiver.try_recv() {
                            Ok(result) => {
                                let lang_inner = *lr_loop.borrow();
                                *is_r_loop.borrow_mut() = false;
                                Self::set_btn_content(&b_run_loop, "media-playback-start-symbolic", lang_inner.tr("run"));
                                fv_loop.widget.queue_draw();

                                match result {
                                    Ok((stdout_str, stderr_str)) => {
                                        if !stdout_str.is_empty() {
                                            con_loop.print_info(&stdout_str);
                                        }
                                        if !stderr_str.is_empty() {
                                            con_loop.print_error(&stderr_str);
                                            stat_loop.set_text(&format!("Python error: {}", stderr_str.lines().last().unwrap_or("Error")));
                                        } else {
                                            con_loop.print_info(lang_inner.tr("python_run_done"));
                                            stat_loop.set_text(lang_inner.tr("python_run_done"));
                                        }
                                    }
                                    Err(e) => {
                                        con_loop.print_error(&format!("Failed to execute python: {}", e));
                                        stat_loop.set_text(&format!("Execution failed: {}", e));
                                    }
                                }
                                glib::ControlFlow::Break
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                if !*is_r_loop.borrow() {
                                    glib::ControlFlow::Break
                                } else {
                                    glib::ControlFlow::Continue
                                }
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                *is_r_loop.borrow_mut() = false;
                                glib::ControlFlow::Break
                            }
                        }
                    });
                    return;
                }

                // -----------------------------------------------------------
                // KuMir Execution Mode
                // -----------------------------------------------------------
                let mut running = is_r.borrow_mut();
                if *running {
                    *running = false;
                    Self::set_btn_content(&b_run, "media-playback-start-symbolic", lang.tr("run"));
                    return;
                }

                // If starting fresh
                if vm_ref.borrow().is_none() {
                    let start = ev_ref.buffer.start_iter();
                    let end = ev_ref.buffer.end_iter();
                    let code = ev_ref.buffer.text(&start, &end, false);
                    con.print_info(lang.tr("compiling"));
                    match compile_source(code.as_str()) {
                        Ok(compiled_vm) => {
                            *vm_ref.borrow_mut() = Some(compiled_vm);
                        }
                        Err(e) => {
                            let prefix = if lang.is_en() { "Compilation error" } else { "Ошибка компиляции" };
                            con.print_error(&format!("{}: {}", prefix, e));
                            return;
                        }
                    }
                }

                *running = true;
                Self::set_btn_content(&b_run, "media-playback-pause-symbolic", lang.tr("pause"));
                drop(running);

                let is_r_loop = is_r.clone();
                let b_run_loop = b_run.clone();
                let s_fn_loop = s_fn.clone();
                let lr_loop = lr.clone();

                glib::timeout_add_local(Duration::from_millis(60), move || {
                    if !*is_r_loop.borrow() {
                        return glib::ControlFlow::Break;
                    }
                    let res = s_fn_loop();
                    match res {
                        StepResult::Finished | StepResult::Error { .. } => {
                            let lang_inner = *lr_loop.borrow();
                            *is_r_loop.borrow_mut() = false;
                            Self::set_btn_content(
                                &b_run_loop,
                                "media-playback-start-symbolic",
                                lang_inner.tr("run"),
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
            let load_fn = do_load_fn.clone();
            let save_fn = do_save_fn.clone();
            let r_fn = do_reset_fn.clone();
            let back_fn = do_step_back_fn.clone();

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
                    load_fn();
                    return glib::Propagation::Stop;
                }
                if cfg.save.matches(keyval, state) {
                    save_fn();
                    return glib::Propagation::Stop;
                }
                // Ctrl+Z or Alt+Left for Step Back
                if (state.contains(gdk::ModifierType::CONTROL_MASK) && (keyval == gdk::Key::z || keyval == gdk::Key::Z))
                    || (state.contains(gdk::ModifierType::ALT_MASK) && keyval == gdk::Key::Left) {
                    back_fn();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
        }
        window.add_controller(key_controller);

        Self { window }
    }

    fn do_step(
        vm: &Rc<RefCell<Option<VirtualMachine>>>,
        field: &Rc<RefCell<RobotField>>,
        editor_view: &Rc<EditorView>,
        field_view: &Rc<RobotFieldView>,
        console_view: &Rc<ConsoleView>,
        status_label: &gtk::Label,
        lang: Language,
    ) -> StepResult {
        let mut vm_mut = vm.borrow_mut();
        if vm_mut.is_none() {
            let start = editor_view.buffer.start_iter();
            let end = editor_view.buffer.end_iter();
            let code = editor_view.buffer.text(&start, &end, false);
            console_view.print_info(lang.tr("compiling"));
            match compile_source(code.as_str()) {
                Ok(compiled_vm) => {
                    *vm_mut = Some(compiled_vm);
                }
                Err(e) => {
                    let prefix = if lang.is_en() { "Compilation error" } else { "Ошибка компиляции" };
                    console_view.print_error(&format!("{}: {}", prefix, e));
                    status_label.set_text(lang.tr("status_compile_err"));
                    return StepResult::Error {
                        message: e,
                        line: 0,
                    };
                }
            }
        }

        let vm_instance = vm_mut.as_mut().unwrap();
        let res = {
            let mut f = field.borrow_mut();
            vm_instance.step(&mut f)
        };

        field_view.widget.queue_draw();

        let prefix_step = if lang.is_en() { "Step" } else { "Шаг" };
        let _prefix_pause = if lang.is_en() { "Pause" } else { "Пауза" };
        let prefix_finished = if lang.is_en() { "Program finished" } else { "Программа завершена" };
        let prefix_err = if lang.is_en() { "Runtime error" } else { "Ошибка выполнения" };

        match &res {
            StepResult::Stepped { line } | StepResult::RobotMoved { line } => {
                editor_view.highlight_line(Some(*line));
                let line_str = format!("{}: {}", prefix_step, line);
                status_label.set_text(&line_str);
            }
            StepResult::Output(text) => {
                console_view.print_output(text);
            }
            StepResult::Finished => {
                editor_view.highlight_line(None);
                status_label.set_text(prefix_finished);
                console_view.print_success(prefix_finished);
            }
            StepResult::Error { message, line } => {
                editor_view.highlight_line(Some(*line));
                let err_msg = format!("{} ({} {}): {}", prefix_err, if lang.is_en() { "line" } else { "строка" }, line, message);
                status_label.set_text(&err_msg);
                console_view.print_error(&err_msg);
            }
        }

        res
    }
}
