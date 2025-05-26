use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use crate::ast::ast::Stmt;

#[derive(Debug)]
pub enum ControlFlow {
    Return(Value),
    Break,
    Continue,
    None,
}

impl ControlFlow {
    pub fn is_none(&self) -> bool {
        match self {
            ControlFlow::None => true,
            _ => false,
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            ControlFlow::None => false,
            _ => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Void,                                        // Primitivo
    Null,                                        // Primitivo
    Bool(bool),                                  // Primitivo
    Number(f64),                                 // Primitivo
    String(String),                              // Primitivo
    Array(Rc<RefCell<Vec<Value>>>),              // Primitivo
    Object(Rc<RefCell<HashMap<String, Value>>>), // Primitivo
    Function(Rc<Function>),

    Class(Rc<Class>),
    Method(Rc<Method>),
    Instance(Rc<Instance>),
    // Method(Rc<Method>),
    This(Box<Value>),
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub this: Rc<RefCell<Environment>>,
    pub is_static: bool,
    pub class: String,
}

impl Method {
    pub fn new(
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        this: Rc<RefCell<Environment>>,
        is_static: bool,
        class: String,
    ) -> Method {
        Method {
            name,
            params,
            body,
            this,
            is_static,
            class,
        }
    }

    pub fn bind(&self, instance: Value) -> Method {
        Method {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            this: self.this.clone(),
            is_static: self.is_static,
            class: self.class.clone(),
        }
        .with_this(instance)
    }

    fn with_this(mut self, instance: Value) -> Self {
        self.this.borrow_mut().define("this".to_string(), instance);
        self
    }
    // pub fn to_function(&self) -> Rc<Function> {
    //     match &self.body {
    //         Value::Function(function) => function.clone(),
    //         _ => panic!("Method body is not a function"),
    //     }
    // }
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
                method.bind(Value::Instance(Rc::new(self.clone()))),
            )))
        } else {
            self.this.borrow().get(name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Rc<Method>>,
    pub statics_methods: Vec<Rc<Method>>,

    pub instance_variables: HashMap<String, Value>,

    pub superclass: Option<Box<Value>>, // Value::Class
    pub this: Rc<RefCell<Environment>>,
}

impl Class {
    pub fn new(
        name: String,
        methods: Vec<Rc<Method>>,
        superclass: Option<Box<Value>>,
        this: Rc<RefCell<Environment>>,
        statics_methods: Vec<Rc<Method>>,
        instance_variables: HashMap<String, Value>,
    ) -> Class {
        Class {
            name,
            methods,
            superclass,
            this,
            statics_methods,
            instance_variables,
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

    pub fn instantiate(&self) -> Value {
        let this = Environment::new_rc();
        this.borrow_mut().copy_from(self.this.clone());

        for (field_name, field_value) in self.instance_variables.clone() {
            this.borrow_mut()
                .define(field_name.clone(), field_value.clone());
        }

        // Vincula métodos com o ambiente correto
        for method in &self.methods {
            if !method.is_static {
                let name = method.name.clone();

                let function_env = method.this.clone();

                function_env.borrow_mut().copy_from(this.clone());
                let this_method_env = function_env.borrow().clone();
                method.this.replace(this_method_env);

                let body = Value::Method(method.to_owned().into());

                this.borrow_mut().define(name.clone(), body.clone());
            }
        }

        let instance = Instance {
            class: Rc::new(self.clone()),
            this: this,
        };

        Value::Instance(Rc::new(instance))
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        for method in &self.methods {
            if method.name == name {
                return Some(method.clone());
            }
        }
        None
    }
    pub fn find_static_method(&self, name: &str) -> Option<Rc<Method>> {
        for method in &self.statics_methods {
            if method.name == name {
                return Some(method.clone());
            }
        }
        None
    }

    pub fn get_all_methods_names(&self) -> Vec<String> {
        let mut methods_names = Vec::new();
        for method in &self.methods {
            methods_names.push(method.name.clone());
        }
        for method in &self.statics_methods {
            methods_names.push(method.name.clone());
        }
        methods_names
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub environment: Rc<RefCell<Environment>>,
    pub is_static: bool,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
        is_static: bool,
    ) -> Function {
        Function {
            name,
            params,
            body,
            environment,
            is_static,
        }
    }
}
impl Value {
    pub fn array(vec: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(vec)))
    }
    pub fn instance(class: Rc<Class>) -> Value {
        let instance = Instance {
            class: class.clone(),
            this: Environment::new_rc(),
        };
        Value::Instance(Rc::new(instance))
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

    pub fn is_instance(&self) -> bool {
        match self {
            Value::Instance(_) => true,
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

    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_) => true,
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
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
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
            Value::Instance { .. } => "instance".to_string(),
            Value::This(value) => value.type_of(),
            Value::Method(_) => "function".to_string(),
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
                for (i, v) in a.borrow().iter().enumerate() {
                    s += &v.to_string();
                    if i != a.borrow().len() - 1 {
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
            Value::Instance(_) => self.convert_class_to_object().stringfy(),
            Value::This(value) => value.to_string(),
            Value::Method(_) => "<function>".to_string(),
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
                for (i, v) in a.borrow().iter().enumerate() {
                    s += &v.stringfy();
                    if i != a.borrow().len() - 1 {
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
                let vars = instance.this.borrow().get_vars();
                // only json valid values
                let vars = vars
                    .iter()
                    .filter(|(_, v)| v.is_primitive() || self.is_object() || self.is_array())
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
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
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

    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::String(s) => {
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
            _ => panic!("Cannot convert {} to number", self.type_of()),
        }
    }

    pub fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(a) => a.borrow().clone(),
            _ => panic!("Cannot convert {} to array", self.type_of()),
        }
    }
    pub fn to_object(&self) -> HashMap<String, Value> {
        match self {
            Value::Object(o) => o.borrow().clone().into(),
            _ => panic!("Cannot convert {} to object", self.type_of()),
        }
    }
    pub fn to_class(&self) -> Rc<Class> {
        match self {
            Value::Class(class) => Rc::clone(class),
            Value::Instance(instance) => instance.class.clone(),
            _ => panic!("Cannot convert {} to class", self.type_of()),
        }
    }

    pub fn to_function(&self) -> Rc<Function> {
        match self {
            Value::Function(f) => f.clone(),
            _ => panic!("Cannot convert {} to function", self.type_of()),
        }
    }

    pub fn to_instance(&self) -> Rc<Instance> {
        match self {
            Value::Instance(instance) => instance.clone(),
            _ => panic!("Cannot convert {} to instance", self.type_of()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

fn global() -> HashMap<String, Value> {
    let mut env: HashMap<String, Value> = HashMap::default();

    env.insert(
        "print".to_string(),
        Value::Builtin(|args: Vec<Value>| {
            let mut s = String::new();
            for arg in args {
                s += &arg.to_string();
                // Add space
                s += " ";
            }
            // Remove last space
            s.pop();
            print!("{}", s);
            Value::Void
        }),
    );

    env.insert(
        "println".to_string(),
        Value::Builtin(|args: Vec<Value>| {
            // Concat all args and print
            let mut s = String::new();
            for arg in args {
                s += &arg.to_string();
                // Add space
                s += " ";
            }
            // Remove last space
            s.pop();
            println!("{}", s);
            Value::Void
        }),
    );

    env.insert(
        "len".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::String(s)] => Value::Number(s.len() as f64),
            [Value::Array(a)] => Value::Number(a.borrow().len() as f64),
            _ => Value::Null,
        }),
    );

    // input
    env.insert(
        "input".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::String(msg)] => {
                print!("{}", msg);
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Value::String(input.trim().to_string())
            }
            [Value::String(msg), value] => {
                print!("{}", msg);
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input.is_empty() {
                    return value.clone();
                }
                Value::String(input.trim().to_string())
            }
            _ => Value::Null,
        }),
    );

    env.insert(
        "range".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::Number(num1), Value::Number(num2)] => {
                let mut array = Vec::new();
                for i in *num1 as i64..*num2 as i64 {
                    array.push(Value::Number(i as f64));
                }
                Value::array(array)
            }
            _ => Value::Null,
        }),
    );

    env.insert(
        "now".to_string(),
        Value::Builtin(|_args: Vec<Value>| {
            let now = std::time::SystemTime::now();
            let since_the_epoch = now
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            let in_ms = since_the_epoch.as_millis();
            Value::Number(in_ms as f64)
        }),
    );

    env.insert(
        "typeof".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [arg] => Value::String(arg.type_of()),
            _ => Value::Null,
        }),
    );
    env.insert(
        "toNumber".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [arg] => Value::Number(arg.to_number()),
            _ => Value::Null,
        }),
    );
    env.insert(
        "push".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::Array(arr), value] => {
                arr.borrow_mut().push(value.clone());
                Value::Void
            }
            _ => Value::Null,
        }),
    );
    env
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: global(),
            parent: None,
        }
    }

    pub fn new_rc() -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment::new()))
    }

    pub fn merge_environments(&mut self, other: Environment) {
        self.variables.extend(other.variables);
    }

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn new_rc_enclosed(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            variables: global(),
            parent: Some(parent),
        }))
    }
    pub fn rc_enclosed(&self, parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            variables: global(),
            parent: Some(parent),
        }))
    }

    pub fn copy_from(&mut self, other: Rc<RefCell<Environment>>) {
        self.variables = other.borrow().variables.clone();
    }

    pub fn exist_in_current_scope(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if self.variables.contains_key(name) {
            Some(self.variables.get(name).unwrap().clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    #[track_caller]
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    pub fn exist(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn merge(&mut self, other: Environment) {
        self.variables.extend(other.variables);
    }

    pub fn from_parent(&mut self, parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            variables: self.variables.to_owned(),
            parent: Some(parent),
        }
    }

    pub fn to_rc(&self) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(self.clone()))
    }
    pub fn get_vars(&self) -> HashMap<String, Value> {
        self.variables.clone()
    }

    pub fn get_vars_name_value(&self) -> Vec<(String, String)> {
        self.variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().to_string()))
            .collect()
    }
}
