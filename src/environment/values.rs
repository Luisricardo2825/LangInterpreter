use core::f64;
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc, vec};

use serde::{Deserialize, Serialize};

use crate::{
    ast::ast::{BinaryOperator, ControlFlow, Expr, MethodModifiersOperations, Modifiers, Stmt},
    environment::stdlib::number::NativeNumberClass,
    interpreter::Interpreter,
};

use super::{
    native::native_callable::NativeCallable, stdlib::array::NativeArrayClass, Environment,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value<T = String> {
    Void,                      // Primitivo
    Null,                      // Primitivo
    Bool(bool),                // Primitivo
    Number(NativeNumberClass), // Primitivo
    String(String),            // Primitivo

    Array(NativeArrayClass),
    Object(Rc<RefCell<Vec<(String, Value)>>>),
    Function(Rc<Function>),

    Error(Rc<RefCell<Value>>),

    Class(Rc<Class>),
    Instance(Rc<RefCell<Instance>>),

    #[serde(skip)]
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
    #[serde(skip)]
    InternalClass(Rc<RefCell<dyn NativeCallable>>),
    #[serde(skip)]
    InternalFunction((String, Rc<RefCell<dyn NativeCallable>>)),

    Internal(T),
    Expr(Expr),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}
#[derive(Clone, Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl RuntimeError {
    pub fn new(message: String, line: usize, column: usize, file: String) -> Self {
        Self {
            message,
            line,
            column,
            file,
        }
    }
}

impl From<std::string::String> for Value {
    fn from(value: std::string::String) -> Self {
        Value::String(value)
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        match value {
            Value::String(s) => s,
            _ => panic!("Cannot convert {:?} to String", value),
        }
    }
}

pub trait NativeObjectTrait {
    fn contains_key(&self, key: &str) -> bool;
    fn get_prop(&self, key: &str) -> Option<Value>;
    fn set_prop(&mut self, key: &str, value: Value) -> Result<(), String>;
}
impl NativeObjectTrait for Vec<(String, Value)> {
    fn contains_key(&self, key: &str) -> bool {
        let mut contains = false;
        for item in self {
            if item.0 == key {
                contains = true;
                break;
            }
        }
        contains
    }

    fn get_prop(&self, key: &str) -> Option<Value> {
        let mut value = None;
        for item in self {
            if item.0 == key {
                value = Some(item.1.clone());
                break;
            }
        }
        value
    }
    fn set_prop(&mut self, key: &str, value: Value) -> Result<(), String> {
        let mut contains = false;
        for item in self.iter_mut() {
            if item.0 == key {
                item.1 = value.clone();
                contains = true;
                break;
            }
        }
        if !contains {
            self.push((key.to_string(), value));
        }
        Ok(())
    }
}
// #[derive(Clone, Debug)]
// pub enum Primitive {
//     Void,
//     Null,
//     Bool(bool),
//     Number(f64),
//     Char(char),
//     String(String),
// }

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
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::InternalClass(a), Value::InternalClass(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::InternalFunction(a), Value::InternalFunction(b)) => {
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
            serde_json::Value::String(s) => Value::String(s.into()),
            serde_json::Value::Array(a) => Value::Array(
                Rc::new(RefCell::new(a.into_iter().map(|v| v.into()).collect())).into(),
            ),
            serde_json::Value::Object(o) => Value::Object(Rc::new(RefCell::new(
                o.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Vec<(String, Value)>>(),
            ))),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub this: Rc<RefCell<Environment>>,
    pub class: Rc<Class>,
}

impl Instance {
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(method) = self.class.find_method(name) {
            // cria uma função com `this` já definido como essa instância
            Some(Value::Function(method))
        } else {
            self.this.borrow().get(name)
        }
    }

    pub fn find_operation(&self, operator: &str) -> Option<Rc<Function>> {
        let class = self.class.clone();

        let method = class.find_method_with_modifiers(&operator, vec![Modifiers::Operator]);

        if method.is_some() {
            return method;
        }

        None
    }

    pub fn is_operation(&self, operator: &str) -> bool {
        match operator {
            "plus" => true,
            "sub" => true,
            "mul" => true,
            "div" => true,
            "exp" => true,
            "mod" => true,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        let class_this = self.class.this.borrow_mut().assign(&name, value.clone());

        if class_this.is_err() {
            return class_this;
        }
        let this = self.this.borrow_mut().assign(&name, value);

        this
    }
    pub fn get_to_string(&self) -> Option<Rc<Function>> {
        let method = self.class.find_method("toString");

        if method.is_none() {
            let method = self.class.get_value_of_method();
            return method;
        }
        method
    }

    pub fn get_value_of(&self) -> Option<Rc<Function>> {
        let method = self.class.find_method("valueOf");

        if method.is_none() {
            let method = self.class.get_value_of_method();
            return method;
        }
        method
    }

    pub fn is_instance_of(&self, class: &Value) -> bool {
        let inst = self;
        let mut current = Some(&inst.class);

        while let Some(cls) = current {
            let class = class.to_class();
            if class.is_none() {
                return false;
            }
            let class = class.unwrap();
            if Rc::ptr_eq(cls, &class) {
                return true;
            }

            current = match &cls.superclass {
                Some(super_class) => Some(&super_class),
                None => None,
            };
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Rc<Function>>,
    pub static_methods: Vec<Rc<Function>>,

    pub instance_variables: Rc<RefCell<HashMap<String, Value>>>,
    pub static_variables: HashMap<String, Value>,

    pub superclass: Option<Rc<Class>>, // Value::Class
    pub this: Rc<RefCell<Environment>>,

    pub closure: Rc<RefCell<Environment>>,
}

impl Class {
    pub fn new(
        name: String,
        methods: Vec<Rc<Function>>,
        superclass: Option<Rc<Class>>,
        this: Rc<RefCell<Environment>>,
        static_methods: Vec<Rc<Function>>,
        instance_variables: Rc<RefCell<HashMap<String, Value>>>,
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

    pub fn instantiate(class: &Rc<Class>, mut args: Vec<Value>) -> Value {
        let this = Environment::new_rc();
        // let closure = interpreter.env.clone();
        this.borrow_mut().copy_from(class.this.clone());
        // this.borrow_mut().copy_from(interpreter.env.clone());
        // let value = this.borrow().get("value");

        // if value.is_some() {
        //     println!("Value: {:?}", value.unwrap());
        // }
        for (field_name, field_value) in class.instance_variables.borrow().clone() {
            this.borrow_mut()
                .define(field_name.clone(), field_value.clone());
        }

        // Vincula métodos com o ambiente correto
        for method in &class.methods {
            if !method.is_static() {
                let name = method.name.clone();

                let body = Value::Function(method.to_owned().into());

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
        // add "value" at start
        args.insert(0, value.clone());
        let constructor = class.get_constructor();
        if let Some(constructor) = constructor {
            // let constructor = constructor.bind(value.clone());

            // for (idx, ele) in args.iter().enumerate() {
            //     println!("Argumento {idx} {ele} {}", class.name)
            // }
            let call = constructor.call(args);

            return call;
        }

        value
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        for method in &self.methods {
            if method.name == name && method.modifiers.is_empty() {
                return Some(method.clone());
            }
        }
        None
    }
    pub fn find_method_with_modifiers(
        &self,
        name: &str,
        modifiers: Vec<Modifiers>,
    ) -> Option<Rc<Function>> {
        for method in &self.methods {
            if method.name == name {
                if method.modifiers.contains_all(modifiers.clone()) {
                    return Some(method.clone());
                }
            }
        }
        None
    }
    pub fn find_constructor_with_args(&self, total_args: usize) -> Option<Rc<Function>> {
        for method in &self.methods {
            if method.name == "constructor" && method.params.len() == total_args {
                return Some(method.clone());
            }
        }
        None
    }

    pub fn get_constructor(&self) -> Option<Rc<Function>> {
        for method in &self.methods {
            if method.name == "constructor" {
                return Some(method.clone());
            }
        }
        None
    }
    pub fn find_static_method(&self, name: &str) -> Option<Rc<Function>> {
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

    pub fn get_value_of_method(&self) -> Option<Rc<Function>> {
        for method in &self.methods {
            if method.name == "valueOf" {
                return Some(method.clone());
            }
        }
        None
    }
    fn mesma_struct_dyn(a: &dyn Any, b: &dyn Any) -> bool {
        a.type_id() == b.type_id()
    }

    pub fn is_instance_of(instance: &Value, class: &Value) -> bool {
        match instance {
            Value::Void => false,
            Value::Null => false,
            Value::Bool(_) => false,
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
            Value::Object(_ref_cell) => todo!(),
            Value::Function(_function) => todo!(),
            Value::Class(this_class) => {
                let class = class.to_class();
                if class.is_none() {
                    return false;
                }
                let class = class.unwrap();
                Rc::ptr_eq(this_class, &class)
            }
            Value::Instance(instance) => {
                let inst = instance.borrow();
                inst.is_instance_of(class)
            }
            Value::Builtin(_) => todo!(),
            Value::InternalClass(_ref_cell) => todo!(),
            Value::InternalFunction(_) => todo!(),
            Value::Internal(_) => todo!(),
            Value::Error(erro) => {
                let error = erro.borrow().clone();
                Self::is_instance_of(&error, class)
            }
            _ => panic!(
                "is_instance_of: Value not supported: {:?}",
                instance.type_of()
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
    pub environment: Rc<RefCell<Environment>>,
    pub prototype: Option<FunctionPrototype>,
    pub modifiers: Vec<Modifiers>,
    pub this: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        modifiers: Vec<Modifiers>,
    ) -> Function {
        let mut func = Function {
            name,
            params,
            vararg,
            body,
            environment,
            prototype: None,
            modifiers,
            this: Value::Null,
        };
        func.generate_proto();
        func
    }

    pub fn unwrap(&self) -> Self {
        Self {
            name: self.name.clone(),
            params: self.params.clone(),
            vararg: self.vararg.clone(),
            body: self.body.clone(),
            environment: self.environment.clone(),
            prototype: self.prototype.clone(),
            modifiers: self.modifiers.clone(),
            this: self.this.clone(),
        }
    }

    pub fn with_this(&self, this: Value) -> Self {
        let mut func = self.clone();
        func.this = this;
        func.generate_proto();
        func
    }
    pub fn from(func: Rc<Function>) -> Self {
        Self {
            name: func.name.clone(),
            params: func.params.clone(),
            vararg: func.vararg.clone(),
            body: func.body.clone(),
            environment: func.environment.clone(),
            prototype: func.prototype.clone(),
            modifiers: func.modifiers.clone(),
            this: func.this.clone(),
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

    pub fn call(&self, mut args: Vec<Value>) -> Value {
        let name = &self.name;
        let body = &self.body;
        let mut interpreter = Interpreter::new_empty();

        let is_initializer = self.name == "constructor";
        // let _guard = DepthGuard::new().map_err(|e| e.to_string())?;

        // let binding = self.environment.borrow();
        let closure = self.environment.borrow(); // evita clone

        let mut local_env = closure.to_rc();

        // remove last arg
        if !self.this.is_null() {
            args.insert(0, self.this.clone());
        }
        let this = if is_initializer && args.len() > 0 {
            args.first().unwrap().clone()
        } else {
            Value::Null
        };
        let mut args_iter = args.clone().into_iter();

        // define parâmetros fixos
        for param in &self.params {
            let val = args_iter.next().unwrap_or(Value::Null);
            // if param == "this" {
            //     local_env.borrow_mut().define(param.clone(), this.clone());
            //     println!(
            //         "Definiu o this para {name} = {}, params: {:?}",
            //         this, self.params
            //     );
            //     continue;
            // }
            local_env.borrow_mut().define(param.clone(), val);
        }

        // define varargs (se houver)
        if let Some(vararg_name) = &self.vararg {
            let vararg_values = Value::array(args_iter.collect());
            local_env
                .borrow_mut()
                .define(vararg_name.clone(), vararg_values);
        }

        // Executa o corpo da função
        for stmt in body {
            match interpreter.eval_stmt(stmt, &mut local_env) {
                ControlFlow::Return(val) => {
                    return val;
                }
                ControlFlow::Break => {
                    return Value::new_error(
                        &mut local_env,
                        format!("Break not allowed in function {}", name).into(),
                    )
                }
                ControlFlow::Continue => {
                    return Value::new_error(
                        &mut local_env,
                        format!("Continue not allowed in function {}", name).into(),
                    )
                }
                ControlFlow::None => {}
                ControlFlow::Error(err) => {
                    return err;
                }
            }
        }

        if is_initializer {
            return this;
        }

        Value::Void
    }

    pub fn is_static(&self) -> bool {
        self.modifiers.contains(&Modifiers::Static)
    }
    pub fn is_private(&self) -> bool {
        self.modifiers.contains(&Modifiers::Private)
    }
    pub fn is_operator(&self) -> bool {
        self.modifiers.contains(&Modifiers::Operator)
    }
}
impl Value {
    #[track_caller]
    pub fn new_error(env: &mut Rc<RefCell<Environment>>, msg: String) -> Value {
        let location = std::panic::Location::caller().to_string()
            + " "
            + std::file!()
            + ":"
            + &std::line!().to_string();
        let env = env.borrow();
        let error_class = env.get("Error");
        if error_class.is_none() {
            panic!("Error class not found {msg} {location}")
        }
        let error_class = error_class.unwrap().to_class();
        if error_class.is_none() {
            panic!("Class 'Error' not found");
        }
        let error_class = error_class.unwrap();

        let throw_method = error_class.find_static_method("throw");

        if throw_method.is_none() {
            panic!("throw method not found");
        }
        let throw_method = throw_method.unwrap();

        let error = throw_method.call(vec![Value::Null, Value::String(msg.into())]);

        if error.is_error() {
            panic!("Error creating error object {:?}", error.to_string());
        }
        Value::Error(Rc::new(error.into()))
    }

    pub fn is_error(&self) -> bool {
        match self {
            Value::Error(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            Value::Error(_) => true,
            _ => false,
        }
    }

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
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::InternalClass(a), Value::InternalClass(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::InternalFunction(a), Value::InternalFunction(b)) => {
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
    pub fn object(map: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(map)))
    }
    pub fn empty_object() -> Value {
        Value::Object(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn error(message: String) -> Value {
        Value::Error(RefCell::new(Value::String(message.into())).into())
    }

    pub fn object_is_empty(&self) -> bool {
        match self {
            Value::Object(o) => o.borrow().is_empty(),
            _ => false,
        }
    }
    pub fn new_object() -> Value {
        Value::Object(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn is_native_class(&self) -> bool {
        match self {
            Value::InternalClass(_) => true,
            Value::Array(_) => true,
            _ => false,
        }
    }

    pub fn get_native_class(&self) -> Option<Rc<RefCell<dyn NativeCallable>>> {
        match self {
            Value::InternalClass(n) => Some(n.clone()),
            Value::Array(a) => Some(Rc::new(RefCell::new(a.clone()))),
            // Value::String(s) => Some(Rc::new(RefCell::new(s.clone()))),
            // Value::Number(n) => Some(Rc::new(RefCell::new(n.clone()))),
            _ => None,
        }
    }

    pub fn call_op(&self, op: BinaryOperator, other: &Value) -> Value {
        let left = self.clone();
        let right = other.clone();
        let op_alias = op.alias();
        if left.is_instance() || right.is_instance() {
            let instance = if left.is_instance() {
                left.to_instance()
            } else {
                right.to_instance()
            };
            let other = if left.is_instance() {
                right.clone()
            } else {
                left.clone()
            };

            let plus_method = instance.borrow().find_operation(&op_alias);
            if let Some(method) = plus_method {
                let args = vec![Value::Instance(instance), other.clone()];
                let val = method.call(args);
                if !val.is_error() {
                    return val;
                }
            }
        }

        match op {
            BinaryOperator::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number((a + b).into()),
                (a, b) => Value::String(format!("{}{}", a, b).into()),
            },
            BinaryOperator::Subtract => {
                Value::Number((left.to_number() - right.to_number()).into())
            }
            BinaryOperator::Multiply => {
                Value::Number((left.to_number() * &right.to_number()).into())
            }
            BinaryOperator::Divide => Value::Number((left.to_number() / &right.to_number()).into()),
            BinaryOperator::Modulo => Value::Number((left.to_number() % &right.to_number()).into()),
            BinaryOperator::Exponentiate => {
                Value::Number((left.to_number().powf(right.to_number())).into())
            }
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
            Value::Instance(instance) => {
                let method = instance.borrow().class.get_value_of_method();
                if method.is_some() {
                    let method = method.unwrap();
                    let value = method.call(vec![]);
                    if value.is_error() {
                        return false;
                    }
                    value.is_number()
                } else {
                    false
                }
            }
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
            Value::Function(_) => true,
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
            Value::InternalClass(_) => "class".to_string(),
            Value::InternalFunction(_) => "function".to_string(),
            Value::Error(error) => error.borrow().type_of(),
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
            Value::String(s) => s.clone(),
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
            Value::Object(_) => self.stringfy(),
            Value::Function(function) => format!("<function {}>", function.name),
            Value::Builtin(_) => "<builtin>".to_string(),
            Value::Class(class) => format!("<class {}>", class.name),
            Value::Instance(instance) => {
                if let Some(method) = instance.borrow().get_to_string().as_ref() {
                    let method = method.clone();

                    let call = method.call(vec![self.clone()]);
                    // if call.is_err() {
                    //     return call.to_string();
                    // }
                    return call.to_string();
                }

                let instance_vars = &instance.borrow().class.instance_variables;
                let vars = instance.borrow().this.borrow().get_vars();

                // only json valid values
                let vars = vars
                    .iter()
                    .filter(|(k, v)| {
                        (v.is_primitive() || self.is_object() || self.is_array())
                            && instance_vars.borrow().contains_key(k.as_str())
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<(String, Value)>>();

                Value::Object(Rc::new(RefCell::new(vars))).to_string()
            }
            Value::InternalClass(c) => format!("<internal class {}>", c.borrow().get_name()),
            Value::InternalFunction(function) => format!("<internal function {}>", function.0),
            Value::Error(error) => error.borrow().to_string(),
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
            Value::InternalClass(_) => "<class>".to_string(),
            Value::InternalFunction(_) => "<function>".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub fn convert_class_to_object(&self) -> Value {
        match self {
            Value::Class(class) => {
                let mut map = Vec::new();
                for (name, value) in &class.instance_variables.borrow().clone() {
                    map.push((name.clone(), value.clone()));
                }
                Value::Object(Rc::new(RefCell::new(map)))
            }
            Value::Instance(instance) => {
                let instance_vars = &instance.borrow().class.instance_variables;
                let vars = instance.borrow().this.borrow().get_vars();

                // only json valid values
                let vars = vars
                    .iter()
                    .filter(|(k, v)| {
                        (v.is_primitive() || self.is_object() || self.is_array())
                            && instance_vars.borrow().contains_key(k.as_str())
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<(String, Value)>>();

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
            // Value::Function(_) => true,
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
                let s = s;
                s.parse::<f64>().unwrap_or(f64::NAN)
            }
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Instance(instance) => {
                let value_of_method = instance.borrow().class.get_value_of_method();
                if let Some(value_of_method) = value_of_method {
                    let call = value_of_method.call(vec![]);

                    if call.is_err() {
                        return f64::NAN;
                    }
                    call.to_number()
                } else {
                    f64::NAN
                }
            }
            _ => f64::NAN,
        }
    }

    pub fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(a) => a.get_value().borrow().clone(),
            _ => panic!("Cannot convert {} to array", self.type_of()),
        }
    }
    pub fn to_object(&self) -> Vec<(String, Value)> {
        match self {
            Value::Object(o) => o.borrow().clone().into(),
            _ => panic!("Cannot convert {} to object", self.type_of()),
        }
    }

    #[track_caller]
    pub fn to_class(&self) -> Option<Rc<Class>> {
        // let caller = std::panic::Location::caller();
        // let location = format!("{}:{}", caller.file(), caller.line());
        match self {
            Value::Class(class) => Some(Rc::clone(class)),
            Value::Instance(instance) => Some(instance.borrow().class.clone()),
            _ => None,
        }
    }

    pub fn to_function(&self) -> Rc<Function> {
        match self {
            Value::Function(f) => f.clone(),
            _ => panic!("Cannot convert {} to function", self.type_of()),
        }
    }

    pub fn to_method(&self) -> Rc<Function> {
        match self {
            Value::Function(m) => m.clone(),
            _ => panic!("Cannot convert {} to method", self.type_of()),
        }
    }
    pub fn to_instance(&self) -> Rc<RefCell<Instance>> {
        match self {
            Value::Instance(instance) => instance.clone(),
            _ => panic!("Cannot convert {} to instance", self.type_of()),
        }
    }

    pub fn get_primitive_class(&self, env: &mut Rc<RefCell<Environment>>) -> Option<Value> {
        match self {
            Value::Bool(_) => {
                let class = env.borrow().get("Boolean");
                if class.is_none() {
                    return None;
                }
                let class = class.unwrap().to_class().unwrap();
                Some(Class::instantiate(&class, vec![self.clone()]))
            }
            Value::Number(_) => {
                let class = env.borrow().get("Number");
                if class.is_none() {
                    return None;
                }
                let class = class.unwrap().to_class().unwrap();
                Some(Class::instantiate(&class, vec![self.clone()]))
            }
            Value::String(_) => {
                let class = env.borrow().get("String");
                if class.is_none() {
                    return None;
                }
                let class = class.unwrap().to_class().unwrap();
                Some(Class::instantiate(&class, vec![self.clone()]))
            }
            _ => None,
        }
    }
}
