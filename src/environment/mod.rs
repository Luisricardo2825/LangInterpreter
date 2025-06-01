pub mod native;
pub mod stdlib;
pub mod values;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use stdlib::{
    array::NativeArrayClass, fs::fs::NativeFsClass, io::io::NativeIoClass,
    json::json::NativeJsonClass,
};
use values::Value;

#[derive(Clone, Debug)]
pub struct Environment {
    pub variables: Vec<(String, Value)>,
    pub parent: Option<Weak<RefCell<Environment>>>,
}

fn global() -> Vec<(String, Value)> {
    let mut env: Vec<(String, Value)> = Vec::new();

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

    env.push((
        "now".to_string(),
        Value::Builtin(|_args: Vec<Value>| {
            let now = std::time::SystemTime::now();
            let since_the_epoch = now
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            let in_ms = since_the_epoch.as_millis();
            Value::Number((in_ms as f64).into())
        }),
    ));

    env.push((
        "Io".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeIoClass::new()))),
    ));
    env.push((
        "Array".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeArrayClass::new()))),
    ));
    env.push((
        "Fs".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeFsClass::new()))),
    ));
    env.push((
        "Json".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(NativeJsonClass::new()))),
    ));
    env.push((
        "String".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(
            stdlib::string::NativeStringClass::new(),
        ))),
    ));
    env.push((
        "Number".to_owned(),
        Value::NativeClass(Rc::new(RefCell::new(
            stdlib::number::NativeNumberClass::new(),
        ))),
    ));
    env
}

impl Environment {
    pub fn new() -> Self {
        let global = global();
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
            variables: global(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }

    pub fn new_rc_enclosed(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            variables: global(),
            parent: Some(Rc::downgrade(&parent)),
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
            parent: Some(Rc::downgrade(&parent)),
        }))
    }

    pub fn copy_from(&mut self, other: Rc<RefCell<Environment>>) {
        self.variables = other.borrow().variables.clone();
    }

    pub fn exist_in_current_scope(&self, name: &str) -> bool {
        self.variables.iter().any(|(n, _)| n == name)
    }

    fn get_parent(&self) -> Option<Rc<RefCell<Environment>>> {
        self.parent
            .as_ref()
            .and_then(|weak_parent| weak_parent.upgrade()) // Tentar obter Rc de Weak
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some((_, v)) = self.variables.iter().find(|(n, _)| n == name) {
            Some(v.clone())
        } else if let Some(parent) = self.get_parent() {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }

    const RESERVED: &[&str] = &["Number", "String", "Boolean", "Array", "Object", "Function"];

    fn is_reserved(name: &str) -> bool {
        Self::RESERVED.contains(&name)
    }

    pub fn define(&mut self, name: String, value: Value) {
        if Environment::is_reserved(&name) {
            panic!("Cannot define reserved word '{}'", name);
        }
        match self.variables.iter_mut().find(|(n, _)| n == &name) {
            Some((_, v)) => *v = value,
            None => self.variables.push((name, value)),
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some((_, v)) = self.variables.iter_mut().find(|(n, _)| n == name) {
            *v = value;
            Ok(())
        } else if let Some(parent) = self.get_parent() {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    pub fn exist(&self, name: &str) -> bool {
        self.variables.iter().any(|(n, _)| n == name)
            || self
                .get_parent()
                .map_or(false, |parent| parent.borrow().exist(name))
    }

    pub fn merge(&mut self, other: Environment) {
        self.variables.extend(other.variables);
    }

    pub fn from_parent(&mut self, parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            variables: self.variables.to_owned(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }

    pub fn to_rc(&self) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(self.clone()))
    }
    pub fn get_vars(&self) -> Vec<(String, Value)> {
        self.variables.clone()
    }

    pub fn get_vars_name_value(&self) -> String {
        let mut vars = HashMap::new();
        for (name, value) in self.variables.iter() {
            vars.insert(name.clone(), value.to_string());
        }

        serde_json::to_string(&vars).unwrap()
    }
    pub fn get_vars_from_parent(&self) -> Vec<(String, Value)> {
        let mut vars = self.variables.clone();
        if let Some(parent) = self.get_parent() {
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
