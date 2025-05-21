use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use crate::ast::ast::{Expr, Stmt};

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
    Void,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),

    Class(Rc<Class>),
    Instance(Rc<RefCell<Instance>>),
    Function(Rc<Function>),

    This(Box<Value>),
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub class: Rc<Class>,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn list_fields(&self) -> Vec<(String, Value)> {
        let mut fields = Vec::new();
        for (name, value) in &self.fields {
            fields.push((name.clone(), value.clone()));
        }
        fields
    }
}
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
    pub statics_methods: Vec<Method>,

    pub instance_variables: HashMap<String, Value>,

    pub superclass: Option<Box<Value>>, // Value::Class
    pub this: Rc<RefCell<Environment>>,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub value: Value,
    pub is_static: bool,
}
impl Method {
    pub fn new(name: String, value: Function, is_static: bool) -> Method {
        Method {
            name,
            value: Value::Function(Rc::new(value)),
            is_static,
        }
    }
}

impl Class {
    pub fn new(
        name: String,
        methods: Vec<Method>,
        superclass: Option<Box<Value>>,
        this: Rc<RefCell<Environment>>,
        statics_methods: Vec<Method>,
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

    pub fn instantiate(&self) -> Value {
        // Cria a instância com os campos (fields)
        let instance = Rc::new(RefCell::new(Instance {
            class: Rc::new(self.clone()),
            fields: self.instance_variables.clone(),
        }));

        // Prepara o ambiente "this" para métodos da instância
        let mut instance_methods_env = Environment::new_enclosed(self.this.clone());
        instance_methods_env.define("this".to_string(), Value::Instance(Rc::clone(&instance)));

        // Vincula métodos com o ambiente correto
        for method in &self.methods {
            if !method.is_static {
                if let Value::Function(function) = method.value.clone() {
                    function
                        .closure
                        .borrow_mut()
                        .merge(instance_methods_env.clone());
                    instance
                        .borrow_mut()
                        .fields
                        .insert(method.name.clone(), Value::Function(function));
                }
            }
        }

        // (Opcional) preparar `constructors` sem executá-los aqui
        // Se quiser executar aqui, você precisa passar os argumentos e um interpretador

        println!("class name: {:?}", instance.borrow().class.name);
        Value::Instance(instance)
    }

    pub fn arity(&self) -> usize {
        if let Some(initializer) = &self.find_method("constructor", false) {
            if let Value::Function(initializer) = initializer {
                return initializer.params.len();
            }
        }
        0
    }

    pub fn find_method(&self, name: &str, is_static: bool) -> Option<Value> {
        for method in &self.methods {
            if method.name == name && method.is_static == is_static {
                return Some(method.value.clone());
            }
        }
        if let Some(superclass) = &self.superclass {
            if let Value::Class(superclass) = &**superclass {
                return superclass.find_method(name, is_static);
            }
        }
        None
    }

    pub fn find_methods_overloading(&self, name: &str, is_static: bool) -> Vec<Value> {
        let mut methods = Vec::new();
        for method in &self.methods {
            if method.name == name && method.is_static == is_static {
                methods.push(method.value.clone());
            }
        }
        if let Some(superclass) = &self.superclass {
            if let Value::Class(superclass) = &**superclass {
                methods.extend(superclass.find_methods_overloading(name, is_static));
            }
        }
        methods
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Function {
        Function {
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }
}
impl Value {
    pub fn array(vec: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(vec)))
    }
    pub fn object(map: HashMap<String, Value>) -> Value {
        Value::Object(Rc::new(RefCell::new(map)))
    }
    pub fn new_object() -> Value {
        Value::Object(Rc::new(RefCell::new(HashMap::new())))
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
            Value::Instance { .. } => "class".to_string(),
            Value::This(value) => value.type_of(),
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
            Value::Object(o) => {
                let mut s = "{".to_string();
                for (i, (k, v)) in o.borrow().iter().enumerate() {
                    s += &format!("{}: {}", k, v.to_string());
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
            Value::Instance(instance) => format!("<instance of {}>", instance.borrow().class.name),
            Value::This(value) => value.to_string(),
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
            Value::Object(o) => o.borrow().clone(),
            _ => panic!("Cannot convert {} to object", self.type_of()),
        }
    }
    pub fn to_class(&self) -> Rc<Class> {
        match self {
            Value::Class(class) => Rc::clone(class),
            Value::Instance(instance) => instance.borrow().class.clone(),
            _ => panic!("Cannot convert {} to class", self.type_of()),
        }
    }

    pub fn to_function(&self) -> Rc<Function> {
        match self {
            Value::Function(f) => f.clone(),
            _ => panic!("Cannot convert {} to function", self.type_of()),
        }
    }
    // pub fn get_constructor(&self, args: &Vec<Value>) -> Option<Function> {
    //     match self {
    //         Value::Instance(instance) => {
    //             if init.is_none() {
    //                 return None;
    //             }
    //             let init = init.clone().unwrap();
    //             for initializer in init {
    //                 if initializer.params.len() == args.len() {
    //                     return Some(initializer.to_owned());
    //                 }
    //             }
    //             None
    //         }
    //         _ => panic!("Cannot convert {} to instance", self.type_of()),
    //     }
    // }
    pub fn set_instance_property(&self, name: &str, value: Value) {
        if let Value::Instance(instance) = self {
            let env = instance.borrow().class.this.clone();
            env.borrow_mut().define(name.to_owned(), value);
        }
    }

    pub fn get_instance_property(&self, name: &str) -> Option<Value> {
        if let Value::Instance(instance) = self {
            let env = instance.borrow().class.this.clone();
            let x = env.borrow().get(name);
            x
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

fn pessoa_class() -> Class {
    let mut methods = Vec::new();
    let class_env = Rc::new(RefCell::new(Environment::new()));
    class_env
        .borrow_mut()
        .define("nome".to_string(), Value::String("Ricardo".to_string()));
    methods.push(Method::new(
        "falar".to_string(),
        Function::new(
            "falar".to_string(),
            vec![],
            vec![Stmt::Return(Some(crate::ast::ast::Expr::Literal(
                crate::ast::ast::Literal::String("Olá, eu sou uma pessoa!".to_string()),
            )))],
            Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
                &class_env,
            )))),
            false,
        ),
        false,
    ));
    methods.push(Method::new(
        "getIdade".to_string(),
        Function::new(
            "getIdade".to_string(),
            vec![],
            vec![Stmt::Return(Some(crate::ast::ast::Expr::Literal(
                crate::ast::ast::Literal::Number(19.0),
            )))],
            Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
                &class_env,
            )))),
            false,
        ),
        true,
    ));

    let mut instance_variables = HashMap::new();

    instance_variables.insert("nome".to_string(), Value::Null);
    methods.push(Method::new(
        "setNome".to_string(),
        Function::new(
            "setNome".to_string(),
            vec!["nome".to_string()],
            vec![Stmt::ExprStmt(Expr::SetProperty {
                object: Box::new(Expr::Identifier("this".to_string())),
                property: Box::new(Expr::Identifier("nome".to_string())),
                value: Box::new(Expr::Identifier("nome".to_string())),
            })],
            Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
                &class_env,
            )))),
            false,
        ),
        false,
    ));

    let static_methods = methods
        .iter()
        .filter(|method| method.is_static)
        .cloned()
        .collect::<Vec<_>>();
    let instance_methods = methods
        .iter()
        .filter(|method| !method.is_static)
        .cloned()
        .collect::<Vec<_>>();

    Class::new(
        "Pessoa".to_string(),
        instance_methods,
        None,
        class_env,
        static_methods,
        instance_variables,
    )
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
        "Pessoa".to_string(),
        Value::Builtin(|_| pessoa_class().instantiate()),
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

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn exist_in_current_scope(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

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

    pub fn get_vars(&self) -> HashMap<String, Value> {
        self.variables.clone()
    }
}
