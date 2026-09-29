use gtk::prelude::*;

pub struct ConsoleView {
    pub container: gtk::Box,
    pub output_buffer: gtk::TextBuffer,
    pub log_buffer: gtk::TextBuffer,
    tag_error: gtk::TextTag,
    tag_success: gtk::TextTag,
    tag_info: gtk::TextTag,
}

impl ConsoleView {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header with title and clear button
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_start(8);
        header.set_margin_end(8);
        header.set_margin_top(4);
        header.set_margin_bottom(4);

        let title = gtk::Label::new(Some("Вывод и Журнал"));
        title.add_css_class("heading");
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Start);
        header.append(&title);

        let btn_clear = gtk::Button::from_icon_name("edit-clear-symbolic");
        btn_clear.set_tooltip_text(Some("Очистить консоль"));
        btn_clear.add_css_class("flat");
        header.append(&btn_clear);

        container.append(&header);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        container.append(&sep);

        // Notebook with Output and Robot Log
        let notebook = gtk::Notebook::new();
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);

        // Output tab
        let output_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let tag_error = output_buffer.create_tag(Some("err"), &[
            ("foreground", &"#e01b24"),
            ("weight", &700i32),
        ]).unwrap();
        let tag_success = output_buffer.create_tag(Some("ok"), &[
            ("foreground", &"#2ec27e"),
            ("weight", &700i32),
        ]).unwrap();
        let tag_info = output_buffer.create_tag(Some("info"), &[
            ("foreground", &"#1c71d8"),
        ]).unwrap();

        let output_tv = gtk::TextView::with_buffer(&output_buffer);
        output_tv.set_monospace(true);
        output_tv.set_editable(false);
        output_tv.set_cursor_visible(false);
        output_tv.set_left_margin(8);
        output_tv.set_top_margin(6);

        let scrolled_output = gtk::ScrolledWindow::new();
        scrolled_output.set_child(Some(&output_tv));
        notebook.append_page(&scrolled_output, Some(&gtk::Label::new(Some("Вывод программы"))));

        // Robot log tab
        let log_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let log_tv = gtk::TextView::with_buffer(&log_buffer);
        log_tv.set_monospace(true);
        log_tv.set_editable(false);
        log_tv.set_cursor_visible(false);
        log_tv.set_left_margin(8);
        log_tv.set_top_margin(6);

        let scrolled_log = gtk::ScrolledWindow::new();
        scrolled_log.set_child(Some(&log_tv));
        notebook.append_page(&scrolled_log, Some(&gtk::Label::new(Some("Журнал Робота"))));

        container.append(&notebook);

        // Connect clear button
        let out_buf_clear = output_buffer.clone();
        let log_buf_clear = log_buffer.clone();
        btn_clear.connect_clicked(move |_| {
            out_buf_clear.set_text("");
            log_buf_clear.set_text("");
        });

        Self {
            container,
            output_buffer,
            log_buffer,
            tag_error,
            tag_success,
            tag_info,
        }
    }

    pub fn print_output(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        self.output_buffer.insert(&mut end, text);
    }

    pub fn print_error(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        self.output_buffer.insert(&mut end, &format!("❌ {}\n", text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_error, &mut start_iter, &end_iter);
    }

    pub fn print_success(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        self.output_buffer.insert(&mut end, &format!("✓ {}\n", text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_success, &mut start_iter, &end_iter);
    }

    pub fn print_info(&self, text: &str) {
        let mut end = self.output_buffer.end_iter();
        let start_offset = end.offset();
        self.output_buffer.insert(&mut end, &format!("ℹ {}\n", text));
        let mut start_iter = self.output_buffer.iter_at_offset(start_offset);
        let end_iter = self.output_buffer.end_iter();
        self.output_buffer.apply_tag(&self.tag_info, &mut start_iter, &end_iter);
    }

    pub fn log_action(&self, text: &str) {
        let mut end = self.log_buffer.end_iter();
        self.log_buffer.insert(&mut end, &format!("• {}\n", text));
    }

    pub fn clear(&self) {
        self.output_buffer.set_text("");
        self.log_buffer.set_text("");
    }
}
