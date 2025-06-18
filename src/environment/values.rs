use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc, vec};

use crate::{
    ast::ast::{
        BinaryOperator, ControlFlow, Expr, MethodModifiers, MethodModifiersOperations, Stmt,
    },
    environment::stdlib::number::NativeNumberClass,
    interpreter::Interpreter,
};

use super::{
    native::native_callable::NativeCallable,
    stdlib::{array::NativeArrayClass, string::NativeStringClass},
    Environment,
};

#[derive(Clone, Debug)]
pub enum Value<T = String> {
    Void,                      // Primitivo
    Null,                      // Primitivo
    Bool(bool),                // Primitivo
    Number(NativeNumberClass), // Primitivo
    String(NativeStringClass), // Primitivo

    Array(NativeArrayClass),
    Object(Rc<RefCell<Vec<(String, Value)>>>),
    Function(Rc<Function>),

    Error(Rc<RefCell<Value>>),

    Class(Rc<Class>),
    Method(Rc<Method>),
    Instance(Rc<RefCell<Instance>>),

    This(Rc<Value>),
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
    InternalClass(Rc<RefCell<dyn NativeCallable>>),
    InternalFunction((String, Rc<RefCell<dyn NativeCallable>>)),

    Internal(T),
    Expr(Expr),
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
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::This(a), Value::This(b)) => Rc::ptr_eq(&*a, &*b),
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
                    .collect::<Vec<(String, Value)>>(),
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
    pub modifiers: Vec<MethodModifiers>,
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
        modifiers: Vec<MethodModifiers>,
        class: String,
        closure: Rc<RefCell<Environment>>,
    ) -> Method {
        Method {
            name,
            params,
            vararg,
            body,
            this,
            modifiers,
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
            modifiers: self.modifiers.clone(),
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

        let closure_rc = self.closure.clone();
        let closure = closure_rc.borrow().clone(); // <- `Ref` de closure é solto aqui

        let local_env = closure
            .with_parent(self.this.clone()) // agora é seguro
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
                ControlFlow::Return(val) => return val,
                ControlFlow::Break => {
                    panic!("Break not allowed in function '{name}'")
                }
                ControlFlow::Continue => {
                    panic!("Continue not allowed function '{name}'")
                }
                ControlFlow::None => {}
                ControlFlow::Error(err) => panic!("{}", err.to_string()),
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

    pub fn is_static(&self) -> bool {
        self.modifiers.contains(&MethodModifiers::Static)
    }
    pub fn is_private(&self) -> bool {
        self.modifiers.contains(&MethodModifiers::Private)
    }
    pub fn is_operator(&self) -> bool {
        self.modifiers.contains(&MethodModifiers::Operator)
    }
}
#[derive(Debug, Clone)]
pub struct Instance {
    pub this: Rc<RefCell<Environment>>,
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

    pub fn find_operation(&self, operator: &str) -> Option<Rc<Method>> {
        let class = self.class.clone();

        let method = class.find_method_with_modifiers(&operator, vec![MethodModifiers::Operator]);

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

        if method.is_none() {
            let method = self.class.get_value_of_method();
            return method;
        }
        method
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Rc<Method>>,
    pub static_methods: Vec<Rc<Method>>,

    pub instance_variables: Rc<RefCell<HashMap<String, Value>>>,
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

    pub fn instantiate(class: &Rc<Class>, args: Vec<Value>, interpreter: Interpreter) -> Value {
        let this = Environment::new_rc();
        // let closure = interpreter.env.clone();
        this.borrow_mut().copy_from(class.this.clone());
        // this.borrow_mut().copy_from(interpreter.env.clone());

        for (field_name, field_value) in class.instance_variables.borrow().clone() {
            this.borrow_mut()
                .define(field_name.clone(), field_value.clone());
        }

        // Vincula métodos com o ambiente correto
        for method in &class.methods {
            if !method.is_static() {
                let name = method.name.clone();

                // let function_env = method.this.clone();

                // function_env.borrow_mut().copy_from(this.clone());
                // let this_method_env = function_env.borrow().clone();
                method.this.replace(this.borrow().clone());

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

        value
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
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
        modifiers: Vec<MethodModifiers>,
    ) -> Option<Rc<Method>> {
        for method in &self.methods {
            if method.name == name {
                if method.modifiers.contains_all(modifiers.clone()) {
                    return Some(method.clone());
                }
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

    pub fn get_value_of_method(&self) -> Option<Rc<Method>> {
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
            Value::Object(_ref_cell) => todo!(),
            Value::Function(_function) => todo!(),
            Value::Class(this_class) => Rc::ptr_eq(this_class, &class.to_class()),
            Value::Method(_method) => todo!(),
            Value::Instance(instance) => {
                let inst = instance.borrow();
                let mut current = Some(&inst.class);

                while let Some(cls) = current {
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
            Value::This(_value) => todo!(),
            Value::Builtin(_) => todo!(),
            Value::InternalClass(_ref_cell) => todo!(),
            Value::InternalFunction(_) => todo!(),
            Value::Internal(_) => todo!(),
            _ => panic!(
                "is_instance_of: Value not supported: {:?}",
                instance.to_string()
            ),
        }
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

        let closure_rc = self.environment.clone();
        let closure = closure_rc.borrow().clone(); // <- `Ref` de closure é solto aqui

        let local_env = closure
            .with_parent(env) // agora é seguro
            .to_rc();

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
                ControlFlow::Return(val) => return val,
                ControlFlow::Break => {
                    panic!("Break not allowed in function {name}")
                }
                ControlFlow::Continue => {
                    panic!("Continue not allowed in function {name}")
                }
                ControlFlow::None => {}
                ControlFlow::Error(err) => panic!("{}", err.to_string()),
            }
        }

        Value::Void
    }
}
impl Value {
    pub fn is_error(&self) -> bool {
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
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(&*a, &*b),
            (Value::This(a), Value::This(b)) => Rc::ptr_eq(&*a, &*b),
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
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn get_native_class(&self) -> Option<Rc<RefCell<dyn NativeCallable>>> {
        match self {
            Value::InternalClass(n) => Some(n.clone()),
            Value::Array(a) => Some(Rc::new(RefCell::new(a.clone()))),
            Value::String(s) => Some(Rc::new(RefCell::new(s.clone()))),
            // Value::Number(n) => Some(Rc::new(RefCell::new(n.clone()))),
            _ => None,
        }
    }

    pub fn call_op(
        &self,
        operator: BinaryOperator,
        other: &Value,
        interpreter: Interpreter,
    ) -> Value {
        match operator {
            BinaryOperator::Add => self.resolve_op("plus", other, interpreter),
            BinaryOperator::Subtract => self.resolve_op("sub", other, interpreter),
            BinaryOperator::Multiply => self.resolve_op("mul", other, interpreter),
            BinaryOperator::Divide => self.resolve_op("div", other, interpreter),
            BinaryOperator::Modulo => self.resolve_op("mod", other, interpreter),
            BinaryOperator::Exponentiate => self.resolve_op("exp", other, interpreter),
        }
    }

    fn resolve_op(&self, op: &str, other: &Value, interpreter: Interpreter) -> Value {
        let left = self.clone();
        let right = other.clone();
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

            let plus_method = instance.borrow().find_operation(op);
            if let Some(method) = plus_method {
                let args = vec![other.clone()];
                return method.call(args, interpreter);
            }
        }
        Value::String((left.to_string() + &right.to_string()).into())
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
                    let value = method.call(vec![], Interpreter::new_empty());
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
            Value::InternalClass(_) => "class".to_string(),
            Value::InternalFunction(_) => "function".to_string(),
            Value::Error(_) => "error".to_string(),
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
                    let interpreter = Interpreter::new_empty();
                    let method = method.clone();

                    let new_method = Method::new(
                        method.name.clone(),
                        method.params.clone(),
                        method.vararg.clone(),
                        method.body.clone(),
                        method.this.clone(),
                        method.modifiers.clone(),
                        method.class.clone(),
                        method.closure.clone(),
                    );

                    let with_this = new_method.with_this(Value::Instance(instance.clone()));

                    let call = with_this.call(vec![], interpreter);
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
            Value::This(value) => value.to_string(),
            Value::Method(method) => format!("<function {}>", method.name),
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
            Value::String(s) => format!("\'{}\'", s.clone()),
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
                if let Some(method) = instance.borrow().get_to_string().as_ref() {
                    let interpreter = Interpreter::new_empty();
                    let method = method.clone();

                    let new_method = Method::new(
                        method.name.clone(),
                        method.params.clone(),
                        method.vararg.clone(),
                        method.body.clone(),
                        method.this.clone(),
                        method.modifiers.clone(),
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
            Value::Instance(instance) => {
                let value_of_method = instance.borrow().class.get_value_of_method();
                if let Some(value_of_method) = value_of_method {
                    value_of_method
                        .call(vec![], Interpreter::new_empty())
                        .to_number()
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
