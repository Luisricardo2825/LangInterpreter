use logos::Logos;
use std::{cell::RefCell, collections::HashMap, env, fs, io::Write, path::Path, rc::Rc};

use crate::{
    ast::ast::{
        debug_stmts, BinaryOperator, CompareOperator, ControlFlow, Expr, FunctionStmt, Literal,
        LogicalOperator, MethodModifiersOperations, ObjectEntry, Operator, Stmt,
    },
    environment::{
        helpers::class::ClassGenerator,
        values::{Class, Function, NativeObjectTrait, Value},
        Environment,
    },
    lexer::tokens::Token,
    parsers::code::parser::Parser,
};
use logos::Lexer;

pub struct LexerWithLocation<'source> {
    inner: Lexer<'source, Token>,
    line: usize,
    col: usize,
    last_offset: usize,
}

impl<'source> LexerWithLocation<'source> {
    pub fn new(source: &'source str) -> Self {
        LexerWithLocation {
            inner: Token::lexer(source),
            line: 1,
            col: 1,
            last_offset: 0,
        }
    }

    pub fn next(
        &mut self,
    ) -> Option<(
        Result<Token, super::lexer::tokens::LexingError>,
        usize,
        usize,
    )> {
        let tok = self.inner.next()?;
        let span = self.inner.span();
        let slice = &self.inner.source()[self.last_offset..span.start];

        // Atualiza linha/coluna baseado no trecho desde o último token
        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        self.last_offset = span.start;

        Some((tok, self.line, self.col))
    }
}
#[derive(Debug, Clone)]
pub struct Interpreter {
    source: String,
    module_cache: HashMap<String, Rc<RefCell<Environment>>>,
    exported_symbols: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new(source: String) -> Self {
        Self {
            source,
            module_cache: HashMap::new(),
            exported_symbols: HashMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self {
            source: String::new(),
            module_cache: HashMap::new(),
            exported_symbols: HashMap::new(),
        }
    }

    pub fn tokenize(&self, src: String, filename: String) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        let mut lexer = LexerWithLocation::new(&src);

        while let Some((token, line, col)) = lexer.next() {
            match token {
                Ok(token) => {
                    if token == Token::Comment {
                        continue;
                    }
                    if let Token::Unknown(c) = token {
                        let filename = Self::normalize_path(&filename, false);
                        panic!("Invalid char '{c}' at {filename}/:{line}:{col}");
                    }
                    tokens.push(token);
                }
                Err(e) => panic!("some error occurred: {:?}", e),
            }
        }
        tokens
    }
    pub fn interpret_from_file(&mut self, filename: String) -> Option<Value> {
        let show_ast = env::args().nth(2).unwrap_or("false".to_owned());
        let generate_classes = env::args().nth(3).unwrap_or(String::new());
        let src = fs::read_to_string(&filename).unwrap_or(self.source.clone());

        let tokens = self.tokenize(src.clone(), filename.clone());
        let mut parser = Parser::new(tokens);

        // let error_class = ClassGenerator::create_error_class();
        let default_stdlib = self.load_stdlib();
        // let default_classes = vec![error_class];

        let mut ast = vec![];

        ast.extend(default_stdlib);
        ast.extend(parser.parse());

        // bench();
        if show_ast == "tree" {
            debug_stmts(&ast, 0);
            // println!("{:#?}", ast);
        }
        if show_ast == "detailed" {
            println!("{:#?}", ast);
        }
        if show_ast == "true" {
            println!("{:#?}", ast);
        }

        if !generate_classes.is_empty() {
            println!(); // Quebra linha

            // Write class to file in generate_classes
            for stmt in ast {
                let class = ClassGenerator::generate_class_function(&stmt);

                if class.is_some() {
                    let line = class.unwrap();
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(generate_classes.clone())
                        .unwrap();
                    file.write_all(line.as_bytes()).unwrap();
                }
            }
            return None;
        }
        // let mut env = self.env.clone();
        let mut env = Environment::new_rc();
        for stmt in ast {
            let val = self.eval_stmt(&stmt, &mut env);
            match val {
                ControlFlow::Error(err) => {
                    panic!("{}", err.to_string());
                }
                _ => {
                    continue;
                }
            }
        }
        None
    }

    // function to read a entire dir and return Vec<Stmt> for each class
    pub fn load_stdlib(&mut self) -> Vec<Stmt> {
        let files = Self::get_files("stdlib");

        let mut ast = vec![];

        for filename in files {
            let src = fs::read_to_string(&filename).unwrap_or(self.source.clone());

            let tokens = self.tokenize(src.clone(), filename.clone());
            let mut parser = Parser::new(tokens);

            ast.extend(parser.parse());
        }
        return ast;
    }

    pub fn get_files(dir: &str) -> Vec<String> {
        let mut files = Vec::new();
        let entries = fs::read_dir(dir).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_str().unwrap().to_string());
            }
        }
        files
    }

    pub fn interpret(&mut self) -> Option<Value> {
        let filename = env::args()
            .nth(1)
            .unwrap_or("./examples/trycatch.x".to_string());
        self.interpret_from_file(filename)
    }

    pub fn interpret_bench(&mut self) {
        let start = std::time::Instant::now();
        self.interpret();
        let elapsed = start.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }

    fn get_absolute_path(path: &str) -> String {
        let filename = Path::new(path);
        let absolute_path = filename.canonicalize().unwrap();
        let mut absolute_path_str = absolute_path.to_str().unwrap().to_string();

        // Remove prefixo Windows \\?\
        if absolute_path_str.starts_with(r"\\?\") {
            absolute_path_str = absolute_path_str[4..].to_string();
        }

        // Converte \ para /
        absolute_path_str.replace("\\", "/")
    }
    fn normalize_path(path: &str, absolute: bool) -> String {
        if absolute {
            return Self::get_absolute_path(path);
        }
        let path = Path::new(path);
        let current_dir = env::current_dir().unwrap();

        let relative_path = path.strip_prefix(&current_dir).unwrap_or(&path);

        relative_path.to_str().unwrap().replace("\\", "/")
    }

    #[allow(unreachable_patterns)]
    pub fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &mut Rc<RefCell<Environment>>,
    ) -> ControlFlow<Value> {
        let ret = match expr {
            Expr::Identifier(name) => {
                let value = self.resolve_variable(name, env);
                if value.is_err() {
                    return ControlFlow::new_error(env, value.unwrap_err());
                }
                let value = value.unwrap();

                value
            }
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(n.into()),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::String(s) => Value::String(s.clone().into()),
                Literal::Null => Value::Null,
                Literal::Void => Value::Void,
                Literal::Object(entries) => {
                    let mut result = Vec::new();

                    for entry in entries {
                        match entry {
                            ObjectEntry::Property { key, value } => {
                                let val = self.eval_expr(value, env);
                                if val.is_error() {
                                    return val;
                                }

                                let val = val.unwrap();
                                result.set_prop(&key, val).unwrap();
                            }
                            ObjectEntry::Shorthand(name) => {
                                // Resolve a variável no ambiente
                                let val = env.borrow().get(name).unwrap_or(Value::Null); // ou trate erro se preferir
                                result.set_prop(&name, val).unwrap();
                            }
                            ObjectEntry::Spread(expr) => {
                                let val = self.eval_expr(expr, env);
                                if val.is_error() {
                                    return val;
                                }
                                let val = val.unwrap();
                                match val {
                                    Value::Object(map) => {
                                        let map = map.borrow_mut().clone();
                                        for (k, v) in map {
                                            result.set_prop(&k, v).unwrap();
                                        }
                                    }
                                    _ => {
                                        // opcional: erro em tempo de execução
                                        return ControlFlow::new_error(
                                            env,
                                            format!("Spread operator must be used with an object"),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    Value::object(result)
                }
                Literal::Array(arr) => {
                    let mut elements = Vec::new();
                    for elem in arr {
                        let val = self.eval_expr(elem, env);
                        if val.is_error() {
                            return val;
                        }
                        let val = val.unwrap();
                        elements.push(val);
                    }
                    Value::array(elements)
                }
            },
            Expr::Block(stmts) => {
                let mut local_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                for stmt in stmts {
                    let ret = self.eval_stmt(stmt, &mut local_env);
                    match ret {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::None => {}
                        // ControlFlow::Error(_) => {
                        //     return ret;
                        // }
                        ret => return ret, // return
                    };
                }
                return ControlFlow::None;
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, env);
                let r = self.eval_expr(right, env);

                if l.is_error() {
                    return l;
                }
                if r.is_error() {
                    return r;
                }

                let l = l.unwrap();
                let r = r.unwrap();

                match (op, l, r) {
                    (Operator::Binary(math_op), left, right) => {
                        left.call_op(math_op.clone(), &right)
                    }

                    (Operator::Compare(comp_op), a, b) => match comp_op {
                        CompareOperator::Eq => Value::Bool(a == b),
                        CompareOperator::Ne => Value::Bool(a != b),
                        CompareOperator::Gt => Value::Bool(a > b),
                        CompareOperator::Ge => Value::Bool(a >= b),
                        CompareOperator::Lt => Value::Bool(a < b),
                        CompareOperator::Le => Value::Bool(a <= b),
                        CompareOperator::InstanceOf => {
                            return ControlFlow::Return(Value::Bool(Class::is_instance_of(&a, &b)));
                        }
                        CompareOperator::In => match (&a, &b) {
                            (Value::String(a), Value::Object(b)) => {
                                let b = b.borrow();
                                Value::Bool(b.contains_key(&a.to_string()))
                            }
                            (Value::Number(idx), Value::Array(b)) => {
                                let idx = idx.get_value() as usize;
                                let b = b.get_value();
                                let b = b.borrow();
                                let item = b.get(idx);
                                Value::Bool(item.is_some())
                            }
                            _ => {
                                return ControlFlow::new_error(
                                    env,
                                    format!("Invalid operands for 'in': {:?} and {:?}", a, b),
                                )
                            }
                        },
                        _ => {
                            return ControlFlow::new_error(
                                env,
                                format!("Invalid comparison operator: {:?}", comp_op),
                            )
                        }
                    },

                    (Operator::Logical(log_op), Value::Bool(a), Value::Bool(b)) => match log_op {
                        LogicalOperator::And => Value::Bool(a && b),
                        LogicalOperator::Or => Value::Bool(a || b),
                    },

                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!("Operation not suported: {op:?}"),
                        )
                    } // fallback
                }
            }
            Expr::UnaryOp { op, expr, postfix } => {
                let val = self.eval_expr(expr, env);
                if val.is_error() {
                    return val;
                }
                let val = val.unwrap();
                match op {
                    crate::ast::ast::UnaryOperator::Negative => {
                        Value::Number((-val.to_number()).into())
                    }
                    crate::ast::ast::UnaryOperator::Not => Value::Bool(!val.to_bool()),
                    crate::ast::ast::UnaryOperator::Typeof => Value::String(val.type_of().into()),
                    crate::ast::ast::UnaryOperator::Increment => {
                        let new_val = val.to_number() + 1.0;
                        match expr.as_ref() {
                            Expr::Identifier(name) => {
                                let name = name.clone();
                                let previous_val = env.borrow().get(&name).unwrap();

                                env.borrow_mut()
                                    .assign(&name, Value::Number(new_val.into()))
                                    .unwrap();

                                if *postfix {
                                    previous_val
                                } else {
                                    Value::Number(new_val.into())
                                }
                            }
                            Expr::Literal(literal) => match literal {
                                Literal::Number(number) => {
                                    let new_val = number + 1.0;

                                    if *postfix {
                                        Value::Number(number.into())
                                    } else {
                                        Value::Number(new_val.into())
                                    }
                                }
                                _ => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Invalid value for increment: {:?}", expr).into(),
                                    )
                                }
                            },
                            Expr::GetProperty { object, property } => {
                                let previous_val = self.eval_expr(
                                    &Expr::GetProperty {
                                        object: object.clone(),
                                        property: property.clone(),
                                    },
                                    env,
                                );

                                let expr = &Expr::Assign {
                                    target: {
                                        Box::new(Expr::GetProperty {
                                            object: object.clone(),
                                            property: property.clone(),
                                        })
                                    },
                                    op: crate::ast::ast::AssignOperator::AddAssign,
                                    value: Box::new(Expr::Literal(Literal::Number(new_val))),
                                };

                                let _ = self.eval_expr(expr, env);

                                if *postfix {
                                    return previous_val;
                                }

                                return ControlFlow::Return(Value::Number((new_val).into()));
                            }
                            _ => {
                                return ControlFlow::new_error(
                                    env,
                                    format!("Invalid operand for increment: {:?}", expr).into(),
                                )
                            }
                        }
                    }
                    crate::ast::ast::UnaryOperator::Decrement => {
                        let new_val = val.to_number() - 1.0;

                        match expr.as_ref() {
                            Expr::Identifier(name) => {
                                let name = name.clone();
                                let previous_val = env.borrow().get(&name).unwrap();
                                env.borrow_mut()
                                    .assign(&name, Value::Number(new_val.into()))
                                    .unwrap();

                                if *postfix {
                                    previous_val
                                } else {
                                    Value::Number(new_val.into())
                                }
                            }
                            Expr::Literal(literal) => match literal {
                                Literal::Number(number) => {
                                    let new_val = number - 1.0;

                                    if *postfix {
                                        Value::Number(number.into())
                                    } else {
                                        Value::Number(new_val.into())
                                    }
                                }
                                _ => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Invalid value for increment: {:?}", expr).into(),
                                    )
                                }
                            },
                            Expr::GetProperty { object, property } => {
                                let previous_val = self.eval_expr(
                                    &Expr::GetProperty {
                                        object: object.clone(),
                                        property: property.clone(),
                                    },
                                    env,
                                );

                                let expr = &Expr::Assign {
                                    target: {
                                        Box::new(Expr::GetProperty {
                                            object: object.clone(),
                                            property: property.clone(),
                                        })
                                    },
                                    op: crate::ast::ast::AssignOperator::SubAssign,
                                    value: Box::new(Expr::Literal(Literal::Number(new_val))),
                                };

                                let _ = self.eval_expr(expr, env);

                                if *postfix {
                                    return previous_val;
                                }

                                return ControlFlow::Return(Value::Number((new_val).into()));
                            }
                            _ => {
                                return ControlFlow::new_error(
                                    env,
                                    format!("Invalid operand for increment: {:?}", expr).into(),
                                )
                            }
                        }
                    }
                    crate::ast::ast::UnaryOperator::Positive => {
                        Value::Number(val.to_number().abs().into())
                    }
                }
            }
            Expr::Call { callee, args } => {
                let evaluated_callee = self.eval_expr(callee, env);

                if evaluated_callee.is_error() {
                    return evaluated_callee;
                }

                let mut evaluated_args = vec![];

                for arg_expr in args {
                    match arg_expr {
                        Expr::Spread(inner_expr) => {
                            let val = self.eval_expr(&inner_expr, env);
                            if val.is_error() {
                                return val;
                            }
                            let val = val.unwrap();

                            match val {
                                Value::Array(arr) => {
                                    let arr = arr.get_value().clone();
                                    evaluated_args.extend(arr.borrow().clone());
                                }
                                Value::Object(map) => {
                                    let map = map.borrow().clone();
                                    for (_, v) in map {
                                        evaluated_args.push(v);
                                    }
                                }
                                _ => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Cannot spread {:?}", val.type_of()).into(),
                                    )
                                }
                            }
                        }
                        _ => {
                            let val = self.eval_expr(&arg_expr, env);

                            if val.is_err() {
                                return val;
                            }

                            let val = val.unwrap();
                            evaluated_args.push(val);
                        }
                    }
                }

                if evaluated_callee.is_err() {
                    return evaluated_callee;
                }
                let evaluated_callee = evaluated_callee.unwrap();
                match evaluated_callee {
                    Value::Function(func) => {
                        let call = func.call(evaluated_args);

                        call
                    }

                    Value::Builtin(func) => func(evaluated_args),
                    Value::InternalFunction((name, native_class)) => {
                        let mut new_args = native_class.borrow().get_args();
                        let is_static = native_class.borrow().is_static();

                        let call = if is_static {
                            native_class.borrow().call_with_args(&name, evaluated_args)
                        } else {
                            // concat new_args and arg_values
                            new_args.extend(evaluated_args.clone());

                            match native_class.borrow_mut().add_args(new_args) {
                                Ok(_) => {}
                                Err(err) => return ControlFlow::new_error(env, err),
                            }

                            let native_method = native_class.borrow().call(&name);
                            native_method
                        };

                        match call {
                            ControlFlow::Return(_) => return call,
                            ControlFlow::Error(val) => {
                                return ControlFlow::new_error(env, val.into());
                            }
                            ControlFlow::None => return ControlFlow::None,
                            _ => return ControlFlow::new_error(env, "Not allowed".into()),
                        }
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!("'{}' is not a function", self.resolve_calle_name(callee))
                                .into(),
                        )
                    }
                }
            }
            Expr::Assign { target, op, value } => {
                let val = self.eval_expr(value, env);

                if val.is_error() {
                    return val;
                }

                let val = val.unwrap();

                match &**target {
                    Expr::Identifier(name) => {
                        // atribuição simples
                        match op {
                            crate::ast::ast::AssignOperator::Assign => {
                                env.borrow_mut().assign(name, val.clone()).unwrap();
                            }
                            crate::ast::ast::AssignOperator::AddAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (Value::Number(a), Value::Number(b)) => {
                                        env.borrow_mut()
                                            .assign(name, Value::Number((a + b).into()))
                                            .unwrap();
                                    }
                                    (Value::Array(a), Value::Array(b)) => {
                                        let a = a.get_value();
                                        let mut a = a.borrow_mut();
                                        let b = b.get_value();
                                        let b = b.borrow();
                                        a.extend(b.to_owned());
                                    }

                                    (Value::Class(class), Value::Number(num)) => {
                                        let value_of_method = class.get_value_of_method();
                                        if value_of_method.is_some() {
                                            let value_of_method = value_of_method.unwrap();
                                            let value = value_of_method.call(vec![]);
                                            if value.is_err() {
                                                let error = value;
                                                return ControlFlow::Error(error);
                                            }
                                            if value.is_number() {
                                                let value = value.to_number() + num.get_value();
                                                env.borrow_mut()
                                                    .assign(name, Value::Number(value.into()))
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    (Value::Instance(instance), b) => {
                                        let class_name = instance.borrow().class.name.clone();
                                        let operator_plus_method =
                                            instance.borrow().find_operation("plus");
                                        if operator_plus_method.is_some() {
                                            let method = operator_plus_method.unwrap();
                                            let call = method.call(vec![b.clone()]);
                                            if call.is_err() {
                                                let error = call;
                                                return ControlFlow::Error(error);
                                            }
                                            return ControlFlow::Return(call);
                                        }
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "operator not implemented for '{class_name}' class {op}",
                                            )
                                            .into(),
                                        );
                                    }
                                    (a, b) => {
                                        let a = a.to_string();

                                        let b = b.to_string();

                                        env.borrow_mut()
                                            .assign(name, Value::String((a + &b).into()))
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::SubAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;
                                match (&old_value, &new_value) {
                                    (Value::Number(a), Value::Number(b)) => {
                                        env.borrow_mut()
                                            .assign(name, Value::Number((a - b).into()))
                                            .unwrap();
                                    }
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                name,
                                                Value::Number(
                                                    (a.to_number() - b.to_number()).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }

                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::MulAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                name,
                                                Value::Number(
                                                    (a.to_number() * b.to_number()).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::DivAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                name,
                                                Value::Number(
                                                    (a.to_number() / b.to_number()).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::ModAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                name,
                                                Value::Number(
                                                    (a.to_number() % b.to_number()).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::PowAssign => {
                                let old_value = env.borrow().get(name);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                name,
                                                Value::Number(
                                                    (a.to_number().powf(b.to_number())).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                        }
                    }
                    Expr::GetProperty { object, property } => {
                        // atribuição a propriedade: obj.prop = val
                        let obj = self.eval_expr(object, env);

                        if obj.is_error() {
                            return obj;
                        }
                        let obj = obj.unwrap();

                        let key = match &**property {
                            Expr::Identifier(name) => name.clone(),
                            _ => {
                                return ControlFlow::new_error(
                                    env,
                                    format!("Propriedade inválida").into(),
                                )
                            }
                        };

                        match op {
                            crate::ast::ast::AssignOperator::Assign => match obj {
                                Value::Object(instance) => {
                                    instance.borrow_mut().set_prop(&key, val.clone()).unwrap();
                                }

                                Value::Instance(instance) => {
                                    let set_result = instance.borrow_mut().set(&key, val.clone());
                                    if set_result.is_err() {
                                        let error = set_result.unwrap_err();
                                        return ControlFlow::new_error(env, error.into());
                                    }
                                }
                                Value::InternalClass(class) => {
                                    class.borrow_mut().add_custom_method(key, val).unwrap();
                                }

                                Value::Error(error) => {
                                    let obj = error.borrow();

                                    if obj.is_instance() {
                                        let instance = obj.to_instance();
                                        let set_result =
                                            instance.borrow_mut().set(&key, val.clone());
                                        if set_result.is_err() {
                                            let error = set_result.unwrap_err();
                                            return ControlFlow::new_error(env, error.into());
                                        }
                                        return ControlFlow::None;
                                    }

                                    return ControlFlow::new_error(
                                        env,
                                        format!("'{}' not found in '{}'", key, obj.type_of())
                                            .into(),
                                    );
                                }
                                ret => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("'{}' not found in '{}'", key, ret.type_of())
                                            .into(),
                                    );
                                }
                            },
                            crate::ast::ast::AssignOperator::PowAssign => {
                                let old_value = env.borrow().get(&key);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                &key,
                                                Value::Number(
                                                    (a.to_number().powf(b.to_number())).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            crate::ast::ast::AssignOperator::MulAssign => {
                                let old_value = env.borrow().get(&key);
                                let old_value = old_value.unwrap_or(Value::Null);
                                let new_value = val;

                                match (&old_value, &new_value) {
                                    (a, b) => {
                                        env.borrow_mut()
                                            .assign(
                                                &key,
                                                Value::Number(
                                                    (a.to_number() * b.to_number()).into(),
                                                ),
                                            )
                                            .unwrap();
                                    }
                                    _ => {
                                        return ControlFlow::new_error(
                                            env,
                                            format!(
                                                "Invalid operation: {:?} {} {:?}",
                                                old_value.type_of(),
                                                op,
                                                new_value.type_of()
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            _ => todo!("Not supported {expr}"),
                        }
                    }
                    Expr::BracketAccess { object, property } => {
                        // atribuição por índice: arr[i] = val
                        let arr = self.eval_expr(object, env);

                        if arr.is_error() {
                            return arr;
                        }

                        let index = self.eval_expr(property, env);

                        if index.is_error() {
                            return index;
                        }

                        let arr = arr.unwrap();
                        let index = index.unwrap();

                        match op {
                            crate::ast::ast::AssignOperator::Assign => match (arr, index) {
                                (Value::Array(array), Value::Number(n)) => {
                                    let idx = n.get_value() as usize;
                                    let get_value = array.get_value();
                                    let mut array = get_value.borrow_mut();
                                    let item = array.get(idx);

                                    if item.is_some() {
                                        array[idx] = val.clone();
                                    } else {
                                        array.push(val.clone());
                                    }
                                }
                                (Value::Object(obj), Value::String(key)) => {
                                    obj.borrow_mut()
                                        .set_prop(&key.to_string(), val.clone())
                                        .unwrap();
                                }
                                (Value::Object(obj), Value::Number(key)) => {
                                    let key = key.get_value() as usize;
                                    obj.borrow_mut().push((key.to_string(), val.clone()));
                                }
                                ac => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Acesso inválido para atribuição {:?}", ac).into(),
                                    )
                                }
                            },
                            crate::ast::ast::AssignOperator::AddAssign => match (arr, index) {
                                (Value::Array(array), Value::Number(n)) => {
                                    let idx = n.get_value() as usize;
                                    let get_value = array.get_value();
                                    let mut array = get_value.borrow_mut();
                                    let item = array.get(idx);

                                    if item.is_some() {
                                        let item = item.unwrap();

                                        let add_expr = self.binary_operation(
                                            Operator::Binary(BinaryOperator::Add),
                                            Expr::Literal(Literal::from_value(item)),
                                            *value.clone(),
                                        );
                                        let new_value = self.eval_expr(&add_expr, env);
                                        if new_value.is_error() {
                                            return new_value;
                                        }

                                        let new_value = new_value.unwrap();

                                        array[idx] = new_value;
                                    } else {
                                        array.push(val.clone());
                                    }
                                }

                                (Value::Object(obj), Value::String(key)) => {
                                    let previous_value = obj.borrow().get_prop(&key.to_string());
                                    if previous_value.is_some() {
                                        let previous_value = previous_value.unwrap();
                                        let add_expr = self.binary_operation(
                                            Operator::Binary(BinaryOperator::Add),
                                            Expr::Literal(Literal::from_value(&previous_value)),
                                            *value.clone(),
                                        );
                                        let new_value = self.eval_expr(&add_expr, env);

                                        if new_value.is_error() {
                                            return new_value;
                                        }
                                        let new_value = new_value.unwrap();

                                        obj.borrow_mut()
                                            .set_prop(&key.to_string(), new_value)
                                            .unwrap();
                                    } else {
                                        obj.borrow_mut()
                                            .set_prop(&key.to_string(), val.clone())
                                            .unwrap();
                                    }
                                }
                                ac => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Acesso inválido para atribuição {:?}", ac).into(),
                                    )
                                }
                            },
                            _ => todo!("Not supported"),
                        }
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!("Expressão inválida no lado esquerdo da atribuição").into(),
                        )
                    }
                }

                return ControlFlow::None;
            }
            Expr::GetProperty { object, property } => {
                let obj = self.eval_expr(object, env);
                if obj.is_error() {
                    return obj;
                }
                let mut obj = obj.unwrap();

                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string().into()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(n.into()),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone().into()),
                    _ => return ControlFlow::Return(Value::Null),
                };

                if obj.is_native_class() && prop.is_string() {
                    let prop = prop.to_string();
                    let native = obj.get_native_class().unwrap();
                    return ControlFlow::Return(Value::InternalFunction((prop, native.clone())));
                }
                if let Value::Error(err) = obj {
                    let err_value = err.clone().borrow().clone();
                    obj = err_value;
                }

                if obj.is_primitive() {
                    let class = obj.get_primitive_class(env);
                    if class.is_some() {
                        obj = class.unwrap();
                    }
                }

                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg =
                            format!("Property '{prop}' not  in found '{}'", object.to_string());
                        let prop = obj.borrow().get_prop(&prop);
                        if prop.is_none() {
                            return ControlFlow::new_error(env, msg);
                        }
                        let prop = prop.unwrap();
                        return ControlFlow::Return(prop);
                    }
                    (Value::Instance(instance), Value::String(prop)) => {
                        let class_name = instance.borrow().class.name.clone();

                        let msg = format!("Cannot find '{prop}' in class {class_name}");

                        let value = instance.borrow().get(&prop.to_string());

                        if value.is_none() {
                            let error = ControlFlow::new_error(env, msg);
                            return error;
                        }
                        let value = value.unwrap();

                        if value.is_function() {
                            let func = value.to_function();

                            let mut new_func = Function::from(func);
                            new_func.this = obj;
                            return ControlFlow::Return(Value::Function(new_func.into()));
                        }
                        return ControlFlow::Return(value);
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        let class_name = &class.name;
                        let msg = format!("Cannot find static '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.get_static_field(&name) {
                            return ControlFlow::Return(val);
                        }
                        if let Some(val) = class.find_static_method(&name) {
                            return ControlFlow::Return(Value::Function(val));
                        }

                        return ControlFlow::new_error(env, format!("{}", msg).into());
                    }
                    (Value::InternalClass(native), Value::String(prop)) => {
                        return ControlFlow::Return(Value::InternalFunction((
                            prop.to_string(),
                            native.clone(),
                        )));
                    }
                    (Value::Function(_func), Value::String(prop)) => {
                        let prop = prop;

                        let msg = format!("Property {prop} not found");
                        todo!("A fazer {prop} {msg}");
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!(
                                "Cannot access property {:?} of {:?} (type: {:?}, {:?}) GetProperty",
                                prop.to_string(),
                                obj.to_string(),
                                &prop.type_of(),
                                &obj.type_of()
                            )
                            .into(),
                        );
                    }
                }
            }
            Expr::BracketAccess { object, property } => {
                let obj = self.eval_expr(object, env);

                if obj.is_error() {
                    return obj;
                }

                let prop = self.eval_expr(property, env);

                if prop.is_error() {
                    return prop;
                }
                let obj = obj.unwrap();
                let prop = prop.unwrap();
                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let prop = prop.to_string();
                        let msg = format!("Property {prop} not found");
                        obj.borrow().get_prop(&prop).expect(&msg)
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = index.get_value() as usize;
                        let value = arr
                            .get_value()
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Null);
                        value
                    }
                    (Value::String(arr), Value::Number(index)) => {
                        let arr = arr.to_string();
                        let index = index.get_value() as usize;
                        arr.chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string().into()))
                            .unwrap_or(Value::Null)
                    }
                    (Value::Object(obj), prop) => {
                        let prop = prop.to_string();
                        // let msg = format!("Property {prop} not found");
                        obj.borrow().get_prop(&prop).unwrap_or(Value::Null)
                    }
                    (Value::Instance(instance), Value::Number(index)) => {
                        let collection_class = env.borrow().get("Collection");
                        let collection_class = collection_class.unwrap();

                        if !instance.borrow().is_instance_of(&collection_class) {
                            return ControlFlow::new_error(env, "Not a collection".into());
                        }
                        let iter_method = instance.borrow().get("iter");

                        let arr = iter_method.unwrap().to_method().call(vec![obj]);
                        let index = index.get_value() as usize;
                        arr.to_array()
                            .get(index)
                            .map(|ch| Value::String(ch.to_string().into()))
                            .unwrap_or(Value::Null)
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!(
                            "Cannot access property {:?} of {:?} (type: {:?}, {:?}) BracketAccess",
                            prop.to_string(),
                            obj.to_string(),
                            &obj.type_of(),
                            &prop.type_of()
                        )
                            .into(),
                        )
                    }
                }
            }
            Expr::SetProperty {
                object,
                property,
                value,
            } => {
                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string().into()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(n.into()),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone().into()),
                    _ => return ControlFlow::Return(Value::Null),
                };

                let obj = self.eval_expr(object, env);
                if obj.is_error() {
                    return obj;
                }
                let obj = obj.unwrap();

                let val = self.eval_expr(value, env);
                if val.is_error() {
                    return val;
                }
                let val = val.unwrap();

                match (obj.clone(), &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg = format!("Property {prop} not found");
                        obj.borrow_mut()
                            .set_prop(&prop.clone().to_string(), val)
                            .expect(&msg);
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = index.get_value() as usize;
                        // let msg = format!("Index {index} out of bounds");
                        let get_value = arr.get_value();
                        let mut arr = get_value.borrow_mut();
                        arr[index] = val;
                    }
                    (Value::Instance(instance), Value::String(prop)) => {
                        instance
                            .borrow()
                            .this
                            .borrow_mut()
                            .assign(&prop.to_string(), val)
                            .unwrap();
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        class
                            .this
                            .borrow()
                            .get(&prop.to_string())
                            .unwrap_or(Value::Null);
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!(
                            "Cannot access property {:?} of {:?} (type: {:?}, {:?}) SetProperty",
                            prop.to_string(),
                            obj.to_string(),
                            &obj.type_of(),
                            &prop.type_of()
                        )
                            .into(),
                        )
                    }
                }
                return ControlFlow::None;
            }
            Expr::New { class_expr } => {
                let empty_args: Vec<Expr> = vec![];
                // Se for uma chamada, separa callee e args
                let (class_callee, args) = match &**class_expr {
                    Expr::Call { callee, args } => (callee, args),
                    Expr::Identifier(name) => {
                        let callee = Expr::Identifier(name.clone());
                        (&Box::new(callee), &empty_args)
                    }
                    _ => {
                        return ControlFlow::new_error(
                            env,
                            format!("Expected a call expression after 'new'"),
                        )
                    }
                };

                let mut arg_values = vec![];

                // Avalia os argumentos
                for arg in args.iter() {
                    let val = self.eval_expr(arg, env);
                    if val.is_error() {
                        return val;
                    }
                    let mut val = val.unwrap();
                    if val.is_void() {
                        val = Value::Null;
                    }
                    arg_values.push(val);
                }

                // Avalia o callee (pode ser Identifier, MemberAccess, etc.)
                let value = self.eval_expr(class_callee, env);

                if value.is_error() {
                    return value;
                }
                let value = value.unwrap();

                match value {
                    Value::Class(class) => {
                        let instance = Class::instantiate(&class, arg_values);
                        return ControlFlow::Return(instance);
                    }
                    Value::InternalClass(native) => {
                        let instance = native.borrow_mut().instantiate(arg_values).unwrap();

                        return ControlFlow::Return(instance);
                    }
                    _ => Value::Error(Rc::new(RefCell::new(value))),
                }
            }
            Expr::This => {
                let this = env.borrow().get("this").unwrap_or(Value::Void);
                this
            }
            Expr::Spread(expr) => Value::Expr(expr.as_ref().clone()),
            _ => {
                todo!("Cannot evaluate expression: {:?}", expr)
            }
        };

        ControlFlow::Return(ret)
    }

    pub fn eval_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut Rc<RefCell<Environment>>,
    ) -> ControlFlow<Value> {
        match stmt {
            Stmt::Let { name, value } => {
                let name = name.clone();
                if env.borrow().exist(&name) {
                    return ControlFlow::new_error(
                        env,
                        format!("Cannot redeclare block-scoped variable '{}'", name),
                    );
                }
                let expr = value.clone();
                let val = self.eval_expr(&expr, env);
                if val.is_err() {
                    return val;
                }
                let val = val.unwrap();

                env.borrow_mut().define(name, val);
                ControlFlow::None
            }
            Stmt::FuncDecl(FunctionStmt {
                name,
                params,
                vararg,
                body,
            }) => {
                let func_env = Environment::new_rc_enclosed(env);
                let function = Function::new(
                    name.clone(),
                    params.clone(),
                    vararg.clone(),
                    body.clone(),
                    func_env,
                    vec![],
                );

                env.borrow_mut()
                    .define(name.clone(), Value::Function(function.into()));
                ControlFlow::None
            }
            Stmt::Return(expr) => {
                if expr.is_none() {
                    return ControlFlow::Return(Value::Void);
                }
                let val = self.eval_expr(&expr.clone().unwrap(), env);

                val
            }
            Stmt::ExprStmt(expr) => {
                // Não retorna valor pois não suporta REPL
                let result = self.eval_expr(expr, env);

                if result.is_err() {
                    return result;
                }
                ControlFlow::None
            }
            Stmt::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                let condition = self.eval_expr(condition, env);
                if condition.is_error() {
                    return condition;
                }

                let condition = condition.unwrap();
                if condition.to_bool() {
                    let inner = Environment::new_rc_enclosed(env);

                    let flow = self.if_block(then_branch, inner);
                    return flow;
                } else {
                    for (cond, branch) in else_ifs {
                        let conditon = self.eval_expr(cond, env);
                        if conditon.is_error() {
                            return conditon;
                        }

                        let conditon = conditon.unwrap();
                        if conditon.to_bool() {
                            let mut local_env =
                                Rc::new(RefCell::new(Environment::new_enclosed(env)));
                            if let Some(branch) = branch {
                                for stmt in branch {
                                    match self.eval_stmt(&stmt, &mut local_env) {
                                        ControlFlow::None => {}
                                        other => return other,
                                    };
                                }
                            }
                            return ControlFlow::None;
                        }
                    }
                    if let Some(else_branch) = else_branch {
                        let mut local_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                        for stmt in else_branch {
                            match self.eval_stmt(&stmt, &mut local_env) {
                                ControlFlow::None => {}
                                other => return other,
                            };
                        }
                    }
                }
                ControlFlow::None
            }
            Stmt::Break => ControlFlow::Break,
            Stmt::Continue => ControlFlow::Continue,
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                let mut loop_env = Environment::new_rc_enclosed(env);
                self.eval_stmt(init, &mut loop_env);

                loop {
                    let cond = match condition {
                        Some(cond) => {
                            let condition = self.eval_expr(cond, &mut loop_env);
                            if condition.is_error() {
                                return condition;
                            }
                            let condition = condition.unwrap();

                            match condition {
                                Value::Bool(b) => b,
                                _ => panic!("Expected boolean {:?}", cond),
                            }
                        }
                        None => true,
                    };

                    if !cond {
                        break;
                    }

                    let inner = Rc::new(RefCell::new(Environment::new_enclosed(&mut loop_env)));

                    let flow = self.loop_block(body, inner);

                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Error(_) | ControlFlow::Return(_) => return flow,
                        ControlFlow::Continue => continue,
                        ControlFlow::None => {}
                    }

                    if let Some(update) = update {
                        self.eval_expr(update, &mut loop_env);
                    }
                }

                ControlFlow::None
            }
            Stmt::ForOf {
                target,
                iterable,
                body,
            } => {
                let iterable_val = self.eval_expr(iterable, env);

                if iterable_val.is_error() {
                    return iterable_val;
                }

                let mut loop_env = Environment::new_rc_enclosed(env);

                let iterable_val = iterable_val.unwrap();

                let iter: Box<dyn Iterator<Item = Value>> = match &iterable_val {
                    Value::Array(arr) => Box::new(arr.get_value().borrow().clone().into_iter()),
                    Value::String(s) => {
                        let s = s.to_string();
                        let chars: Vec<_> = s
                            .chars()
                            .map(|c| Value::String(c.to_string().into()))
                            .collect();
                        Box::new(chars.into_iter())
                    }
                    Value::Instance(instance) => {
                        let collection_class = env.borrow().get("Collection");
                        let collection_class = collection_class.unwrap();

                        if !instance.borrow().is_instance_of(&collection_class) {
                            return ControlFlow::new_error(env, "Not a collection".into());
                        }
                        let iter_method = instance.borrow().get("iter");

                        let arr = iter_method.unwrap().to_method().call(vec![iterable_val]);
                        Box::new(arr.to_array().into_iter())
                    }
                    _ => panic!("Expected array or string in for-of"),
                };

                for val in iter {
                    let mut inner = Rc::new(RefCell::new(Environment::new_enclosed(&mut loop_env)));

                    // Aplicar o padrão de atribuição (identificador ou destructuring)
                    self.destructure(&target, val, &mut inner);

                    let flow = self.loop_block(body, inner);

                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Error(_) | ControlFlow::Return(_) => return flow,
                        ControlFlow::Continue => continue,
                        ControlFlow::None => {}
                    }
                }

                ControlFlow::None
            }
            Stmt::ForIn {
                target,
                object,
                body,
            } => {
                let object_val = self.eval_expr(object, env);

                if object_val.is_error() {
                    return object_val;
                }

                let object_val = object_val.unwrap();
                let mut loop_env = Environment::new_rc_enclosed(env);

                let iter: Box<dyn Iterator<Item = Value>> = match object_val {
                    Value::Object(obj) => {
                        let keys_and_values: Vec<_> = obj
                            .borrow()
                            .iter()
                            .map(|(key, _)| {
                                Value::array(vec![
                                    Value::String(key.clone().into()),
                                    obj.borrow().get_prop(key).unwrap().clone(),
                                ])
                            })
                            .collect();
                        Box::new(keys_and_values.into_iter())
                    }
                    Value::Array(arr) => {
                        let indices_and_values: Vec<_> = arr
                            .get_value()
                            .borrow()
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| Value::array(vec![Value::Number(i.into()), v]))
                            .collect();
                        Box::new(indices_and_values.into_iter())
                    }
                    _ => panic!("Expected object in for-in"),
                };

                for val in iter {
                    let mut inner = Rc::new(RefCell::new(Environment::new_enclosed(&mut loop_env)));

                    // Aplicar o padrão de atribuição (identificador ou destructuring)
                    self.destructure(&target, val, &mut inner);

                    let flow = self.loop_block(body, inner);

                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Error(_) | ControlFlow::Return(_) => return flow,
                        ControlFlow::Continue => continue,
                        ControlFlow::None => {}
                    }
                }

                ControlFlow::None
            }
            Stmt::ClassDecl {
                name,
                superclass,
                methods,
                static_fields,
                instance_fields,
            } => {
                // Primeiro definimos a classe com valor `null` para permitir referências recursivas
                env.borrow_mut().define(name.clone(), Value::Null);

                let mut class_env = Environment::new_rc_enclosed(env);
                let class_closure = class_env.clone();

                // Herdar de outra classe, se houver
                let super_class_value = if let Some(expr) = superclass {
                    let val = self.eval_expr(expr, &mut class_env);
                    if val.is_error() {
                        return val;
                    }

                    let val = val.unwrap();
                    match val {
                        Value::Class(class) => Some(class),
                        _ => panic!("Superclass must be a class"),
                    }
                } else {
                    None
                };

                let instance_variables = Rc::new(RefCell::new(HashMap::new()));

                let instace_env = Environment::new_rc();

                let mut super_class_static_methods = vec![];
                let mut super_class_methods = vec![];

                if super_class_value.is_some() {
                    let super_class = super_class_value.clone().unwrap();

                    for method in super_class.static_methods.clone() {
                        let method = Function::new(
                            method.name.clone(),
                            method.params.clone(),
                            method.vararg.clone(),
                            method.body.clone(),
                            env.clone(),
                            method.modifiers.clone(),
                        );

                        super_class_static_methods.push(Rc::new(method));
                    }
                    for method in super_class.methods.clone() {
                        let mut method_name = method.name.clone();

                        if method_name == "constructor" {
                            method_name = "super".to_owned();
                        }
                        let method = Function::new(
                            method_name,
                            method.params.clone(),
                            method.vararg.clone(),
                            method.body.clone(),
                            class_closure.clone(),
                            method.modifiers.clone(),
                        );

                        super_class_methods.push(Rc::new(method));
                    }

                    for (field_name, value) in super_class.instance_variables.borrow().clone() {
                        instance_variables
                            .borrow_mut()
                            .insert(field_name.to_owned(), value.clone());
                        instace_env
                            .borrow_mut()
                            .define(field_name.to_owned(), value);
                    }
                }

                let mut static_variables = HashMap::new();
                // // Avaliar e definir campos estáticos no ambiente da classe
                for (field_name, initializer) in static_fields {
                    let value = self.eval_expr(initializer, &mut class_env);

                    if value.is_error() {
                        return value;
                    }

                    let value = value.unwrap();
                    static_variables.insert(field_name.to_owned(), value.clone());
                }

                // Avaliar valores iniciais dos campos de instância (armazenados para uso em `instantiate`)
                for (field_name, initializer) in instance_fields {
                    let value = self.eval_expr(initializer, &mut class_env);

                    if value.is_error() {
                        return value;
                    }
                    let value = value.unwrap();

                    instance_variables
                        .borrow_mut()
                        .insert(field_name.to_owned(), value.clone());
                    instace_env
                        .borrow_mut()
                        .define(field_name.to_owned(), value);
                }

                let mut is_constructor_declared = false;
                let mut method_array: Vec<Rc<Function>> = vec![];
                let mut static_method_array: Vec<Rc<Function>> = vec![];

                for method in methods {
                    let method_name = method.name.clone();

                    if method.modifiers.contains_str("static") {
                        let method = Function::new(
                            method.name.clone(),
                            method.params.clone(),
                            method.vararg.clone(),
                            method.body.clone(),
                            env.clone(),
                            method.modifiers.clone(),
                        );

                        static_method_array.push(Rc::new(method))
                    } else {
                        if method_name == "constructor" {
                            if is_constructor_declared {
                                panic!("Only one constructor can be declared");
                            }
                            is_constructor_declared = true;
                        }

                        let method = Function::new(
                            method_name,
                            method.params.clone(),
                            method.vararg.clone(),
                            method.body.clone(),
                            env.clone(),
                            method.modifiers.clone(),
                        );
                        method_array.push(Rc::new(method));
                    }
                }

                let super_class = if let Some(super_class) = super_class_value {
                    Some(super_class)
                } else {
                    None
                };

                // add super statics to static_method_array
                static_method_array.extend(super_class_static_methods);
                method_array.extend(super_class_methods);

                let class = Class {
                    name: name.clone(),
                    superclass: super_class,

                    methods: method_array,
                    static_methods: static_method_array,

                    this: Rc::clone(&instace_env),
                    instance_variables,
                    static_variables,
                    closure: class_closure.clone(),
                };

                let class = Value::Class(class.into());
                env.borrow_mut().define(name.clone(), class.clone());

                ControlFlow::Return(class)
                // ControlFlow::None
            }
            Stmt::Export(inner) => {
                self.eval_stmt(inner, env);

                // Registra o símbolo exportado, se aplicável
                if let Some(name) = self.get_export_name(inner) {
                    self.register_export(&name);
                }
                ControlFlow::None
            }
            Stmt::ExportDefault(expr) => {
                let value = self.eval_stmt(&expr, env);
                let value = value.unwrap().clone();
                env.borrow_mut()
                    .define("default".to_string(), value.clone());

                self.register_export_with_value("default", value);

                ControlFlow::None
            }
            Stmt::ImportNamed { items, from } => {
                let module_env = self.load_module(from).unwrap();

                for (exported, local) in items {
                    let val = module_env
                        .borrow()
                        .get(exported)
                        .expect(&format!("'{}' not exported by '{}'", exported, from));
                    env.borrow_mut().define(local.clone(), val.clone());
                }
                ControlFlow::None
            }

            Stmt::ImportDefault { local_name, from } => {
                let module_env = self.load_module(from).unwrap();
                let val = module_env
                    .borrow()
                    .get("default")
                    .expect(&format!("No default export in '{}'", from));
                env.borrow_mut().define(local_name.clone(), val.clone());
                ControlFlow::None
            }

            Stmt::ImportAll { local_name, from } => {
                let module_env = self.load_module(from).unwrap();
                let mut obj = Vec::new();
                for (k, v) in module_env.borrow().get_vars().iter() {
                    obj.set_prop(k, v.clone()).unwrap();
                }
                env.borrow_mut().define(
                    local_name.clone(),
                    Value::Object(Rc::new(RefCell::new(obj))),
                );
                ControlFlow::None
            }
            Stmt::ImportMixed {
                default,
                items,
                from,
            } => {
                let module_env = self.load_module(from).unwrap();

                let val = module_env
                    .borrow()
                    .get("default")
                    .expect(&format!("No default export in '{}'", from));
                env.borrow_mut().define(default.clone(), val.clone());

                // Named exports
                for (imported, local) in items {
                    let val = module_env
                        .borrow()
                        .get(imported)
                        .expect(&format!("Symbol '{}' not exported by '{}'", imported, from));

                    env.borrow_mut().define(local.clone(), val.clone());
                }
                ControlFlow::None
            }
            Stmt::While { condition, body } => {
                let mut loop_env = Environment::new_rc_enclosed(env);

                let condition = self.eval_expr(condition, &mut loop_env);
                if condition.is_error() {
                    return condition;
                }

                let condition = condition.unwrap();
                while condition.is_truthy() {
                    // let loop_env = Rc::clone(&loop_env);
                    for stmt in body.iter() {
                        match self.eval_stmt(stmt, &mut loop_env) {
                            ControlFlow::Return(v) => return ControlFlow::Return(v),
                            ControlFlow::Break => return ControlFlow::None,
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                            ControlFlow::Error(err) => {
                                panic!("{:?}", err);
                            }
                        }
                    }
                }

                ControlFlow::None
            }
            Stmt::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
            } => {
                let mut try_env = Environment::new_rc_enclosed(env);
                // Tenta executar o bloco `try`
                let result = self.execute_try_block(try_block, &mut try_env);

                let mut catch_env = Environment::new_rc_enclosed(env);

                if let Err(error) = result {
                    // Se houve erro e temos um bloco catch
                    if let Some((err_name, catch_stmts)) = catch_block {
                        let error = if error.is_err() {
                            error
                        } else {
                            Value::Error(Rc::new(RefCell::new(error)))
                        };
                        catch_env.borrow_mut().define(err_name.to_string(), error);

                        for stmt in catch_stmts {
                            let val = self.eval_stmt(stmt, &mut catch_env);
                            match val {
                                ControlFlow::Return(val) => return ControlFlow::Return(val),
                                ControlFlow::Break => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Break not allowed catch block").into(),
                                    )
                                }
                                ControlFlow::Continue => {
                                    return ControlFlow::new_error(
                                        env,
                                        format!("Continue not allowed catch block ").into(),
                                    )
                                }
                                ControlFlow::None => {}
                                error => return error,
                            }
                        }
                    } else {
                        // Sem catch, propaga o erro depois do finally
                        if let Some(finally_stmts) = finally_block {
                            let _ = self.execute_block(finally_stmts, env);
                        }

                        return ControlFlow::Error(error);
                    }
                }

                // Executa o bloco finally sempre
                if let Some(finally_stmts) = finally_block {
                    let _ = self.execute_block(finally_stmts, env);
                }

                ControlFlow::None
            }
            Stmt::Throw(expr) => {
                let value = self.eval_expr(expr, env);
                // let error_class = env.borrow().get("Error");
                // let error_class = error_class.unwrap();
                if value.is_err() {
                    return value;
                } else if value.is_return() {
                    return ControlFlow::Error(value.unwrap());
                } else {
                    return ControlFlow::new_error(
                        env,
                        format!("Throw must be a value. got: {}", value.name()),
                    );
                }
            }
            _ => {
                todo!()
            }
        }
    }

    fn loop_block(
        &mut self,
        body: &Vec<Stmt>,
        mut inner: Rc<RefCell<Environment>>,
    ) -> ControlFlow<Value> {
        for stmt in body {
            match self.eval_stmt(stmt, &mut inner) {
                ControlFlow::Break => return ControlFlow::Break,
                ControlFlow::Continue => break,
                ControlFlow::Return(v) => {
                    return ControlFlow::Return(v);
                }
                ControlFlow::Error(err) => return ControlFlow::Error(err),
                ControlFlow::None => {}
            }
        }
        ControlFlow::None
    }

    fn if_block(
        &mut self,
        body: &Vec<Stmt>,
        mut inner: Rc<RefCell<Environment>>,
    ) -> ControlFlow<Value> {
        for stmt in body {
            match self.eval_stmt(stmt, &mut inner) {
                ControlFlow::Break => return ControlFlow::Break,
                ControlFlow::Continue => break,
                ControlFlow::Return(v) => {
                    if matches!(stmt, &Stmt::Return(_)) {
                        return ControlFlow::Return(v);
                    }
                }
                ControlFlow::Error(err) => return ControlFlow::Error(err),
                ControlFlow::None => {}
            }
        }
        ControlFlow::None
    }

    pub fn execute_try_block(
        &mut self,
        stmts: &Vec<Stmt>,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, Value> {
        for stmt in stmts {
            let val = self.eval_stmt(stmt, env);
            match val {
                ControlFlow::Error(err) => return Err(err),
                _ => {}
            }
        }
        Ok(Value::Void)
    }
    pub fn execute_block(
        &mut self,
        stmts: &Vec<Stmt>,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, Value> {
        for stmt in stmts {
            let val = self.eval_stmt(stmt, env);
            // println!("Executando {:?} {:?}", stmt, val);
            match val {
                ControlFlow::Return(val) => return Ok(val),
                ControlFlow::Break => return Ok(Value::Void),
                ControlFlow::Continue => return Ok(Value::Void),
                ControlFlow::None => return Ok(Value::Void),
                ControlFlow::Error(err) => return Err(err),
            }
        }
        Ok(Value::Void)
    }
    fn register_export_with_value(&mut self, name: &str, value: Value) {
        self.exported_symbols.insert(name.to_string(), value);
    }
    fn load_module(&mut self, path: &str) -> Result<Rc<RefCell<Environment>>, String> {
        if let Some(cached) = self.module_cache.get(path) {
            return Ok(cached.clone());
        }

        let source = std::fs::read_to_string(path).unwrap();
        let tokens = self.tokenize(source, path.to_string());
        let ast = Parser::new(tokens).parse();

        let mut module_env = Environment::new_rc();
        self.exported_symbols.clear();

        for stmt in ast {
            self.eval_stmt(&stmt, &mut module_env);
        }

        // println!("Env: {:?}",module_env.borrow_mut().get_vars_name_value());
        let export_only_env = Environment::new_rc();
        for (name, value) in &self.exported_symbols {
            if let Some(val) = module_env.borrow().get(name) {
                let value = if value.is_null() { val } else { value.clone() };
                export_only_env.borrow_mut().define(name.clone(), value);
            }
        }

        self.module_cache
            .insert(path.to_string(), export_only_env.clone());
        Ok(export_only_env)
    }
    fn register_export(&mut self, name: &str) {
        self.exported_symbols.insert(name.to_string(), Value::Null);
    }
    fn get_export_name(&self, stmt: &Stmt) -> Option<String> {
        match stmt {
            Stmt::Let { name, .. } => Some(name.clone()),
            Stmt::FuncDecl(FunctionStmt { name, .. }) => Some(name.clone()),
            Stmt::ClassDecl { name, .. } => Some(name.clone()),
            // adicione outras formas se precisar
            _ => None,
        }
    }

    fn binary_operation(&self, op: Operator, left: Expr, right: Expr) -> Expr {
        return Expr::BinaryOp {
            op: op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    fn resolve_variable(
        &self,
        name: &str,
        env: &mut Rc<RefCell<Environment>>,
    ) -> Result<Value, String> {
        // 1. tenta no ambiente atual (local)
        if let Some(val) = env.borrow().get(name) {
            return Ok(val);
        }

        // 3. erro variável não encontrada
        Err(format!("Undefined variable '{}'.", name))
    }

    fn destructure(&self, pattern: &Expr, value: Value, env: &mut Rc<RefCell<Environment>>) {
        match pattern {
            Expr::Identifier(name) => {
                env.borrow_mut().define(name.clone(), value);
            }
            Expr::Literal(Literal::Array(patterns)) => {
                let val_arr = match value {
                    Value::Array(arr) => arr,
                    _ => panic!("Expected array for destructuring"),
                };
                for (i, pat) in patterns.iter().enumerate() {
                    if let Some(v) = val_arr.get_value().borrow().get(i) {
                        self.destructure(pat, v.clone(), env);
                    }
                }
            }
            Expr::Literal(Literal::Object(entries)) => {
                let val_obj = match value {
                    Value::Object(obj) => obj,
                    _ => panic!("Expected object for destructuring"),
                };

                for entry in entries {
                    match entry {
                        ObjectEntry::Property { key, value } => {
                            let val = val_obj.borrow().get_prop(key).unwrap_or(Value::Null);
                            self.destructure(value, val, env);
                        }
                        ObjectEntry::Shorthand(name) => {
                            let val = val_obj.borrow().get_prop(name).unwrap_or(Value::Null);
                            self.destructure(&Expr::Identifier(name.clone()), val, env);
                        }
                        ObjectEntry::Spread(_) => {
                            panic!("Unexpected spread in object literal during destructuring");
                        }
                    }
                }
            }
            _ => panic!("Invalid destructuring target"),
        }
    }

    pub fn resolve_calle_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Identifier(name) => name.clone(),
            Expr::GetProperty { object, property } => {
                let object_name = self.resolve_calle_name(object);
                let property_name = match property.as_ref() {
                    Expr::Identifier(name) => name.clone(),
                    _ => panic!("Expected identifier for property name"),
                };
                format!("{}.{}", object_name, property_name)
            }
            Expr::Literal(l) => format!("{:?}", l.to_string()),
            _ => panic!("Expected identifier for callee name"),
        }
    }
}
