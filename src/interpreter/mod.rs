use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::ast::{
        BinaryOperator, CompareOperator, Expr, FunctionStmt, Literal, LogicalOperator,
        MathOperator, Stmt,
    },
    environment::{Class, ControlFlow, Environment, Function, Instance, Method, Value},
};

pub struct Interpreter {
    ast: Vec<Stmt>,
    classes: HashMap<String, Class>,
}

impl Interpreter {
    pub fn new(ast: Vec<Stmt>) -> Self {
        Self {
            ast,
            classes: HashMap::new(),
        }
    }

    pub fn interpret(&mut self) {
        let ast = self.ast.clone();

        let global = Rc::new(RefCell::new(Environment::new()));
        let env = Rc::new(RefCell::new(Environment::new_enclosed(global.clone())));

        for stmt in ast {
            let val = self.exec_stmt(&stmt, env.clone());
            if let ControlFlow::Return(v) = val {
                println!("{:?}", v);
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Environment>>) -> Value {
        match expr {
            Expr::Identifier(name) => {
                let value = self.resolve_variable(name, env);
                value
            }
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(*n),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Null => Value::Null,
                Literal::Void => Value::Void,
                Literal::Object(obj) => {
                    let mut properties = std::collections::HashMap::new();
                    for (key, value) in obj {
                        let val = self.eval_expr(value, env.clone());
                        properties.insert(key.clone(), val);
                    }
                    Value::object(properties)
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
                    let ret = self.exec_stmt(stmt, local_env.clone());
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
                    (BinaryOperator::Math(math_op), Value::Number(a), Value::Number(b)) => {
                        match math_op {
                            MathOperator::Add => Value::Number(a + b),
                            MathOperator::Sub => Value::Number(a - b),
                            MathOperator::Mul => Value::Number(a * b),
                            MathOperator::Div => Value::Number(a / b),
                        }
                    }

                    (BinaryOperator::Compare(comp_op), Value::Number(a), Value::Number(b)) => {
                        match comp_op {
                            CompareOperator::Eq => Value::Bool(a == b),
                            CompareOperator::Ne => Value::Bool(a != b),
                            CompareOperator::Gt => Value::Bool(a > b),
                            CompareOperator::Ge => Value::Bool(a >= b),
                            CompareOperator::Lt => Value::Bool(a < b),
                            CompareOperator::Le => Value::Bool(a <= b),
                        }
                    }

                    (BinaryOperator::Logical(log_op), Value::Bool(a), Value::Bool(b)) => {
                        match log_op {
                            LogicalOperator::And => Value::Bool(a && b),
                            LogicalOperator::Or => Value::Bool(a || b),
                        }
                    }

                    _ => Value::Null, // fallback
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

                        let is_static = func.is_static;

                        let local_env = func.environment.borrow().clone().from_parent(env).to_rc();

                        for (param, val) in params.iter().zip(arg_values) {
                            local_env.borrow_mut().define(param.clone(), val);
                        }

                        for stmt in body {
                            match self.exec_stmt(stmt, local_env.clone()) {
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
                    Value::Method(method) => {
                        let name = &method.name;
                        let params = &method.params;
                        let body = &method.body;

                        let is_initializer = name == "init";

                        let local_env = method.this.borrow().clone().from_parent(env).to_rc();

                        for (param, val) in params.iter().zip(arg_values) {
                            local_env.borrow_mut().define(param.clone(), val);
                        }
                        // local_env
                        //     .borrow_mut()
                        //     .merge_environments(env.borrow().clone());

                        for stmt in body {
                            match self.exec_stmt(stmt, local_env.clone()) {
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

                        //    local_env.borrow_mut().clear();
                        if is_initializer {
                            return local_env
                                .borrow()
                                .get("this")
                                .unwrap_or(Value::Null)
                                .clone();
                        }
                        Value::Null
                    }

                    Value::Builtin(func) => func(arg_values),

                    other => panic!("Cannot call non-function value: {:?}", other),
                }
            }
            Expr::Assign { name, value } => {
                let val = self.eval_expr(value, env.clone());
                let mut env_mut = env.borrow_mut();

                env_mut.assign(name, val).unwrap();
                Value::Void
            }
            Expr::GetProperty { object, property } => {
                let obj = self.eval_expr(object, env.clone());

                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone()),
                    _ => return Value::Null,
                };

                match (&obj, &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg = format!(
                            "Property {prop} not found {:?}",
                            env.borrow().get_vars_name_value()
                        );
                        obj.borrow().get(prop).cloned().expect(&msg)
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        let msg = format!("Index {index} out of bounds");
                        arr.borrow().get(index).cloned().expect(&msg)
                    }
                    (Value::String(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        let msg = format!("Index {index} out of bounds");
                        arr.chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .expect(&msg)
                    }
                    (Value::Instance(instance), Value::String(prop)) => {
                        let class_name = instance.class.name.clone();

                        let msg = format!(
                            "Cannot find '{prop}' in class {class_name} {:?}",
                            env.borrow().get_vars_name_value()
                        );

                        let value = instance.get(prop);

                        return value.expect(&msg);
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        let class_name = &class.name;
                        let msg = format!("Cannot find static '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.find_static_method(&name) {
                            return Value::Method(val);
                        }
                        panic!("{}", msg)
                    }
                    (Value::Builtin(func), Value::String(prop)) => {
                        let class = func(vec![]).to_class();
                        let class_name = &class.name;
                        let msg = format!("Cannot find '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.find_static_method(&name) {
                            return Value::Method(val);
                        }
                        // 1. Tentar buscar o campo estático no ambiente 'this' da classe
                        if let Some(val) = class.this.borrow().get(&name) {
                            return val;
                        }

                        panic!("{}", msg)
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
                        let msg = format!("Property {prop} not found");
                        obj.borrow().get(prop).cloned().expect(&msg)
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        let length = arr.borrow().len();
                        let msg = format!("Array out of bounds. Index: {index} length: {length}");
                        let value = arr.borrow().get(index).cloned().expect(&msg);

                        value
                    }
                    (Value::String(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        let msg = format!("Index {index} out of bounds");
                        arr.chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .expect(&msg)
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
                    Expr::Identifier(name) => Value::String(name.to_string()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone()),
                    _ => return Value::Null,
                };

                let obj = self.eval_expr(object, env.clone());

                let val = self.eval_expr(value, env.clone());

                match (obj.clone(), &prop) {
                    (Value::Object(obj), Value::String(prop)) => {
                        let msg = format!("Property {prop} not found");
                        obj.borrow_mut().insert(prop.clone(), val).expect(&msg);
                    }
                    (Value::Array(arr), Value::Number(index)) => {
                        let index = *index as usize;
                        // let msg = format!("Index {index} out of bounds");
                        let mut arr = arr.borrow_mut();
                        arr[index] = val;
                    }
                    (Value::Instance(instance), Value::String(prop)) => {
                        instance.this.borrow_mut().assign(prop, val).unwrap();
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        class.this.borrow().get(prop).unwrap_or(Value::Null);
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
                let class = self
                    .classes
                    .get(class_name)
                    .ok_or("Class '{class_name}' not found ");

                let class = class.unwrap();

                let instance = class.instantiate();

                return instance;
            }
            Expr::This => env.borrow().get("this").unwrap_or(Value::Null),
            _ => {
                todo!("Cannot evaluate expression: {:?}", expr)
            }
        }
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt, env: Rc<RefCell<Environment>>) -> ControlFlow {
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
                let function = Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    environment: Environment::new_rc_enclosed(env.clone()),
                    is_static: false,
                };

                env.borrow_mut()
                    .define(name.clone(), Value::Function(function.into()));
                ControlFlow::None
            }
            Stmt::Return(expr) => {
                let val = self.eval_expr(
                    &expr.clone().unwrap_or(Expr::Literal(Literal::Null)),
                    env.clone(),
                );
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
                        match self.exec_stmt(stmt, local_env) {
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
                                    return self.exec_stmt(&stmt, local_env);
                                }
                            }
                            return ControlFlow::None;
                        }
                    }
                    if let Some(else_branch) = else_branch {
                        for stmt in else_branch {
                            let local_env =
                                Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                            return self.exec_stmt(&stmt, local_env);
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
                // let loop_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                self.exec_stmt(init, env.clone());

                loop {
                    let cond = match condition {
                        Some(cond) => match self.eval_expr(cond, env.clone()) {
                            Value::Bool(b) => b,
                            _ => panic!("Expected boolean {:?}", cond),
                        },
                        None => true,
                    };

                    if !cond {
                        break;
                    }

                    let inner = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));

                    for stmt in body {
                        match self.exec_stmt(stmt, inner.clone()) {
                            ControlFlow::Return(v) => return ControlFlow::Return(v),
                            ControlFlow::Break => return ControlFlow::None,
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }

                    if let Some(update) = update {
                        self.eval_expr(update, env.clone());
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
                    Value::Array(arr) => Box::new(arr.borrow().clone().into_iter()),
                    Value::String(s) => {
                        let chars: Vec<_> =
                            s.chars().map(|c| Value::String(c.to_string())).collect();
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
                        match self.exec_stmt(stmt, inner_env.clone()) {
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

                let class_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));

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
                // // Avaliar e definir campos estáticos no ambiente da classe
                // for (field_name, initializer) in static_fields {
                //     let value = self.eval_expr(initializer, Rc::clone(&class_env));
                //     class_env.borrow_mut().define(field_name.to_string(), value);
                // }

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
                        );

                        static_method_array.push(Rc::new(method))
                    } else {
                        let method = Method::new(
                            method.name.clone(),
                            method.params.clone(),
                            method.body.clone(),
                            Environment::new_rc_enclosed(instace_env.clone()),
                            method.is_static,
                            name.clone(),
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
                };

                self.classes.insert(name.clone(), class);
                // let result = env.borrow_mut().assign(&name, Value::Class(Rc::new(class)));

                // if result.is_err() {
                //     let msg = result.unwrap_err();
                //     panic!("{}", msg);
                // }

                ControlFlow::None
            }
            _ => {
                todo!()
            }
        }
    }

    fn resolve_variable(&self, name: &str, env: Rc<RefCell<Environment>>) -> Value {
        // 1. tenta no ambiente atual (local)
        if let Some(val) = env.borrow().get(name) {
            // println!(
            //     "Entrou aqui com {name}  {:?}",
            //     val
            // );
            return val;
        }

        // let this = env.borrow().get("this");

        // // 2. tenta buscar em this se variável não encontrada
        // if let Some(this_val) = this {
        //     if let Value::Object(fields) = this_val {
        //         if let Some(field_val) = fields.borrow().get(name) {
        //             return field_val.clone();
        //         }
        //     }
        // }

        // if is_this {
        //     let parent = env;
        //     if let Some(parent) = parent {
        //         if let Some(field_val) = parent.borrow().get(name) {
        //             return field_val.clone();
        //         }
        //         println!("Parent: {:?}", parent.borrow().get_vars_name_value());
        //     }
        // }
        // 3. erro variável não encontrada
        panic!(
            "Undefined variable '{}'. values: {:?}",
            name,
            env.borrow().get_vars_name_value()
        );
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
                    if let Some(v) = val_arr.borrow().get(i) {
                        self.destructure(pat, v.clone(), env.clone());
                    }
                }
            }
            Expr::Literal(Literal::Object(pairs)) => {
                let val_obj = match value {
                    Value::Object(obj) => obj,
                    _ => panic!("Expected object for destructuring"),
                };
                for (key, pat) in pairs {
                    if let Some(v) = val_obj.borrow().get(key) {
                        self.destructure(pat, v.clone(), env.clone());
                    } else {
                        self.destructure(pat, Value::Null, env.clone());
                    }
                }
            }
            _ => panic!("Invalid destructuring target"),
        }
    }
}
