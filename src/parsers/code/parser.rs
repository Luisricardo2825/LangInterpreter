use std::collections::HashMap;

use crate::ast::ast::{
    BinaryOperator, CompareOperator, Expr, FunctionStmt, Literal, LogicalOperator, MathOperator,
    Method, Stmt,
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
            Token::Break => self.parse_break_stmt(),
            Token::Continue => self.parse_continue_stmt(),
            Token::For => self.parse_for_stmt(), // 👈 Adiciona isso
            Token::If => self.parse_if_stmt(),
            Token::BraceOpen => Some(Stmt::ExprStmt(self.parse_brace()?)),
            Token::Class => self.parse_class_decl(),
            _ => Some(Stmt::ExprStmt(self.parse_expr()?)),
        };
        // Se houver um ponto e vírgula depois do statement, consome
        self.expect(&Token::Semicolon);

        stmt
    }

    fn parse_class_decl(&mut self) -> Option<Stmt> {
        self.next(); // consume 'class'

        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        // Suporte a herança: class Nome extends SuperClasse
        let superclass = if self.consume(&Token::Extends) {
            Some(self.parse_primary()?)
        } else {
            None
        };

        self.consume(&Token::BraceOpen);

        let mut methods = vec![];
        let mut static_fields = HashMap::new();
        let mut instance_fields = HashMap::new();

        while self.peek() != Some(&Token::BraceClose) {
            if self.check_identifier() && self.peek_next() == Some(&Token::ParenOpen) {
                let method = self.parse_method(false)?;
                methods.push(method);
            } else if self.expect(&Token::Static) {
                let prev = self.peek();
                let next = self.peek_next();

                match (prev, next) {
                    (Some(Token::Identifier(_)), Some(Token::ParenOpen)) => {
                        let method = self.parse_method(true)?;
                        methods.push(method);
                    }
                    (Some(Token::Identifier(_)), Some(Token::Assign)) => {
                        let (name, expr) = self.parse_field()?;
                        static_fields.insert(name, expr);
                    }
                    _ => {
                        return None;
                    }
                }
            } else if self.check_identifier() {
                let (name, expr) = self.parse_field()?;
                instance_fields.insert(name, expr);
            } else {
                return None; // erro de sintaxe
            }
        }

        self.expect(&Token::BraceClose);

        Some(Stmt::ClassDecl {
            name,
            superclass,
            methods,
            static_fields,
            instance_fields,
        })
    }

    fn parse_field(&mut self) -> Option<(String, Expr)> {
        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => {
                return None;
            }
        };
        if self.check(&Token::Assign) {
            self.consume(&Token::Assign);
            let expr = self.parse_expr()?;
            self.consume(&Token::Semicolon);

            return Some((name, expr));
        }
        let expr = Expr::Literal(Literal::Null);
        self.consume(&Token::Semicolon);
        Some((name, expr))
    }

    fn check_identifier(&self) -> bool {
        matches!(self.peek(), Some(Token::Identifier(_)))
    }
    fn parse_method(&mut self, is_static: bool) -> Option<Method> {
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

        Some(Method {
            name,
            params,
            body,
            is_static,
        })
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "if"
        self.expect(&Token::ParenOpen);
        let condition = self.parse_expr()?;
        self.expect(&Token::ParenClose);

        let then_branch = self.parse_block();

        let mut else_ifs = vec![];

        while self.peek() == Some(&Token::Else) && self.peek_next() == Some(&Token::If) {
            self.next(); // consume "else"
            self.next(); // consume "if"
            self.expect(&Token::ParenOpen);
            let condition = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let then_branch = self.parse_block();
            else_ifs.push((condition, Some(then_branch)));
        }

        let mut else_branch = None;
        let peek = self.peek();
        if peek == Some(&Token::Else) {
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

        let is_let = self.consume(&Token::Let);
        let pattern = if is_let {
            self.parse_primary()? // novo método para suportar destructuring
        } else {
            self.parse_expr()? // para casos como `for (item of list)`
        };

        if self.consume(&Token::Of) {
            let iterable = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let body = self.parse_block();
            return Some(Stmt::ForOf {
                target: pattern,
                iterable,
                body: body,
            });
        }

        // fallback para for tradicional
        let init = if is_let {
            Stmt::Let {
                name: self.extract_identifier(&pattern)?,
                value: Some(self.parse_expr()?),
            }
        } else {
            self.parse_stmt()?
        };

        self.expect(&Token::Semicolon);
        let condition = self.parse_expr();
        self.expect(&Token::Semicolon);
        let update = self.parse_expr();
        self.expect(&Token::ParenClose);
        let body = self.parse_block();

        Some(Stmt::For {
            init: Box::new(init),
            condition,
            update,
            body,
        })
    }

    fn extract_identifier(&mut self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => Some(name.to_string()),
            _ => None,
        }
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
        Some(Stmt::FuncDecl(FunctionStmt { name, params, body }))
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

    fn parse_break_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "break"
        Some(Stmt::Break)
    }

    fn parse_continue_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "continue"
        Some(Stmt::Continue)
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Option<Expr> {
        let expr = self.parse_binary_expr(0)?;

        if let Some(Token::Assign) = self.peek() {
            self.next(); // consume '='

            let expr = expr;

            match expr {
                Expr::Identifier(name) => {
                    let value = self.parse_assignment_expr()?;
                    return Some(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                Expr::GetProperty { object, property } => {
                    let value = self.parse_assignment_expr()?;
                    return Some(Expr::SetProperty {
                        object,
                        property,
                        value: Box::new(value),
                    });
                }
                Expr::BracketAccess { object, property } => {
                    let value = self.parse_assignment_expr()?;
                    return Some(Expr::SetProperty {
                        object,
                        property,
                        value: Box::new(value),
                    });
                }
                _ => {
                    // só pode atribuir a um identificador
                    panic!("Invalid assignment target");
                }
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.next()? {
            Token::This => Some(Expr::This),
            Token::New => {
                let constructor = self.parse_primary()?;
                let mut args = vec![];

                if self.expect(&Token::ParenOpen) {
                    while self.peek() != Some(&Token::ParenClose) {
                        args.push(self.parse_expr()?);
                        if !self.expect(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::ParenClose);
                }

                Some(Expr::New {
                    constructor: Box::new(constructor),
                    args,
                })
            }
            Token::Number(n) => Some(Expr::Literal(Literal::Number(n))),
            Token::String(s) => Some(Expr::Literal(Literal::String(s))),
            Token::Bool(b) => Some(Expr::Literal(Literal::Bool(b))),
            Token::Null => Some(Expr::Literal(Literal::Null)),
            Token::Identifier(name) => {
                if name == "else" {
                    println!("Teste")
                }
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
            Token::BracketOpen => self.parse_bracket(),
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
                    expr = Expr::GetProperty {
                        object: Box::new(expr),
                        property: Box::new(property),
                    };
                }
                Some(Token::BracketOpen) => {
                    self.next(); // consume '['
                    let property = self.parse_expr()?;
                    self.expect(&Token::BracketClose);
                    expr = Expr::BracketAccess {
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
        let mut elements = Vec::new();
        while let Some(tok) = self.peek() {
            if tok == &Token::BracketClose {
                break;
            }
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

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(tok)
    }

    fn consume(&mut self, expected: &Token) -> bool {
        if self.peek() == Some(expected) {
            self.next(); // avança o cursor
            true
        } else {
            false
        }
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
    fn check(&self, expected: &Token) -> bool {
        if let Some(tok) = self.peek() {
            if tok == expected {
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
