use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use crate::environment::{native::native_callable::NativeCallable, values::Value};

#[derive(Debug, Clone)]
pub struct NativeNumberClass {
    pub args: Vec<Value>,
    pub value: Option<f64>,
    pub is_static: bool,
}

impl NativeNumberClass {
    pub fn new() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: true,
        }
    }
    pub fn new_with_value(value: f64) -> Self {
        Self {
            args: vec![],
            value: Some(value),
            is_static: false,
        }
    }
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self {
            args,
            value: None,
            is_static: true,
        }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }

    pub fn get_value(&self) -> f64 {
        if self.is_static {
            return self.args[0].clone().to_number();
        }
        self.value.clone().unwrap()
    }

    pub fn get_this(&self) -> Value {
        Value::Number(self.get_value().into())
    }

    pub fn get_method_info(&self, method_name: &str) -> (String, usize) {
        for method_info in methods_config() {
            let MethodInfo {
                name,
                num_of_args,
                is_static,
            } = method_info;
            if name == method_name && is_static == self.is_static {
                return (name.to_string(), num_of_args);
            }
        }
        panic!("Método '{}' não encontrado", method_name);
    }
}

impl Add<&NativeNumberClass> for NativeNumberClass {
    type Output = NativeNumberClass;

    fn add(self, rhs: &NativeNumberClass) -> Self::Output {
        let mut value = self.get_value();
        value += rhs.get_value();
        NativeNumberClass::new_with_value(value)
    }
}
impl Add<NativeNumberClass> for &NativeNumberClass {
    type Output = NativeNumberClass;

    fn add(self, rhs: NativeNumberClass) -> Self::Output {
        let mut value = self.get_value();
        value += rhs.get_value();
        NativeNumberClass::new_with_value(value)
    }
}
impl Add<&NativeNumberClass> for &NativeNumberClass {
    type Output = NativeNumberClass;

    fn add(self, rhs: &NativeNumberClass) -> Self::Output {
        let mut value = self.get_value();
        value += rhs.get_value();
        NativeNumberClass::new_with_value(value)
    }
}
impl Sub<&NativeNumberClass> for NativeNumberClass {
    type Output = NativeNumberClass;

    fn sub(self, rhs: &NativeNumberClass) -> Self::Output {
        let mut value = self.get_value();
        value -= rhs.get_value();
        NativeNumberClass::new_with_value(value)
    }
}
impl Sub<NativeNumberClass> for &NativeNumberClass {
    type Output = NativeNumberClass;

    fn sub(self, rhs: NativeNumberClass) -> Self::Output {
        let mut value = self.get_value();
        value -= rhs.get_value();
        NativeNumberClass::new_with_value(value)
    }
}

impl From<f64> for NativeNumberClass {
    fn from(value: f64) -> Self {
        Self::new_with_value(value)
    }
}
impl From<&f64> for NativeNumberClass {
    fn from(value: &f64) -> Self {
        Self::new_with_value(*value)
    }
}

impl Display for NativeNumberClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_value())
    }
}

impl PartialEq for NativeNumberClass {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl NativeCallable for NativeNumberClass {
    fn call(&self, method_name: &str) -> Result<Value, String> {
        let mut args = self.get_args();
        if args.len() < 1 && self.args.len() > 0 {
            args = self.args.clone();
        }
        match method_name {
            "valueOf" => {
                let (_, num_of_args) = self.get_method_info(method_name);

                if args.len() != num_of_args {
                    return Err(format!(
                        "Método nativo '{method_name}' esperava {num_of_args} argumentos, mas recebeu {}",
                        args.len()
                    ));
                }
                let arg = self.get_this();

                Ok(Value::Number(arg.to_number().into()))
            }
            "toString" => {
                let (_, num_of_args) = self.get_method_info(method_name);

                if args.len() != num_of_args {
                    return Err(format!(
                        "Método nativo '{method_name}' esperava {num_of_args} argumentos, mas recebeu {}",
                        args.len()
                    ));
                }
                let arg = self.get_this();

                Ok(Value::String(arg.to_string().into()))
            }
            _ => Err(format!("Método nativo desconhecido: {}", method_name)),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        let methods = vec!["valueOf", "toString"];

        methods.iter().map(|s| s.to_string()).collect()
    }

    fn get_args(&self) -> Vec<Value> {
        self.args.clone()
    }

    fn add_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args.extend(args);
        Ok(())
    }

    fn instantiate(&self, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!(
                "Class 'Number' expected 1 argument but received {}",
                args.len()
            ));
        }
        let arg = args[0].clone();
        Ok(Value::Number(arg.to_number().into()))
    }

    fn get_name(&self) -> String {
        "Number".to_string()
    }
}

pub struct MethodInfo {
    pub name: String,
    pub num_of_args: usize,
    pub is_static: bool,
}
impl MethodInfo {
    pub fn new(name: &str, num_of_args: usize, is_static: bool) -> Self {
        Self {
            name: name.to_string(),
            num_of_args,
            is_static,
        }
    }
}

fn methods_config() -> Vec<MethodInfo> {
    return vec![
        MethodInfo::new("valueOf", 1, true),
        MethodInfo::new("toString", 0, false),
    ];
}
