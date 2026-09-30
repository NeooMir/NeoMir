use crate::plugins::PluginManager;
use crate::robot::{PerformerMode, RobotField};
use crate::ui::editor_view::EditorView;
use crate::ui::field_view::RobotFieldView;
use crate::ui::keybindings::{prompt_shortcut_recording, KeybindingsConfig};
use crate::ui::theme::ThemeId;
use adw::prelude::*;
use gtk::gio;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub performer: String,
    pub vim_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Ptyxis / Adwaita Dark".to_string(),
            language: "ru".to_string(),
            performer: "robot".to_string(),
            vim_mode: false,
        }
    }
}

impl AppSettings {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/vptr".to_string());
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("neomir");
        let _ = fs::create_dir_all(&path);
        path.push("settings.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&data) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data)
        } else {
            Ok(())
        }
    }
}

pub struct SettingsDialog;

impl SettingsDialog {
    pub fn show(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
        field: &Rc<RefCell<RobotField>>,
        field_view: &Rc<RobotFieldView>,
        on_performer_changed: Rc<dyn Fn()>,
        app_settings: &Rc<RefCell<AppSettings>>,
    ) {
        Self::show_internal(
            parent,
            editor_view,
            keybindings,
            on_keybindings_changed,
            field,
            field_view,
            on_performer_changed,
            app_settings,
            false,
        );
    }

    pub fn show_extensions(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
        field: &Rc<RefCell<RobotField>>,
        field_view: &Rc<RobotFieldView>,
        on_performer_changed: Rc<dyn Fn()>,
        app_settings: &Rc<RefCell<AppSettings>>,
    ) {
        Self::show_internal(
            parent,
            editor_view,
            keybindings,
            on_keybindings_changed,
            field,
            field_view,
            on_performer_changed,
            app_settings,
            true,
        );
    }

    fn show_internal(
        parent: &adw::ApplicationWindow,
        editor_view: &Rc<EditorView>,
        keybindings: &Rc<RefCell<KeybindingsConfig>>,
        on_keybindings_changed: Rc<dyn Fn()>,
        field: &Rc<RefCell<RobotField>>,
        field_view: &Rc<RobotFieldView>,
        on_performer_changed: Rc<dyn Fn()>,
        app_settings: &Rc<RefCell<AppSettings>>,
        open_extensions: bool,
    ) {
        let dialog = adw::PreferencesDialog::builder()
            .title("Настройки NeoMir")
            .build();

        let win_parent = parent.clone();

        // ==========================================
        // Tab 1: General (Themes, Language, Performer, Editor, Vim)
        // ==========================================
        let page_general = adw::PreferencesPage::new();
        page_general.set_title("Основные");
        page_general.set_icon_name(Some("preferences-system-symbolic"));

        // Group 1: Appearance & themes (Ptyxis themes)
        let theme_group = adw::PreferencesGroup::new();
        theme_group.set_title("Внешний вид и темы Ptyxis");
        theme_group.set_description(Some("Выбор цветовой схемы Ptyxis для редактора кода и интерфейса"));

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
        let s_theme = app_settings.clone();
        theme_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if let Some(theme) = themes.get(idx) {
                ed_theme.apply_theme(*theme);
                s_theme.borrow_mut().theme = theme.name().to_string();
                let _ = s_theme.borrow().save();
            }
        });

        theme_group.add(&theme_combo);
        page_general.add(&theme_group);

        // Group 2: Language & Localization
        let lang_group = adw::PreferencesGroup::new();
        lang_group.set_title("Язык и локализация");
        lang_group.set_description(Some("Выбор языка интерфейса приложения"));

        let lang_combo = adw::ComboRow::new();
        lang_combo.set_title("Язык интерфейса");
        let languages = ["Русский (Russian)", "English (Английский)"];
        let lang_list = gtk::StringList::new(&languages);
        lang_combo.set_model(Some(&lang_list));

        if app_settings.borrow().language == "en" {
            lang_combo.set_selected(1);
            lang_combo.set_subtitle("Current language: English");
        } else {
            lang_combo.set_selected(0);
            lang_combo.set_subtitle("Текущий язык: Русский");
        }

        let parent_alert = parent.clone();
        let lang_combo_clone = lang_combo.clone();
        let s_lang = app_settings.clone();
        lang_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            if idx == 0 {
                lang_combo_clone.set_subtitle("Текущий язык: Русский");
                s_lang.borrow_mut().language = "ru".to_string();
            } else {
                lang_combo_clone.set_subtitle("Current language: English");
                s_lang.borrow_mut().language = "en".to_string();
                let alert = adw::AlertDialog::builder()
                    .heading("Язык интерфейса")
                    .body("Выбран язык: English. Интерфейс адаптирован.")
                    .build();
                alert.add_response("ok", "OK");
                alert.choose(&parent_alert, Option::<&gio::Cancellable>::None, |_| {});
            }
            let _ = s_lang.borrow().save();
        });

        lang_group.add(&lang_combo);
        page_general.add(&lang_group);

        // Group 3: Performer selection (Робот <-> Черепаха, NO EMOJIS)
        let performer_group = adw::PreferencesGroup::new();
        performer_group.set_title("Исполнитель алгоритма");
        performer_group.set_description(Some("Выбор активного исполнителя: клетчатое поле Робота или векторная графика Черепахи"));

        let performer_combo = adw::ComboRow::new();
        performer_combo.set_title("Активный исполнитель");
        let performer_options = [
            "Робот (клетчатое поле, закрашивание клеток, стены)",
            "Черепаха (векторное рисование, перо/хвост, углы поворота)",
        ];
        let performer_list = gtk::StringList::new(&performer_options);
        performer_combo.set_model(Some(&performer_list));

        let cur_performer = field.borrow().performer;
        if cur_performer == PerformerMode::Turtle {
            performer_combo.set_selected(1);
            performer_combo.set_subtitle("Текущий исполнитель: Черепаха");
        } else {
            performer_combo.set_selected(0);
            performer_combo.set_subtitle("Текущий исполнитель: Робот");
        }

        let f_perf = field.clone();
        let fv_perf = field_view.clone();
        let on_change = on_performer_changed.clone();
        let perf_combo_clone = performer_combo.clone();
        let s_perf = app_settings.clone();
        performer_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            if idx == 1 {
                f_perf.borrow_mut().performer = PerformerMode::Turtle;
                perf_combo_clone.set_subtitle("Текущий исполнитель: Черепаха");
                s_perf.borrow_mut().performer = "turtle".to_string();
            } else {
                f_perf.borrow_mut().performer = PerformerMode::Robot;
                perf_combo_clone.set_subtitle("Текущий исполнитель: Робот");
                s_perf.borrow_mut().performer = "robot".to_string();
            }
            let _ = s_perf.borrow().save();
            fv_perf.widget.queue_draw();
            on_change();
        });

        performer_group.add(&performer_combo);
        page_general.add(&performer_group);

        // Group 4: Editor & Vim mode
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
        let s_vim = app_settings.clone();

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
                let switch_inner = switch_clone.clone();
                let s_inner = s_vim.clone();

                alert.choose(&parent_win, Option::<&gio::Cancellable>::None, move |response| {
                    if response == "enable" {
                        ed_inner.vim.set_enabled(true);
                        s_inner.borrow_mut().vim_mode = true;
                        let _ = s_inner.borrow().save();
                    } else {
                        flag.set(true);
                        switch_inner.set_active(false);
                        flag.set(false);
                        ed_inner.vim.set_enabled(false);
                        s_inner.borrow_mut().vim_mode = false;
                        let _ = s_inner.borrow().save();
                    }
                });
            } else if !is_active && ed_vim.vim.is_enabled() {
                ed_vim.vim.set_enabled(false);
                s_vim.borrow_mut().vim_mode = false;
                let _ = s_vim.borrow().save();
            }
        });

        editor_group.add(&vim_switch);
        page_general.add(&editor_group);

        dialog.add(&page_general);

        // ==========================================
        // Tab 2: Keyboard shortcuts
        // ==========================================
        let page_shortcuts = adw::PreferencesPage::new();
        page_shortcuts.set_title("Горячие клавиши");
        page_shortcuts.set_icon_name(Some("input-keyboard-symbolic"));

        let shortcuts_group = adw::PreferencesGroup::new();
        shortcuts_group.set_title("Управление средой");
        shortcuts_group.set_description(Some("Нажмите кнопку комбинации для назначения новой клавиши"));

        let kb_cfg = keybindings.clone();
        let on_change_kb = on_keybindings_changed.clone();
        let win_sc = parent.clone();

        // 1. Run (F5)
        let row_run = adw::ActionRow::new();
        row_run.set_title("Выполнить программу");
        let btn_run = gtk::Button::with_label(&kb_cfg.borrow().run.to_display_string());
        btn_run.add_css_class("flat");
        btn_run.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_run.clone();
            let win = win_sc.clone();
            btn_run.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().run.clone();
                prompt_shortcut_recording(&win, "Выполнить", &current_sc, move |sc| {
                    btn.set_label(&sc.to_display_string());
                    kb_inner.borrow_mut().run = sc;
                    let _ = kb_inner.borrow().save();
                    on_ch_inner();
                });
            });
        }
        row_run.add_suffix(&btn_run);
        shortcuts_group.add(&row_run);

        // 2. Step (F10)
        let row_step = adw::ActionRow::new();
        row_step.set_title("Шаг программы");
        let btn_step = gtk::Button::with_label(&kb_cfg.borrow().step.to_display_string());
        btn_step.add_css_class("flat");
        btn_step.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_step.clone();
            let win = win_sc.clone();
            btn_step.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().step.clone();
                prompt_shortcut_recording(&win, "Шаг", &current_sc, move |sc| {
                    btn.set_label(&sc.to_display_string());
                    kb_inner.borrow_mut().step = sc;
                    let _ = kb_inner.borrow().save();
                    on_ch_inner();
                });
            });
        }
        row_step.add_suffix(&btn_step);
        shortcuts_group.add(&row_step);

        // 3. Reset (F8)
        let row_reset = adw::ActionRow::new();
        row_reset.set_title("Сброс поля и выполнения");
        let btn_reset = gtk::Button::with_label(&kb_cfg.borrow().reset.to_display_string());
        btn_reset.add_css_class("flat");
        btn_reset.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_reset.clone();
            let win = win_sc.clone();
            btn_reset.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().reset.clone();
                prompt_shortcut_recording(&win, "Сброс", &current_sc, move |sc| {
                    btn.set_label(&sc.to_display_string());
                    kb_inner.borrow_mut().reset = sc;
                    let _ = kb_inner.borrow().save();
                    on_ch_inner();
                });
            });
        }
        row_reset.add_suffix(&btn_reset);
        shortcuts_group.add(&row_reset);

        // 4. Save (Ctrl+S)
        let row_save = adw::ActionRow::new();
        row_save.set_title("Сохранить файл");
        let btn_save = gtk::Button::with_label(&kb_cfg.borrow().save.to_display_string());
        btn_save.add_css_class("flat");
        btn_save.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_save.clone();
            let win = win_sc.clone();
            btn_save.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().save.clone();
                prompt_shortcut_recording(&win, "Сохранить", &current_sc, move |sc| {
                    btn.set_label(&sc.to_display_string());
                    kb_inner.borrow_mut().save = sc;
                    let _ = kb_inner.borrow().save();
                    on_ch_inner();
                });
            });
        }
        row_save.add_suffix(&btn_save);
        shortcuts_group.add(&row_save);

        // Reset to defaults
        let btn_defaults = gtk::Button::with_label("Сбросить по умолчанию");
        btn_defaults.add_css_class("destructive-action");
        btn_defaults.set_halign(gtk::Align::Center);
        btn_defaults.set_margin_top(12);

        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let b_run = btn_run.clone();
            let b_step = btn_step.clone();
            let b_reset = btn_reset.clone();
            let b_save = btn_save.clone();
            btn_defaults.connect_clicked(move |_| {
                let def = KeybindingsConfig::default();
                *kb_clone.borrow_mut() = def.clone();
                let _ = def.save();
                b_run.set_label(&def.run.to_display_string());
                b_step.set_label(&def.step.to_display_string());
                b_reset.set_label(&def.reset.to_display_string());
                b_save.set_label(&def.save.to_display_string());
                on_ch();
            });
        }
        shortcuts_group.add(&btn_defaults);

        page_shortcuts.add(&shortcuts_group);
        dialog.add(&page_shortcuts);

        // ==========================================
        // Tab 3: Extensions / Plugins (.plug support)
        // ==========================================
        let page_plugins = adw::PreferencesPage::new();
        page_plugins.set_title("Расширения");
        page_plugins.set_icon_name(Some("application-x-addon-symbolic"));

        let install_group = adw::PreferencesGroup::new();
        install_group.set_title("Установка пакетов");
        install_group.set_description(Some("Поддерживаются самостоятельные плагины NeoMir в формате пакетов .plug"));

        let install_row = adw::ActionRow::new();
        install_row.set_title("Установить расширение из файла");
        install_row.set_subtitle("Выберите пакет .plug для установки");

        let btn_choose_plug = gtk::Button::with_label("Выбрать .plug...");
        btn_choose_plug.add_css_class("suggested-action");
        btn_choose_plug.set_valign(gtk::Align::Center);
        install_row.add_suffix(&btn_choose_plug);
        install_group.add(&install_row);

        page_plugins.add(&install_group);

        let list_group = adw::PreferencesGroup::new();
        list_group.set_title("Установленные расширения");

        // Helper to refresh plugin list
        let plugins_state = Rc::new(RefCell::new(Vec::new()));
        let list_group_rc = Rc::new(list_group.clone());

        let refresh_list = {
            let win = parent.clone();
            let lg = list_group_rc.clone();
            let ps = plugins_state.clone();
            Rc::new(move || {
                let list = PluginManager::list_plugins();
                *ps.borrow_mut() = list.clone();

                let mut child = lg.first_child();
                while let Some(c) = child {
                    let next = c.next_sibling();
                    lg.remove(&c);
                    child = next;
                }

                if list.is_empty() {
                    let empty_row = adw::ActionRow::new();
                    empty_row.set_title("Нет установленных расширений");
                    empty_row.set_subtitle("Установите пакет .plug через кнопку выше");
                    lg.add(&empty_row);
                    return;
                }

                for meta in list {
                    let row = adw::ActionRow::new();
                    row.set_title(&meta.name);
                    let sub = format!(
                        "v{} • {} • {}",
                        meta.version,
                        meta.author,
                        meta.description
                    );
                    row.set_subtitle(&sub);

                    let sw = gtk::Switch::new();
                    sw.set_active(meta.enabled);
                    sw.set_valign(gtk::Align::Center);

                    let plugin_id = meta.id.clone();
                    let win_alert = win.clone();
                    sw.connect_state_set(move |_, state| {
                        if let Err(e) = PluginManager::set_plugin_enabled(&plugin_id, state) {
                            let alert = adw::AlertDialog::builder()
                                .heading("Ошибка")
                                .body(&format!("Не удалось изменить статус расширения: {}", e))
                                .build();
                            alert.add_response("ok", "OK");
                            alert.choose(&win_alert, Option::<&gio::Cancellable>::None, |_| {});
                        }
                        glib::Propagation::Proceed
                    });

                    row.add_suffix(&sw);

                    let btn_del = gtk::Button::from_icon_name("user-trash-symbolic");
                    btn_del.add_css_class("flat");
                    btn_del.add_css_class("destructive-action");
                    btn_del.set_valign(gtk::Align::Center);
                    btn_del.set_tooltip_text(Some("Удалить расширение"));

                    let del_id = meta.id.clone();
                    let win_del = win.clone();
                    let refresh_del = lg.clone();
                    btn_del.connect_clicked(move |_| {
                        if let Err(e) = PluginManager::delete_plugin(&del_id) {
                            let alert = adw::AlertDialog::builder()
                                .heading("Ошибка удаления")
                                .body(&format!("Не удалось удалить расширение: {}", e))
                                .build();
                            alert.add_response("ok", "OK");
                            alert.choose(&win_del, Option::<&gio::Cancellable>::None, |_| {});
                        } else {
                            let mut child = refresh_del.first_child();
                            while let Some(c) = child {
                                let next = c.next_sibling();
                                refresh_del.remove(&c);
                                child = next;
                            }
                            let l = PluginManager::list_plugins();
                            if l.is_empty() {
                                let empty_row = adw::ActionRow::new();
                                empty_row.set_title("Нет установленных расширений");
                                refresh_del.add(&empty_row);
                            }
                        }
                    });

                    row.add_suffix(&btn_del);
                    lg.add(&row);
                }
            })
        };

        refresh_list();

        // Connect .plug chooser
        {
            let win = parent.clone();
            let rf = refresh_list.clone();
            btn_choose_plug.connect_clicked(move |_| {
                let chooser = gtk::FileChooserNative::new(
                    Some("Выберите пакет расширения NeoMir (*.plug)"),
                    Some(&win),
                    gtk::FileChooserAction::Open,
                    Some("Установить"),
                    Some("Отмена"),
                );

                let filter = gtk::FileFilter::new();
                filter.set_name(Some("Пакеты расширений NeoMir (*.plug)"));
                filter.add_pattern("*.plug");
                chooser.add_filter(&filter);

                let all_filter = gtk::FileFilter::new();
                all_filter.set_name(Some("Все файлы (*.*)"));
                all_filter.add_pattern("*");
                chooser.add_filter(&all_filter);

                let win_clone = win.clone();
                let rf_clone = rf.clone();
                chooser.connect_response(move |dialog, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                match PluginManager::install_plug_file(&path) {
                                    Ok(meta) => {
                                        let alert = adw::AlertDialog::builder()
                                            .heading("Расширение установлено!")
                                            .body(&format!(
                                                "«{}» (версия {}) успешно установлено и готово к работе.",
                                                meta.name, meta.version
                                            ))
                                            .build();
                                        alert.add_response("ok", "OK");
                                        alert.choose(&win_clone, Option::<&gio::Cancellable>::None, |_| {});
                                        rf_clone();
                                    }
                                    Err(err) => {
                                        let alert = adw::AlertDialog::builder()
                                            .heading("Ошибка установки")
                                            .body(&format!("Не удалось установить расширение:\n{}", err))
                                            .build();
                                        alert.add_response("ok", "OK");
                                        alert.choose(&win_clone, Option::<&gio::Cancellable>::None, |_| {});
                                    }
                                }
                            }
                        }
                    }
                    dialog.destroy();
                });
                chooser.show();
            });
        }

        page_plugins.add(&list_group);
        dialog.add(&page_plugins);

        // Pre-select page
        if open_extensions {
            dialog.set_visible_page(&page_plugins);
        } else {
            dialog.set_visible_page(&page_general);
        }

        dialog.present(Some(&win_parent));
    }
}
