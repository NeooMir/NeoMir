#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct RobotField {
    pub width: usize,
    pub height: usize,
    pub robot_x: usize,
    pub robot_y: usize,
    pub start_x: usize,
    pub start_y: usize,
    // h_walls[y][x]: wall between cell (x, y) and (x, y+1). Size: (height-1) x width
    pub h_walls: Vec<Vec<bool>>,
    // v_walls[y][x]: wall between cell (x, y) and (x+1, y). Size: height x (width-1)
    pub v_walls: Vec<Vec<bool>>,
    // painted cells
    pub painted: Vec<Vec<bool>>,
    pub initial_painted: Vec<Vec<bool>>,
    // trail of visited cells
    pub trace: Vec<(usize, usize)>,
    pub crashed: bool,
    pub crash_message: Option<String>,
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
        }
    }

    pub fn reset_execution(&mut self) {
        self.robot_x = self.start_x;
        self.robot_y = self.start_y;
        self.painted = self.initial_painted.clone();
        self.trace = vec![(self.robot_x, self.robot_y)];
        self.crashed = false;
        self.crash_message = None;
    }

    pub fn set_start_pos(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.start_x = x;
            self.start_y = y;
            self.reset_execution();
        }
    }

    pub fn has_wall(&self, dir: &Direction) -> bool {
        match dir {
            Direction::Up => {
                if self.robot_y == 0 {
                    true
                } else {
                    self.h_walls[self.robot_y - 1][self.robot_x]
                }
            }
            Direction::Down => {
                if self.robot_y + 1 >= self.height {
                    true
                } else {
                    self.h_walls[self.robot_y][self.robot_x]
                }
            }
            Direction::Left => {
                if self.robot_x == 0 {
                    true
                } else {
                    self.v_walls[self.robot_y][self.robot_x - 1]
                }
            }
            Direction::Right => {
                if self.robot_x + 1 >= self.width {
                    true
                } else {
                    self.v_walls[self.robot_y][self.robot_x]
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
            let dir_name = match dir {
                Direction::Up => "вверх в стену",
                Direction::Down => "вниз в стену",
                Direction::Left => "влево в стену",
                Direction::Right => "вправо в стену",
            };
            let err = format!("Робот разбился: попытка движения {}! Позиция: ({}, {})", dir_name, self.robot_x + 1, self.robot_y + 1);
            self.crash_message = Some(err.clone());
            return Err(err);
        }

        match dir {
            Direction::Up => self.robot_y -= 1,
            Direction::Down => self.robot_y += 1,
            Direction::Left => self.robot_x -= 1,
            Direction::Right => self.robot_x += 1,
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
        self.painted[self.robot_y][self.robot_x]
    }

    pub fn toggle_h_wall(&mut self, row: usize, col: usize) {
        if row < self.h_walls.len() && col < self.width {
            self.h_walls[row][col] = !self.h_walls[row][col];
        }
    }

    pub fn toggle_v_wall(&mut self, row: usize, col: usize) {
        if row < self.height && col < self.v_walls[0].len() {
            self.v_walls[row][col] = !self.v_walls[row][col];
        }
    }

    pub fn toggle_paint_cell(&mut self, col: usize, row: usize) {
        if row < self.height && col < self.width {
            self.initial_painted[row][col] = !self.initial_painted[row][col];
            self.painted[row][col] = self.initial_painted[row][col];
        }
    }

    pub fn clear_walls(&mut self) {
        for row in self.h_walls.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
        for row in self.v_walls.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
    }

    pub fn clear_painted(&mut self) {
        for row in self.initial_painted.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
        self.painted = self.initial_painted.clone();
    }

    // Presets
    pub fn preset_oge_corridor() -> Self {
        let mut field = Self::new(12, 8);
        field.set_start_pos(1, 3);
        // Horizontal wall below corridor: from col 1 to 9 on row 3 (so bottom of row 3 is blocked)
        for col in 1..9 {
            field.h_walls[3][col] = true;
        }
        // Vertical wall at the end of corridor (col 9, row 3)
        field.v_walls[3][8] = true;
        field
    }

    pub fn preset_stairs() -> Self {
        let mut field = Self::new(12, 10);
        field.set_start_pos(1, 1);
        // Stair walls
        field.v_walls[1][2] = true;
        field.h_walls[1][2] = true;

        field.v_walls[2][4] = true;
        field.h_walls[2][4] = true;

        field.v_walls[3][6] = true;
        field.h_walls[3][6] = true;

        field.v_walls[4][8] = true;
        field.h_walls[4][8] = true;
        field
    }

    pub fn preset_maze() -> Self {
        let mut field = Self::new(10, 8);
        field.set_start_pos(0, 0);

        // row 0
        field.v_walls[0][1] = true;
        field.v_walls[0][4] = true;
        field.h_walls[0][2] = true;
        field.h_walls[0][3] = true;

        // row 1
        field.h_walls[1][0] = true;
        field.h_walls[1][1] = true;
        field.v_walls[1][3] = true;
        field.v_walls[1][6] = true;

        // row 2
        field.h_walls[2][4] = true;
        field.h_walls[2][5] = true;
        field.h_walls[2][6] = true;

        // row 3
        field.v_walls[3][1] = true;
        field.v_walls[3][2] = true;
        field.v_walls[3][7] = true;

        // row 4
        field.h_walls[4][1] = true;
        field.h_walls[4][2] = true;
        field.h_walls[4][3] = true;
        field.v_walls[4][5] = true;

        field
    }
}
