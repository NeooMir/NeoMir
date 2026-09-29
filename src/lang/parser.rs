use crate::lang::ast::*;
use crate::lang::token::{Token, TokenKind};
use crate::robot::Direction;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn match_kind(&mut self, kind: &TokenKind) -> bool {
        if self.peek_kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: TokenKind, msg: &str) -> Result<Token, String> {
        let tok = self.peek().clone();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(tok)
        } else {
            Err(format!("Ошибка на строке {}: {}, получено {:?}", tok.line, msg, tok.kind))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut uses_robot = false;
        let mut main_body = Vec::new();
        let mut subroutines = Vec::new();

        // Optional semicolons
        while self.match_kind(&TokenKind::Semicolon) {}

        // Check for 'исп Робот'
        while self.peek_kind() == &TokenKind::Isp {
            self.advance();
            if let TokenKind::Ident(name) = self.peek_kind() {
                if name.to_lowercase() == "робот" {
                    uses_robot = true;
                }
                self.advance();
            }
            while self.match_kind(&TokenKind::Semicolon) {}
        }

        // If top-level starts with 'алг'
        let mut found_alg = false;
        while self.peek_kind() == &TokenKind::Alg {
            found_alg = true;
            let alg_tok = self.advance();
            let mut alg_name = String::new();
            if let TokenKind::Ident(name) = self.peek_kind() {
                alg_name = name.clone();
                self.advance();
            }

            // Skip annotations like 'дано', 'надо' until 'нач'
            while self.peek_kind() != &TokenKind::Nach && self.peek_kind() != &TokenKind::Eof {
                self.advance();
            }

            self.expect(TokenKind::Nach, "Ожидалось 'нач'")?;
            let body = self.parse_stmt_list(&[TokenKind::Kon])?;
            self.expect(TokenKind::Kon, "Ожидалось 'кон'")?;

            if alg_name.is_empty() && main_body.is_empty() {
                main_body = body;
            } else {
                subroutines.push(Subroutine {
                    name: alg_name,
                    body,
                    line: alg_tok.line,
                });
            }

            while self.match_kind(&TokenKind::Semicolon) {}
        }

        if !found_alg {
            // Check if there is a 'нач' ... 'кон' block without 'алг'
            if self.peek_kind() == &TokenKind::Nach {
                self.advance();
                main_body = self.parse_stmt_list(&[TokenKind::Kon])?;
                self.expect(TokenKind::Kon, "Ожидалось 'кон'")?;
            } else {
                // Freeform statements until EOF
                main_body = self.parse_stmt_list(&[TokenKind::Eof])?;
            }
        } else if main_body.is_empty() && !subroutines.is_empty() {
            // If the first subroutine had a name, but no explicit unnamed main,
            // treat the first one as main
            let first = subroutines.remove(0);
            main_body = first.body;
        }

        Ok(Program {
            uses_robot,
            main_body,
            subroutines,
        })
    }

    fn parse_stmt_list(&mut self, stop_kinds: &[TokenKind]) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();

        while !stop_kinds.iter().any(|k| std::mem::discriminant(k) == std::mem::discriminant(self.peek_kind())) {
            while self.match_kind(&TokenKind::Semicolon) {}
            if stop_kinds.iter().any(|k| std::mem::discriminant(k) == std::mem::discriminant(self.peek_kind())) {
                break;
            }
            if self.peek_kind() == &TokenKind::Eof {
                break;
            }

            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            while self.match_kind(&TokenKind::Semicolon) {}
        }

        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Vlevo => {
                self.advance();
                Ok(Stmt::RobotCmd { action: RobotAction::Vlevo, line: tok.line })
            }
            TokenKind::Vpravo => {
                self.advance();
                Ok(Stmt::RobotCmd { action: RobotAction::Vpravo, line: tok.line })
            }
            TokenKind::Vverh => {
                self.advance();
                Ok(Stmt::RobotCmd { action: RobotAction::Vverh, line: tok.line })
            }
            TokenKind::Vniz => {
                self.advance();
                Ok(Stmt::RobotCmd { action: RobotAction::Vniz, line: tok.line })
            }
            TokenKind::Zakrasit => {
                self.advance();
                Ok(Stmt::RobotCmd { action: RobotAction::Zakrasit, line: tok.line })
            }
            TokenKind::Cel | TokenKind::Veshch | TokenKind::Log | TokenKind::Sim | TokenKind::Lit => {
                self.parse_var_decl()
            }
            TokenKind::Esli => {
                self.parse_if()
            }
            TokenKind::Nc => {
                self.parse_loop()
            }
            TokenKind::Vyvod => {
                self.parse_print()
            }
            TokenKind::Ident(ref name) => {
                let name = name.clone();
                // Check if next is ':='
                if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].kind == TokenKind::Assign {
                    self.advance(); // consume ident
                    self.advance(); // consume :=
                    let expr = self.parse_expr()?;
                    Ok(Stmt::Assign { var: name, expr, line: tok.line })
                } else {
                    // Call subroutine
                    self.advance();
                    Ok(Stmt::Call { name, line: tok.line })
                }
            }
            _ => {
                Err(format!("Неожиданная инструкция на строке {}: {:?}", tok.line, tok.kind))
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, String> {
        let tok = self.advance();
        let var_type = match tok.kind {
            TokenKind::Cel => VarType::Cel,
            TokenKind::Veshch => VarType::Veshch,
            TokenKind::Log => VarType::Log,
            TokenKind::Sim => VarType::Sim,
            TokenKind::Lit => VarType::Lit,
            _ => unreachable!(),
        };

        let mut names = Vec::new();
        loop {
            let name_tok = self.expect(TokenKind::Ident(String::new()), "Ожидалось имя переменной")?;
            if let TokenKind::Ident(name) = name_tok.kind {
                names.push(name);
            }
            if self.match_kind(&TokenKind::Comma) {
                continue;
            } else {
                break;
            }
        }

        Ok(Stmt::VarDecl { names, var_type, line: tok.line })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        let if_tok = self.advance(); // consume 'если'
        let cond = self.parse_expr()?;
        self.expect(TokenKind::To, "Ожидалось 'то' после условия")?;

        let then_branch = self.parse_stmt_list(&[TokenKind::Inache, TokenKind::Vse])?;
        let else_branch = if self.match_kind(&TokenKind::Inache) {
            let branch = self.parse_stmt_list(&[TokenKind::Vse])?;
            Some(branch)
        } else {
            None
        };

        self.expect(TokenKind::Vse, "Ожидалось 'все' в конце условия 'если'")?;
        Ok(Stmt::If { cond, then_branch, else_branch, line: if_tok.line })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        let nc_tok = self.advance(); // consume 'нц'

        match self.peek_kind() {
            TokenKind::Poka => {
                self.advance(); // consume 'пока'
                let cond = self.parse_expr()?;
                let body = self.parse_stmt_list(&[TokenKind::Kc])?;
                self.expect(TokenKind::Kc, "Ожидалось 'кц' в конце цикла")?;
                Ok(Stmt::While { cond, body, line: nc_tok.line })
            }
            TokenKind::Dlya => {
                self.advance(); // consume 'для'
                let var_tok = self.expect(TokenKind::Ident(String::new()), "Ожидалась переменная цикла")?;
                let var_name = match var_tok.kind {
                    TokenKind::Ident(n) => n,
                    _ => unreachable!(),
                };
                self.expect(TokenKind::Ot, "Ожидалось 'от'")?;
                let from = self.parse_expr()?;
                self.expect(TokenKind::Do, "Ожидалось 'до'")?;
                let to = self.parse_expr()?;

                let step = if self.match_kind(&TokenKind::Shag) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                let body = self.parse_stmt_list(&[TokenKind::Kc])?;
                self.expect(TokenKind::Kc, "Ожидалось 'кц' в конце цикла")?;
                Ok(Stmt::For { var: var_name, from, to, step, body, line: nc_tok.line })
            }
            _ => {
                // Could be 'нц <expr> раз'
                let times_expr = self.parse_expr()?;
                self.expect(TokenKind::Raz, "Ожидалось 'раз' или 'пока'/'для' после 'нц'")?;
                let body = self.parse_stmt_list(&[TokenKind::Kc])?;
                self.expect(TokenKind::Kc, "Ожидалось 'кц' в конце цикла")?;
                Ok(Stmt::RepeatTimes { times: times_expr, body, line: nc_tok.line })
            }
        }
    }

    fn parse_print(&mut self) -> Result<Stmt, String> {
        let tok = self.advance(); // consume 'вывод'
        let mut items = Vec::new();

        loop {
            if self.match_kind(&TokenKind::Ns) {
                items.push(PrintItem::Newline);
            } else {
                let expr = self.parse_expr()?;
                items.push(PrintItem::Expr(expr));
            }

            if self.match_kind(&TokenKind::Comma) {
                continue;
            } else {
                break;
            }
        }

        Ok(Stmt::Print { items, line: tok.line })
    }

    // Expressions
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;

        while self.match_kind(&TokenKind::Ili) {
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;

        while self.match_kind(&TokenKind::I) {
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = if self.match_kind(&TokenKind::Equal) {
                BinOp::Equal
            } else if self.match_kind(&TokenKind::NotEqual) {
                BinOp::NotEqual
            } else {
                break;
            };

            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;

        loop {
            let op = if self.match_kind(&TokenKind::Less) {
                BinOp::Less
            } else if self.match_kind(&TokenKind::Greater) {
                BinOp::Greater
            } else if self.match_kind(&TokenKind::LessEq) {
                BinOp::LessEq
            } else if self.match_kind(&TokenKind::GreaterEq) {
                BinOp::GreaterEq
            } else {
                break;
            };

            let right = self.parse_term()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;

        loop {
            let op = if self.match_kind(&TokenKind::Plus) {
                BinOp::Add
            } else if self.match_kind(&TokenKind::Minus) {
                BinOp::Sub
            } else {
                break;
            };

            let right = self.parse_factor()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = if self.match_kind(&TokenKind::Star) {
                BinOp::Mul
            } else if self.match_kind(&TokenKind::Slash) {
                BinOp::Div
            } else if self.match_kind(&TokenKind::Div) {
                BinOp::IntDiv
            } else if self.match_kind(&TokenKind::Mod) {
                BinOp::Mod
            } else {
                break;
            };

            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.match_kind(&TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(expr),
            });
        }
        if self.match_kind(&TokenKind::Ne) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.peek().clone();

        match tok.kind {
            TokenKind::IntNumber(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            TokenKind::FloatNumber(f) => {
                self.advance();
                Ok(Expr::Float(f))
            }
            TokenKind::StringLit(s) => {
                self.advance();
                Ok(Expr::String(s))
            }
            TokenKind::Da => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::Net => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen, "Ожидалась закрывающая скобка ')'")?;
                Ok(expr)
            }
            // Robot sensor condition patterns:
            // слева/справа/сверху/снизу свободно/стена
            TokenKind::Sleva | TokenKind::Sprava | TokenKind::Sverhu | TokenKind::Snizu => {
                self.advance();
                let dir = match tok.kind {
                    TokenKind::Sleva => Direction::Left,
                    TokenKind::Sprava => Direction::Right,
                    TokenKind::Sverhu => Direction::Up,
                    TokenKind::Snizu => Direction::Down,
                    _ => unreachable!(),
                };

                let next = self.peek().clone();
                match next.kind {
                    TokenKind::Svobodno => {
                        self.advance();
                        Ok(Expr::RobotWallCheck { dir, expect_wall: false })
                    }
                    TokenKind::Stena => {
                        self.advance();
                        Ok(Expr::RobotWallCheck { dir, expect_wall: true })
                    }
                    _ => Err(format!(
                        "Ошибка на строке {}: после направления ожидалось 'свободно' или 'стена'",
                        tok.line
                    )),
                }
            }
            // клетка закрашена / клетка чистая
            TokenKind::Kletka => {
                self.advance();
                let next = self.peek().clone();
                match next.kind {
                    TokenKind::Zakrashena => {
                        self.advance();
                        Ok(Expr::RobotPaintedCheck { expect_painted: true })
                    }
                    TokenKind::Chistaya => {
                        self.advance();
                        Ok(Expr::RobotPaintedCheck { expect_painted: false })
                    }
                    _ => Err(format!(
                        "Ошибка на строке {}: после 'клетка' ожидалось 'закрашена' или 'чистая'",
                        tok.line
                    )),
                }
            }
            TokenKind::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Var(name))
            }
            _ => Err(format!("Неожиданное выражение на строке {}: {:?}", tok.line, tok.kind)),
        }
    }
}
