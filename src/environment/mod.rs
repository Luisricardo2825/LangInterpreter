use std::{collections::HashMap, io::Write};

use crate::ast::ast::Stmt;

#[derive(Clone, Debug)]
pub enum Value {
    Void,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Builtin(fn(Vec<Value>) -> Value), // função Rust nativa
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function { .. } => true,
            Value::Builtin(_) => true,
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
                for (i, v) in a.iter().enumerate() {
                    s += &v.to_string();
                    if i != a.len() - 1 {
                        s += ", ";
                    }
                }
                s += "]";
                s
            }
            Value::Object(o) => {
                let mut s = "{".to_string();
                for (i, (k, v)) in o.iter().enumerate() {
                    s += &format!("{}: {}", k, v.to_string());
                    if i != o.len() - 1 {
                        s += ", ";
                    }
                }
                s += "}";
                s
            }
            Value::Function { .. } => "<function>".to_string(),
            Value::Builtin(_) => "<internal>".to_string(),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function { .. } => true,
            Value::Builtin(_) => true,
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
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub id: String,
}

fn global() -> HashMap<String, Value> {
    let mut env: HashMap<String, Value> = HashMap::default();

    env.insert(
        "print".to_string(),
        Value::Builtin(|args: Vec<Value>| {
            let mut s = String::new();
            for arg in args {
                s += &arg.to_string();
            }
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
            }
            println!("{}", s);
            Value::Void
        }),
    );

    env.insert(
        "len".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::String(s)] => Value::Number(s.len() as f64),
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
                Value::Array(array)
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

    env
}

impl Environment {
    pub fn new() -> Self {
        let env = global();
        Self {
            variables: env,
            id: Environment::random_id(),
        }
    }

    pub fn new_enclosed(env: Environment) -> Self {
        let mut new_env = Environment::new();
        new_env.merge(env);
        new_env
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    pub fn merge(&mut self, other: Environment) {
        self.variables.extend(other.variables);
    }
    fn random_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const ID_LEN: usize = 10;
        let mut rng = rand::rng();

        (0..ID_LEN)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
