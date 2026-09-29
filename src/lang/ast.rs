use crate::robot::Direction;

#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    Cel,
    Veshch,
    Log,
    Sim,
    Lit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RobotAction {
    Vlevo,
    Vpravo,
    Vverh,
    Vniz,
    Zakrasit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintItem {
    Expr(Expr),
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Var(String),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    // Robot sensors
    RobotWallCheck {
        dir: Direction,
        expect_wall: bool,
    },
    RobotPaintedCheck {
        expect_painted: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    RobotCmd {
        action: RobotAction,
        line: usize,
    },
    Assign {
        var: String,
        expr: Expr,
        line: usize,
    },
    VarDecl {
        names: Vec<String>,
        var_type: VarType,
        line: usize,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        line: usize,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    RepeatTimes {
        times: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    For {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
        line: usize,
    },
    Print {
        items: Vec<PrintItem>,
        line: usize,
    },
    Call {
        name: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Subroutine {
    pub name: String,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub uses_robot: bool,
    pub main_body: Vec<Stmt>,
    pub subroutines: Vec<Subroutine>,
}
