use crate::plugins::PluginManager;
use crate::robot::{PerformerMode, RobotField};
use crate::ui::editor_view::EditorView;
use crate::ui::field_view::RobotFieldView;
use crate::ui::i18n::Language;
use crate::ui::keybindings::{prompt_shortcut_recording, KeybindingsConfig};
use crate::ui::theme::ThemeId;
use adw::prelude::*;
use gtk::gio;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

fn default_syntax_str() -> String {
    "kumir".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub performer: String,
    pub vim_mode: bool,
    #[serde(default = "default_syntax_str")]
    pub syntax_mode: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Ptyxis / Adwaita Dark".to_string(),
            language: "ru".to_string(),
            performer: "robot".to_string(),
            vim_mode: false,
            syntax_mode: "kumir".to_string(),
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
        on_language_changed: Rc<dyn Fn()>,
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
            on_language_changed,
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
        on_language_changed: Rc<dyn Fn()>,
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
            on_language_changed,
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
        on_language_changed: Rc<dyn Fn()>,
        open_extensions: bool,
    ) {
        let lang = Language::from_code(&app_settings.borrow().language);

        let dialog = adw::PreferencesDialog::builder()
            .title(lang.tr("settings_title"))
            .build();

        let win_parent = parent.clone();

        // ==========================================
        // Tab 1: General (Themes, Language, Performer, Editor, Vim)
        // ==========================================
        let page_general = adw::PreferencesPage::new();
        page_general.set_title(lang.tr("tab_general"));
        page_general.set_icon_name(Some("preferences-system-symbolic"));

        // Group 1: Appearance & themes (Ptyxis themes)
        let theme_group = adw::PreferencesGroup::new();
        theme_group.set_title(lang.tr("grp_appearance"));
        theme_group.set_description(Some(lang.tr("desc_appearance")));

        let theme_combo = adw::ComboRow::new();
        theme_combo.set_title(lang.tr("theme_title"));
        theme_combo.set_subtitle(lang.tr("theme_sub"));

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
        lang_group.set_title(lang.tr("grp_language"));
        lang_group.set_description(Some(lang.tr("desc_language")));

        let lang_combo = adw::ComboRow::new();
        lang_combo.set_title(lang.tr("lang_title"));
        let languages = ["Русский (Russian)", "English"];
        let lang_list = gtk::StringList::new(&languages);
        lang_combo.set_model(Some(&lang_list));

        if app_settings.borrow().language == "en" {
            lang_combo.set_selected(1);
            lang_combo.set_subtitle(lang.tr("cur_lang_en"));
        } else {
            lang_combo.set_selected(0);
            lang_combo.set_subtitle(lang.tr("cur_lang_ru"));
        }

        let parent_alert = parent.clone();
        let lang_combo_clone = lang_combo.clone();
        let s_lang = app_settings.clone();
        let on_lang_change_cb = on_language_changed.clone();

        lang_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            let new_lang = if idx == 0 { Language::Ru } else { Language::En };
            let lang_code = new_lang.code().to_string();

            if s_lang.borrow().language == lang_code {
                return;
            }

            s_lang.borrow_mut().language = lang_code;
            let _ = s_lang.borrow().save();

            if new_lang.is_en() {
                lang_combo_clone.set_subtitle("Current language: English");
            } else {
                lang_combo_clone.set_subtitle("Текущий язык: Русский");
            }

            // Update main window UI components dynamically
            on_lang_change_cb();

            let alert = adw::AlertDialog::builder()
                .heading(new_lang.tr("lang_changed_title"))
                .body(new_lang.tr("lang_changed_body"))
                .build();
            alert.add_response("ok", "OK");
            alert.choose(&parent_alert, Option::<&gio::Cancellable>::None, |_| {});
        });

        lang_group.add(&lang_combo);
        page_general.add(&lang_group);

        // Group 3: Performer selection (Robot <-> Turtle)
        let performer_group = adw::PreferencesGroup::new();
        performer_group.set_title(lang.tr("grp_performer"));
        performer_group.set_description(Some(lang.tr("desc_performer")));

        let performer_combo = adw::ComboRow::new();
        performer_combo.set_title(lang.tr("active_performer"));
        let performer_options = [
            lang.tr("perf_opt_robot"),
            lang.tr("perf_opt_turtle"),
        ];
        let performer_list = gtk::StringList::new(&performer_options);
        performer_combo.set_model(Some(&performer_list));

        let cur_performer = field.borrow().performer;
        if cur_performer == PerformerMode::Turtle {
            performer_combo.set_selected(1);
            performer_combo.set_subtitle(lang.tr("cur_perf_turtle"));
        } else {
            performer_combo.set_selected(0);
            performer_combo.set_subtitle(lang.tr("cur_perf_robot"));
        }

        let f_perf = field.clone();
        let fv_perf = field_view.clone();
        let on_change = on_performer_changed.clone();
        let perf_combo_clone = performer_combo.clone();
        let s_perf = app_settings.clone();
        let cur_lang_for_perf = lang;

        performer_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            if idx == 1 {
                f_perf.borrow_mut().performer = PerformerMode::Turtle;
                perf_combo_clone.set_subtitle(cur_lang_for_perf.tr("cur_perf_turtle"));
                s_perf.borrow_mut().performer = "turtle".to_string();
            } else {
                f_perf.borrow_mut().performer = PerformerMode::Robot;
                perf_combo_clone.set_subtitle(cur_lang_for_perf.tr("cur_perf_robot"));
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
        editor_group.set_title(lang.tr("grp_editor"));
        editor_group.set_description(Some(lang.tr("desc_editor")));

        let vim_switch = adw::SwitchRow::new();
        vim_switch.set_title(lang.tr("vim_title"));
        vim_switch.set_subtitle(lang.tr("vim_sub"));
        vim_switch.set_active(editor_view.vim.is_enabled());

        let ed_vim = editor_view.clone();
        let parent_win = parent.clone();
        let switch_clone = vim_switch.clone();
        let reverting = Rc::new(Cell::new(false));
        let s_vim = app_settings.clone();
        let vim_lang = lang;

        vim_switch.connect_active_notify(move |switch| {
            if reverting.get() {
                return;
            }
            let is_active = switch.is_active();
            if is_active && !ed_vim.vim.is_enabled() {
                let alert = adw::AlertDialog::builder()
                    .heading(vim_lang.tr("vim_alert_title"))
                    .body(vim_lang.tr("vim_alert_body"))
                    .build();

                alert.add_response("cancel", vim_lang.tr("cancel"));
                alert.add_response("enable", vim_lang.tr("enable"));
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
        page_shortcuts.set_title(lang.tr("tab_shortcuts"));
        page_shortcuts.set_icon_name(Some("input-keyboard-symbolic"));

        let shortcuts_group = adw::PreferencesGroup::new();
        shortcuts_group.set_title(lang.tr("grp_shortcuts"));
        shortcuts_group.set_description(Some(lang.tr("desc_shortcuts")));

        let kb_cfg = keybindings.clone();
        let on_change_kb = on_keybindings_changed.clone();
        let win_sc = parent.clone();
        let is_en = lang.is_en();

        // 1. Run (F5)
        let row_run = adw::ActionRow::new();
        row_run.set_title(lang.tr("sc_run"));
        let btn_run = gtk::Button::with_label(&kb_cfg.borrow().run.to_display_string());
        btn_run.add_css_class("flat");
        btn_run.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_run.clone();
            let win = win_sc.clone();
            let title = lang.tr("run").to_string();
            btn_run.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().run.clone();
                prompt_shortcut_recording(&win, &title, &current_sc, is_en, move |sc| {
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
        row_step.set_title(lang.tr("sc_step"));
        let btn_step = gtk::Button::with_label(&kb_cfg.borrow().step.to_display_string());
        btn_step.add_css_class("flat");
        btn_step.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_step.clone();
            let win = win_sc.clone();
            let title = lang.tr("step").to_string();
            btn_step.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().step.clone();
                prompt_shortcut_recording(&win, &title, &current_sc, is_en, move |sc| {
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
        row_reset.set_title(lang.tr("sc_reset"));
        let btn_reset = gtk::Button::with_label(&kb_cfg.borrow().reset.to_display_string());
        btn_reset.add_css_class("flat");
        btn_reset.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_reset.clone();
            let win = win_sc.clone();
            let title = lang.tr("reset").to_string();
            btn_reset.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().reset.clone();
                prompt_shortcut_recording(&win, &title, &current_sc, is_en, move |sc| {
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
        row_save.set_title(lang.tr("sc_save"));
        let btn_save = gtk::Button::with_label(&kb_cfg.borrow().save.to_display_string());
        btn_save.add_css_class("flat");
        btn_save.set_valign(gtk::Align::Center);
        {
            let kb_clone = kb_cfg.clone();
            let on_ch = on_change_kb.clone();
            let btn_clone = btn_save.clone();
            let win = win_sc.clone();
            let title = lang.tr("save").to_string();
            btn_save.connect_clicked(move |_| {
                let kb_inner = kb_clone.clone();
                let on_ch_inner = on_ch.clone();
                let btn = btn_clone.clone();
                let current_sc = kb_inner.borrow().save.clone();
                prompt_shortcut_recording(&win, &title, &current_sc, is_en, move |sc| {
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
        let btn_defaults = gtk::Button::with_label(lang.tr("btn_defaults"));
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
        page_plugins.set_title(lang.tr("tab_extensions"));
        page_plugins.set_icon_name(Some("application-x-addon-symbolic"));

        let install_group = adw::PreferencesGroup::new();
        install_group.set_title(lang.tr("grp_pkg_install"));
        install_group.set_description(Some(lang.tr("desc_pkg_install")));

        let install_row = adw::ActionRow::new();
        install_row.set_title(lang.tr("install_from_file"));
        install_row.set_subtitle(lang.tr("choose_plug_sub"));

        let btn_choose_plug = gtk::Button::with_label(lang.tr("btn_choose_plug"));
        btn_choose_plug.add_css_class("suggested-action");
        btn_choose_plug.set_valign(gtk::Align::Center);
        install_row.add_suffix(&btn_choose_plug);
        install_group.add(&install_row);

        page_plugins.add(&install_group);

        let list_group = adw::PreferencesGroup::new();
        list_group.set_title(lang.tr("grp_installed_ext"));

        // Helper to refresh plugin list
        let plugins_state = Rc::new(RefCell::new(Vec::new()));
        let list_group_rc = Rc::new(list_group.clone());
        let plug_lang = lang;

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
                    empty_row.set_title(plug_lang.tr("no_installed_ext"));
                    empty_row.set_subtitle(plug_lang.tr("no_ext_sub"));
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
                                .heading(plug_lang.tr("error"))
                                .body(&format!("{}: {}", plug_lang.tr("ext_status_fail"), e))
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
                    let del_tip = if plug_lang.is_en() { "Delete extension" } else { "Удалить расширение" };
                    btn_del.set_tooltip_text(Some(del_tip));

                    let del_id = meta.id.clone();
                    let win_del = win.clone();
                    let refresh_del = lg.clone();
                    btn_del.connect_clicked(move |_| {
                        if let Err(e) = PluginManager::delete_plugin(&del_id) {
                            let alert = adw::AlertDialog::builder()
                                .heading(plug_lang.tr("error"))
                                .body(&format!("{}: {}", plug_lang.tr("ext_status_fail"), e))
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
                                empty_row.set_title(plug_lang.tr("no_installed_ext"));
                                empty_row.set_subtitle(plug_lang.tr("no_ext_sub"));
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
                let chooser_title = if plug_lang.is_en() {
                    "Select NeoMir Extension Package (*.plug)"
                } else {
                    "Выберите пакет расширения NeoMir (*.plug)"
                };
                let accept_label = if plug_lang.is_en() { "Install" } else { "Установить" };
                let cancel_label = plug_lang.tr("cancel");

                let chooser = gtk::FileChooserNative::new(
                    Some(chooser_title),
                    Some(&win),
                    gtk::FileChooserAction::Open,
                    Some(accept_label),
                    Some(cancel_label),
                );

                let filter = gtk::FileFilter::new();
                filter.set_name(Some(plug_lang.tr("filter_plug")));
                filter.add_pattern("*.plug");
                chooser.add_filter(&filter);

                let all_filter = gtk::FileFilter::new();
                all_filter.set_name(Some(plug_lang.tr("filter_all")));
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
                                        let alert_body = if plug_lang.is_en() {
                                            format!("\"{}\" (version {}) successfully installed and ready.", meta.name, meta.version)
                                        } else {
                                            format!("«{}» (версия {}) успешно установлено и готово к работе.", meta.name, meta.version)
                                        };
                                        let alert = adw::AlertDialog::builder()
                                            .heading(plug_lang.tr("ext_installed"))
                                            .body(&alert_body)
                                            .build();
                                        alert.add_response("ok", "OK");
                                        alert.choose(&win_clone, Option::<&gio::Cancellable>::None, |_| {});
                                        rf_clone();
                                    }
                                    Err(err) => {
                                        let alert = adw::AlertDialog::builder()
                                            .heading(plug_lang.tr("error"))
                                            .body(&format!("{}:\n{}", plug_lang.tr("ext_install_fail"), err))
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

        if open_extensions {
            dialog.set_visible_page(&page_plugins);
        }

        dialog.present(Some(&win_parent));
    }
}
