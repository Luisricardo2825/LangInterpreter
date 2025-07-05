use std::collections::HashMap;
// use std::rc::Rc; // Troca para BOX para export e ExportAll

use crate::ast::ast::{
    AssignOperator, BinaryOperator, CompareOperator, Expr, FunctionStmt, Literal, LogicalOperator,
    MethodDecl, Modifiers, ObjectEntry, Operator, Stmt, UnaryOperator,
};
use crate::lexer::tokens::Token;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[allow(unused)]
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
            Token::Identifier(s) if s == "for" => self.parse_for_stmt(),
            Token::Identifier(s) if s == "while" => self.parse_while_stmt(),
            Token::Identifier(s) if s == "if" => self.parse_if_stmt(),
            Token::Identifier(s) if s == "try" => self.parse_try_stmt(),
            Token::Identifier(s) if s == "throw" => self.parse_throw_stmt(),
            Token::BraceOpen => Some(Stmt::ExprStmt(self.parse_brace()?)),
            Token::Identifier(s) if s == "class" => self.parse_class_decl(),
            _ => Some(Stmt::ExprStmt(self.parse_expr()?)),
        };
        // Se houver um ponto e vírgula depois do statement, consome
        self.consume(&Token::Semicolon);

        stmt
    }

    fn parse_throw_stmt(&mut self) -> Option<Stmt> {
        self.expect_keyword("throw");
        let expr = self.parse_expr()?;
        self.consume(&Token::Semicolon);
        Some(Stmt::Throw(expr))
    }

    fn parse_try_stmt(&mut self) -> Option<Stmt> {
        self.expect_keyword("try");
        let try_block = self.parse_block();

        let catch_block = if self.expect_keyword("catch") {
            self.expect(&Token::ParenOpen);

            let identifier = match self.next()? {
                Token::Identifier(name) => name,
                _ => panic!("Expected identifier after '('"),
            };

            self.expect(&Token::ParenClose);

            let block = self.parse_block();
            Some((identifier, block))
        } else {
            None
        };

        let finally_block = if self.expect_keyword("finally") {
            Some(self.parse_block())
        } else {
            None
        };

        Some(Stmt::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
        })
    }
    fn parse_export_stmt(&mut self) -> Option<Stmt> {
        if self.expect_keyword("export") {
            if self.expect_keyword("default") {
                let value = self.parse_stmt()?;
                self.expect(&Token::Semicolon);
                return Some(Stmt::ExportDefault(Box::new(value)));
            }

            let inner = self.parse_stmt()?; // let, fn, etc.
            return Some(Stmt::Export(Box::new(inner)));
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
                    if self.is(&Token::Comma) {
                        if self.is(&Token::BraceOpen) {
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

                                if !self.is(&Token::Comma) {
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

                    if !self.is(&Token::Comma) {
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
                let method = self.parse_method(false, false)?;
                methods.push(method);
            } else if self.expect_keyword("static") {
                let prev = self.peek();
                let next = self.peek_next();

                match (prev, next) {
                    (Some(Token::Identifier(_)), Some(Token::ParenOpen)) => {
                        let method = self.parse_method(true, false)?;
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
            } else if self.expect_keyword("operator") || self.expect_keyword("@Operator") {
                let method = self.parse_method(false, true)?;
                methods.push(method);
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
    fn parse_method(&mut self, is_static: bool, is_operator: bool) -> Option<MethodDecl> {
        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        self.expect(&Token::ParenOpen);
        let mut params = vec![];
        let mut vararg: Option<String> = None;

        loop {
            match self.peek()? {
                Token::ParenClose => break,
                Token::Ellipsis => {
                    self.next(); // consume "..."
                    if vararg.is_some() {
                        panic!("Error: Only one '...' parameter is allowed");
                    }

                    match self.next()? {
                        Token::Identifier(name) => {
                            vararg = Some(name);
                        }
                        _ => {
                            panic!("Error: Expected identifier after '...'");
                        }
                    }

                    if self.peek() == Some(&Token::Comma) && vararg.is_some() {
                        panic!("Error: A rest parameter must be last in a parameter list.");
                    }
                    // After vararg, no more parameters are allowed
                    if self.peek() != Some(&Token::ParenClose) {
                        panic!(
                            "Error: Unexpected token: {:?}",
                            self.peek().unwrap().to_string()
                        );
                    }

                    break;
                }
                Token::Identifier(name) => {
                    params.push(name.clone());
                    self.next(); // consume identifier

                    if self.peek() == Some(&Token::Comma) {
                        self.next(); // consume comma
                    } else {
                        break;
                    }
                }
                _ => {
                    panic!("Error: Unexpected token in parameter list");
                }
            }
        }

        self.expect(&Token::ParenClose);
        let body = self.parse_block();

        let mut modifiers: Vec<Modifiers> = vec![];

        if is_static {
            modifiers.push(Modifiers::Static);
        }
        if is_operator {
            modifiers.push(Modifiers::Operator);
        }
        Some(MethodDecl {
            name,
            params,
            vararg,
            body,
            modifiers,
        })
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "if"
        self.expect(&Token::ParenOpen);
        let condition = self.parse_expr()?;
        self.expect(&Token::ParenClose);
        let then_branch = self.parse_block();

        let mut else_ifs = vec![];
        let mut else_branch = None;

        while self.peek_is_keyword("else") {
            self.next(); // consume "else"
            if self.peek_is_keyword("if") {
                self.next(); // consume "if"
                self.expect(&Token::ParenOpen);
                let else_if_cond = self.parse_expr()?;
                self.expect(&Token::ParenClose);
                let else_if_block = self.parse_block();
                else_ifs.push((else_if_cond, Some(else_if_block)));
            } else {
                else_branch = Some(self.parse_block());
                break;
            }
        }

        Some(Stmt::If {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        })
    }

    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "while"
        self.expect(&Token::ParenOpen);
        let condition = self.parse_expr()?;
        self.expect(&Token::ParenClose);
        let body = self.parse_block();

        Some(Stmt::While { condition, body })
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume 'for'
        self.expect(&Token::ParenOpen);

        let is_let = self.consume_keyword("let");
        let pattern = if is_let {
            self.parse_primary()? // suporte a destructuring
        } else {
            self.parse_expr()? // para casos como for (item of list)
        };

        if self.consume_keyword("in") {
            let object = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let body = self.parse_block();
            return Some(Stmt::ForIn {
                target: pattern,
                object,
                body,
            });
        }

        if self.consume_keyword("of") {
            let iterable = self.parse_expr()?;
            self.expect(&Token::ParenClose);
            let body = self.parse_block();
            return Some(Stmt::ForOf {
                target: pattern,
                iterable,
                body,
            });
        }

        // fallback para for tradicional
        let init = if self.check(&Token::Semicolon) {
            None
        } else if is_let {
            self.next(); // Consume Token::Assign;
            let value = self.parse_expr().unwrap_or(Expr::Literal(Literal::Null));
            Some(Stmt::Let {
                name: self.extract_identifier(&pattern)?,
                value: value,
            })
        } else {
            Some(self.parse_stmt()?)
        };
        self.expect(&Token::Semicolon);

        let condition = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(&Token::Semicolon);

        let update = if self.check(&Token::ParenClose) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(&Token::ParenClose);

        let body = self.parse_block();

        Some(Stmt::For {
            init: init.map(Box::new)?,
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
        let value = self.parse_expr();
        Some(Stmt::Let {
            name,
            value: value.unwrap_or(Expr::Literal(Literal::Null)),
        })
    }

    fn parse_func_decl(&mut self) -> Option<Stmt> {
        self.next(); // consume "fn" or "function"

        let name = match self.next()? {
            Token::Identifier(name) => name,
            _ => return None,
        };

        self.expect(&Token::ParenOpen);
        let mut params = vec![];
        let mut vararg: Option<String> = None;

        loop {
            match self.peek()? {
                Token::ParenClose => break,
                Token::Ellipsis => {
                    self.next(); // consume "..."
                    if vararg.is_some() {
                        panic!("Error: Only one '...' parameter is allowed");
                    }
                    let token = self.next()?;
                    match token {
                        Token::Identifier(name) => {
                            vararg = Some(name);
                        }
                        _ => {
                            panic!("Error: Expected identifier after '...'. {:?}", token);
                        }
                    }

                    if self.peek() == Some(&Token::Comma) && vararg.is_some() {
                        panic!("Error: A rest parameter must be last in a parameter list.");
                    }
                    // After vararg, no more parameters are allowed
                    if self.peek() != Some(&Token::ParenClose) {
                        panic!(
                            "Error: Unexpected token: {:?}",
                            self.peek().unwrap().to_string()
                        );
                    }

                    break;
                }
                Token::Identifier(name) => {
                    params.push(name.clone());
                    self.next(); // consume identifier

                    if self.peek() == Some(&Token::Comma) {
                        self.next(); // consume comma
                    } else {
                        break;
                    }
                }
                _ => {
                    panic!("Error: Unexpected token in parameter list");
                }
            }
        }

        self.expect(&Token::ParenClose);
        let body = self.parse_block();

        Some(Stmt::FuncDecl(FunctionStmt {
            name,
            params,
            vararg,
            body,
        }))
    }
    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.next(); // consume "return"
        let value = if let Some(Token::BraceClose) = self.peek() {
            None
        } else if let Some(Token::Semicolon) = self.peek() {
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

        let operator: Option<AssignOperator> = get_assign_op(self.peek());

        // TODO: Assign operators
        if let Some(op) = operator {
            self.next(); // consume '='
            let value = self.parse_assignment_expr()?;
            return Some(Expr::Assign {
                target: Box::new(expr),
                op: op,
                value: Box::new(value),
            });
        }

        Some(expr)
    }

    fn parse_arguments(&mut self) -> Vec<Expr> {
        let mut args: Vec<Expr> = vec![];

        self.expect(&Token::ParenOpen);

        while self.peek() != Some(&Token::ParenClose) {
            let arg = if self.peek() == Some(&Token::Ellipsis) {
                let inner = self.parse_expr().expect("Expected expression after '...'");
                Expr::Spread(Box::new(inner))
            } else {
                self.parse_expr().expect("Expected argument expression")
            };

            args.push(arg);

            if self.peek() != Some(&Token::Comma) {
                break;
            }

            self.next(); // consume comma
        }

        self.expect(&Token::ParenClose);

        args
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.next()? {
            Token::Identifier(s) if s == "this" => Some(Expr::This),
            Token::Identifier(s) if s == "new" => self.parse_new_keyword(),
            Token::Number(n) => Some(Expr::Literal(Literal::Number(n))),
            Token::String(s) => Some(Expr::Literal(Literal::String(s))),
            Token::Bool(b) => Some(Expr::Literal(Literal::Bool(b))),
            Token::Null => Some(Expr::Literal(Literal::Null)),
            Token::Identifier(name) => {
                if let Some(Token::ParenOpen) = self.peek() {
                    let args = self.parse_arguments();
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

    fn parse_new_keyword(&mut self) -> Option<Expr> {
        let class_expr = self.parse_expr()?;

        Some(Expr::New {
            class_expr: Box::new(class_expr),
        })
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
                        let arg = if self.peek() == Some(&Token::Ellipsis) {
                            self.next(); // consume '...'
                            let inner = self.parse_expr()?;
                            Expr::Spread(Box::new(inner))
                        } else {
                            self.parse_expr()?
                        };
                        args.push(arg);

                        if !self.is(&Token::Comma) {
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
            if !self.is(&Token::Comma) {
                break;
            }
        }

        self.is(&Token::BracketClose);
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
        self.is(&Token::BraceOpen);

        let mut properties = vec![];

        while self.peek()? != &Token::BraceClose {
            if self.is(&Token::Ellipsis) {
                // ...expr
                let expr = self.parse_expr()?;
                properties.push(ObjectEntry::Spread(expr));
            } else {
                // ident or ident: expr
                let key = match self.next()? {
                    Token::Identifier(name) => name,
                    Token::String(name) => name,
                    _ => return None,
                };

                if self.is(&Token::Colon) {
                    let value = self.parse_expr()?;
                    properties.push(ObjectEntry::Property { key, value });
                } else {
                    // shorthand: { a }  ->  { a: a }
                    properties.push(ObjectEntry::Shorthand(key));
                }
            }

            // Optional comma
            if !self.is(&Token::Comma) {
                break;
            }
        }

        self.is(&Token::BraceClose);
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

    fn peek_is_keyword(&self, keyword: &str) -> bool {
        matches!(
            self.peek(),
            Some(Token::Identifier(k)) if k == keyword
        )
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

    #[track_caller]
    fn expect(&mut self, expected: &Token) {
        let caller = std::panic::Location::caller();
        let location = format!("{}:{}", caller.file(), caller.line());
        if self.peek() != Some(expected) {
            panic!(
                "Unexpected token: {:?}, expected: {:?} {location}",
                self.peek(),
                expected
            );
        }
        self.next(); // avança o cursor
    }

    fn insert_next(&mut self, token: Token) {
        self.tokens.insert(self.pos, token);
    }
    #[track_caller]
    fn expect_any(&mut self, expected: &[Token]) {
        let caller = std::panic::Location::caller();
        let location = format!("{}:{}", caller.file(), caller.line());

        let current = self.peek();

        if expected.iter().any(|t| Some(t) == current) {
            self.next(); // avança o cursor se houver match
        } else {
            panic!(
                "Unexpected token: {:?}, expected one of: {:?} {location}",
                current, expected
            );
        }
    }

    fn is(&mut self, expected: &Token) -> bool {
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

fn get_assign_op(tok: Option<&Token>) -> Option<AssignOperator> {
    match tok {
        Some(Token::Assign) => Some(AssignOperator::Assign), // #[token("=")]
        Some(Token::AddAssign) => Some(AssignOperator::AddAssign), // #[token("+=")]
        Some(Token::SubAssign) => Some(AssignOperator::SubAssign), //  #[token("-=")]
        Some(Token::MulAssign) => Some(AssignOperator::MulAssign), //  #[token("*=")]
        Some(Token::DivAssign) => Some(AssignOperator::DivAssign), // #[token("/=")]
        Some(Token::ModAssign) => Some(AssignOperator::ModAssign), //  #[token("%=")]
        Some(Token::PowAssign) => Some(AssignOperator::PowAssign), // #[token("**=")]

        _ => None,
    }
}

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
        Token::Identifier(i) if i == "instanceof" => {
            Some(Operator::Compare(CompareOperator::InstanceOf))
        }
        Token::Identifier(i) if i == "in" => Some(Operator::Compare(CompareOperator::In)),

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
