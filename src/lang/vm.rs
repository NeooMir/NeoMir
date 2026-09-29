use crate::lang::ast::*;
use crate::robot::{Direction, RobotField};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", if *b { "да" } else { "нет" }),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Line(usize),
    RobotCmd(RobotAction, usize),
    Assign(String, Expr, usize),
    Print(Vec<PrintItem>, usize),
    Jump(usize),
    JumpIfFalse(Expr, usize), // (condition, jump_target)
    ForInit {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        jump_if_empty: usize,
    },
    ForNext {
        var: String,
        to: Expr,
        step: Option<Expr>,
        loop_top: usize,
    },
    RepeatInit {
        counter_idx: usize,
        times: Expr,
        jump_if_zero: usize,
    },
    RepeatNext {
        counter_idx: usize,
        loop_top: usize,
    },
    Call(usize),
    Return,
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Stepped { line: usize },
    RobotMoved { line: usize },
    Output(String),
    Finished,
    Error { line: usize, message: String },
}

pub struct VirtualMachine {
    pub instructions: Vec<Instruction>,
    pub pc: usize,
    pub current_line: usize,
    pub variables: HashMap<String, Value>,
    pub call_stack: Vec<usize>,
    pub repeat_counters: Vec<i64>,
    pub halted: bool,
}

impl VirtualMachine {
    pub fn compile(program: &Program) -> Self {
        let mut compiler = Compiler::new();
        compiler.compile_program(program);

        Self {
            instructions: compiler.instructions,
            pc: 0,
            current_line: 1,
            variables: HashMap::new(),
            call_stack: Vec::new(),
            repeat_counters: Vec::new(),
            halted: false,
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.current_line = 1;
        self.variables.clear();
        self.call_stack.clear();
        self.repeat_counters.clear();
        self.halted = false;
    }

    pub fn step(&mut self, robot: &mut RobotField) -> StepResult {
        if self.halted {
            return StepResult::Finished;
        }

        let mut ops_count = 0;
        let start_line = self.current_line;

        loop {
            ops_count += 1;
            if ops_count > 100_000 {
                return StepResult::Error {
                    line: self.current_line,
                    message: "Превышен лимит инструкций (возможно, зацикливание)".to_string(),
                };
            }

            if self.pc >= self.instructions.len() {
                self.halted = true;
                return StepResult::Finished;
            }

            let instr = self.instructions[self.pc].clone();
            match instr {
                Instruction::Line(l) => {
                    self.current_line = l;
                    self.pc += 1;
                    if self.current_line != start_line && ops_count > 1 {
                        return StepResult::Stepped { line: self.current_line };
                    }
                }
                Instruction::RobotCmd(action, line) => {
                    self.current_line = line;
                    self.pc += 1;
                    match action {
                        RobotAction::Vlevo => {
                            if let Err(e) = robot.move_robot(&Direction::Left) {
                                self.halted = true;
                                return StepResult::Error { line, message: e };
                            }
                        }
                        RobotAction::Vpravo => {
                            if let Err(e) = robot.move_robot(&Direction::Right) {
                                self.halted = true;
                                return StepResult::Error { line, message: e };
                            }
                        }
                        RobotAction::Vverh => {
                            if let Err(e) = robot.move_robot(&Direction::Up) {
                                self.halted = true;
                                return StepResult::Error { line, message: e };
                            }
                        }
                        RobotAction::Vniz => {
                            if let Err(e) = robot.move_robot(&Direction::Down) {
                                self.halted = true;
                                return StepResult::Error { line, message: e };
                            }
                        }
                        RobotAction::Zakrasit => {
                            robot.paint_cell();
                        }
                    }
                    return StepResult::RobotMoved { line };
                }
                Instruction::Assign(var, expr, line) => {
                    self.current_line = line;
                    self.pc += 1;
                    match self.eval_expr(&expr, robot) {
                        Ok(val) => {
                            self.variables.insert(var, val);
                        }
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error { line, message: e };
                        }
                    }
                }
                Instruction::Print(items, line) => {
                    self.current_line = line;
                    self.pc += 1;
                    let mut out = String::new();
                    for item in items {
                        match item {
                            PrintItem::Newline => out.push('\n'),
                            PrintItem::Expr(expr) => match self.eval_expr(&expr, robot) {
                                Ok(val) => out.push_str(&format!("{} ", val)),
                                Err(e) => {
                                    self.halted = true;
                                    return StepResult::Error { line, message: e };
                                }
                            },
                        }
                    }
                    return StepResult::Output(out);
                }
                Instruction::Jump(target) => {
                    self.pc = target;
                }
                Instruction::JumpIfFalse(cond, target) => {
                    match self.eval_expr(&cond, robot) {
                        Ok(val) => {
                            if !val.is_truthy() {
                                self.pc = target;
                            } else {
                                self.pc += 1;
                            }
                        }
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error {
                                line: self.current_line,
                                message: e,
                            };
                        }
                    }
                }
                Instruction::ForInit { var, from, to, step, jump_if_empty } => {
                    self.pc += 1;
                    let from_val = match self.eval_expr(&from, robot).and_then(|v| v.as_int()) {
                        Ok(v) => v,
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error { line: self.current_line, message: e };
                        }
                    };
                    let to_val = match self.eval_expr(&to, robot).and_then(|v| v.as_int()) {
                        Ok(v) => v,
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error { line: self.current_line, message: e };
                        }
                    };
                    let step_val = if let Some(s) = step {
                        match self.eval_expr(&s, robot).and_then(|v| v.as_int()) {
                            Ok(v) => v,
                            Err(e) => {
                                self.halted = true;
                                return StepResult::Error { line: self.current_line, message: e };
                            }
                        }
                    } else {
                        1
                    };

                    self.variables.insert(var.clone(), Value::Int(from_val));
                    if (step_val > 0 && from_val > to_val) || (step_val < 0 && from_val < to_val) {
                        self.pc = jump_if_empty;
                    }
                }
                Instruction::ForNext { var, to, step, loop_top } => {
                    let to_val = match self.eval_expr(&to, robot).and_then(|v| v.as_int()) {
                        Ok(v) => v,
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error { line: self.current_line, message: e };
                        }
                    };
                    let step_val = if let Some(s) = step {
                        match self.eval_expr(&s, robot).and_then(|v| v.as_int()) {
                            Ok(v) => v,
                            Err(e) => {
                                self.halted = true;
                                return StepResult::Error { line: self.current_line, message: e };
                            }
                        }
                    } else {
                        1
                    };

                    let curr = self.variables.get(&var).and_then(|v| v.as_int().ok()).unwrap_or(0);
                    let next = curr + step_val;
                    self.variables.insert(var, Value::Int(next));

                    if (step_val > 0 && next <= to_val) || (step_val < 0 && next >= to_val) {
                        self.pc = loop_top;
                    } else {
                        self.pc += 1;
                    }
                }
                Instruction::RepeatInit { counter_idx, times, jump_if_zero } => {
                    self.pc += 1;
                    let count = match self.eval_expr(&times, robot).and_then(|v| v.as_int()) {
                        Ok(v) => v,
                        Err(e) => {
                            self.halted = true;
                            return StepResult::Error { line: self.current_line, message: e };
                        }
                    };
                    if counter_idx >= self.repeat_counters.len() {
                        self.repeat_counters.resize(counter_idx + 1, 0);
                    }
                    self.repeat_counters[counter_idx] = count;
                    if count <= 0 {
                        self.pc = jump_if_zero;
                    }
                }
                Instruction::RepeatNext { counter_idx, loop_top } => {
                    if counter_idx < self.repeat_counters.len() {
                        self.repeat_counters[counter_idx] -= 1;
                        if self.repeat_counters[counter_idx] > 0 {
                            self.pc = loop_top;
                            continue;
                        }
                    }
                    self.pc += 1;
                }
                Instruction::Call(target) => {
                    self.call_stack.push(self.pc + 1);
                    self.pc = target;
                }
                Instruction::Return => {
                    if let Some(ret_pc) = self.call_stack.pop() {
                        self.pc = ret_pc;
                    } else {
                        self.halted = true;
                        return StepResult::Finished;
                    }
                }
                Instruction::Halt => {
                    self.halted = true;
                    return StepResult::Finished;
                }
            }
        }
    }

    fn eval_expr(&self, expr: &Expr, robot: &RobotField) -> Result<Value, String> {
        match expr {
            Expr::Int(i) => Ok(Value::Int(*i)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Var(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Неопределенная переменная '{}'", name)),
            Expr::RobotWallCheck { dir, expect_wall } => {
                let wall = robot.has_wall(dir);
                Ok(Value::Bool(wall == *expect_wall))
            }
            Expr::RobotPaintedCheck { expect_painted } => {
                let painted = robot.is_painted();
                Ok(Value::Bool(painted == *expect_painted))
            }
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr, robot)?;
                match op {
                    UnOp::Neg => match val {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err("Унарный минус применим только к числам".to_string()),
                    },
                    UnOp::Not => match val {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err("Логическое 'не' применимо только к логическим значениям".to_string()),
                    },
                }
            }
            Expr::Binary { op, left, right } => {
                let left_val = self.eval_expr(left, robot)?;
                let right_val = self.eval_expr(right, robot)?;

                match op {
                    BinOp::Add => match (&left_val, &right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                        _ => Err("Несовместимые типы для сложения".to_string()),
                    },
                    BinOp::Sub => match (&left_val, &right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                        _ => Err("Несовместимые типы для вычитания".to_string()),
                    },
                    BinOp::Mul => match (&left_val, &right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                        _ => Err("Несовместимые типы для умножения".to_string()),
                    },
                    BinOp::Div => {
                        let a = left_val.as_float()?;
                        let b = right_val.as_float()?;
                        if b == 0.0 {
                            return Err("Деление на ноль".to_string());
                        }
                        Ok(Value::Float(a / b))
                    }
                    BinOp::IntDiv => {
                        let a = left_val.as_int()?;
                        let b = right_val.as_int()?;
                        if b == 0 {
                            return Err("Деление на ноль (div)".to_string());
                        }
                        Ok(Value::Int(a / b))
                    }
                    BinOp::Mod => {
                        let a = left_val.as_int()?;
                        let b = right_val.as_int()?;
                        if b == 0 {
                            return Err("Деление на ноль (mod)".to_string());
                        }
                        Ok(Value::Int(a % b))
                    }
                    BinOp::Equal => Ok(Value::Bool(left_val == right_val)),
                    BinOp::NotEqual => Ok(Value::Bool(left_val != right_val)),
                    BinOp::Less => Ok(Value::Bool(left_val.as_float()? < right_val.as_float()?)),
                    BinOp::Greater => Ok(Value::Bool(left_val.as_float()? > right_val.as_float()?)),
                    BinOp::LessEq => Ok(Value::Bool(left_val.as_float()? <= right_val.as_float()?)),
                    BinOp::GreaterEq => Ok(Value::Bool(left_val.as_float()? >= right_val.as_float()?)),
                    BinOp::And => Ok(Value::Bool(left_val.is_truthy() && right_val.is_truthy())),
                    BinOp::Or => Ok(Value::Bool(left_val.is_truthy() || right_val.is_truthy())),
                }
            }
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
        }
    }

    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            _ => Err("Ожидалось целое число".to_string()),
        }
    }

    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Int(i) => Ok(*i as f64),
            Value::Float(f) => Ok(*f),
            _ => Err("Ожидалось число".to_string()),
        }
    }
}

struct Compiler {
    instructions: Vec<Instruction>,
    subroutines: HashMap<String, usize>,
    pending_calls: Vec<(usize, String)>,
    repeat_counter_alloc: usize,
}

impl Compiler {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            subroutines: HashMap::new(),
            pending_calls: Vec::new(),
            repeat_counter_alloc: 0,
        }
    }

    fn compile_program(&mut self, program: &Program) {
        // Compile main body
        for stmt in &program.main_body {
            self.compile_stmt(stmt);
        }
        self.instructions.push(Instruction::Halt);

        // Compile subroutines
        for sub in &program.subroutines {
            let start_pc = self.instructions.len();
            self.subroutines.insert(sub.name.clone(), start_pc);
            self.instructions.push(Instruction::Line(sub.line));
            for stmt in &sub.body {
                self.compile_stmt(stmt);
            }
            self.instructions.push(Instruction::Return);
        }

        // Patch calls
        for (call_pc, name) in &self.pending_calls {
            if let Some(&target) = self.subroutines.get(name) {
                self.instructions[*call_pc] = Instruction::Call(target);
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::RobotCmd { action, line } => {
                self.instructions.push(Instruction::Line(*line));
                self.instructions.push(Instruction::RobotCmd(action.clone(), *line));
            }
            Stmt::Assign { var, expr, line } => {
                self.instructions.push(Instruction::Line(*line));
                self.instructions.push(Instruction::Assign(var.clone(), expr.clone(), *line));
            }
            Stmt::VarDecl { .. } => {
                // Declarations don't produce runtime instructions
            }
            Stmt::Print { items, line } => {
                self.instructions.push(Instruction::Line(*line));
                self.instructions.push(Instruction::Print(items.clone(), *line));
            }
            Stmt::Call { name, line } => {
                self.instructions.push(Instruction::Line(*line));
                let call_pc = self.instructions.len();
                self.instructions.push(Instruction::Call(0)); // patched later
                self.pending_calls.push((call_pc, name.clone()));
            }
            Stmt::If { cond, then_branch, else_branch, line } => {
                self.instructions.push(Instruction::Line(*line));
                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(cond.clone(), 0));

                for s in then_branch {
                    self.compile_stmt(s);
                }

                if let Some(else_stmts) = else_branch {
                    let jump_end_idx = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));

                    let else_start = self.instructions.len();
                    self.instructions[jump_false_idx] = Instruction::JumpIfFalse(cond.clone(), else_start);

                    for s in else_stmts {
                        self.compile_stmt(s);
                    }

                    let end_idx = self.instructions.len();
                    self.instructions[jump_end_idx] = Instruction::Jump(end_idx);
                } else {
                    let end_idx = self.instructions.len();
                    self.instructions[jump_false_idx] = Instruction::JumpIfFalse(cond.clone(), end_idx);
                }
            }
            Stmt::While { cond, body, line } => {
                let loop_top = self.instructions.len();
                self.instructions.push(Instruction::Line(*line));

                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(cond.clone(), 0));

                for s in body {
                    self.compile_stmt(s);
                }

                self.instructions.push(Instruction::Jump(loop_top));
                let loop_end = self.instructions.len();
                self.instructions[jump_false_idx] = Instruction::JumpIfFalse(cond.clone(), loop_end);
            }
            Stmt::RepeatTimes { times, body, line } => {
                self.instructions.push(Instruction::Line(*line));
                let counter_idx = self.repeat_counter_alloc;
                self.repeat_counter_alloc += 1;

                let init_idx = self.instructions.len();
                self.instructions.push(Instruction::RepeatInit {
                    counter_idx,
                    times: times.clone(),
                    jump_if_zero: 0,
                });

                let loop_top = self.instructions.len();
                for s in body {
                    self.compile_stmt(s);
                }

                self.instructions.push(Instruction::RepeatNext {
                    counter_idx,
                    loop_top,
                });

                let loop_end = self.instructions.len();
                self.instructions[init_idx] = Instruction::RepeatInit {
                    counter_idx,
                    times: times.clone(),
                    jump_if_zero: loop_end,
                };
            }
            Stmt::For { var, from, to, step, body, line } => {
                self.instructions.push(Instruction::Line(*line));
                let init_idx = self.instructions.len();
                self.instructions.push(Instruction::ForInit {
                    var: var.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    step: step.clone(),
                    jump_if_empty: 0,
                });

                let loop_top = self.instructions.len();
                for s in body {
                    self.compile_stmt(s);
                }

                self.instructions.push(Instruction::ForNext {
                    var: var.clone(),
                    to: to.clone(),
                    step: step.clone(),
                    loop_top,
                });

                let loop_end = self.instructions.len();
                self.instructions[init_idx] = Instruction::ForInit {
                    var: var.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    step: step.clone(),
                    jump_if_empty: loop_end,
                };
            }
        }
    }
}
