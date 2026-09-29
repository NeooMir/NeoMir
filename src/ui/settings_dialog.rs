use crate::plugins::PluginManager;
use crate::ui::editor_view::EditorView;
use crate::ui::keybindings::{prompt_shortcut_recording, KeybindingsConfig};
use crate::ui::theme::ThemeId;
use adw::prelude::*;
use gtk::gio;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct SettingsDialog;

impl SettingsDialog {
    pub fn show(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
    ) {
        Self::show_internal(parent, editor_view, keybindings, on_keybindings_changed, false);
    }

    pub fn show_extensions(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
    ) {
        Self::show_internal(parent, editor_view, keybindings, on_keybindings_changed, true);
    }

    fn show_internal(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
        open_extensions: bool,
    ) {
        let dialog = adw::PreferencesDialog::builder()
            .title("Настройки NeoMir")
            .build();

        let win_parent = parent.clone();

        // ==========================================
        // Tab 1: General (Themes, Editor, Vim)
        // ==========================================
        let page_general = adw::PreferencesPage::new();
        page_general.set_title("Основные");
        page_general.set_icon_name(Some("preferences-system-symbolic"));

        // Group 1: Appearance & themes
        let theme_group = adw::PreferencesGroup::new();
        theme_group.set_title("Внешний вид и темы");
        theme_group.set_description(Some("Выбор цветовой схемы для редактора кода и интерфейса"));

        let theme_combo = adw::ComboRow::new();
        theme_combo.set_title("Тема оформления");
        theme_combo.set_subtitle("Цвета подсветки синтаксиса и фона");

        let themes = ThemeId::all();
        let theme_names: Vec<&str> = themes.iter().map(|t| t.name()).collect();
        let string_list = gtk::StringList::new(&theme_names);
        theme_combo.set_model(Some(&string_list));

        // Set active theme
        let current_theme = *editor_view.current_theme.borrow();
        if let Some(pos) = themes.iter().position(|t| *t == current_theme) {
            theme_combo.set_selected(pos as u32);
        }

        let ed_theme = editor_view.clone();
        theme_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if let Some(theme) = themes.get(idx) {
                ed_theme.apply_theme(*theme);
            }
        });

        theme_group.add(&theme_combo);
        page_general.add(&theme_group);

        // Group 2: Editor & Vim mode
        let editor_group = adw::PreferencesGroup::new();
        editor_group.set_title("Редактор");
        editor_group.set_description(Some("Параметры ввода текста и режимы управления"));

        let vim_switch = adw::SwitchRow::new();
        vim_switch.set_title("Режим Vim (модальное редактирование)");
        vim_switch.set_subtitle("Клавиши h, j, k, l, i, w, b, x, u, dd для профессионалов");
        vim_switch.set_active(editor_view.vim.is_enabled());

        let ed_vim = editor_view.clone();
        let parent_win = parent.clone();
        let switch_clone = vim_switch.clone();
        let reverting = Rc::new(Cell::new(false));

        vim_switch.connect_active_notify(move |switch| {
            if reverting.get() {
                return;
            }
            let is_active = switch.is_active();
            if is_active && !ed_vim.vim.is_enabled() {
                let alert = adw::AlertDialog::builder()
                    .heading("Включить режим Vim?")
                    .body("Режим Vim предназначен для опытных пользователей, знакомых с модальным управлением (NORMAL / INSERT / VISUAL).\n\nНажмите i для ввода текста, Esc для возврата в командный режим.")
                    .build();

                alert.add_response("cancel", "Отмена");
                alert.add_response("enable", "Включить");
                alert.set_response_appearance("enable", adw::ResponseAppearance::Suggested);

                let ed_inner = ed_vim.clone();
                let flag = reverting.clone();
                let switch_clone = switch_clone.clone();

                alert.choose(&parent_win, Option::<&gio::Cancellable>::None, move |response| {
                    if response == "enable" {
                        ed_inner.vim.set_enabled(true);
                    } else {
                        flag.set(true);
                        switch_clone.set_active(false);
                        flag.set(false);
                        ed_inner.vim.set_enabled(false);
                    }
                });
            } else if !is_active && ed_vim.vim.is_enabled() {
                ed_vim.vim.set_enabled(false);
            }
        });

        editor_group.add(&vim_switch);
        page_general.add(&editor_group);

        dialog.add(&page_general);

        // ==========================================
        // Tab 2: Keyboard shortcuts (Keybindings)
        // ==========================================
        let page_shortcuts = adw::PreferencesPage::new();
        page_shortcuts.set_title("Горячие клавиши");
        page_shortcuts.set_icon_name(Some("input-keyboard-symbolic"));

        let shortcuts_group = adw::PreferencesGroup::new();
        shortcuts_group.set_title("Управление клавиатурой");
        shortcuts_group.set_description(Some(
            "Нажмите на кнопку с комбинацией клавиш, чтобы назначить свой бинд (например, Shift + F10)",
        ));

        // 1. Run (Start / Pause)
        let btn_run_sc = gtk::Button::builder()
            .label(&keybindings.borrow().run.to_display_string())
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let row_run = adw::ActionRow::new();
        row_run.set_title("Запуск программы");
        row_run.set_subtitle("Запустить выполнение программы или приостановить");
        row_run.add_suffix(&btn_run_sc);
        row_run.set_activatable_widget(Some(&btn_run_sc));

        let kb_run = keybindings.clone();
        let on_ch_run = on_keybindings_changed.clone();
        let btn_run_lbl = btn_run_sc.clone();
        let win_rec_run = win_parent.clone();
        btn_run_sc.connect_clicked(move |_| {
            let current = kb_run.borrow().run.clone();
            let kb_inner = kb_run.clone();
            let on_ch_inner = on_ch_run.clone();
            let btn_inner = btn_run_lbl.clone();
            prompt_shortcut_recording(
                &win_rec_run,
                "Запуск программы",
                &current,
                move |new_shortcut| {
                    kb_inner.borrow_mut().run = new_shortcut.clone();
                    let _ = kb_inner.borrow().save();
                    btn_inner.set_label(&new_shortcut.to_display_string());
                    on_ch_inner();
                },
            );
        });
        shortcuts_group.add(&row_run);

        // 2. Step
        let btn_step_sc = gtk::Button::builder()
            .label(&keybindings.borrow().step.to_display_string())
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let row_step = adw::ActionRow::new();
        row_step.set_title("Шаг программы");
        row_step.set_subtitle("Выполнить одну команду КуМир");
        row_step.add_suffix(&btn_step_sc);
        row_step.set_activatable_widget(Some(&btn_step_sc));

        let kb_step = keybindings.clone();
        let on_ch_step = on_keybindings_changed.clone();
        let btn_step_lbl = btn_step_sc.clone();
        let win_rec_step = win_parent.clone();
        btn_step_sc.connect_clicked(move |_| {
            let current = kb_step.borrow().step.clone();
            let kb_inner = kb_step.clone();
            let on_ch_inner = on_ch_step.clone();
            let btn_inner = btn_step_lbl.clone();
            prompt_shortcut_recording(
                &win_rec_step,
                "Шаг программы",
                &current,
                move |new_shortcut| {
                    kb_inner.borrow_mut().step = new_shortcut.clone();
                    let _ = kb_inner.borrow().save();
                    btn_inner.set_label(&new_shortcut.to_display_string());
                    on_ch_inner();
                },
            );
        });
        shortcuts_group.add(&row_step);

        // 3. Reset
        let btn_reset_sc = gtk::Button::builder()
            .label(&keybindings.borrow().reset.to_display_string())
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let row_reset = adw::ActionRow::new();
        row_reset.set_title("Сброс робота");
        row_reset.set_subtitle("Вернуть робота в исходное положение");
        row_reset.add_suffix(&btn_reset_sc);
        row_reset.set_activatable_widget(Some(&btn_reset_sc));

        let kb_reset = keybindings.clone();
        let on_ch_reset = on_keybindings_changed.clone();
        let btn_reset_lbl = btn_reset_sc.clone();
        let win_rec_reset = win_parent.clone();
        btn_reset_sc.connect_clicked(move |_| {
            let current = kb_reset.borrow().reset.clone();
            let kb_inner = kb_reset.clone();
            let on_ch_inner = on_ch_reset.clone();
            let btn_inner = btn_reset_lbl.clone();
            prompt_shortcut_recording(
                &win_rec_reset,
                "Сброс робота",
                &current,
                move |new_shortcut| {
                    kb_inner.borrow_mut().reset = new_shortcut.clone();
                    let _ = kb_inner.borrow().save();
                    btn_inner.set_label(&new_shortcut.to_display_string());
                    on_ch_inner();
                },
            );
        });
        shortcuts_group.add(&row_reset);

        // 4. Load
        let btn_load_sc = gtk::Button::builder()
            .label(&keybindings.borrow().load.to_display_string())
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let row_load = adw::ActionRow::new();
        row_load.set_title("Загрузить программу");
        row_load.set_subtitle("Открыть файл программы (*.kum)");
        row_load.add_suffix(&btn_load_sc);
        row_load.set_activatable_widget(Some(&btn_load_sc));

        let kb_load = keybindings.clone();
        let on_ch_load = on_keybindings_changed.clone();
        let btn_load_lbl = btn_load_sc.clone();
        let win_rec_load = win_parent.clone();
        btn_load_sc.connect_clicked(move |_| {
            let current = kb_load.borrow().load.clone();
            let kb_inner = kb_load.clone();
            let on_ch_inner = on_ch_load.clone();
            let btn_inner = btn_load_lbl.clone();
            prompt_shortcut_recording(
                &win_rec_load,
                "Загрузить программу",
                &current,
                move |new_shortcut| {
                    kb_inner.borrow_mut().load = new_shortcut.clone();
                    let _ = kb_inner.borrow().save();
                    btn_inner.set_label(&new_shortcut.to_display_string());
                    on_ch_inner();
                },
            );
        });
        shortcuts_group.add(&row_load);

        // 5. Save
        let btn_save_sc = gtk::Button::builder()
            .label(&keybindings.borrow().save.to_display_string())
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let row_save = adw::ActionRow::new();
        row_save.set_title("Сохранить программу");
        row_save.set_subtitle("Сохранить файл программы (*.kum)");
        row_save.add_suffix(&btn_save_sc);
        row_save.set_activatable_widget(Some(&btn_save_sc));

        let kb_save = keybindings.clone();
        let on_ch_save = on_keybindings_changed.clone();
        let btn_save_lbl = btn_save_sc.clone();
        let win_rec_save = win_parent.clone();
        btn_save_sc.connect_clicked(move |_| {
            let current = kb_save.borrow().save.clone();
            let kb_inner = kb_save.clone();
            let on_ch_inner = on_ch_save.clone();
            let btn_inner = btn_save_lbl.clone();
            prompt_shortcut_recording(
                &win_rec_save,
                "Сохранить программу",
                &current,
                move |new_shortcut| {
                    kb_inner.borrow_mut().save = new_shortcut.clone();
                    let _ = kb_inner.borrow().save();
                    btn_inner.set_label(&new_shortcut.to_display_string());
                    on_ch_inner();
                },
            );
        });
        shortcuts_group.add(&row_save);

        page_shortcuts.add(&shortcuts_group);

        // Presets Group
        let presets_group = adw::PreferencesGroup::new();
        presets_group.set_title("Готовые схемы");
        presets_group.set_description(Some("Быстрое переключение стандартных раскладок"));

        let row_ide = adw::ActionRow::new();
        row_ide.set_title("Стиль IDE / JetBrains");
        row_ide.set_subtitle("Старт: Shift+F10 • Шаг: F10 • Сброс: F8 • Файлы: Ctrl+O / Ctrl+S");
        let btn_apply_ide = gtk::Button::builder()
            .label("Применить")
            .css_classes(["suggested-action"])
            .valign(gtk::Align::Center)
            .build();
        row_ide.add_suffix(&btn_apply_ide);
        row_ide.set_activatable_widget(Some(&btn_apply_ide));

        let kb_ide = keybindings.clone();
        let on_ch_ide = on_keybindings_changed.clone();
        let b_run_ide = btn_run_sc.clone();
        let b_step_ide = btn_step_sc.clone();
        let b_reset_ide = btn_reset_sc.clone();
        let b_load_ide = btn_load_sc.clone();
        let b_save_ide = btn_save_sc.clone();
        btn_apply_ide.connect_clicked(move |_| {
            let mut cfg = kb_ide.borrow_mut();
            cfg.run = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F10, false, true, false); // Shift+F10
            cfg.step = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F10, false, false, false); // F10
            cfg.reset = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F8, false, false, false); // F8
            cfg.load = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::o, true, false, false); // Ctrl+O
            cfg.save = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::s, true, false, false); // Ctrl+S
            let _ = cfg.save();
            b_run_ide.set_label(&cfg.run.to_display_string());
            b_step_ide.set_label(&cfg.step.to_display_string());
            b_reset_ide.set_label(&cfg.reset.to_display_string());
            b_load_ide.set_label(&cfg.load.to_display_string());
            b_save_ide.set_label(&cfg.save.to_display_string());
            drop(cfg);
            on_ch_ide();
        });
        presets_group.add(&row_ide);

        let row_classic = adw::ActionRow::new();
        row_classic.set_title("Классический стиль");
        row_classic.set_subtitle("Старт: F5 • Шаг: F10 • Сброс: F8 • Файлы: Ctrl+O / Ctrl+S");
        let btn_apply_classic = gtk::Button::builder()
            .label("Сбросить")
            .valign(gtk::Align::Center)
            .build();
        row_classic.add_suffix(&btn_apply_classic);
        row_classic.set_activatable_widget(Some(&btn_apply_classic));

        let kb_classic = keybindings.clone();
        let on_ch_classic = on_keybindings_changed.clone();
        let b_run_classic = btn_run_sc.clone();
        let b_step_classic = btn_step_sc.clone();
        let b_reset_classic = btn_reset_sc.clone();
        let b_load_classic = btn_load_sc.clone();
        let b_save_classic = btn_save_sc.clone();
        btn_apply_classic.connect_clicked(move |_| {
            let mut cfg = kb_classic.borrow_mut();
            cfg.run = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F5, false, false, false); // F5
            cfg.step = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F10, false, false, false); // F10
            cfg.reset = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::F8, false, false, false); // F8
            cfg.load = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::o, true, false, false); // Ctrl+O
            cfg.save = crate::ui::keybindings::Shortcut::new(gtk::gdk::Key::s, true, false, false); // Ctrl+S
            let _ = cfg.save();
            b_run_classic.set_label(&cfg.run.to_display_string());
            b_step_classic.set_label(&cfg.step.to_display_string());
            b_reset_classic.set_label(&cfg.reset.to_display_string());
            b_load_classic.set_label(&cfg.load.to_display_string());
            b_save_classic.set_label(&cfg.save.to_display_string());
            drop(cfg);
            on_ch_classic();
        });
        presets_group.add(&row_classic);

        page_shortcuts.add(&presets_group);
        dialog.add(&page_shortcuts);

        // ==========================================
        // Tab 3: Extensions (Install .plug packages)
        // ==========================================
        let page_extensions = adw::PreferencesPage::new();
        page_extensions.set_title("Расширения");
        page_extensions.set_icon_name(Some("application-x-addon-symbolic"));

        let install_group = adw::PreferencesGroup::new();
        install_group.set_title("Установка пакетов (.plug)");
        install_group.set_description(Some("Установка дополнительных исполнителей, окон и скриптов управления Роботом"));

        let install_row = adw::ActionRow::new();
        install_row.set_title("Установить расширение");
        install_row.set_subtitle("Выберите локальный файл пакета (*.plug)");

        let btn_browse = gtk::Button::builder()
            .label("Выбрать файл...")
            .css_classes(["suggested-action"])
            .valign(gtk::Align::Center)
            .build();
        install_row.add_suffix(&btn_browse);
        install_row.set_activatable_widget(Some(&btn_browse));
        install_group.add(&install_row);

        let folder_row = adw::ActionRow::new();
        folder_row.set_title("Папка расширений");
        let plugins_path = PluginManager::plugins_dir();
        folder_row.set_subtitle(&plugins_path.display().to_string());

        let btn_open_folder = gtk::Button::builder()
            .icon_name("folder-open-symbolic")
            .tooltip_text("Открыть каталог с расширениями")
            .valign(gtk::Align::Center)
            .build();
        let path_for_open = plugins_path.clone();
        btn_open_folder.connect_clicked(move |_| {
            let uri = format!("file://{}", path_for_open.display());
            let _ = gio::AppInfo::launch_default_for_uri(&uri, Option::<&gio::AppLaunchContext>::None);
        });
        folder_row.add_suffix(&btn_open_folder);
        install_group.add(&folder_row);

        page_extensions.add(&install_group);

        // List of installed extensions
        let list_group = adw::PreferencesGroup::new();
        list_group.set_title("Установленные расширения");
        list_group.set_description(Some("Список активных плагинов и модулей NeoMir. Нажмите ▶ для запуска окна расширения."));

        let list_group_rc = Rc::new(list_group.clone());
        let added_rows: Rc<RefCell<Vec<gtk::Widget>>> = Rc::new(RefCell::new(Vec::new()));
        let win_window = parent.clone();

        Self::render_plugins_list(&list_group_rc, &added_rows, &win_window);

        let list_group_for_browse = list_group_rc.clone();
        let added_rows_for_browse = added_rows.clone();
        let win_for_browse = win_window.clone();

        btn_browse.connect_clicked(move |_| {
            let chooser = gtk::FileChooserNative::new(
                Some("Выберите файл расширения NeoMir"),
                Some(&win_for_browse),
                gtk::FileChooserAction::Open,
                Some("Установить"),
                Some("Отмена"),
            );

            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Расширения NeoMir (*.plug)"));
            filter.add_pattern("*.plug");
            chooser.add_filter(&filter);

            let all_filter = gtk::FileFilter::new();
            all_filter.set_name(Some("Все файлы (*.*)"));
            all_filter.add_pattern("*");
            chooser.add_filter(&all_filter);

            let lg = list_group_for_browse.clone();
            let ar = added_rows_for_browse.clone();
            let parent_window = win_for_browse.clone();

            chooser.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            match PluginManager::install_plug_file(&path) {
                                Ok(meta) => {
                                    let alert = adw::AlertDialog::builder()
                                        .heading("Расширение установлено")
                                        .body(&format!(
                                            "Расширение «{}» версии {} успешно установлено!",
                                            meta.name, meta.version
                                        ))
                                        .build();
                                    alert.add_response("ok", "OK");
                                    alert.choose(&parent_window, Option::<&gio::Cancellable>::None, |_| {});
                                    Self::render_plugins_list(&lg, &ar, &parent_window);
                                }
                                Err(err) => {
                                    let alert = adw::AlertDialog::builder()
                                        .heading("Ошибка установки расширения")
                                        .body(&format!("Не удалось установить расширение:\n\n{}", err))
                                        .build();
                                    alert.add_response("ok", "Закрыть");
                                    alert.choose(&parent_window, Option::<&gio::Cancellable>::None, |_| {});
                                }
                            }
                        }
                    }
                }
            });

            chooser.show();
        });

        page_extensions.add(&list_group);
        dialog.add(&page_extensions);

        if open_extensions {
            dialog.set_visible_page(&page_extensions);
        }

        dialog.present(Some(parent));
    }

    fn render_plugins_list(
        group: &Rc<adw::PreferencesGroup>,
        added_rows: &Rc<RefCell<Vec<gtk::Widget>>>,
        parent_window: &adw::ApplicationWindow,
    ) {
        for row in added_rows.borrow_mut().drain(..) {
            group.remove(&row);
        }

        let plugins = PluginManager::list_plugins();

        if plugins.is_empty() {
            let empty_row = adw::ActionRow::new();
            empty_row.set_title("Нет установленных расширений");
            empty_row.set_subtitle("Нажмите кнопку выше, чтобы установить файл пакета (*.plug)");
            empty_row.set_sensitive(false);
            group.add(&empty_row);
            added_rows.borrow_mut().push(empty_row.upcast());
            return;
        }

        for plugin in plugins {
            let row = adw::SwitchRow::new();
            let title = if plugin.entry.is_some() {
                format!("🧩 {} (v{})", plugin.name, plugin.version)
            } else {
                format!("📦 {} (v{})", plugin.name, plugin.version)
            };
            row.set_title(&title);

            let desc = if plugin.description.is_empty() {
                "Без описания"
            } else {
                &plugin.description
            };
            row.set_subtitle(desc);
            row.set_active(plugin.enabled);

            let plugin_id = plugin.id.clone();
            let parent_win = parent_window.clone();

            row.connect_active_notify(move |switch| {
                let active = switch.is_active();
                if let Err(err) = PluginManager::set_plugin_enabled(&plugin_id, active) {
                    let alert = adw::AlertDialog::builder()
                        .heading("Ошибка изменения статуса")
                        .body(&format!("Не удалось переключить плагин: {}", err))
                        .build();
                    alert.add_response("ok", "Закрыть");
                    alert.choose(&parent_win, Option::<&gio::Cancellable>::None, |_| {});
                }
            });

            // If plugin has an entry point, show Launch button
            if plugin.entry.is_some() {
                let btn_run_plugin = gtk::Button::builder()
                    .icon_name("media-playback-start-symbolic")
                    .tooltip_text("Запустить расширение")
                    .css_classes(["flat"])
                    .valign(gtk::Align::Center)
                    .build();

                let pid = plugin.id.clone();
                let parent_w = parent_window.clone();
                btn_run_plugin.connect_clicked(move |_| {
                    if let Err(err) = PluginManager::launch_plugin(&pid) {
                        let alert = adw::AlertDialog::builder()
                            .heading("Ошибка запуска плагина")
                            .body(&format!("Не удалось запустить плагин:\n\n{}", err))
                            .build();
                        alert.add_response("ok", "Закрыть");
                        alert.choose(&parent_w, Option::<&gio::Cancellable>::None, |_| {});
                    }
                });
                row.add_suffix(&btn_run_plugin);
            }

            // Uninstall button
            let btn_uninstall = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Удалить расширение")
                .css_classes(["flat", "destructive-action"])
                .valign(gtk::Align::Center)
                .build();

            let pid_del = plugin.id.clone();
            let p_name = plugin.name.clone();
            let parent_w = parent_window.clone();
            let group_reload = group.clone();
            let rows_reload = added_rows.clone();

            btn_uninstall.connect_clicked(move |_| {
                let alert = adw::AlertDialog::builder()
                    .heading("Удалить расширение?")
                    .body(&format!("Вы уверены, что хотите удалить расширение «{}»?", p_name))
                    .build();
                alert.add_response("cancel", "Отмена");
                alert.add_response("delete", "Удалить");
                alert.set_response_appearance("delete", adw::ResponseAppearance::Destructive);

                let pid = pid_del.clone();
                let pw = parent_w.clone();
                let gr = group_reload.clone();
                let rr = rows_reload.clone();

                alert.choose(&parent_w, Option::<&gio::Cancellable>::None, move |resp| {
                    if resp == "delete" {
                        let _ = PluginManager::delete_plugin(&pid);
                        Self::render_plugins_list(&gr, &rr, &pw);
                    }
                });
            });

            row.add_suffix(&btn_uninstall);
            group.add(&row);
            added_rows.borrow_mut().push(row.upcast());
        }
    }
}
