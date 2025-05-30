use logos::Logos;
use std::{cell::RefCell, collections::HashMap, env, fs, path::Path, rc::Rc};

use crate::{
    ast::ast::{
        BinaryOperator, CompareOperator, ControlFlow, Expr, FunctionStmt, Literal, LogicalOperator,
        ObjectEntry, Operator, Stmt,
    },
    environment::{
        values::{Class, Function, Method, Value},
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
    pub env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new(source: String) -> Self {
        let env = Environment::new_rc();
        // let env = Rc::new(RefCell::new(Environment::new_enclosed(global.clone())));

        Self {
            source,
            module_cache: HashMap::new(),
            exported_symbols: HashMap::new(),
            env: env,
        }
    }

    pub fn new_empty() -> Self {
        let env = Environment::new_rc();

        Self {
            source: String::new(),
            module_cache: HashMap::new(),
            exported_symbols: HashMap::new(),
            env,
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
        let show_ast = show_ast == "true";
        let src = fs::read_to_string(&filename).unwrap_or(self.source.clone());

        let tokens = self.tokenize(src.clone(), filename.clone());
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        // bench();
        if show_ast {
            println!("{:#?}", ast);
        }

        for stmt in ast {
            let val = self.eval_stmt(&stmt, self.env.clone());
            if let ControlFlow::Return(v) = val {
                return Some(v);
            }
        }
        None
    }

    pub fn interpret(&mut self) -> Option<Value> {
        let filename = env::args().nth(1).expect("File not found");
        self.interpret_from_file(filename)
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
    pub fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Environment>>) -> Value {
        match expr {
            Expr::Identifier(name) => {
                let value = self.resolve_variable(name, env);
                value
            }
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(*n),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::String(s) => Value::String(s.clone().into()),
                Literal::Null => Value::Null,
                Literal::Void => Value::Void,
                Literal::Object(entries) => {
                    let mut result = std::collections::HashMap::new();

                    for entry in entries {
                        match entry {
                            ObjectEntry::Property { key, value } => {
                                let val = self.eval_expr(value, env.clone());
                                result.insert(key.clone(), val);
                            }
                            ObjectEntry::Shorthand(name) => {
                                // Resolve a variável no ambiente
                                let val = env.borrow().get(name).unwrap_or(Value::Null); // ou trate erro se preferir
                                result.insert(name.clone(), val);
                            }
                            ObjectEntry::Spread(expr) => {
                                let val = self.eval_expr(expr, env.clone());
                                match val {
                                    Value::Object(map) => {
                                        let map = map.borrow_mut().clone();
                                        for (k, v) in map {
                                            result.insert(k, v);
                                        }
                                    }
                                    _ => {
                                        // opcional: erro em tempo de execução
                                        panic!("Spread operator must be used with an object");
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
                        let val = self.eval_expr(elem, env.clone());
                        elements.push(val);
                    }
                    Value::array(elements)
                }
            },
            Expr::Block(stmts) => {
                let local_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                for stmt in stmts {
                    let ret = self.eval_stmt(stmt, local_env.clone());
                    match ret {
                        ControlFlow::Return(value) => return value,
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::None => {}
                    };
                }
                Value::Void
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, env.clone());
                let r = self.eval_expr(right, env.clone());
                match (op, l, r) {
                    (Operator::Binary(math_op), left, right) => match math_op {
                        BinaryOperator::Add => match (&left, &right) {
                            (Value::String(a), Value::String(b)) => Value::String((a + b).into()),
                            (Value::String(a), b) => Value::String(a + b.to_string()),
                            (a, Value::String(b)) => {
                                Value::String((a.to_string() + &b.to_string()).into())
                            }
                            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                            _ => panic!("Invalid operands for +: {:?} and {:?}", left, right),
                        },
                        BinaryOperator::Subtract => {
                            Value::Number(left.to_number() - right.to_number())
                        }
                        BinaryOperator::Multiply => {
                            Value::Number(left.to_number() * right.to_number())
                        }
                        BinaryOperator::Divide => {
                            Value::Number(left.to_number() / right.to_number())
                        }
                        BinaryOperator::Exponentiate => {
                            Value::Number(left.to_number().powf(right.to_number()))
                        }
                        BinaryOperator::Modulo => {
                            Value::Number(left.to_number() % right.to_number())
                        }
                    },

                    (Operator::Compare(comp_op), Value::Number(a), Value::Number(b)) => {
                        match comp_op {
                            CompareOperator::Eq => Value::Bool(a == b),
                            CompareOperator::Ne => Value::Bool(a != b),
                            CompareOperator::Gt => Value::Bool(a > b),
                            CompareOperator::Ge => Value::Bool(a >= b),
                            CompareOperator::Lt => Value::Bool(a < b),
                            CompareOperator::Le => Value::Bool(a <= b),
                        }
                    }

                    (Operator::Logical(log_op), Value::Bool(a), Value::Bool(b)) => match log_op {
                        LogicalOperator::And => Value::Bool(a && b),
                        LogicalOperator::Or => Value::Bool(a || b),
                    },

                    _ => Value::Null, // fallback
                }
            }
            Expr::UnaryOp { op, expr, postfix } => {
                let val = self.eval_expr(expr, env.clone());
                match op {
                    crate::ast::ast::UnaryOperator::Negative => Value::Number(-val.to_number()),
                    crate::ast::ast::UnaryOperator::Not => Value::Bool(!val.to_bool()),
                    crate::ast::ast::UnaryOperator::Typeof => Value::String(val.type_of().into()),
                    crate::ast::ast::UnaryOperator::Increment => {
                        let new_val = val.to_number() + 1.0;
                        match expr.as_ref() {
                            Expr::Identifier(name) => {
                                let name = name.clone();
                                let previous_val = env.borrow().get(&name).unwrap();
                                env.borrow_mut()
                                    .assign(&name, Value::Number(new_val))
                                    .unwrap();

                                if *postfix {
                                    previous_val
                                } else {
                                    Value::Number(new_val)
                                }
                            }
                            Expr::Literal(literal) => match literal {
                                Literal::Number(number) => {
                                    let new_val = number + 1.0;

                                    if *postfix {
                                        Value::Number(*number)
                                    } else {
                                        Value::Number(new_val)
                                    }
                                }
                                _ => {
                                    panic!("Invalid value for increment: {:?}", expr)
                                }
                            },
                            _ => panic!("Invalid operand for increment: {:?}", expr),
                        }
                    }
                    crate::ast::ast::UnaryOperator::Decrement => {
                        let new_val = val.to_number() - 1.0;
                        let name = match expr.as_ref() {
                            Expr::Identifier(name) => name.clone(),
                            _ => panic!("Invalid operand for decrement: {:?}", expr),
                        };
                        env.borrow_mut()
                            .assign(&name, Value::Number(new_val))
                            .unwrap();
                        Value::Number(new_val)
                    }
                    crate::ast::ast::UnaryOperator::Positive => Value::Number(val.to_number()),
                }
            }
            Expr::Call { callee, args } => {
                let evaluated_callee = self.eval_expr(callee, env.clone());

                let arg_values: Vec<_> = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env.clone()))
                    .collect();

                match evaluated_callee {
                    Value::Function(func) => {
                        let name = &func.name;
                        let params = &func.params;
                        let body = &func.body;

                        let is_initializer = name == "init";

                        let local_env = func.environment.borrow().clone().from_parent(env).to_rc();

                        for (param, val) in params.iter().zip(arg_values) {
                            local_env.borrow_mut().define(param.clone(), val);
                        }

                        for stmt in body {
                            match self.eval_stmt(stmt, local_env.clone()) {
                                ControlFlow::Return(_val) if is_initializer => {
                                    return local_env
                                        .borrow()
                                        .get("this")
                                        .unwrap_or(Value::Null)
                                        .clone();
                                }
                                ControlFlow::Return(val) => return val,
                                ControlFlow::Break => {
                                    panic!("Break not allowed in function {name}")
                                }
                                ControlFlow::Continue => {
                                    panic!("Continue not allowed in function {name}")
                                }
                                ControlFlow::None => {}
                            }
                        }

                        if is_initializer {
                            return local_env
                                .borrow()
                                .get("this")
                                .unwrap_or(Value::Null)
                                .clone();
                        }
                        Value::Null
                    }
                    Value::Method(method) => method.call(arg_values, self.clone()),

                    Value::Builtin(func) => func(arg_values),
                    Value::NativeFunction((name, native_class)) => {
                        let mut new_args = native_class.borrow().get_args();
                        let is_static = native_class.borrow().is_static();

                        if is_static {
                            return native_class
                                .borrow()
                                .call_with_args(&name, arg_values)
                                .unwrap();
                        }
                        // concat new_args and arg_values
                        new_args.extend(arg_values);

                        native_class.borrow_mut().add_args(new_args).unwrap();

                        native_class.borrow().call(&name).unwrap()
                    }
                    _ => panic!("'{}' is not a function", self.resolve_calle_name(callee)),
                }
            }
            Expr::Assign {
                target,
                op: _,
                value,
            } => {
                let val = self.eval_expr(value, env.clone());

                match &**target {
                    Expr::Identifier(name) => {
                        // atribuição simples
                        let mut env_mut = env.borrow_mut();
                        env_mut.assign(name, val.clone()).unwrap();
                    }
                    Expr::GetProperty { object, property } => {
                        // atribuição a propriedade: obj.prop = val
                        let obj = self.eval_expr(object, env.clone());
                        let key = match &**property {
                            Expr::Identifier(name) => name.clone(),
                            _ => panic!("Propriedade inválida"),
                        };

                        match obj {
                            Value::Object(instance) => {
                                instance.borrow_mut().insert(key, val.clone());
                            }

                            Value::Instance(instance) => {
                                instance.borrow_mut().set(&key, val.clone());
                            }
                            _ => panic!("Não é um objeto {:?}", obj),
                        }
                    }
                    Expr::BracketAccess { object, property } => {
                        // atribuição por índice: arr[i] = val
                        let arr = self.eval_expr(object, env.clone());
                        let index = self.eval_expr(property, env.clone());

                        match (arr, index) {
                            (Value::Array(array), Value::Number(n)) => {
                                let idx = n as usize;
                                let get_value = array.get_value();
                                let mut array = get_value.borrow_mut();
                                let item = array.get(idx);
                                if item.is_some() {
                                    array[idx] = val.clone();
                                } else {
                                    array.insert(idx, val.clone());
                                }
                            }
                            (Value::Object(obj), Value::String(key)) => {
                                obj.borrow_mut().insert(key.to_string(), val.clone());
                            }
                            _ => panic!("Acesso inválido para atribuição"),
                        }
                    }
                    _ => panic!("Expressão inválida no lado esquerdo da atribuição"),
                }

                Value::Void
            }
            Expr::GetProperty { object, property } => {
                let obj = self.eval_expr(object, env.clone());

                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string().into()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone().into()),
                    _ => return Value::Null,
                };

                if obj.is_native_class() && prop.is_string() {
                    let prop = prop.to_string();

                    let native = obj.get_native_class().unwrap();
                    return Value::NativeFunction((prop, native.clone()));
                }
                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let prop = prop.get_value();
                        let msg = format!(
                            "Property {prop} not found {}",
                            env.borrow().get_vars_name_value()
                        );
                        obj.borrow().get(&prop).cloned().expect(&msg)
                    }
                    (Value::Instance(instance), Value::String(prop)) => {
                        let class_name = instance.borrow().class.name.clone();

                        let msg = format!("Cannot find '{prop}' in class {class_name}");

                        let value = instance.borrow().get(&prop.to_string());

                        return value.expect(&msg);
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        let class_name = &class.name;
                        let msg = format!("Cannot find static '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.get_static_field(&name) {
                            return val;
                        }
                        if let Some(val) = class.find_static_method(&name) {
                            return Value::Method(val);
                        }
                        panic!("{}", msg)
                    }
                    (Value::NativeClass(native), Value::String(prop)) => {
                        return Value::NativeFunction((prop.to_string(), native.clone()));
                    }
                    (Value::Function(_func), Value::String(prop)) => {
                        let prop = prop.get_value();

                        let msg = format!("Property {prop} not found");
                        todo!("Entrou aqui {prop} {msg}");
                    }
                    _ => {
                        panic!(
                            "Cannot access property {:?} of {:?} (type: {:?}, {:?})",
                            prop.to_string(),
                            obj.to_string(),
                            &obj.type_of(),
                            &prop.type_of()
                        )
                    }
                }
            }
            Expr::BracketAccess { object, property } => {
                let obj = self.eval_expr(object, env.clone());
                let prop = self.eval_expr(property, env.clone());

                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let prop = prop.to_string();
                        let msg = format!("Property {prop} not found");
                        obj.borrow().get(&prop).cloned().expect(&msg)
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
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
                        let index = *index as usize;
                        arr.chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string().into()))
                            .unwrap_or(Value::Null)
                    }
                    _ => panic!(
                        "Cannot access property {:?} of {:?} (type: {:?}, {:?})",
                        prop.to_string(),
                        obj.to_string(),
                        &obj.type_of(),
                        &prop.type_of()
                    ),
                }
            }
            Expr::SetProperty {
                object,
                property,
                value,
            } => {
                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string().into()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone().into()),
                    _ => return Value::Null,
                };

                let obj = self.eval_expr(object, env.clone());

                let val = self.eval_expr(value, env.clone());

                match (obj.clone(), &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg = format!("Property {prop} not found");
                        obj.borrow_mut()
                            .insert(prop.clone().to_string(), val)
                            .expect(&msg);
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
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
                    _ => panic!(
                        "Cannot access property {:?} of {:?} (type: {:?}, {:?})",
                        prop.to_string(),
                        obj.to_string(),
                        &obj.type_of(),
                        &prop.type_of()
                    ),
                }
                Value::Void
            }
            Expr::New { class_name, args } => {
                let arg_values: Vec<_> = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env.clone()))
                    .collect();

                let value = env
                    .borrow()
                    .get(class_name)
                    .expect(&format!("Class '{}' not found", class_name));

                match value {
                    Value::Class(class) => {
                        let mut interpreter = self.clone();
                        let closure = class.closure.clone();
                        // println!("Env: {:?}", closure.borrow().get_vars_name_value());
                        interpreter.env = closure;
                        let instance = class.instantiate(arg_values, interpreter);

                        return instance;
                    }
                    Value::NativeClass(native) => {
                        let instance = native.borrow().instantiate(arg_values).unwrap();

                        return instance;
                    }
                    _ => panic!("'{}' is not a class.", class_name),
                }
            }
            Expr::This => env.borrow().get("this").unwrap_or(Value::Null),
            _ => {
                todo!("Cannot evaluate expression: {:?}", expr)
            }
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt, env: Rc<RefCell<Environment>>) -> ControlFlow<Value> {
        match stmt {
            Stmt::Let { name, value } => {
                let name = name.clone();
                if env.borrow().exist(&name) {
                    panic!("Cannot redeclare block-scoped variable '{}'", name);
                }
                let expr = value.clone().unwrap_or(Expr::Literal(Literal::Null));
                let val = self.eval_expr(&expr, env.clone());
                env.borrow_mut().define(name, val);
                ControlFlow::None
            }
            Stmt::FuncDecl(FunctionStmt { name, params, body }) => {
                let mut function = Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    environment: Environment::new_rc_enclosed(env.clone()),
                    prototype: None,
                };
                let _ = function.generate_proto();

                env.borrow_mut()
                    .define(name.clone(), Value::Function(function.into()));
                ControlFlow::None
            }
            Stmt::Return(expr) => {
                let val = self.eval_expr(&expr.clone().unwrap(), env.clone());
                ControlFlow::Return(val)
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr, env);
                ControlFlow::None
            }
            Stmt::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                if self.eval_expr(condition, env.clone()).to_bool() {
                    for stmt in then_branch {
                        let local_env =
                            Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                        match self.eval_stmt(stmt, local_env) {
                            ControlFlow::None => {}
                            other => return other,
                        }
                    }
                } else {
                    for (cond, branch) in else_ifs {
                        if self.eval_expr(cond, env.clone()).to_bool() {
                            if let Some(branch) = branch {
                                for stmt in branch {
                                    let local_env = Rc::new(RefCell::new(
                                        Environment::new_enclosed(env.clone()),
                                    ));
                                    return self.eval_stmt(&stmt, local_env);
                                }
                            }
                            return ControlFlow::None;
                        }
                    }
                    if let Some(else_branch) = else_branch {
                        for stmt in else_branch {
                            let local_env =
                                Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                            return self.eval_stmt(&stmt, local_env);
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
                let loop_env = Environment::new_rc_enclosed(env.clone());
                self.eval_stmt(init, loop_env.clone());

                loop {
                    let cond = match condition {
                        Some(cond) => match self.eval_expr(cond, loop_env.clone()) {
                            Value::Bool(b) => b,
                            _ => panic!("Expected boolean {:?}", cond),
                        },
                        None => true,
                    };

                    if !cond {
                        break;
                    }

                    let inner = Rc::new(RefCell::new(Environment::new_enclosed(loop_env.clone())));

                    for stmt in body {
                        match self.eval_stmt(stmt, inner.clone()) {
                            ControlFlow::Return(v) => return ControlFlow::Return(v),
                            ControlFlow::Break => return ControlFlow::None,
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }

                    if let Some(update) = update {
                        self.eval_expr(update, loop_env.clone());
                    }
                }

                ControlFlow::None
            }
            Stmt::ForOf {
                target,
                iterable,
                body,
            } => {
                let iterable_val = self.eval_expr(iterable, env.clone());

                let loop_env = Environment::new_rc_enclosed(env);

                let iter: Box<dyn Iterator<Item = Value>> = match iterable_val {
                    Value::Array(arr) => Box::new(arr.get_value().borrow().clone().into_iter()),
                    Value::String(s) => {
                        let s = s.to_string();
                        let chars: Vec<_> = s
                            .chars()
                            .map(|c| Value::String(c.to_string().into()))
                            .collect();
                        Box::new(chars.into_iter())
                    }
                    _ => panic!("Expected array or string in for-of"),
                };

                for val in iter {
                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
                        &loop_env,
                    ))));

                    // Aplicar o padrão de atribuição (identificador ou destructuring)
                    self.destructure(&target, val, inner_env.clone());

                    for stmt in body.iter() {
                        match self.eval_stmt(stmt, inner_env.clone()) {
                            ControlFlow::Return(v) => return ControlFlow::Return(v),
                            ControlFlow::Break => return ControlFlow::None,
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }
                }

                ControlFlow::None
            }
            Stmt::ForIn {
                target,
                object,
                body,
            } => {
                let object_val = self.eval_expr(object, env.clone());

                let loop_env = Environment::new_rc_enclosed(env);

                let iter: Box<dyn Iterator<Item = Value>> = match object_val {
                    Value::Object(obj) => {
                        let keys_and_values: Vec<_> = obj
                            .borrow()
                            .keys()
                            .map(|key| {
                                Value::array(vec![
                                    Value::String(key.clone().into()),
                                    obj.borrow().get(key).unwrap().clone(),
                                ])
                            })
                            .collect();
                        Box::new(keys_and_values.into_iter())
                    }
                    _ => panic!("Expected object in for-in"),
                };

                for val in iter {
                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
                        &loop_env,
                    ))));

                    // Aplicar o padrão de atribuição (identificador ou destructuring)
                    self.destructure(&target, val, inner_env.clone());

                    for stmt in body.iter() {
                        match self.eval_stmt(stmt, inner_env.clone()) {
                            ControlFlow::Return(v) => return ControlFlow::Return(v),
                            ControlFlow::Break => return ControlFlow::None,
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
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
                // env.borrow_mut().define(name.clone(), Value::Null);

                let class_env = Environment::new_rc_enclosed(Rc::clone(&env));

                // Herdar de outra classe, se houver
                let super_class_value = if let Some(expr) = superclass {
                    let val = self.eval_expr(expr, Rc::clone(&class_env));
                    match val {
                        Value::Class(class) => Some(class),
                        _ => panic!("Superclass must be a class"),
                    }
                } else {
                    None
                };

                // Construção do mapa de métodos

                let static_method_env = Environment::new_rc();
                let mut static_variables = HashMap::new();
                // // Avaliar e definir campos estáticos no ambiente da classe
                for (field_name, initializer) in static_fields {
                    let value = self.eval_expr(initializer, Rc::clone(&class_env));
                    static_variables.insert(field_name.to_owned(), value.clone());
                    static_method_env
                        .borrow_mut()
                        .define(field_name.to_string(), value);
                }

                // Avaliar valores iniciais dos campos de instância (armazenados para uso em `instantiate`)
                let mut instance_variables = HashMap::new();
                let instace_env = Environment::new_rc();
                for (field_name, initializer) in instance_fields {
                    let value = self.eval_expr(initializer, Rc::clone(&class_env));
                    instance_variables.insert(field_name.to_owned(), value.clone());
                    instace_env
                        .borrow_mut()
                        .define(field_name.to_owned(), value);
                }

                // // Criar novo escopo para métodos (especialmente para `super`)
                // let method_env = if super_class_value.is_some() {
                //     instace_env.borrow_mut().define(
                //         "super".to_string(),
                //         Value::Class(super_class_value.clone().unwrap()),
                //     );
                //     Rc::clone(&instace_env)
                // } else {
                //     Rc::clone(&instace_env)
                // };

                let mut method_array: Vec<Rc<Method>> = vec![];
                let mut static_method_array: Vec<Rc<Method>> = vec![];
                for method in methods {
                    if method.is_static {
                        let method = Method::new(
                            method.name.clone(),
                            method.params.clone(),
                            method.body.clone(),
                            Rc::clone(&static_method_env),
                            method.is_static,
                            name.clone(),
                            env.clone(),
                        );

                        static_method_array.push(Rc::new(method))
                    } else {
                        let method = Method::new(
                            method.name.clone(),
                            method.params.clone(),
                            method.body.clone(),
                            instace_env.clone(),
                            method.is_static,
                            name.clone(),
                            env.clone(),
                        );
                        method_array.push(Rc::new(method));
                    }
                }

                let super_class = if let Some(super_class) = super_class_value {
                    Some(Box::new(Value::Class(super_class)))
                } else {
                    None
                };

                let class = Class {
                    name: name.clone(),
                    superclass: super_class,

                    methods: method_array,
                    statics_methods: static_method_array,

                    this: Rc::clone(&instace_env),
                    instance_variables,
                    static_variables,
                    closure: env.clone(),
                };

                // self.classes.insert(name.clone(), class.clone());
                env.borrow_mut()
                    .define(name.clone(), Value::Class(Rc::new(class.clone())));

                // ControlFlow::Return(Value::Class(Rc::new(class)))
                ControlFlow::None
            }
            Stmt::Export(inner) => {
                self.eval_stmt(inner, env.clone());

                // Registra o símbolo exportado, se aplicável
                if let Some(name) = self.get_export_name(inner) {
                    // println!("Exporting {:?}", name);
                    self.register_export(&name);
                }
                ControlFlow::None
            }
            Stmt::ExportDefault(expr) => {
                let value = self.eval_stmt(&expr, env.clone());
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
                let mut obj = std::collections::HashMap::new();
                for (k, v) in module_env.borrow().get_vars().iter() {
                    obj.insert(k.clone(), v.clone());
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
            _ => {
                todo!()
            }
        }
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

        let module_env = Environment::new_rc();
        self.exported_symbols.clear();

        for stmt in ast {
            self.eval_stmt(&stmt, module_env.clone());
        }

        // println!("Env: {:?}",module_env.borrow_mut().get_vars_name_value());
        let export_only_env = Environment::new_rc();
        for (name, value) in &self.exported_symbols {
            if let Some(val) = module_env.borrow().get(name) {
                let value = if value.is_null() {
                    val.clone()
                } else {
                    value.clone()
                };
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
    fn resolve_variable(&self, name: &str, env: Rc<RefCell<Environment>>) -> Value {
        // 1. tenta no ambiente atual (local)
        if let Some(val) = env.borrow().get(name) {
            return val;
        }

        // 3. erro variável não encontrada
        panic!("Undefined variable '{}'.", name,);
    }

    fn destructure(&self, pattern: &Expr, value: Value, env: Rc<RefCell<Environment>>) {
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
                        self.destructure(pat, v.clone(), env.clone());
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
                            let val = val_obj.borrow().get(key).cloned().unwrap_or(Value::Null);
                            self.destructure(value, val, env.clone());
                        }
                        ObjectEntry::Shorthand(name) => {
                            let val = val_obj.borrow().get(name).cloned().unwrap_or(Value::Null);
                            self.destructure(&Expr::Identifier(name.clone()), val, env.clone());
                        }
                        ObjectEntry::Spread(_) => {
                            // Ignore here — spread in destructuring is handled via Pattern::Rest
                            // If reached, this is likely a misuse.
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
