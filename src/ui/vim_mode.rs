use gtk::gdk::Key;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
}

pub struct VimController {
    pub enabled: Rc<RefCell<bool>>,
    pub mode: Rc<RefCell<VimMode>>,
    pub pending_op: Rc<RefCell<Option<char>>>,
    pub register: Rc<RefCell<Option<String>>>,
    pub status_box: gtk::Box,
    pub badge_label: gtk::Label,
    pub info_label: gtk::Label,
}

impl VimController {
    pub fn new() -> Self {
        let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        status_box.set_margin_start(8);
        status_box.set_margin_end(8);
        status_box.set_margin_top(2);
        status_box.set_margin_bottom(2);
        status_box.set_visible(false); // hidden until enabled

        let badge_label = gtk::Label::new(Some("NORMAL"));
        badge_label.add_css_class("vim-normal-badge");

        let info_label = gtk::Label::new(Some("Vim-режим активен (h,j,k,l для навигации, i для ввода, Esc для выхода)"));
        info_label.add_css_class("dim-label");

        status_box.append(&badge_label);
        status_box.append(&info_label);

        Self {
            enabled: Rc::new(RefCell::new(false)),
            mode: Rc::new(RefCell::new(VimMode::Normal)),
            pending_op: Rc::new(RefCell::new(None)),
            register: Rc::new(RefCell::new(None)),
            status_box,
            badge_label,
            info_label,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.borrow_mut() = enabled;
        *self.mode.borrow_mut() = VimMode::Normal;
        *self.pending_op.borrow_mut() = None;
        self.status_box.set_visible(enabled);
        self.update_ui();
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.borrow()
    }

    pub fn update_ui(&self) {
        let mode = *self.mode.borrow();
        match mode {
            VimMode::Normal => {
                self.badge_label.set_text("NORMAL");
                self.badge_label.remove_css_class("vim-insert-badge");
                self.badge_label.add_css_class("vim-normal-badge");
                let pending = *self.pending_op.borrow();
                if let Some(op) = pending {
                    self.info_label.set_text(&format!("Ожидание: {}-", op));
                } else {
                    self.info_label.set_text("NORMAL: h,j,k,l, w,b, 0,$, x, dd, yy, p, u, i (ввод)");
                }
            }
            VimMode::Insert => {
                self.badge_label.set_text("INSERT");
                self.badge_label.remove_css_class("vim-normal-badge");
                self.badge_label.add_css_class("vim-insert-badge");
                self.info_label.set_text("INSERT: ввод текста. Нажмите Esc для возврата в NORMAL");
            }
        }
    }

    pub fn handle_key(
        &self,
        key: Key,
        buffer: &gtk::TextBuffer,
        text_view: &gtk::TextView,
    ) -> glib::Propagation {
        if !*self.enabled.borrow() {
            return glib::Propagation::Proceed;
        }

        let current_mode = *self.mode.borrow();

        // In Insert mode
        if current_mode == VimMode::Insert {
            if key == Key::Escape {
                *self.mode.borrow_mut() = VimMode::Normal;
                *self.pending_op.borrow_mut() = None;
                // Move cursor 1 char back
                let mark = buffer.get_insert();
                let mut iter = buffer.iter_at_mark(&mark);
                if !iter.starts_line() {
                    iter.backward_char();
                    buffer.place_cursor(&iter);
                }
                self.update_ui();
                return glib::Propagation::Stop;
            }
            return glib::Propagation::Proceed;
        }

        // In Normal mode
        let mark = buffer.get_insert();
        let mut iter = buffer.iter_at_mark(&mark);

        // Escape clears pending ops
        if key == Key::Escape {
            *self.pending_op.borrow_mut() = None;
            self.update_ui();
            return glib::Propagation::Stop;
        }

        let pending = *self.pending_op.borrow();

        // Check pending 2-key operations
        if let Some(op) = pending {
            *self.pending_op.borrow_mut() = None;
            match (op, key) {
                ('d', Key::d) => {
                    // dd: Delete current line
                    let mut start = iter;
                    start.set_line_offset(0);
                    let mut end = start;
                    if !end.forward_line() {
                        end.forward_to_line_end();
                    }
                    let deleted = buffer.text(&start, &end, false).to_string();
                    *self.register.borrow_mut() = Some(deleted);
                    buffer.delete(&mut start, &mut end);
                    self.info_label.set_text("Строка удалена");
                    return glib::Propagation::Stop;
                }
                ('y', Key::y) => {
                    // yy: Yank current line
                    let mut start = iter;
                    start.set_line_offset(0);
                    let mut end = start;
                    if !end.forward_line() {
                        end.forward_to_line_end();
                    }
                    let yanked = buffer.text(&start, &end, false).to_string();
                    *self.register.borrow_mut() = Some(yanked);
                    self.info_label.set_text("Строка скопирована");
                    return glib::Propagation::Stop;
                }
                ('g', Key::g) => {
                    // gg: Go to top of document
                    let start = buffer.start_iter();
                    buffer.place_cursor(&start);
                    text_view.scroll_to_mark(&buffer.get_insert(), 0.0, true, 0.0, 0.0);
                    self.update_ui();
                    return glib::Propagation::Stop;
                }
                _ => {
                    self.update_ui();
                    return glib::Propagation::Stop;
                }
            }
        }

        // Handle single keys in Normal mode
        match key {
            // Mode switches
            Key::i => {
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::a => {
                if !iter.ends_line() {
                    iter.forward_char();
                    buffer.place_cursor(&iter);
                }
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::A => {
                iter.forward_to_line_end();
                buffer.place_cursor(&iter);
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::I => {
                iter.set_line_offset(0);
                buffer.place_cursor(&iter);
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::o => {
                iter.forward_to_line_end();
                buffer.insert(&mut iter, "\n");
                buffer.place_cursor(&iter);
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::O => {
                iter.set_line_offset(0);
                buffer.insert(&mut iter, "\n");
                let mut target = iter;
                target.backward_line();
                buffer.place_cursor(&target);
                *self.mode.borrow_mut() = VimMode::Insert;
                self.update_ui();
                glib::Propagation::Stop
            }

            // Motions
            Key::h | Key::Left => {
                if !iter.starts_line() {
                    iter.backward_char();
                    buffer.place_cursor(&iter);
                    text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                }
                glib::Propagation::Stop
            }
            Key::l | Key::Right => {
                if !iter.ends_line() {
                    iter.forward_char();
                    buffer.place_cursor(&iter);
                    text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                }
                glib::Propagation::Stop
            }
            Key::j | Key::Down => {
                let line_offset = iter.line_offset();
                if iter.forward_line() {
                    let chars_in_line = iter.chars_in_line().saturating_sub(1);
                    iter.set_line_offset(line_offset.min(chars_in_line));
                    buffer.place_cursor(&iter);
                    text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                }
                glib::Propagation::Stop
            }
            Key::k | Key::Up => {
                let line_offset = iter.line_offset();
                if iter.backward_line() {
                    let chars_in_line = iter.chars_in_line().saturating_sub(1);
                    iter.set_line_offset(line_offset.min(chars_in_line));
                    buffer.place_cursor(&iter);
                    text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                }
                glib::Propagation::Stop
            }
            Key::w => {
                iter.forward_word_end();
                iter.forward_char();
                buffer.place_cursor(&iter);
                text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                glib::Propagation::Stop
            }
            Key::b => {
                iter.backward_word_start();
                buffer.place_cursor(&iter);
                text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                glib::Propagation::Stop
            }
            Key::_0 | Key::asciicircum => {
                iter.set_line_offset(0);
                buffer.place_cursor(&iter);
                text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                glib::Propagation::Stop
            }
            Key::dollar => {
                iter.forward_to_line_end();
                buffer.place_cursor(&iter);
                text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                glib::Propagation::Stop
            }
            Key::G => {
                let end = buffer.end_iter();
                buffer.place_cursor(&end);
                text_view.scroll_to_mark(&buffer.get_insert(), 0.0, false, 0.0, 0.0);
                glib::Propagation::Stop
            }

            // Edit actions
            Key::x => {
                let mut end = iter;
                if !end.ends_line() && end.forward_char() {
                    buffer.delete(&mut iter, &mut end);
                }
                glib::Propagation::Stop
            }
            Key::d => {
                *self.pending_op.borrow_mut() = Some('d');
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::y => {
                *self.pending_op.borrow_mut() = Some('y');
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::g => {
                *self.pending_op.borrow_mut() = Some('g');
                self.update_ui();
                glib::Propagation::Stop
            }
            Key::p => {
                if let Some(text) = self.register.borrow().as_ref() {
                    let mut line_end = iter;
                    line_end.forward_to_line_end();
                    let to_insert = if text.ends_with('\n') {
                        format!("\n{}", text.trim_end_matches('\n'))
                    } else {
                        text.clone()
                    };
                    buffer.insert(&mut line_end, &to_insert);
                    self.info_label.set_text("Вставлено");
                }
                glib::Propagation::Stop
            }
            Key::P => {
                if let Some(text) = self.register.borrow().as_ref() {
                    let mut line_start = iter;
                    line_start.set_line_offset(0);
                    let to_insert = if text.ends_with('\n') {
                        text.clone()
                    } else {
                        format!("{}\n", text)
                    };
                    buffer.insert(&mut line_start, &to_insert);
                    self.info_label.set_text("Вставлено");
                }
                glib::Propagation::Stop
            }
            Key::u => {
                buffer.undo();
                self.info_label.set_text("Отмена (undo)");
                glib::Propagation::Stop
            }

            // Catch any other typing keys to prevent insertion in normal mode
            _ => glib::Propagation::Stop,
        }
    }
}
