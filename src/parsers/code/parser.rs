use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::ast::{
    AssignOperator, BinaryOperator, CompareOperator, Expr, FunctionStmt, Literal, LogicalOperator,
    MethodDecl, ObjectEntry, Operator, Stmt, UnaryOperator,
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
            Token::Identifier(s) if s == "let" => self.parse_var_decl(),
            Token::Identifier(s) if ["fn", "function"].contains(&s.as_str()) => {
                self.parse_func_decl()
            }
            Token::Identifier(s) if s == "return" => self.parse_return_stmt(),
            Token::Identifier(s) if s == "import" => self.parse_import_stmt(),
            Token::Identifier(s) if s == "export" => self.parse_export_stmt(),
            Token::Identifier(s) if s == "break" => self.parse_break_stmt(),
            Token::Identifier(s) if s == "continue" => self.parse_continue_stmt(),
            Token::Identifier(s) if s == "for" => self.parse_for_stmt(), // 👈 Adiciona isso
            Token::Identifier(s) if s == "if" => self.parse_if_stmt(),
            Token::BraceOpen => Some(Stmt::ExprStmt(self.parse_brace()?)),
            Token::Identifier(s) if s == "class" => self.parse_class_decl(),
            _ => Some(Stmt::ExprStmt(self.parse_expr()?)),
        };
        // Se houver um ponto e vírgula depois do statement, consome
        self.expect(&Token::Semicolon);

        stmt
    }

    fn parse_export_stmt(&mut self) -> Option<Stmt> {
        if self.expect_keyword("export") {
            if self.expect_keyword("default") {
                let value = self.parse_stmt()?;
                self.expect(&Token::Semicolon);
                return Some(Stmt::ExportDefault(Rc::new(value)));
            }

            let inner = self.parse_stmt()?; // let, fn, etc.
            return Some(Stmt::Export(Rc::new(inner)));
        }

        panic!("Invalid export syntax")
    }
    fn parse_import_stmt(&mut self) -> Option<Stmt> {
        if !self.expect_keyword("import") {
            return None;
        }

        // Flags para identificar os casos
        let mut default_import: Option<String> = None;
        let mut named_imports: Vec<(String, String)> = vec![];

        // Primeira parte: pode ser identifier (default), '*' ou '{'
        match self.peek()? {
            Token::Identifier(_) => {
                if let Some(Token::Identifier(name)) = self.next() {
                    default_import = Some(name);

                    // Pode vir uma vírgula antes do named import
                    if self.expect(&Token::Comma) {
                        if self.expect(&Token::BraceOpen) {
                            while self.peek() != Some(&Token::BraceClose) {
                                let imported = match self.next()? {
                                    Token::Identifier(name) => name,
                                    _ => panic!("Expected identifier in import list"),
                                };

                                let local = if self.expect_keyword("as") {
                                    match self.next()? {
                                        Token::Identifier(name) => name,
                                        _ => panic!("Expected identifier after 'as'"),
                                    }
                                } else {
                                    imported.clone()
                                };

                                named_imports.push((imported, local));

                                if !self.expect(&Token::Comma) {
                                    break;
                                }
                            }
                            self.expect(&Token::BraceClose);
                        } else {
                            panic!("Expected '{{' after ',' in import");
                        }
                    }
                }
            }

            Token::Asterisk => {
                self.next(); // consume '*'
                self.expect_keyword("as");
                let local_name = match self.next()? {
                    Token::Identifier(name) => name,
                    _ => panic!("Expected identifier after 'as'"),
                };
                self.expect_keyword("from");
                let path = match self.next()? {
                    Token::String(path) => path,
                    _ => panic!("Expected string after 'from'"),
                };
                self.expect(&Token::Semicolon);
                return Some(Stmt::ImportAll {
                    local_name,
                    from: path,
                });
            }

            Token::BraceOpen => {
                self.next(); // consume '{'
                while self.peek() != Some(&Token::BraceClose) {
                    let imported = match self.next()? {
                        Token::Identifier(name) => name,
                        _ => panic!("Expected identifier in import list"),
                    };

                    let local = if self.expect_keyword("as") {
                        match self.next()? {
                            Token::Identifier(name) => name,
                            _ => panic!("Expected identifier after 'as'"),
                        }
                    } else {
                        imported.clone()
                    };

                    named_imports.push((imported, local));

                    if !self.expect(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::BraceClose);
            }

            _ => panic!("Unexpected token after 'import'"),
        }

        self.expect_keyword("from");

        let path = match self.next()? {
            Token::String(path) => path,
            _ => panic!("Expected string after 'from'"),
        };

        self.expect(&Token::Semicolon);

        match (default_import, named_imports.is_empty()) {
            (Some(local), true) => Some(Stmt::ImportDefault {
                local_name: local,
                from: path,
            }),
            (None, false) => Some(Stmt::ImportNamed {
                items: named_imports,
                from: path,
            }),
            (Some(default), false) => Some(Stmt::ImportMixed {
                default,
                items: named_imports,
                from: path,
            }),
            _ => None,
        }
    }

    fn parse_unary(&mut self, min_prec: u8) -> Option<Expr> {
        while let Some(op) = self.peek().and_then(get_unary_op) {
            self.next();
            let expr = self.parse_unary(min_prec)?; // recursivo para múltiplos unários como `!!a`
            return Some(Expr::UnaryOp {
                op,
                expr: Box::new(expr),
                postfix: false,
            });
        }
        self.parse_postfix_expr()
    }

    fn parse_class_decl(&mut self) -> Option<Stmt> {
        self.next(); // consume 'class'

        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        // Suporte a herança: class Nome extends SuperClasse
        let superclass = if self.consume(&Token::Identifier("extends".to_string())) {
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
            } else if self.expect_keyword("static") {
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
    fn parse_method(&mut self, is_static: bool) -> Option<MethodDecl> {
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

        Some(MethodDecl {
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

        // TODO: Else if
        // while self.expect_keyword("else") {
        //     self.next(); // consume "else"
        //     self.next(); // consume "if"
        //     self.expect(&Token::ParenOpen);
        //     let condition = self.parse_expr()?;
        //     self.expect(&Token::ParenClose);
        //     let then_branch = self.parse_block();
        //     else_ifs.push((condition, Some(then_branch)));
        // }

        let mut else_branch = None;

        if self.expect_keyword("else") {
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

        let is_let = self.consume_keyword("let");
        let pattern = if is_let {
            self.parse_primary()? // novo método para suportar destructuring
        } else {
            self.parse_expr()? // para casos como `for (item of list)`
        };

        if self.consume_keyword("in") {
            let object = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let body = self.parse_block();
            return Some(Stmt::ForIn {
                target: pattern,
                object,
                body: body,
            });
        }
        if self.consume_keyword("of") {
            let iterable = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let body = self.parse_block();
            return Some(Stmt::ForOf {
                target: pattern,
                iterable,
                body: body,
            });
        }

        self.next(); // Token::Assign

        // fallback para for tradicional
        let init = if is_let {
            Stmt::Let {
                name: self.extract_identifier(&pattern)?,
                value: self.parse_expr(),
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
            let value = self.parse_assignment_expr()?;
            return Some(Expr::Assign {
                target: Box::new(expr),
                op: AssignOperator::Assign,
                value: Box::new(value),
            });
        }

        Some(expr)
    }

    fn parse_identifier(&mut self) -> Option<Expr> {
        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };
        Some(Expr::Identifier(name))
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        let mut args: Vec<Expr> = vec![];
        while self.peek() != Some(&Token::ParenClose) {
            let expr = self.parse_expr();
            if expr.is_some() {
                args.push(expr.unwrap());
            }
            if !self.expect(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::ParenClose);
        args
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.next()? {
            Token::Identifier(s) if s == "this" => Some(Expr::This),
            Token::Identifier(s) if s == "new" => {
                let class_name = self.parse_identifier()?;
                let args = self.parse_arguments();
                let class_name = class_name.to_string();
                Some(Expr::New { class_name, args })
            }
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
            Token::BracketOpen => self.parse_bracket(),
            _ => None,
        }
    }

    // fn parse_this_expr(&mut self) -> Option<Expr> {
    //     if self.peek() == Some(&Token::Dot) {
    //         self.next(); // consume '.'
    //         let property = match self.next()? {
    //             Token::Identifier(name) => Expr::Identifier(name),
    //             _ => return None,
    //         };
    //         Some(Expr::This(Some(Box::new(property))))
    //     } else {
    //         Some(Expr::This(None))
    //     }
    // }
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
                Some(Token::Increment) => {
                    self.next();
                    expr = Expr::UnaryOp {
                        op: UnaryOperator::Increment,
                        expr: Box::new(expr),
                        postfix: true, // ⬅️ é pós-fixado
                    };
                }
                Some(Token::Decrement) => {
                    self.next();
                    expr = Expr::UnaryOp {
                        op: UnaryOperator::Decrement,
                        expr: Box::new(expr),
                        postfix: true,
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
        let token = self.tokens.get(i);
        match token {
            Some(Token::Identifier(_)) => {
                i += 1;
                let colon = self.tokens.get(i);
                matches!(colon, Some(Token::Colon))
            }
            Some(Token::Ellipsis) => {
                i += 1;
                let token = self.tokens.get(i);
                matches!(token, Some(Token::Identifier(_)))
                    || matches!(token, Some(Token::BraceOpen))
            }
            Some(Token::BraceClose) => true,
            _ => false,
        }
    }

    fn parse_object_literal(&mut self) -> Option<Expr> {
        self.expect(&Token::BraceOpen);

        let mut properties = vec![];

        while self.peek()? != &Token::BraceClose {
            if self.expect(&Token::Ellipsis) {
                // ...expr
                let expr = self.parse_expr()?;
                properties.push(ObjectEntry::Spread(expr));
            } else {
                // ident or ident: expr
                let key = match self.next()? {
                    Token::Identifier(name) => name,
                    _ => return None,
                };

                if self.expect(&Token::Colon) {
                    let value = self.parse_expr()?;
                    properties.push(ObjectEntry::Property { key, value });
                } else {
                    // shorthand: { a }  ->  { a: a }
                    properties.push(ObjectEntry::Shorthand(key));
                }
            }

            // Optional comma
            if !self.expect(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::BraceClose);
        Some(Expr::Literal(Literal::Object(properties)))
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Option<Expr> {
        let mut left: Expr = self.parse_unary(7)?;

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

    fn peek_prev(&self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.tokens.get(self.pos - 1)
        }
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
    fn consume_keyword<S: AsRef<str>>(&mut self, expected: S) -> bool {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                if name == expected.as_ref() {
                    self.next(); // avança o cursor
                    return true;
                }
                false
            }
            _ => false,
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

    fn expect_keyword<S: AsRef<str>>(&mut self, expected: S) -> bool {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                if name == expected.as_ref() {
                    self.pos += 1;
                    return true;
                }
                false
            }
            _ => false,
        }
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

fn get_unary_op(tok: &Token) -> Option<UnaryOperator> {
    match tok {
        Token::Minus => Some(UnaryOperator::Negative),
        Token::Plus => Some(UnaryOperator::Positive),
        Token::Not => Some(UnaryOperator::Not),
        Token::Increment => Some(UnaryOperator::Increment),
        Token::Decrement => Some(UnaryOperator::Decrement),
        Token::Identifier(i) if i == "typeof" => Some(UnaryOperator::Typeof),
        // inclua typeof se necessário
        _ => None,
    }
}
fn get_bin_op(token: &Token) -> Option<Operator> {
    match token {
        Token::Plus => Some(Operator::Binary(BinaryOperator::Add)),
        Token::Minus => Some(Operator::Binary(BinaryOperator::Subtract)),
        Token::Asterisk => Some(Operator::Binary(BinaryOperator::Multiply)),
        Token::Slash => Some(Operator::Binary(BinaryOperator::Divide)),
        Token::Modulo => Some(Operator::Binary(BinaryOperator::Modulo)),
        Token::Exponentiation => Some(Operator::Binary(BinaryOperator::Exponentiate)),

        Token::Equal => Some(Operator::Compare(CompareOperator::Eq)),
        Token::NotEqual => Some(Operator::Compare(CompareOperator::Ne)),
        Token::Less => Some(Operator::Compare(CompareOperator::Lt)),
        Token::Greater => Some(Operator::Compare(CompareOperator::Gt)),
        Token::LessEqual => Some(Operator::Compare(CompareOperator::Le)),
        Token::GreaterEqual => Some(Operator::Compare(CompareOperator::Ge)),

        Token::And => Some(Operator::Logical(LogicalOperator::And)),
        Token::Or => Some(Operator::Logical(LogicalOperator::Or)),

        Token::Increment => Some(Operator::Unary(UnaryOperator::Increment)),
        Token::Decrement => Some(Operator::Unary(UnaryOperator::Decrement)),
        Token::Not => Some(Operator::Unary(UnaryOperator::Not)),
        _ => None,
    }
}

fn get_precedence(op: &Operator) -> u8 {
    match op {
        Operator::Logical(LogicalOperator::Or) => 1,
        Operator::Logical(LogicalOperator::And) => 2,
        Operator::Compare(_) => 3,
        Operator::Binary(BinaryOperator::Add) | Operator::Binary(BinaryOperator::Subtract) => 4,
        Operator::Binary(BinaryOperator::Multiply)
        | Operator::Binary(BinaryOperator::Divide)
        | Operator::Binary(BinaryOperator::Modulo) => 5,
        Operator::Binary(BinaryOperator::Exponentiate) => 6,
        Operator::Unary(_) => 7,
    }
}
