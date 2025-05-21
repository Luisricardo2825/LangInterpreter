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
}

impl Interpreter {
    pub fn new(ast: Vec<Stmt>) -> Self {
        Self { ast }
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
            Expr::Identifier(name) => self.resolve_variable(name, env),
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
                        let closure = &func.closure;
                        let is_initializer = name == "init";
                        let local_env =
                            Rc::new(RefCell::new(Environment::new_enclosed(closure.clone())));
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

                    Value::Class(class) => {
                        let instance = class.instantiate();
                        // let constructor = instance.get_constructor(&arg_values);
                        // if constructor.is_some() {
                        //     self.call_function(Rc::new(constructor.unwrap()), None, arg_values);
                        // }

                        instance
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
                        let msg = format!("Property {prop} not found");
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
                        let instance_rc = instance;

                        let instance = instance.borrow();

                        let class = instance.class.clone();
                        let class_name = &class.name;
                        let msg = format!("Cannot find '{prop}' in class {class_name}");
                        let name = prop.to_string();
                        // println!("methods: {:?}",class.methods);
                        let this = class.this.borrow();

                        let value = this.get(&name).expect(&msg);
                        if let Value::Function(function_rc) = &value {
                            let function = function_rc;

                            // Crie um ambiente com 'this' apontando para a instância
                            let mut environment =
                                Environment::new_enclosed(function.closure.clone());
                            environment
                                .define("this".to_string(), Value::Instance(instance_rc.clone()));

                            // Execute a função (sem argumentos, ou passe se necessário)
                            self.execute_block(
                                &function.body.clone(), // você precisa ter isso salvo no Function
                                Rc::new(RefCell::new(environment)),
                            )
                            .expect("Erro ao executar bloco")
                        }
                        value
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        let class_name = &class.name;
                        let msg = format!("Cannot find '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.find_method(&name, true) {
                            return val;
                        }
                        // 1. Tentar buscar o campo estático no ambiente 'this' da classe
                        if let Some(val) = class.this.borrow().get(&name) {
                            return val;
                        }

                        // 2. Se não encontrar, tentar método estático

                        panic!("{}", msg)
                    }
                    (Value::Builtin(func), Value::String(prop)) => {
                        let class = func(vec![]).to_class();
                        let class_name = &class.name;
                        let msg = format!("Cannot find '{prop}' in class {class_name}");
                        let name = prop.to_string();

                        if let Some(val) = class.find_method(&name, true) {
                            return val;
                        }
                        // 1. Tentar buscar o campo estático no ambiente 'this' da classe
                        if let Some(val) = class.this.borrow().get(&name) {
                            return val;
                        }

                        // 2. Se não encontrar, tentar método estático

                        panic!("{}", msg)
                    }
                    (Value::This(this), Value::String(prop)) => {
                        let value = this.as_ref();

                        let msg = format!("Cannot find '{prop}' in {}", value.to_string());

                        match value {
                            Value::Object(obj) => {
                                let msg =
                                    format!("Property {prop} not found in {}", value.to_string());
                                obj.borrow().get(prop).cloned().expect(&msg)
                            }
                            _ => panic!("{}", msg),
                        }
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
                let obj = self.eval_expr(object, env.clone());
                let prop = match property.as_ref() {
                    Expr::Identifier(name) => Value::String(name.to_string()),
                    Expr::Literal(Literal::Number(n)) => Value::Number(*n),
                    Expr::Literal(Literal::String(s)) => Value::String(s.clone()),
                    _ => return Value::Null,
                };
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
                        todo!()
                    }
                    (Value::Class(class), Value::String(prop)) => {
                        class.this.borrow().get(prop).unwrap_or(Value::Null);
                    }
                    (Value::This(value), Value::String(prop)) => {
                        if let Value::Object(this) = value.as_ref() {
                            let mut this = this.borrow_mut();
                            this.insert(prop.clone(), val);
                        }
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
            Expr::This => {
                let mut env = env.borrow_mut();
                println!("env: {:?}", env.variables.len());

                let variables = env.variables.clone();
                env.define("this".to_owned(), Value::object(variables));
                let this = env.get("this").unwrap_or(Value::Null);

                if this.type_of() == "object" || this.type_of() == "class" {
                    Value::This(Box::new(this))
                } else {
                    let this = this
                        .to_class()
                        .this
                        .borrow()
                        .get("this")
                        .unwrap_or(Value::Null);
                    Value::This(Box::new(this))
                }
            }
            Expr::New { constructor, args } => {
                let class_val = self.eval_expr(constructor, env.clone());
                let arg_vals = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env.clone()))
                    .collect::<Vec<_>>();

                match class_val {
                    Value::Class(class) => {
                        let instance = Rc::new(RefCell::new(Instance {
                            class: class.clone(),
                            fields: HashMap::new(),
                        }));

                        // Se houver método 'init', chame-o
                        if let Some(init) = class
                            .methods
                            .iter()
                            .find(|method| method.name == "constructor")
                        {
                            let this = Value::Instance(instance.clone());
                            let method = init.clone();

                            self.call_function(
                                method.value.to_function().into(),
                                Some(this),
                                arg_vals,
                            );
                        }

                        Value::Instance(instance)
                    }
                    Value::Instance(instance) => {
                        let class = instance.borrow().class.clone();

                        let fields = instance.borrow_mut().fields.clone();

                        for (name, value) in fields {
                            class.this.borrow_mut().define(name, value);
                        }
                        // Se houver método 'init', chame-o
                        if let Some(init) = class
                            .methods
                            .iter()
                            .find(|method| method.name == "constructor")
                        {
                            let this = Value::Instance(instance.clone());
                            let method = init.clone();

                            let value = self.call_function(
                                method.value.to_function().into(),
                                Some(this),
                                arg_vals,
                            );
                            let value = value.unwrap();
                            instance
                                .borrow_mut()
                                .fields
                                .insert("this".to_string(), value);
                        }
                        // println!("fields: {:?}",instance.borrow_mut().list_fields());
                        Value::Instance(instance)
                    }
                    _ => panic!("Only classes can be constructed with 'new'"),
                }
            }
            _ => todo!(),
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
                    closure: Rc::clone(&env),
                    is_initializer: false,
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
                let loop_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                self.exec_stmt(init, loop_env.clone());

                loop {
                    let cond = match condition {
                        Some(cond) => match self.eval_expr(cond, loop_env.clone()) {
                            Value::Bool(b) => b,
                            _ => panic!("Expected boolean"),
                        },
                        None => true,
                    };

                    if !cond {
                        break;
                    }

                    let inner = Rc::new(RefCell::new(Environment::new_enclosed(loop_env.clone())));

                    for stmt in body {
                        match self.exec_stmt(stmt, inner.clone()) {
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

                let loop_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));

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
                env.borrow_mut().define(name.clone(), Value::Null);

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

                // Avaliar e definir campos estáticos no ambiente da classe
                for (field_name, initializer) in static_fields {
                    let value = self.eval_expr(initializer, Rc::clone(&class_env));
                    class_env.borrow_mut().define(field_name.to_string(), value);
                }

                // Avaliar valores iniciais dos campos de instância (armazenados para uso em `instantiate`)
                let mut instance_variables = HashMap::new();
                for (field_name, initializer) in instance_fields {
                    let value = self.eval_expr(initializer, Rc::clone(&class_env));
                    instance_variables.insert(field_name.to_owned(), value);
                }

                // Criar novo escopo para métodos (especialmente para `super`)
                let method_env = if super_class_value.is_some() {
                    class_env.borrow_mut().define(
                        "super".to_string(),
                        Value::Class(super_class_value.clone().unwrap()),
                    );
                    Rc::clone(&class_env)
                } else {
                    Rc::clone(&class_env)
                };

                let mut method_array: Vec<Method> = vec![];
                let mut static_method_array: Vec<Method> = vec![];
                for method in methods {
                    let func = Function::new(
                        method.name.clone(),
                        method.params.clone(),
                        method.body.clone(),
                        Rc::clone(&method_env),
                        /* is_initializer */ method.name == "constructor",
                    );
                    if method.is_static {
                        static_method_array.push(Method {
                            name: method.name.clone(),
                            value: Value::Function(func.into()),
                            is_static: method.is_static.clone(),
                        })
                    } else {
                        method_array.push(Method {
                            name: method.name.clone(),
                            value: Value::Function(func.into()),
                            is_static: method.is_static.clone(),
                        });
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

                    this: Rc::clone(&class_env),
                    instance_variables,
                };

                let result = env.borrow_mut().assign(&name, Value::Class(Rc::new(class)));

                if result.is_err() {
                    let msg = result.unwrap_err();
                    panic!("{}", msg);
                }

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
            return val;
        }
        // 2. tenta buscar em this se variável não encontrada
        if let Some(this_val) = env.borrow().get("this") {
            if let Value::Object(fields) = this_val {
                if let Some(field_val) = fields.borrow().get(name) {
                    return field_val.clone();
                }
            }
        }
        // 3. erro variável não encontrada
        panic!("Undefined variable {}", name);
    }

    fn call_function(
        &mut self,
        func: Rc<Function>,
        this: Option<Value>,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        let mut env = Environment::new_enclosed(func.closure.clone());

        // Vincular argumentos
        for (name, value) in func.params.iter().zip(args) {
            env.define(name.clone(), value);
        }

        // Se for método de instância, defina `this`
        if let Some(this_val) = this {
            env.define("this".to_string(), this_val);
        }

        match self.execute_block(&func.body, Rc::new(RefCell::new(env))) {
            Ok(_) => Ok(Value::Null),
            Err(ControlFlow::Return(val)) => Ok(val),
            Err(e) => panic!("Erro ao executar bloco"),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), ControlFlow> {
        let result = (|| {
            for stmt in statements {
                self.exec_stmt(stmt, env.clone()); // Pode retornar ControlFlow::Return, Break, etc.
            }
            Ok(())
        })();

        result
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
