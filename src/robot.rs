#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformerMode {
    Robot,
    Turtle,
}

#[derive(Debug, Clone)]
pub struct RobotField {
    pub width: usize,
    pub height: usize,
    pub robot_x: usize,
    pub robot_y: usize,
    pub start_x: usize,
    pub start_y: usize,
    pub h_walls: Vec<Vec<bool>>,
    pub v_walls: Vec<Vec<bool>>,
    pub painted: Vec<Vec<bool>>,
    pub initial_painted: Vec<Vec<bool>>,
    pub trace: Vec<(usize, usize)>,
    pub crashed: bool,
    pub crash_message: Option<String>,
    pub performer: PerformerMode,
    pub turtle_angle: f64,
    pub turtle_pen_down: bool,
    pub turtle_lines: Vec<((f64, f64), (f64, f64))>,
}

impl RobotField {
    pub fn new(width: usize, height: usize) -> Self {
        let width = width.max(2);
        let height = height.max(2);
        let h_walls = vec![vec![false; width]; height.saturating_sub(1)];
        let v_walls = vec![vec![false; width.saturating_sub(1)]; height];
        let painted = vec![vec![false; width]; height];
        let initial_painted = painted.clone();

        Self {
            width,
            height,
            robot_x: 0,
            robot_y: 0,
            start_x: 0,
            start_y: 0,
            h_walls,
            v_walls,
            painted,
            initial_painted,
            trace: vec![(0, 0)],
            crashed: false,
            crash_message: None,
            performer: PerformerMode::Robot,
            turtle_angle: 0.0,
            turtle_pen_down: true,
            turtle_lines: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let performer = self.performer;
        *self = Self::new(width, height);
        self.performer = performer;
    }

    pub fn reset_execution(&mut self) {
        self.robot_x = self.start_x;
        self.robot_y = self.start_y;
        self.painted = self.initial_painted.clone();
        self.trace = vec![(self.robot_x, self.robot_y)];
        self.crashed = false;
        self.crash_message = None;
        self.turtle_angle = 0.0;
        self.turtle_pen_down = true;
        self.turtle_lines.clear();
    }

    pub fn set_start_pos(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.start_x = x;
            self.start_y = y;
            self.reset_execution();
        }
    }

    pub fn turtle_forward(&mut self, dist: f64) {
        let rad = self.turtle_angle.to_radians();
        let old_x = self.robot_x as f64 + 0.5;
        let old_y = self.robot_y as f64 + 0.5;
        let new_x = (old_x + dist * rad.cos()).clamp(0.5, self.width as f64 - 0.5);
        let new_y = (old_y - dist * rad.sin()).clamp(0.5, self.height as f64 - 0.5);
        if self.turtle_pen_down {
            self.turtle_lines.push(((old_x, old_y), (new_x, new_y)));
        }
        self.robot_x = (new_x - 0.5).round().clamp(0.0, (self.width - 1) as f64) as usize;
        self.robot_y = (new_y - 0.5).round().clamp(0.0, (self.height - 1) as f64) as usize;
        self.trace.push((self.robot_x, self.robot_y));
    }

    pub fn turtle_backward(&mut self, dist: f64) {
        self.turtle_forward(-dist);
    }

    pub fn turtle_turn_left(&mut self, angle: f64) {
        self.turtle_angle = (self.turtle_angle + angle).rem_euclid(360.0);
    }

    pub fn turtle_turn_right(&mut self, angle: f64) {
        self.turtle_angle = (self.turtle_angle - angle).rem_euclid(360.0);
    }

    pub fn turtle_pen_down(&mut self) {
        self.turtle_pen_down = true;
    }

    pub fn turtle_pen_up(&mut self) {
        self.turtle_pen_down = false;
    }

    pub fn has_wall(&self, dir: &Direction) -> bool {
        match dir {
            Direction::Up => {
                if self.robot_y == 0 {
                    return true;
                }
                let wall_row = self.robot_y - 1;
                if wall_row < self.h_walls.len() && self.robot_x < self.h_walls[wall_row].len() {
                    self.h_walls[wall_row][self.robot_x]
                } else {
                    false
                }
            }
            Direction::Down => {
                if self.robot_y + 1 >= self.height {
                    return true;
                }
                let wall_row = self.robot_y;
                if wall_row < self.h_walls.len() && self.robot_x < self.h_walls[wall_row].len() {
                    self.h_walls[wall_row][self.robot_x]
                } else {
                    false
                }
            }
            Direction::Left => {
                if self.robot_x == 0 {
                    return true;
                }
                let wall_col = self.robot_x - 1;
                if self.robot_y < self.v_walls.len() && wall_col < self.v_walls[self.robot_y].len() {
                    self.v_walls[self.robot_y][wall_col]
                } else {
                    false
                }
            }
            Direction::Right => {
                if self.robot_x + 1 >= self.width {
                    return true;
                }
                let wall_col = self.robot_x;
                if self.robot_y < self.v_walls.len() && wall_col < self.v_walls[self.robot_y].len() {
                    self.v_walls[self.robot_y][wall_col]
                } else {
                    false
                }
            }
        }
    }

    pub fn move_robot(&mut self, dir: &Direction) -> Result<(), String> {
        if self.crashed {
            return Err("Робот уже разбит!".to_string());
        }

        if self.has_wall(dir) {
            self.crashed = true;
            let msg = format!("Робот врезался в стену при движении {:?}", dir);
            self.crash_message = Some(msg.clone());
            return Err(msg);
        }

        match dir {
            Direction::Up => {
                if self.robot_y > 0 {
                    self.robot_y -= 1;
                }
            }
            Direction::Down => {
                if self.robot_y + 1 < self.height {
                    self.robot_y += 1;
                }
            }
            Direction::Left => {
                if self.robot_x > 0 {
                    self.robot_x -= 1;
                }
            }
            Direction::Right => {
                if self.robot_x + 1 < self.width {
                    self.robot_x += 1;
                }
            }
        }

        self.trace.push((self.robot_x, self.robot_y));
        Ok(())
    }

    pub fn paint_cell(&mut self) {
        if self.robot_y < self.height && self.robot_x < self.width {
            self.painted[self.robot_y][self.robot_x] = true;
        }
    }

    pub fn is_painted(&self) -> bool {
        if self.robot_y < self.height && self.robot_x < self.width {
            self.painted[self.robot_y][self.robot_x]
        } else {
            false
        }
    }

    pub fn toggle_h_wall(&mut self, row: usize, col: usize) {
        if row < self.h_walls.len() && col < self.h_walls[row].len() {
            self.h_walls[row][col] = !self.h_walls[row][col];
        }
    }

    pub fn toggle_v_wall(&mut self, row: usize, col: usize) {
        if row < self.v_walls.len() && col < self.v_walls[row].len() {
            self.v_walls[row][col] = !self.v_walls[row][col];
        }
    }

    pub fn toggle_paint_cell(&mut self, col: usize, row: usize) {
        if row < self.height && col < self.width {
            self.painted[row][col] = !self.painted[row][col];
            self.initial_painted[row][col] = self.painted[row][col];
        }
    }

    pub fn clear_walls(&mut self) {
        for row in self.h_walls.iter_mut() {
            for w in row.iter_mut() {
                *w = false;
            }
        }
        for row in self.v_walls.iter_mut() {
            for w in row.iter_mut() {
                *w = false;
            }
        }
    }

    pub fn clear_painted(&mut self) {
        for row in self.painted.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
        self.initial_painted = self.painted.clone();
    }

    pub fn preset_oge_corridor() -> Self {
        let mut f = Self::new(12, 8);
        f.robot_x = 1;
        f.robot_y = 4;
        f.start_x = 1;
        f.start_y = 4;
        f.trace = vec![(1, 4)];

        for c in 1..10 {
            f.h_walls[3][c] = true;
            f.h_walls[4][c] = true;
        }
        f
    }

    pub fn preset_stairs() -> Self {
        let mut f = Self::new(10, 10);
        f.robot_x = 1;
        f.robot_y = 8;
        f.start_x = 1;
        f.start_y = 8;
        f.trace = vec![(1, 8)];

        for i in 0..4 {
            let col = 2 + i * 2;
            let row = 7 - i * 2;
            f.v_walls[row][col - 1] = true;
            f.h_walls[row - 1][col] = true;
        }
        f
    }

    pub fn preset_maze() -> Self {
        let mut f = Self::new(8, 8);
        f.robot_x = 0;
        f.robot_y = 0;
        f.start_x = 0;
        f.start_y = 0;
        f.trace = vec![(0, 0)];

        f.h_walls[0][0] = true;
        f.v_walls[1][0] = true;
        f.h_walls[2][1] = true;
        f.v_walls[0][2] = true;
        f.h_walls[1][3] = true;
        f.v_walls[3][2] = true;
        f.h_walls[4][3] = true;
        f.v_walls[5][4] = true;
        f
    }
}
