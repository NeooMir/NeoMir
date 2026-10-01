use crate::ui::theme::ThemeId;
use crate::ui::vim_mode::VimController;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxMode {
    Kumir,
    Python,
}

impl SyntaxMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "python" | "py" => SyntaxMode::Python,
            _ => SyntaxMode::Kumir,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SyntaxMode::Kumir => "kumir",
            SyntaxMode::Python => "python",
        }
    }
}


pub struct EditorView {
    pub container: gtk::Box,
    pub text_view: gtk::TextView,
    pub buffer: gtk::TextBuffer,
    pub line_numbers_view: gtk::TextView,
    pub line_numbers_buffer: gtk::TextBuffer,
    pub vim: Rc<VimController>,
    pub current_theme: Rc<RefCell<ThemeId>>,
    css_provider: gtk::CssProvider,
    tag_keyword: gtk::TextTag,
    tag_robot: gtk::TextTag,
    tag_sensor: gtk::TextTag,
    tag_string: gtk::TextTag,
    tag_comment: gtk::TextTag,
    tag_number: gtk::TextTag,
    tag_current_line: gtk::TextTag,
    line_tag: gtk::TextTag,
    current_highlighted_line: Rc<RefCell<Option<usize>>>,
    pub syntax_mode: Rc<RefCell<SyntaxMode>>,
}

impl EditorView {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);

        let editor_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        editor_row.set_vexpand(true);
        editor_row.set_hexpand(true);

        // Text buffers
        let buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let line_numbers_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let syntax_mode = Rc::new(RefCell::new(SyntaxMode::Kumir));

        let initial_theme = ThemeId::VsCode;

        // Tags
        let tag_keyword = buffer.create_tag(Some("keyword"), &[
            ("weight", &700i32),
            ("foreground", &initial_theme.keyword_color()),
        ]).unwrap();

        let tag_robot = buffer.create_tag(Some("robot"), &[
            ("weight", &700i32),
            ("foreground", &initial_theme.robot_color()),
        ]).unwrap();

        let tag_sensor = buffer.create_tag(Some("sensor"), &[
            ("weight", &700i32),
            ("foreground", &initial_theme.sensor_color()),
        ]).unwrap();

        let tag_string = buffer.create_tag(Some("string"), &[
            ("foreground", &initial_theme.string_color()),
        ]).unwrap();

        let tag_comment = buffer.create_tag(Some("comment"), &[
            ("style", &pango::Style::Italic),
            ("foreground", &initial_theme.comment_color()),
        ]).unwrap();

        let tag_number = buffer.create_tag(Some("number"), &[
            ("foreground", &initial_theme.number_color()),
        ]).unwrap();

        let tag_current_line = buffer.create_tag(Some("current_line"), &[
            ("background", &initial_theme.current_line_bg()),
            ("foreground", &initial_theme.current_line_fg()),
        ]).unwrap();

        // Main TextView
        let text_view = gtk::TextView::with_buffer(&buffer);
        text_view.set_monospace(true);
        text_view.set_left_margin(8);
        text_view.set_right_margin(8);
        text_view.set_top_margin(6);
        text_view.set_bottom_margin(6);
        text_view.set_wrap_mode(gtk::WrapMode::None);
        text_view.add_css_class("neomir-editor");

        // Line numbers TextView
        let line_numbers_view = gtk::TextView::with_buffer(&line_numbers_buffer);
        line_numbers_view.set_monospace(true);
        line_numbers_view.set_editable(false);
        line_numbers_view.set_cursor_visible(false);
        line_numbers_view.set_can_focus(false);
        line_numbers_view.set_left_margin(10);
        line_numbers_view.set_right_margin(8);
        line_numbers_view.set_top_margin(6);
        line_numbers_view.set_bottom_margin(6);
        line_numbers_view.add_css_class("neomir-gutter");

        // Gutter styling
        let line_tag = line_numbers_buffer.create_tag(Some("line_num"), &[
            ("foreground", &initial_theme.gutter_fg()),
            ("justification", &gtk::Justification::Right),
        ]).unwrap();

        let scrolled_editor = gtk::ScrolledWindow::new();
        scrolled_editor.set_child(Some(&text_view));
        scrolled_editor.set_vexpand(true);
        scrolled_editor.set_hexpand(true);

        let scrolled_gutter = gtk::ScrolledWindow::new();
        scrolled_gutter.set_child(Some(&line_numbers_view));
        scrolled_gutter.set_vexpand(true);
        scrolled_gutter.set_hexpand(false);
        scrolled_gutter.set_hscrollbar_policy(gtk::PolicyType::Never);
        scrolled_gutter.set_vscrollbar_policy(gtk::PolicyType::Never);

        // Sync vertical scrolling
        let vadj = scrolled_editor.vadjustment();
        scrolled_gutter.set_vadjustment(Some(&vadj));

        editor_row.append(&scrolled_gutter);
        editor_row.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        editor_row.append(&scrolled_editor);

        container.append(&editor_row);

        // Vim Controller and bottom badge
        let vim = Rc::new(VimController::new());
        container.append(&vim.status_box);

        // Key Controller for Vim Mode
        let key_controller = gtk::EventControllerKey::new();
        let vim_key = vim.clone();
        let buf_key = buffer.clone();
        let tv_key = text_view.clone();
        key_controller.connect_key_pressed(move |_, keyval, _keycode, _state| {
            vim_key.handle_key(keyval, &buf_key, &tv_key)
        });
        text_view.add_controller(key_controller);

        // CSS Provider for themes
        let css_provider = gtk::CssProvider::new();
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let current_highlighted_line = Rc::new(RefCell::new(None));
        let current_theme = Rc::new(RefCell::new(initial_theme));

        let view = Self {
            container,
            text_view,
            buffer,
            line_numbers_view,
            line_numbers_buffer,
            vim,
            current_theme,
            css_provider,
            tag_keyword,
            tag_robot,
            tag_sensor,
            tag_string,
            tag_comment,
            tag_number,
            tag_current_line,
            line_tag: line_tag.clone(),
            current_highlighted_line,
            syntax_mode,
        };

        view.apply_theme(initial_theme);
        view.setup_signals(line_tag);
        view.update_gutter();
        view
    }

    pub fn apply_theme(&self, theme: ThemeId) {
        *self.current_theme.borrow_mut() = theme;

        // Apply CSS
        let css = theme.generate_css();
        self.css_provider.load_from_data(&css);

        // Update tags
        self.tag_keyword.set_property("foreground", theme.keyword_color());
        self.tag_robot.set_property("foreground", theme.robot_color());
        self.tag_sensor.set_property("foreground", theme.sensor_color());
        self.tag_string.set_property("foreground", theme.string_color());
        self.tag_comment.set_property("foreground", theme.comment_color());
        self.tag_number.set_property("foreground", theme.number_color());
        self.tag_current_line.set_property("background", theme.current_line_bg());
        self.tag_current_line.set_property("foreground", theme.current_line_fg());
        self.line_tag.set_property("foreground", theme.gutter_fg());

        // Re-highlight syntax
        Self::highlight_syntax(
            &self.buffer,
            *self.syntax_mode.borrow(),
            &self.tag_keyword,
            &self.tag_robot,
            &self.tag_sensor,
            &self.tag_string,
            &self.tag_comment,
            &self.tag_number,
        );

        // Update Libadwaita color scheme
        let style_manager = adw::StyleManager::default();
        if theme.is_dark() {
            style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
        } else {
            style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
        }
    }

    pub fn update_gutter(&self) {
        let line_count = self.buffer.line_count().max(1);
        let mut numbers_str = String::new();
        for i in 1..=line_count {
            numbers_str.push_str(&format!("{:>3}\n", i));
        }
        self.line_numbers_buffer.set_text(&numbers_str);
        let start = self.line_numbers_buffer.start_iter();
        let end = self.line_numbers_buffer.end_iter();
        self.line_numbers_buffer.apply_tag(&self.line_tag, &start, &end);
    }

    fn setup_signals(&self, line_tag: gtk::TextTag) {
        let gutter_buf = self.line_numbers_buffer.clone();
        let kw_tag = self.tag_keyword.clone();
        let rb_tag = self.tag_robot.clone();
        let sn_tag = self.tag_sensor.clone();
        let str_tag = self.tag_string.clone();
        let cmt_tag = self.tag_comment.clone();
        let num_tag = self.tag_number.clone();
        let syn_mode = self.syntax_mode.clone();

        self.buffer.connect_changed(move |buffer| {
            // Update line numbers
            let line_count = buffer.line_count().max(1);
            let mut numbers_str = String::new();
            for i in 1..=line_count {
                numbers_str.push_str(&format!("{:>3}\n", i));
            }
            gutter_buf.set_text(&numbers_str);
            let start = gutter_buf.start_iter();
            let end = gutter_buf.end_iter();
            gutter_buf.apply_tag(&line_tag, &start, &end);

            // Re-apply syntax highlighting
            let mode = *syn_mode.borrow();
            Self::highlight_syntax(buffer, mode, &kw_tag, &rb_tag, &sn_tag, &str_tag, &cmt_tag, &num_tag);
        });
    }

    pub fn syntax_mode(&self) -> SyntaxMode {
        *self.syntax_mode.borrow()
    }

    pub fn set_syntax_mode(&self, mode: SyntaxMode) {
        *self.syntax_mode.borrow_mut() = mode;
        self.rehighlight();
    }

    pub fn rehighlight(&self) {
        Self::highlight_syntax(
            &self.buffer,
            *self.syntax_mode.borrow(),
            &self.tag_keyword,
            &self.tag_robot,
            &self.tag_sensor,
            &self.tag_string,
            &self.tag_comment,
            &self.tag_number,
        );
    }

    fn highlight_syntax(
        buffer: &gtk::TextBuffer,
        mode: SyntaxMode,
        tag_kw: &gtk::TextTag,
        tag_rb: &gtk::TextTag,
        tag_sn: &gtk::TextTag,
        tag_str: &gtk::TextTag,
        tag_cmt: &gtk::TextTag,
        tag_num: &gtk::TextTag,
    ) {
        let (start, end) = buffer.bounds();
        buffer.remove_tag(tag_kw, &start, &end);
        buffer.remove_tag(tag_rb, &start, &end);
        buffer.remove_tag(tag_sn, &start, &end);
        buffer.remove_tag(tag_str, &start, &end);
        buffer.remove_tag(tag_cmt, &start, &end);
        buffer.remove_tag(tag_num, &start, &end);

        let text = buffer.text(&start, &end, true).to_string();

        let kumir_keywords = [
            "алг", "нач", "кон", "исп", "использовать", "цел", "вещ", "лог", "сим", "лит",
            "если", "то", "иначе", "все", "всё", "нц", "кц", "пока", "раз", "для", "от", "до",
            "шаг", "вывод", "ввод", "нс", "да", "нет", "и", "или", "не", "div", "mod",
            // English equivalents
            "alg", "begin", "end", "use", "int", "float", "bool", "char", "string",
            "if", "then", "else", "fi", "loop", "pool", "while", "times", "for", "from", "to",
            "step", "output", "print", "input", "newline", "true", "false", "and", "or", "not",
        ];

        let python_keywords = [
            "def", "class", "import", "from", "as", "return", "if", "elif", "else",
            "while", "for", "in", "try", "except", "finally", "with", "pass", "break",
            "continue", "lambda", "yield", "none", "true", "false", "is", "not", "and",
            "or", "self", "async", "await", "global", "nonlocal", "del", "raise", "assert",
        ];

        let robot_cmds = [
            "вверх", "вниз", "влево", "вправо", "закрасить", "робот", "сброс",
            "up", "down", "left", "right", "paint", "robot", "reset",
            "forward", "backward", "turn_left", "turn_right", "turtle",
            "вперед", "назад", "налево", "направо", "черепаха",
            "pen_down", "pen_up", "tail_down", "tail_up", "step_back",
        ];

        let sensors = [
            "сверху_свободно", "снизу_свободно", "слева_свободно", "справа_свободно",
            "сверху_стена", "снизу_стена", "слева_стена", "справа_стена",
            "клетка_закрашена", "клетка_чистая", "температура", "радиация",
            "free", "wall", "painted", "clean", "cell_painted", "cell_clean",
            "print", "range", "len", "int", "str", "float", "list", "dict", "set", "bool", "input", "open",
        ];

        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];

            // Comments
            let is_comment = match mode {
                SyntaxMode::Kumir => c == '|',
                SyntaxMode::Python => c == '#' || c == '|',
            };
            if is_comment {
                let start_idx = i;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                let start_iter = buffer.iter_at_offset(start_idx as i32);
                let end_iter = buffer.iter_at_offset(i as i32);
                buffer.apply_tag(tag_cmt, &start_iter, &end_iter);
                continue;
            }

            // Strings
            let is_string_start = match mode {
                SyntaxMode::Kumir => c == '"',
                SyntaxMode::Python => c == '"' || c == '\'',
            };
            if is_string_start {
                let quote = c;
                let start_idx = i;
                i += 1;
                while i < len && chars[i] != quote && chars[i] != '\n' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < len && chars[i] == quote {
                    i += 1;
                }
                let start_iter = buffer.iter_at_offset(start_idx as i32);
                let end_iter = buffer.iter_at_offset(i as i32);
                buffer.apply_tag(tag_str, &start_iter, &end_iter);
                continue;
            }

            // Numbers
            if c.is_ascii_digit() {
                let start_idx = i;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let start_iter = buffer.iter_at_offset(start_idx as i32);
                let end_iter = buffer.iter_at_offset(i as i32);
                buffer.apply_tag(tag_num, &start_iter, &end_iter);
                continue;
            }

            // Identifiers / Keywords
            if c.is_alphabetic() || c == '_' {
                let start_char_idx = i;
                let mut word = String::new();
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    word.push(chars[i]);
                    i += 1;
                }
                let end_char_idx = i;

                let lower = word.to_lowercase();
                let is_kw = match mode {
                    SyntaxMode::Kumir => kumir_keywords.contains(&lower.as_str()),
                    SyntaxMode::Python => python_keywords.contains(&lower.as_str()),
                };

                let matched_tag = if is_kw {
                    Some(tag_kw)
                } else if robot_cmds.contains(&lower.as_str()) {
                    Some(tag_rb)
                } else if sensors.contains(&lower.as_str()) {
                    Some(tag_sn)
                } else {
                    None
                };

                if let Some(tag) = matched_tag {
                    let start_iter = buffer.iter_at_offset(start_char_idx as i32);
                    let end_iter = buffer.iter_at_offset(end_char_idx as i32);
                    buffer.apply_tag(tag, &start_iter, &end_iter);
                }
                continue;
            }

            i += 1;
        }
    }

    pub fn text(&self) -> String {
        let (start, end) = self.buffer.bounds();
        self.buffer.text(&start, &end, false).to_string()
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
        self.update_gutter();
    }

    pub fn insert_at_cursor(&self, text: &str) {
        self.buffer.insert_at_cursor(text);
        self.text_view.grab_focus();
    }

    pub fn highlight_line(&self, line: Option<usize>) {
        if let Some(prev) = *self.current_highlighted_line.borrow() {
            if prev >= 1 && prev <= self.buffer.line_count() as usize {
                if let Some(start) = self.buffer.iter_at_line((prev - 1) as i32) {
                    let mut end = start;
                    if !end.forward_line() {
                        end.forward_to_end();
                    }
                    self.buffer.remove_tag(&self.tag_current_line, &start, &end);
                }
            }
        }

        *self.current_highlighted_line.borrow_mut() = line;

        if let Some(l) = line {
            if l >= 1 && l <= self.buffer.line_count() as usize {
                if let Some(mut start) = self.buffer.iter_at_line((l - 1) as i32) {
                    let mut end = start;
                    if !end.forward_line() {
                        end.forward_to_end();
                    }
                    self.buffer.apply_tag(&self.tag_current_line, &start, &end);
                    self.text_view.scroll_to_iter(&mut start, 0.1, false, 0.0, 0.0);
                }
            }
        }
    }
}
