use crate::robot::{PerformerMode, RobotField};
use gtk::cairo::Context;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct RobotFieldView {
    pub widget: gtk::DrawingArea,
    pub field: Rc<RefCell<RobotField>>,
}

impl RobotFieldView {
    pub fn new(field: Rc<RefCell<RobotField>>) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        drawing_area.set_content_width(450);
        drawing_area.set_content_height(350);

        let field_clone = field.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let f = field_clone.borrow();
            Self::draw_field(&f, cr, width as f64, height as f64);
        });

        // Click gesture for editing field
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(0); // accept all buttons
        let field_click = field.clone();
        let da_weak = drawing_area.downgrade();

        click_gesture.connect_pressed(move |gesture, n_press, x, y| {
            if n_press != 1 {
                return;
            }
            let button = gesture.current_button();
            let da = match da_weak.upgrade() {
                Some(d) => d,
                None => return,
            };

            let width = da.width() as f64;
            let height = da.height() as f64;

            let mut f = field_click.borrow_mut();
            let (cell_size, offset_x, offset_y) = Self::compute_layout(&f, width, height);

            let rel_x = x - offset_x;
            let rel_y = y - offset_y;

            if rel_x < 0.0 || rel_y < 0.0 {
                return;
            }

            let col = (rel_x / cell_size) as usize;
            let row = (rel_y / cell_size) as usize;

            if col >= f.width || row >= f.height {
                return;
            }

            let in_cell_x = rel_x - (col as f64 * cell_size);
            let in_cell_y = rel_y - (row as f64 * cell_size);
            let wall_threshold = (cell_size * 0.22).clamp(6.0, 14.0);

            // Right click: Move robot start position
            if button == 3 {
                f.set_start_pos(col, row);
                da.queue_draw();
                return;
            }

            // Left click
            let near_right = in_cell_x >= cell_size - wall_threshold;
            let near_left = in_cell_x <= wall_threshold;
            let near_bottom = in_cell_y >= cell_size - wall_threshold;
            let near_top = in_cell_y <= wall_threshold;

            if near_right && col + 1 < f.width {
                f.toggle_v_wall(row, col);
            } else if near_left && col > 0 {
                f.toggle_v_wall(row, col - 1);
            } else if near_bottom && row + 1 < f.height {
                f.toggle_h_wall(row, col);
            } else if near_top && row > 0 {
                f.toggle_h_wall(row - 1, col);
            } else {
                // Click in center of cell: toggle paint
                f.toggle_paint_cell(col, row);
            }

            da.queue_draw();
        });

        drawing_area.add_controller(click_gesture);

        Self {
            widget: drawing_area,
            field,
        }
    }

    fn compute_layout(field: &RobotField, width: f64, height: f64) -> (f64, f64, f64) {
        let padding = 16.0;
        let avail_w = (width - padding * 2.0).max(10.0);
        let avail_h = (height - padding * 2.0).max(10.0);

        let cell_w = avail_w / field.width as f64;
        let cell_h = avail_h / field.height as f64;
        let cell_size = cell_w.min(cell_h).clamp(16.0, 75.0);

        let grid_w = cell_size * field.width as f64;
        let grid_h = cell_size * field.height as f64;
        let offset_x = (width - grid_w) / 2.0;
        let offset_y = (height - grid_h) / 2.0;

        (cell_size, offset_x, offset_y)
    }

    pub fn draw_field(field: &RobotField, cr: &Context, width: f64, height: f64) {
        // Background fill
        cr.set_source_rgb(0.96, 0.96, 0.97);
        cr.paint().unwrap();

        let (cell_size, offset_x, offset_y) = Self::compute_layout(field, width, height);

        // Draw painted cells
        for r in 0..field.height {
            for c in 0..field.width {
                if field.painted[r][c] {
                    let x = offset_x + c as f64 * cell_size;
                    let y = offset_y + r as f64 * cell_size;
                    cr.set_source_rgb(0.82, 0.88, 0.95);
                    cr.rectangle(x, y, cell_size, cell_size);
                    let _ = cr.fill();

                    // Pattern dots inside painted cell
                    cr.set_source_rgb(0.68, 0.76, 0.88);
                    let step = cell_size / 4.0;
                    for i in 1..4 {
                        for j in 1..4 {
                            cr.arc(x + i as f64 * step, y + j as f64 * step, 1.2, 0.0, std::f64::consts::TAU);
                            let _ = cr.fill();
                        }
                    }
                }
            }
        }

        // Draw grid lines
        cr.set_source_rgb(0.78, 0.80, 0.84);
        cr.set_line_width(1.0);

        for c in 0..=field.width {
            let x = offset_x + c as f64 * cell_size;
            cr.move_to(x, offset_y);
            cr.line_to(x, offset_y + field.height as f64 * cell_size);
            let _ = cr.stroke();
        }

        for r in 0..=field.height {
            let y = offset_y + r as f64 * cell_size;
            cr.move_to(offset_x, y);
            cr.line_to(offset_x + field.width as f64 * cell_size, y);
            let _ = cr.stroke();
        }

        // Draw trace of visited cells (Robot or Turtle)
        if field.trace.len() > 1 {
            cr.set_source_rgba(0.2, 0.5, 0.9, 0.35);
            for &(tx, ty) in &field.trace {
                let cx = offset_x + tx as f64 * cell_size + cell_size * 0.5;
                let cy = offset_y + ty as f64 * cell_size + cell_size * 0.5;
                cr.arc(cx, cy, 3.0, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }
        }

        // Draw Turtle vector lines if any
        if !field.turtle_lines.is_empty() {
            cr.set_source_rgba(0.18, 0.76, 0.49, 0.95); // Emerald green trace
            cr.set_line_width(3.5);
            cr.set_line_cap(gtk::cairo::LineCap::Round);
            cr.set_line_join(gtk::cairo::LineJoin::Round);

            for &((x1, y1), (x2, y2)) in &field.turtle_lines {
                let sx1 = offset_x + x1 * cell_size;
                let sy1 = offset_y + y1 * cell_size;
                let sx2 = offset_x + x2 * cell_size;
                let sy2 = offset_y + y2 * cell_size;
                cr.move_to(sx1, sy1);
                cr.line_to(sx2, sy2);
                let _ = cr.stroke();
            }
        }

        // Draw internal horizontal walls
        cr.set_source_rgb(0.15, 0.18, 0.22);
        cr.set_line_width(4.5);
        cr.set_line_cap(gtk::cairo::LineCap::Round);

        for r in 0..field.h_walls.len() {
            for c in 0..field.width {
                if field.h_walls[r][c] {
                    let x1 = offset_x + c as f64 * cell_size;
                    let x2 = x1 + cell_size;
                    let y = offset_y + (r + 1) as f64 * cell_size;
                    cr.move_to(x1, y);
                    cr.line_to(x2, y);
                    let _ = cr.stroke();
                }
            }
        }

        // Draw internal vertical walls
        for r in 0..field.height {
            for c in 0..field.v_walls[0].len() {
                if field.v_walls[r][c] {
                    let x = offset_x + (c + 1) as f64 * cell_size;
                    let y1 = offset_y + r as f64 * cell_size;
                    let y2 = y1 + cell_size;
                    cr.move_to(x, y1);
                    cr.line_to(x, y2);
                    let _ = cr.stroke();
                }
            }
        }

        // Draw outer boundary walls (Thick KuMir boundary)
        cr.set_source_rgb(0.12, 0.14, 0.18);
        cr.set_line_width(5.0);
        let grid_w = cell_size * field.width as f64;
        let grid_h = cell_size * field.height as f64;
        cr.rectangle(offset_x, offset_y, grid_w, grid_h);
        let _ = cr.stroke();

        // Coordinates for current performer
        let rx = offset_x + field.robot_x as f64 * cell_size + cell_size * 0.5;
        let ry = offset_y + field.robot_y as f64 * cell_size + cell_size * 0.5;
        let radius = cell_size * 0.36;

        if field.performer == PerformerMode::Turtle {
            // ==========================================
            // Render Turtle Performer (Черепаха)
            // ==========================================
            let rad = field.turtle_angle.to_radians();

            // Shell shadow
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
            cr.arc(rx, ry + 2.0, radius * 0.9, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();

            // 4 Flippers (Lappies)
            cr.set_source_rgb(0.18, 0.62, 0.38);
            for flipper_angle in [rad + 0.7, rad - 0.7, rad + 2.4, rad - 2.4] {
                let fx = rx + (radius * 0.85) * flipper_angle.cos();
                let fy = ry - (radius * 0.85) * flipper_angle.sin();
                cr.arc(fx, fy, radius * 0.30, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }

            // Turtle Shell (Green dome)
            cr.set_source_rgb(0.15, 0.68, 0.37);
            cr.arc(rx, ry, radius * 0.80, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();

            // Shell rim
            cr.set_source_rgb(0.10, 0.48, 0.26);
            cr.set_line_width(2.0);
            cr.arc(rx, ry, radius * 0.80, 0.0, std::f64::consts::TAU);
            let _ = cr.stroke();

            // Turtle Head pointing in direction of movement
            let head_x = rx + (radius * 0.95) * rad.cos();
            let head_y = ry - (radius * 0.95) * rad.sin();
            cr.set_source_rgb(0.20, 0.75, 0.42);
            cr.arc(head_x, head_y, radius * 0.38, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();

            cr.set_source_rgb(0.10, 0.48, 0.26);
            cr.set_line_width(1.5);
            cr.arc(head_x, head_y, radius * 0.38, 0.0, std::f64::consts::TAU);
            let _ = cr.stroke();

            // Pen indicator in center of shell
            if field.turtle_pen_down {
                cr.set_source_rgb(0.9, 0.2, 0.2); // Red pen down dot
                cr.arc(rx, ry, radius * 0.22, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            } else {
                cr.set_source_rgb(0.7, 0.7, 0.7); // Gray pen up ring
                cr.set_line_width(1.8);
                cr.arc(rx, ry, radius * 0.22, 0.0, std::f64::consts::TAU);
                let _ = cr.stroke();
            }
        } else {
            // ==========================================
            // Render Robot Performer (Робот)
            // ==========================================
            if field.crashed {
                // Crashed Robot representation (red warning marker)
                cr.set_source_rgb(0.9, 0.2, 0.2);
                cr.arc(rx, ry, radius, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();

                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(3.0);
                let d = radius * 0.55;
                cr.move_to(rx - d, ry - d);
                cr.line_to(rx + d, ry + d);
                cr.move_to(rx + d, ry - d);
                cr.line_to(rx - d, ry + d);
                let _ = cr.stroke();
            } else {
                // KuMir Robot: Diamond with inner eye & robot antenna
                // Shadow
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
                cr.arc(rx, ry + 2.0, radius * 0.9, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();

                // Diamond body
                cr.set_source_rgb(0.18, 0.45, 0.85); // Classic deep KuMir blue
                cr.move_to(rx, ry - radius);
                cr.line_to(rx + radius, ry);
                cr.line_to(rx, ry + radius);
                cr.line_to(rx - radius, ry);
                cr.close_path();
                let _ = cr.fill();

                // Diamond border
                cr.set_source_rgb(0.08, 0.25, 0.55);
                cr.set_line_width(2.0);
                cr.move_to(rx, ry - radius);
                cr.line_to(rx + radius, ry);
                cr.line_to(rx, ry + radius);
                cr.line_to(rx - radius, ry);
                cr.close_path();
                let _ = cr.stroke();

                // Inner eye/circle
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.arc(rx, ry, radius * 0.38, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();

                // Pupil
                cr.set_source_rgb(0.15, 0.2, 0.3);
                cr.arc(rx, ry, radius * 0.18, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }
        }
    }
}
