use std::collections::HashMap;

use crate::ast::ast::{
    BinaryOperator, CompareOperator, Expr, Literal, LogicalOperator, MathOperator, Stmt,
};
use crate::lexer::tokens::Token;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        let stmt = match self.peek()? {
            Token::Let => self.parse_var_decl(),
            Token::Fn => self.parse_func_decl(),
            Token::Return => self.parse_return_stmt(),
            Token::For => self.parse_for_stmt(), // 👈 Adiciona isso
            Token::If => self.parse_if_stmt(),
            Token::BraceOpen => Some(Stmt::ExprStmt(self.parse_brace()?)),
            _ => Some(Stmt::ExprStmt(self.parse_expr()?)),
        };
        // Se houver um ponto e vírgula depois do statement, consome
        self.expect(&Token::Semicolon);

        stmt
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "if"
        self.expect(&Token::ParenOpen);
        let condition = self.parse_expr()?;
        self.expect(&Token::ParenClose);

        let then_branch = self.parse_block();

        let mut else_ifs = vec![];

        while self.peek() == Some(&Token::ElseIf) {
            self.next(); // consume "else if"
            let condition = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let then_branch = self.parse_block();
            else_ifs.push((condition, Some(then_branch)));
        }

        let mut else_branch = None;
        if self.peek() == Some(&Token::Else) {
            self.next(); // consume "else"
            else_branch = Some(self.parse_block());
        }

        Some(Stmt::If {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        })
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume 'for'
        self.expect(&Token::ParenOpen);

        let init = self.parse_stmt()?;

        let condition = if self.peek() == Some(&Token::Semicolon) {
            self.next(); // consume ';'
            let s = self.parse_expr();
            s
        } else {
            let cond = self.parse_expr()?;
            self.expect(&Token::Semicolon);
            Some(cond)
        };

        let update = if self.peek() == Some(&Token::Semicolon) {
            self.next(); // consume ';'
            let s = self.parse_expr();
            s
        } else {
            Some(self.parse_expr()?)
        };

        self.expect(&Token::ParenClose);

        // println!(
        //     "init: {:?}, condition: {:?}, update: {:?}",
        //     init, condition, update
        // );

        // println!("peek: {:?}", self.peek());
        let body = self.parse_block();

        Some(Stmt::For {
            init: Box::new(init),
            condition,
            update,
            body,
        })
    }

    fn parse_var_decl(&mut self) -> Option<Stmt> {
        self.next(); // consume "let"
        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };
        self.expect(&Token::Assign);
        let value = self.parse_expr()?;
        Some(Stmt::Let {
            name,
            value: Some(value),
        })
    }

    fn parse_func_decl(&mut self) -> Option<Stmt> {
        self.next(); // consume "fn"
        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        self.expect(&Token::ParenOpen);
        let mut params = vec![];
        while let Some(Token::Identifier(param)) = self.peek() {
            params.push(param.clone());
            self.next();
            if !self.expect(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::ParenClose);

        let body = self.parse_block();
        Some(Stmt::FuncDecl { name, params, body })
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "return"
        let value = if let Some(Token::BraceClose) = self.peek() {
            None
        } else {
            Some(self.parse_expr()?)
        };
        Some(Stmt::Return(value))
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Option<Expr> {
        let expr = self.parse_binary_expr(0)?;

        if let Some(Token::Assign) = self.peek() {
            self.next(); // consume '='
            if let Expr::Identifier(name) = expr {
                let value = self.parse_assignment_expr()?;
                return Some(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else {
                // só pode atribuir a um identificador
                panic!("Invalid assignment target");
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.next()? {
            Token::Number(n) => Some(Expr::Literal(Literal::Number(n))),
            Token::String(s) => Some(Expr::Literal(Literal::String(s))),
            Token::Bool(b) => Some(Expr::Literal(Literal::Bool(b))),
            Token::Null => Some(Expr::Literal(Literal::Null)),
            Token::Identifier(name) => {
                if let Some(Token::ParenOpen) = self.peek() {
                    self.next(); // consume "("
                    let mut args = vec![];
                    while self.peek() != Some(&Token::ParenClose) {
                        args.push(self.parse_expr()?);
                        if !self.expect(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::ParenClose);
                    Some(Expr::Call {
                        callee: Box::new(Expr::Identifier(name)),
                        args,
                    })
                } else {
                    Some(Expr::Identifier(name))
                }
            }
            Token::ParenOpen => {
                let expr = self.parse_expr()?;
                self.expect(&Token::ParenClose);
                Some(expr)
            }
            Token::BraceOpen => self.parse_brace(),
            Token::BracketOpen => Some(self.parse_bracket()?),
            _ => None,
        }
    }

    fn parse_postfix_expr(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.next(); // consume '.'
                    let property = match self.next()? {
                        Token::Identifier(name) => Expr::Identifier(name),
                        _ => return None,
                    };
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        property: Box::new(property),
                    };
                }
                Some(Token::BracketOpen) => {
                    self.next(); // consume '['
                    let property = self.parse_expr()?;
                    self.expect(&Token::BracketClose);
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        property: Box::new(property),
                    };
                }
                Some(Token::ParenOpen) => {
                    self.next(); // consume '('
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::ParenClose) {
                        args.push(self.parse_expr()?);
                        if !self.expect(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::ParenClose);
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }

        Some(expr)
    }

    fn parse_brace(&mut self) -> Option<Expr> {
        if self.is_next_object() {
            self.parse_object_literal()
        } else {
            Some(Expr::Block(self.parse_block()))
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(&Token::BraceOpen);
        let mut stmts = Vec::new();
        while let Some(tok) = self.peek() {
            if let Token::BraceClose = tok {
                break;
            }
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
        }
        self.expect(&Token::BraceClose);
        stmts
    }

    fn parse_bracket(&mut self) -> Option<Expr> {
        self.expect(&Token::BracketOpen);
        let mut elements = Vec::new();
        while self.peek() != Some(&Token::BracketClose) {
            elements.push(self.parse_expr()?);
            if !self.expect(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::BracketClose);
        Some(Expr::Literal(Literal::Array(elements)))
    }

    fn is_next_object(&self) -> bool {
        let mut i = self.pos;
        match self.tokens.get(i) {
            Some(Token::Identifier(_)) => {
                i += 1;
                let colon = self.tokens.get(i);
                matches!(colon, Some(Token::Colon))
            }
            _ => false,
        }
    }

    fn parse_object_literal(&mut self) -> Option<Expr> {
        self.expect(&Token::BraceOpen);
        let mut properties = HashMap::default();
        while self.peek() != Some(&Token::BraceClose) {
            let key = match self.next()? {
                Token::Identifier(key) => key,
                _ => return None,
            };
            self.expect(&Token::Colon);
            let value = self.parse_expr()?;
            properties.insert(key, value);
            if !self.expect(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::BraceClose);
        Some(Expr::Literal(Literal::Object(properties)))
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Option<Expr> {
        let mut left: Expr = self.parse_postfix_expr()?;

        while let Some(op) = self.peek().and_then(get_bin_op) {
            let prec = get_precedence(&op);
            if prec < min_prec {
                break;
            }

            self.next(); // consume operator
            let right = self.parse_binary_expr(prec + 1)?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(tok)
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if let Some(tok) = self.peek() {
            if tok == expected {
                self.pos += 1;
                return true;
            }
        }
        false
    }
}

// === Helpers ===

fn get_bin_op(token: &Token) -> Option<BinaryOperator> {
    match token {
        Token::Plus => Some(BinaryOperator::Math(MathOperator::Add)),
        Token::Minus => Some(BinaryOperator::Math(MathOperator::Sub)),
        Token::Star => Some(BinaryOperator::Math(MathOperator::Mul)),
        Token::Slash => Some(BinaryOperator::Math(MathOperator::Div)),

        Token::Equal => Some(BinaryOperator::Compare(CompareOperator::Eq)),
        Token::NotEqual => Some(BinaryOperator::Compare(CompareOperator::Ne)),
        Token::Less => Some(BinaryOperator::Compare(CompareOperator::Lt)),
        Token::Greater => Some(BinaryOperator::Compare(CompareOperator::Gt)),
        Token::LessEqual => Some(BinaryOperator::Compare(CompareOperator::Le)),
        Token::GreaterEqual => Some(BinaryOperator::Compare(CompareOperator::Ge)),

        Token::And => Some(BinaryOperator::Logical(LogicalOperator::And)),
        Token::Or => Some(BinaryOperator::Logical(LogicalOperator::Or)),

        _ => None,
    }
}

fn get_precedence(op: &BinaryOperator) -> u8 {
    match op {
        BinaryOperator::Logical(LogicalOperator::Or) => 1,
        BinaryOperator::Logical(LogicalOperator::And) => 2,
        BinaryOperator::Compare(_) => 3,
        BinaryOperator::Math(MathOperator::Add) | BinaryOperator::Math(MathOperator::Sub) => 4,
        BinaryOperator::Math(MathOperator::Mul) | BinaryOperator::Math(MathOperator::Div) => 5,
    }
}
