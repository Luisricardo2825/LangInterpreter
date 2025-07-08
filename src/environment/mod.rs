pub mod environment;
pub mod helpers;
pub mod native;
pub mod stdlib;
pub mod test;
pub mod values;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use values::Value;

use crate::environment::values::NativeObjectTrait;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Environment {
    pub variables: Vec<(String, Value)>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

// export "EnvironmentMap" as "Environment"

fn global() -> Vec<(String, Value)> {
    let mut env: Vec<(String, Value)> = Vec::new();

    env.push((
        "len".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::String(s)] => Value::Number(s.chars().count().into()),
            [Value::Array(a)] => Value::Number(a.get_value().borrow().len().into()),
            _ => Value::Null,
        }),
    ));

    env.push((
        "range".to_string(),
        Value::Builtin(|args: Vec<Value>| match &args[..] {
            [Value::Number(num1), Value::Number(num2)] => {
                let num1 = num1.get_value();
                let num2 = num2.get_value();
                let mut array = Vec::new();
                for i in num1 as i64..num2 as i64 {
                    array.push(Value::Number(i.into()));
                }
                Value::array(array)
            }
            _ => Value::Null,
        }),
    ));

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

    // Importa todos os modulos nativos declarados
    for module_name in stdlib::list_modules() {
        if let Some(native_class) = stdlib::get_module_by_name(module_name) {
            // let borrow = native_class.borrow();
            // let module = borrow.new();
            let name = native_class.borrow().get_name().to_string();
            let value = Value::InternalClass(native_class);

            // println!("Declarando modulo {name} valor: {:?}", value.to_string());
            env.push((name, value));
        }
    }
    env.set_prop("NaN", Value::Number(f64::NAN.into())).unwrap();

    // find "Io"
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

    pub fn new_enclosed(parent: &mut Rc<RefCell<Environment>>) -> Self {
        Environment {
            variables: global(),
            parent: Some(Rc::clone(parent)),
        }
    }

    pub fn new_rc_enclosed(parent: &mut Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            variables: global(),
            parent: Some(Rc::clone(parent)),
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
        self.variables.iter().any(|(n, _)| n == name)
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<Environment>>> {
        self.parent.as_ref().and_then(|p| Some(p)).cloned()
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some((_, v)) = self.variables.iter().find(|(n, _)| n == name) {
            Some(v.clone())
        } else if let Some(parent) = self.get_parent() {
            let ret = parent.borrow().get(name);
            ret
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }

    // const RESERVED: &[&str] = &["String", "Boolean", "Array", "Object", "Function"];

    // fn is_reserved(name: &str) -> bool {
    //     Self::RESERVED.contains(&name)
    // }

    pub fn define(&mut self, name: String, value: Value) {
        // if Environment::is_reserved(&name) {
        //     panic!("Cannot define reserved word '{}'", name);
        // }
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
    }

    pub fn merge(&mut self, other: Environment) {
        self.variables.extend(other.variables);
    }

    pub fn with_parent(&self, parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            variables: self.variables.clone(), // ou shallow copy se possível
            parent: Some(parent),
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

    pub fn get_vars_name_value_from_parent(&self) -> String {
        let mut vars = HashMap::new();
        for (name, value) in self.get_vars_from_parent().iter() {
            vars.insert(name.clone(), value.to_string());
        }

        serde_json::to_string(&vars).unwrap()
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
