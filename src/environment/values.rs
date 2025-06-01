use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::ast::{ControlFlow, Expr, Stmt},
    interpreter::Interpreter,
};

use super::{
    native::native_callable::NativeCallable,
    stdlib::{array::NativeArrayClass, number::NativeNumberClass, string::NativeStringClass},
    Environment,
};

#[derive(Clone, Debug)]
pub enum Value<T = String> {
    Void,                                        // Primitivo
    Null,                                        // Primitivo
    Bool(bool),                                  // Primitivo
    Number(NativeNumberClass),                   // Primitivo
    String(NativeStringClass),                   // Primitivo
    Array(NativeArrayClass),                     // Primitivo
    Object(Rc<RefCell<HashMap<String, Value>>>), // Primitivo
    Function(Rc<Function>),

    Class(Rc<Class>),
    Method(Rc<Method>),
    Instance(Rc<RefCell<Instance>>),

    This(Rc<Value>),
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
    NativeClass(Rc<RefCell<dyn NativeCallable>>),
    NativeFunction((String, Rc<RefCell<dyn NativeCallable>>)),

    Internal(T),
    Expr(Expr),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Void, Value::Void) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::This(a), Value::This(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::NativeClass(a), Value::NativeClass(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => {
                let a = &a.1;
                let b = &b.1;
                Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None, // Comparações entre tipos diferentes ou não ordenáveis retornam None
        }
    }
}
impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap().into()),
            serde_json::Value::String(s) => {
                Value::String(NativeStringClass::new_with_args(vec![Value::String(
                    s.into(),
                )]))
            }
            serde_json::Value::Array(a) => Value::Array(
                Rc::new(RefCell::new(a.into_iter().map(|v| v.into()).collect())).into(),
            ),
            serde_json::Value::Object(o) => Value::Object(Rc::new(RefCell::new(
                o.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<HashMap<String, Value>>(),
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
    pub this: Rc<RefCell<Environment>>,
    pub is_static: bool,
    pub class: String,
    pub closure: Rc<RefCell<Environment>>,
}

impl Method {
    pub fn new(
        name: String,
        params: Vec<String>,
        vararg: Option<String>,

        body: Vec<Stmt>,
        this: Rc<RefCell<Environment>>,
        is_static: bool,
        class: String,
        closure: Rc<RefCell<Environment>>,
    ) -> Method {
        Method {
            name,
            params,
            vararg,
            body,
            this,
            is_static,
            class,
            closure,
        }
    }

    pub fn bind(&self, instance: Value) -> Method {
        Method {
            name: self.name.clone(),
            params: self.params.clone(),
            vararg: self.vararg.clone(),
            body: self.body.clone(),
            this: self.this.clone(),
            is_static: self.is_static,
            class: self.class.clone(),
            closure: self.closure.clone(),
        }
        .with_this(instance)
    }

    fn with_this(self, instance: Value) -> Self {
        self.this.borrow_mut().define("this".to_string(), instance);
        self
    }

    pub fn call(&self, mut args: Vec<Value>, mut interpreter: Interpreter) -> Value {
        let name = &self.name;
        let params = &self.params;
        let body = &self.body;
        let vararg = &self.vararg;
        let is_initializer = name == "init";

        let local_env = self
            .closure
            .clone()
            .borrow()
            .clone()
            .from_parent(self.this.clone())
            .to_rc();

        for idx in 0..params.len() {
            let param = params.get(idx).unwrap();
            let val = if !args.is_empty() {
                args.remove(0)
            } else {
                Value::Null
            };

            local_env.borrow_mut().define(param.clone(), val);
            // remove from param_value
        }

        if vararg.is_some() {
            let vararg = vararg.as_ref().unwrap();
            let vararg_name = vararg.clone();

            let vararg_values = Value::array(args);
            local_env.borrow_mut().define(vararg_name, vararg_values);
        }
        // local_env
        //     .borrow_mut()
        //     .merge_environments(env.borrow().clone());

        for stmt in body {
            match interpreter.eval_stmt(stmt, local_env.clone()) {
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

        // //    local_env.borrow_mut().clear();
        // if is_initializer {
        //     return local_env
        //         .borrow()
        //         .get("this")
        //         .unwrap_or(Value::Null)
        //         .clone();
        // }
        Value::Void
    }
}
#[derive(Debug, Clone)]
pub struct Instance {
    pub this: Rc<RefCell<Environment>>,
    // pub static_methods: HashMap<String, Value>,
    pub class: Rc<Class>,
}

impl Instance {
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(method) = self.class.find_method(name) {
            // cria uma função com `this` já definido como essa instância
            Some(Value::Method(Rc::new(
                method.bind(Value::Instance(Rc::new(self.clone().into()))),
            )))
        } else {
            self.this.borrow().get(name)
        }
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.class
            .this
            .borrow_mut()
            .assign(&name, value.clone())
            .unwrap();
        self.this.borrow_mut().assign(&name, value).unwrap();
    }
    pub fn get_to_string(&self) -> Option<Rc<Method>> {
        let method = self.class.find_method("toString");

        method
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Rc<Method>>,
    pub static_methods: Vec<Rc<Method>>,

    pub instance_variables: HashMap<String, Value>,
    pub static_variables: HashMap<String, Value>,

    pub superclass: Option<Rc<Class>>, // Value::Class
    pub this: Rc<RefCell<Environment>>,

    pub closure: Rc<RefCell<Environment>>,
}

impl Class {
    pub fn new(
        name: String,
        methods: Vec<Rc<Method>>,
        superclass: Option<Rc<Class>>,
        this: Rc<RefCell<Environment>>,
        static_methods: Vec<Rc<Method>>,
        instance_variables: HashMap<String, Value>,
        static_variables: HashMap<String, Value>,
        closure: Rc<RefCell<Environment>>,
    ) -> Class {
        Class {
            name,
            methods,
            superclass,
            this,
            static_methods,
            instance_variables,
            static_variables,
            closure,
        }
    }

    pub fn get_all_vars_in_this(&self) -> Vec<(String, Value)> {
        let this = self.this.borrow();

        let mut vars = Vec::new();

        for (name, value) in this.get_vars() {
            vars.push((name.clone(), value.clone()));
        }
        vars
    }

    pub fn instantiate(class: &Rc<Class>, args: Vec<Value>, interpreter: Interpreter) -> Value {
        let this = Environment::new_rc();
        // let closure = interpreter.env.clone();
        this.borrow_mut().copy_from(class.this.clone());
        // this.borrow_mut().copy_from(interpreter.env.clone());

        for (field_name, field_value) in class.instance_variables.clone() {
            this.borrow_mut()
                .define(field_name.clone(), field_value.clone());
        }

        // Add toString method

        // find toString in methods
        // let mut has_to_string = false;

        // Vincula métodos com o ambiente correto
        for method in &class.methods {
            if !method.is_static {
                let name = method.name.clone();

                // if name == "toString" {
                //     has_to_string = true;
                // }
                let function_env = method.this.clone();

                function_env.borrow_mut().copy_from(this.clone());
                let this_method_env = function_env.borrow().clone();
                method.this.replace(this_method_env);

                let body = Value::Method(method.to_owned().into());

                this.borrow_mut().define(name.clone(), body.clone());
            }
        }

        // Get constructor

        let instance = Instance {
            class: class.clone(),
            this: this.clone(),
        };

        let instance = Rc::new(RefCell::new(instance));
        let value = Value::Instance(instance.clone());
        let constructor = class.get_constructor();
        if let Some(constructor) = constructor {
            let constructor = constructor.bind(value.clone());
            constructor.call(args, interpreter);
        }

        // if !has_to_string {
        //     // println!("Criando toString para {}", class.name.clone());
        //     let method = Method::new(
        //         "toString".to_string(),
        //         vec![],
        //         vec![Stmt::Return(Some(crate::ast::ast::Expr::Literal(
        //             crate::ast::ast::Literal::String(value.to_string()),
        //         )))],
        //         this.clone(),
        //         false,
        //         class.name.clone(),
        //         closure,
        //     );

        //     let body = Value::Method(method.to_owned().into());

        //     this.borrow_mut()
        //         .define("toString".to_string(), body.clone());
        // }
        // println!("this: {}", this.borrow().get_vars_string());
        value
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        for method in &self.methods {
            if method.name == name {
                return Some(method.clone());
            }
        }
        None
    }
    pub fn find_constructor_with_args(&self, total_args: usize) -> Option<Rc<Method>> {
        for method in &self.methods {
            if method.name == "constructor" && method.params.len() == total_args {
                return Some(method.clone());
            }
        }
        None
    }

    pub fn get_constructor(&self) -> Option<Rc<Method>> {
        for method in &self.methods {
            if method.name == "constructor" {
                return Some(method.clone());
            }
        }
        None
    }
    pub fn find_static_method(&self, name: &str) -> Option<Rc<Method>> {
        for method in &self.static_methods {
            if method.name == name {
                return Some(method.clone());
            }
        }
        None
    }

    pub fn get_static_field(&self, name: &str) -> Option<Value> {
        self.static_variables.get(name).cloned()
    }

    pub fn get_all_methods_names(&self) -> Vec<String> {
        let mut methods_names = Vec::new();
        for method in &self.methods {
            methods_names.push(method.name.clone());
        }
        for method in &self.static_methods {
            methods_names.push(method.name.clone());
        }
        methods_names
    }

    fn mesma_struct_dyn(a: &dyn Any, b: &dyn Any) -> bool {
        a.type_id() == b.type_id()
    }
    pub fn is_instance_of(instance: &Value, class: &Value) -> bool {
        match instance {
            Value::Void => todo!(),
            Value::Null => todo!(),
            Value::Bool(_) => todo!(),
            Value::Number(instance) => {
                let native_class_rc = class.get_native_class(); // guarda o Rc
                if native_class_rc.is_none() {
                    return false;
                }
                let native_class_rc = native_class_rc.unwrap();
                let native_class_ref = native_class_rc.borrow(); // faz o borrow depois
                Self::mesma_struct_dyn(instance, &*native_class_ref)
            }
            Value::String(instance) => {
                let native_class_rc = class.get_native_class(); // guarda o Rc
                if native_class_rc.is_none() {
                    return false;
                }
                let native_class_rc = native_class_rc.unwrap();
                let native_class_ref = native_class_rc.borrow(); // faz o borrow depois
                Self::mesma_struct_dyn(instance, &*native_class_ref)
            }
            Value::Array(instance) => {
                let native_class_rc = class.get_native_class(); // guarda o Rc
                if native_class_rc.is_none() {
                    return false;
                }
                let native_class_rc = native_class_rc.unwrap();
                let native_class_ref = native_class_rc.borrow(); // faz o borrow depois
                Self::mesma_struct_dyn(instance, &*native_class_ref)
            }
            Value::Object(ref_cell) => todo!(),
            Value::Function(function) => todo!(),
            Value::Class(class) => todo!(),
            Value::Method(method) => todo!(),
            Value::Instance(instance) => {
                let inst = instance.borrow();
                let mut current = Some(&inst.class);

                while let Some(cls) = current {
                    if class.is_native_class() {
                        return false;
                    }
                    if Rc::ptr_eq(cls, &class.to_class()) {
                        return true;
                    }

                    current = match &cls.superclass {
                        Some(super_class) => Some(&super_class),
                        None => None,
                    };
                }

                false
            }
            Value::This(value) => todo!(),
            Value::Builtin(_) => todo!(),
            Value::NativeClass(ref_cell) => todo!(),
            Value::NativeFunction(_) => todo!(),
            Value::Internal(_) => todo!(),
            Value::Expr(expr) => todo!(),
        }
    }

    pub fn create_number() {
        let class_name = "Number".to_string();
        let methods: Vec<Rc<Method>> = vec![]; // Nenhum método por enquanto
        let statics_methods: Vec<Rc<Method>> = vec![];
        let superclass = None;
        let instance_variables = HashMap::new();
        let static_variables = HashMap::new();
        let this_env = Environment::new_rc();
        let closure = Environment::new_rc();

        // Criar a classe
        let number_class = Rc::new(Class::new(
            class_name,
            methods,
            superclass,
            this_env.clone(),
            statics_methods,
            instance_variables,
            static_variables,
            closure.clone(),
        ));
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
    pub environment: Rc<RefCell<Environment>>,
    pub prototype: Option<FunctionPrototype>,
}

#[derive(Debug, Clone)]
pub struct FunctionPrototype {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
}
impl FunctionPrototype {
    pub fn new(name: String, params: Vec<String>, body: String) -> FunctionPrototype {
        FunctionPrototype { name, params, body }
    }
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        vararg: Option<String>,
        body: Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
        prototype: Option<FunctionPrototype>,
    ) -> Function {
        Function {
            name,
            params,
            vararg,
            body,
            environment,
            prototype,
        }
    }

    pub fn generate_proto(&mut self) -> FunctionPrototype {
        let name = self.name.clone();
        let params = self.params.clone();
        let body = self.body_to_string();
        let new_proto = FunctionPrototype::new(name, params, body);
        if self.prototype.is_none() {
            self.prototype = Some(new_proto.clone());
        }
        new_proto
    }

    pub fn body_to_string(&self) -> String {
        let mut body = String::new();
        for stmt in &self.body {
            body.push_str(&format!("{stmt}\n"));
        }
        body
    }

    pub fn call(&self, mut args: Vec<Value>, mut interpreter: Interpreter) -> Value {
        let env = interpreter.env.clone();
        let name = &self.name;
        let params = self.params.clone();
        let body = &self.body;
        let vararg = &self.vararg;

        let is_initializer = name == "init";

        let local_env = self.environment.borrow().clone().from_parent(env).to_rc();

        for idx in 0..params.len() {
            let param = params.get(idx).unwrap();
            let val = args.remove(0);

            local_env.borrow_mut().define(param.clone(), val);
            // remove from param_value
        }

        if vararg.is_some() {
            let vararg = vararg.as_ref().unwrap();
            let vararg_name = vararg.clone();

            let vararg_values = Value::array(args);
            local_env.borrow_mut().define(vararg_name, vararg_values);
        }

        for stmt in body {
            match interpreter.eval_stmt(stmt, local_env.clone()) {
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

        // if is_initializer {
        //     return local_env
        //         .borrow()
        //         .get("this")
        //         .unwrap_or(Value::Null)
        //         .clone();
        // }
        Value::Void
    }
}
impl Value {
    pub fn equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Void, Value::Void) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::This(a), Value::This(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::NativeClass(a), Value::NativeClass(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => {
                let a = &a.1;
                let b = &b.1;
                Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
    pub fn array(vec: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(vec)).into())
    }
    pub fn instance(class: Rc<Class>) -> Value {
        let instance = Instance {
            class: class.clone(),
            this: Environment::new_rc(),
        };
        Value::Instance(Rc::new(instance.into()))
    }
    pub fn object(map: HashMap<String, Value>) -> Value {
        Value::Object(Rc::new(RefCell::new(map)))
    }
    pub fn empty_object() -> Value {
        Value::Object(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn object_is_empty(&self) -> bool {
        match self {
            Value::Object(o) => o.borrow().is_empty(),
            _ => false,
        }
    }
    pub fn new_object() -> Value {
        Value::Object(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn is_native_class(&self) -> bool {
        match self {
            Value::NativeClass(_) => true,
            Value::Array(_) => true,
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn get_native_class(&self) -> Option<Rc<RefCell<dyn NativeCallable>>> {
        match self {
            Value::NativeClass(n) => Some(n.clone()),
            Value::Array(a) => Some(Rc::new(RefCell::new(a.clone()))),
            Value::String(s) => Some(Rc::new(RefCell::new(s.clone()))),
            Value::Number(n) => Some(Rc::new(RefCell::new(n.clone()))),
            _ => None,
        }
    }

    pub fn is_instance(&self) -> bool {
        match self {
            Value::Instance(_) => true,
            _ => false,
        }
    }

    pub fn is_class(&self) -> bool {
        match self {
            Value::Class(_) => true,
            _ => false,
        }
    }
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }
    pub fn is_primitive(&self) -> bool {
        match self {
            Value::Void => true,
            Value::Null => true,
            Value::Bool(_) => true,
            Value::Number(_) => true,
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            Value::Void => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_) => true,
            Value::Class(_) => true,
            Value::Instance(_) => true,
            _ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match self {
            Value::Function(_) => true,
            _ => false,
        }
    }
    pub fn is_method(&self) -> bool {
        match self {
            Value::Method(_) => true,
            _ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false,
        }
    }
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0.into(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.get_value().borrow().is_empty(),
            Value::Object(o) => !o.borrow().is_empty(),
            Value::Function { .. } => true,
            Value::Builtin(_) => true,
            Value::Class(_) => true,
            Value::Instance { .. } => true,
            Value::This(value) => value.is_truthy(),
            _ => false,
        }
    }

    pub fn type_of(&self) -> String {
        match self {
            Value::Void => "void".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
            Value::Function { .. } => "function".to_string(),
            Value::Builtin(_) => "function".to_string(),
            Value::Class(_) => "class".to_string(),
            Value::Instance { .. } => "object".to_string(),
            Value::This(value) => value.type_of(),
            Value::Method(_) => "function".to_string(),
            Value::NativeClass(_) => "class".to_string(),
            Value::NativeFunction(_) => "function".to_string(),
            _ => "unknown".to_string(),
        }
    }
    // Printable
    pub fn to_string(&self) -> String {
        match self {
            Value::Void => "void".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.get_value().clone(),
            Value::Array(a) => {
                let mut s = "[".to_string();
                for (i, v) in a.get_value().borrow().iter().enumerate() {
                    s += &v.to_string();
                    if i != a.get_value().borrow().len() - 1 {
                        s += ", ";
                    }
                }
                s += "]";
                s
            }
            Value::Object(_) => self.stringfy(),
            Value::Function { .. } => "<function>".to_string(),
            Value::Builtin(_) => "<builtin>".to_string(),
            Value::Class(_) => "<class>".to_string(),
            Value::Instance(instance) => {
                if let Some(method) = instance.borrow().get_to_string().as_ref() {
                    let interpreter = Interpreter::new_empty();
                    let method = method.clone();

                    let new_method = Method::new(
                        method.name.clone(),
                        method.params.clone(),
                        method.vararg.clone(),
                        method.body.clone(),
                        method.this.clone(),
                        method.is_static.clone(),
                        method.class.clone(),
                        method.closure.clone(),
                    );

                    return new_method
                        .with_this(Value::Instance(instance.clone()))
                        .call(vec![], interpreter)
                        .to_string();
                }

                let instance_vars = &instance.borrow().class.instance_variables;
                let vars = instance.borrow().this.borrow().get_vars();

                // only json valid values
                let vars = vars
                    .iter()
                    .filter(|(k, v)| {
                        (v.is_primitive() || self.is_object() || self.is_array())
                            && instance_vars.contains_key(k.as_str())
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, Value>>();

                Value::Object(Rc::new(RefCell::new(vars))).to_string()
            }
            Value::This(value) => value.to_string(),
            Value::Method(_) => "<method>".to_string(),
            Value::NativeClass(c) => format!("<Native class {}>", c.borrow().get_name()),
            Value::NativeFunction(_) => "<function>".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub fn stringfy(&self) -> String {
        match self {
            Value::Void => "void".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s.clone()),
            Value::Array(a) => {
                let mut s = "[".to_string();
                for (i, v) in a.get_value().borrow().iter().enumerate() {
                    s += &v.stringfy();
                    if i != a.get_value().borrow().len() - 1 {
                        s += ", ";
                    }
                }
                s += "]";
                s
            }
            Value::Object(o) => {
                let mut s = "{".to_string();
                for (i, (k, v)) in o.borrow().iter().enumerate() {
                    s += &format!("\"{}\": {}", k, v.stringfy());
                    if i != o.borrow().len() - 1 {
                        s += ", ";
                    }
                }
                s += "}";
                s
            }
            Value::Function { .. } => "<function>".to_string(),
            Value::Builtin(_) => "<builtin>".to_string(),
            Value::Class(_) => "<class>".to_string(),
            Value::Instance(_) => self.convert_class_to_object().stringfy(),
            Value::This(value) => value.stringfy(),
            Value::Method(_) => "<function>".to_string(),
            Value::NativeClass(_) => "<class>".to_string(),
            Value::NativeFunction(_) => "<function>".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub fn convert_class_to_object(&self) -> Value {
        match self {
            Value::Class(class) => {
                let mut map = HashMap::new();
                for (name, value) in &class.instance_variables {
                    map.insert(name.clone(), value.clone());
                }
                Value::Object(Rc::new(RefCell::new(map)))
            }
            Value::Instance(instance) => {
                if let Some(method) = instance.borrow().get_to_string().as_ref() {
                    let interpreter = Interpreter::new_empty();
                    let method = method.clone();

                    let new_method = Method::new(
                        method.name.clone(),
                        method.params.clone(),
                        method.vararg.clone(),
                        method.body.clone(),
                        method.this.clone(),
                        method.is_static.clone(),
                        method.class.clone(),
                        method.closure.clone(),
                    );

                    return new_method
                        .with_this(Value::Instance(instance.clone()))
                        .call(vec![], interpreter);
                }

                let instance_vars = &instance.borrow().class.instance_variables;
                let vars = instance.borrow().this.borrow().get_vars();

                // only json valid values
                let vars = vars
                    .iter()
                    .filter(|(k, v)| {
                        (v.is_primitive() || self.is_object() || self.is_array())
                            && instance_vars.contains_key(k.as_str())
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, Value>>();

                Value::Object(Rc::new(RefCell::new(vars)))
            }
            _ => panic!("Cannot convert {} to object", self.type_of()),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0.into(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.get_value().borrow().is_empty(),
            Value::Object(o) => !o.borrow().is_empty(),
            Value::Function { .. } => true,
            Value::Builtin(_) => true,
            Value::Class(_) => true,
            Value::Instance { .. } => true,
            Value::This(value) => value.to_bool(),
            // Value::Method(_) => true,
            _ => false,
        }
    }

    #[track_caller]
    pub fn to_number(&self) -> f64 {
        // let caller = std::panic::Location::caller();
        // let location = format!("{}:{}", caller.file(), caller.line());
        match self {
            Value::Number(n) => n.get_value(),
            Value::String(s) => {
                let s = s.get_value();
                let msg = format!("Cannot convert string '{}' to number", s);
                s.parse::<f64>().expect(&msg)
            }
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => f64::NAN,
        }
    }

    #[track_caller]
    pub fn to_number_class(&self) -> NativeNumberClass {
        // let caller = std::panic::Location::caller();
        // let location = format!("{}:{}", caller.file(), caller.line());
        match self {
            Value::Number(n) => n.to_owned(),

            _ => panic!("Not a number"),
        }
    }

    pub fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(a) => a.get_value().borrow().clone(),
            _ => panic!("Cannot convert {} to array", self.type_of()),
        }
    }
    pub fn to_object(&self) -> HashMap<String, Value> {
        match self {
            Value::Object(o) => o.borrow().clone().into(),
            _ => panic!("Cannot convert {} to object", self.type_of()),
        }
    }

    #[track_caller]
    pub fn to_class(&self) -> Rc<Class> {
        let caller = std::panic::Location::caller();
        let location = format!("{}:{}", caller.file(), caller.line());
        match self {
            Value::Class(class) => Rc::clone(class),
            Value::Instance(instance) => instance.borrow().class.clone(),
            _ => panic!(
                "Cannot convert {} to class. called: {location}",
                self.type_of()
            ),
        }
    }

    pub fn to_function(&self) -> Rc<Function> {
        match self {
            Value::Function(f) => f.clone(),
            _ => panic!("Cannot convert {} to function", self.type_of()),
        }
    }

    pub fn to_method(&self) -> Rc<Method> {
        match self {
            Value::Method(m) => m.clone(),
            _ => panic!("Cannot convert {} to method", self.type_of()),
        }
    }
    pub fn to_instance(&self) -> Rc<RefCell<Instance>> {
        match self {
            Value::Instance(instance) => instance.clone(),
            _ => panic!("Cannot convert {} to instance", self.type_of()),
        }
    }
}
