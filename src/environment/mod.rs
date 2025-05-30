pub mod native;
pub mod stdlib;
pub mod values;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use stdlib::{
    array::NativeArrayClass, fs::fs::NativeFsClass, io::io::NativeIoClass,
    json::json::NativeJsonClass,
};
use values::Value;

#[derive(Clone, Debug)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

fn global() -> HashMap<String, Value> {
    let mut env: HashMap<String, Value> = HashMap::default();

    // env.insert(
    //     "print".to_string(),
    //     Value::Builtin(|args: Vec<Value>| {
    //         let mut s = String::new();
    //         for arg in args {
    //             s += &arg.to_string();
    //             // Add space
    //             s += " ";
    //         }
    //         // Remove last space
    //         s.pop();
    //         print!("{}", s);
    //         Value::Void
    //     }),
    // );

    // env.insert(
    //     "println".to_string(),
    //     Value::Builtin(|args: Vec<Value>| {
    //         // Concat all args and print
    //         let mut s = String::new();
    //         for arg in args {
    //             s += &arg.to_string();
    //             // Add space
    //             s += " ";
    //         }
    //         // Remove last space
    //         s.pop();
    //         println!("{}", s);
    //         Value::Void
    //     }),
    // );

    // env.insert(
    //     "len".to_string(),
    //     Value::Builtin(|args: Vec<Value>| match &args[..] {
    //         [Value::String(s)] => Value::Number(s.len() as f64),
    //         [Value::Array(a)] => Value::Number(a.get_value().borrow().len() as f64),
    //         _ => Value::Null,
    //     }),
    // );

    // // input
    // env.insert(
    //     "input".to_string(),
    //     Value::Builtin(|args: Vec<Value>| match &args[..] {
    //         [Value::String(msg)] => {
    //             print!("{}", msg);
    //             std::io::stdout().flush().unwrap();

    //             let mut input = String::new();
    //             std::io::stdin().read_line(&mut input).unwrap();
    //             Value::String(input.trim().to_string().into())
    //         }
    //         [Value::String(msg), value] => {
    //             print!("{}", msg);
    //             std::io::stdout().flush().unwrap();

    //             let mut input = String::new();
    //             std::io::stdin().read_line(&mut input).unwrap();
    //             let input = input.trim();
    //             if input.is_empty() {
    //                 return value.clone();
    //             }
    //             Value::String(input.trim().to_string().into())
    //         }
    //         _ => Value::Null,
    //     }),
    // );

    // env.insert(
    //     "range".to_string(),
    //     Value::Builtin(|args: Vec<Value>| match &args[..] {
    //         [Value::Number(num1), Value::Number(num2)] => {
    //             let mut array = Vec::new();
    //             for i in *num1 as i64..*num2 as i64 {
    //                 array.push(Value::Number(i as f64));
    //             }
    //             Value::array(array)
    //         }
    //         _ => Value::Null,
    //     }),
    // );

    env.insert(
        "now".to_string(),
        Value::Builtin(|_args: Vec<Value>| {
            let now = std::time::SystemTime::now();
            let since_the_epoch = now
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            let in_ms = since_the_epoch.as_millis();
            Value::Number((in_ms as f64).into())
        }),
    );

    env.insert(
        "Io".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeIoClass::new()))),
    );
    env.insert(
        "Array".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeArrayClass::new()))),
    );
    env.insert(
        "Fs".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeFsClass::new()))),
    );
    env.insert(
        "Json".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeJsonClass::new()))),
    );
    env.insert(
        "String".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(
            stdlib::string::NativeStringClass::new(),
        ))),
    );
    env.insert(
        "Number".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(
            stdlib::number::NativeNumberClass::new(),
        ))),
    );
    env
}

impl Environment {
    pub fn new() -> Self {
        let global = global();
        // let io = io::io();
        // global.extend(io);
        Environment {
            variables: global,
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

    pub fn new_rc_merged(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let mut env = Environment::new();
        env.merge_environments(parent.borrow().clone());
        Rc::new(RefCell::new(env))
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

    pub fn get_vars_name_value(&self) -> String {
        let mut vars = HashMap::new();
        for (name, value) in self.variables.iter() {
            vars.insert(name.clone(), value.to_string());
        }

        serde_json::to_string(&vars).unwrap()
    }
    pub fn get_vars_from_parent(&self) -> HashMap<String, Value> {
        let mut vars = self.variables.clone();
        if let Some(parent) = &self.parent {
            vars.extend(parent.borrow().get_vars_from_parent());
        }
        vars
    }

    pub fn get_vars_string(&self) -> String {
        let old_vars = self.get_vars_from_parent();
        let mut vars = HashMap::new();
        for (name, value) in old_vars.iter() {
            vars.insert(name.clone(), value.to_string());
        }
        serde_json::to_string(&vars).unwrap()
    }
}
