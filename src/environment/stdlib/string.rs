use std::{fmt::Display, ops::Add};

use crate::environment::{native::native_callable::NativeCallable, values::Value};

#[derive(Debug, Clone)]
pub struct NativeStringClass {
    pub args: Vec<Value>,
    pub value: Option<String>,
    pub is_static: bool,
}

impl NativeStringClass {
    pub fn new() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: true,
        }
    }
    pub fn new_with_value(value: String) -> Self {
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
    pub fn len(&self) -> usize {
        self.get_value().len()
    }
    pub fn is_empty(&self) -> bool {
        self.get_value().is_empty()
    }

    pub fn get_value(&self) -> String {
        if self.is_static {
            return self.args[0].clone().to_string();
        }
        self.value.clone().unwrap()
    }

    pub fn get_this(&self) -> Value {
        Value::String(self.get_value().into())
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

impl Add<&NativeStringClass> for &NativeStringClass {
    type Output = NativeStringClass;

    fn add(self, rhs: &NativeStringClass) -> Self::Output {
        let mut value = self.get_value();
        value.push_str(&rhs.get_value());
        NativeStringClass::new_with_value(value)
    }
}

impl Add<&str> for &NativeStringClass {
    type Output = NativeStringClass;

    fn add(self, rhs: &str) -> Self::Output {
        let mut value = self.get_value();
        value.push_str(rhs);
        NativeStringClass::new_with_value(value)
    }
}

impl Add<String> for &NativeStringClass {
    type Output = NativeStringClass;

    fn add(self, rhs: String) -> Self::Output {
        let mut value = self.get_value();
        value.push_str(&rhs);
        NativeStringClass::new_with_value(value)
    }
}

impl From<String> for NativeStringClass {
    fn from(value: String) -> Self {
        Self::new_with_value(value)
    }
}

impl From<&str> for NativeStringClass {
    fn from(value: &str) -> Self {
        Self::new_with_value(value.to_string())
    }
}

impl Display for NativeStringClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_value())
    }
}

impl PartialEq for NativeStringClass {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl NativeCallable for NativeStringClass {
    fn call(&self, method_name: &str) -> Result<Value, String> {
        let mut args = self.get_args();
        if args.len() < 1 && self.args.len() > 0 {
            args = self.args.clone();
        }
        match method_name {
            "length" => {
                let this = self.get_value();

                Ok(Value::Number(this.len() as f64))
            }
            "toUpperCase" => {
                let Value::String(s) = self.get_this() else {
                    unreachable!("Expected Value::String");
                };

                Ok(Value::String(s.get_value().to_uppercase().into()))
            }
            "toLowerCase" => {
                let arg = self.get_this();

                match arg {
                    Value::String(s) => Ok(Value::String(s.get_value().to_lowercase().into())),
                    _ => Err(format!(
                        "Método nativo 'toLowerCase' esperava um argumento do tipo String, mas recebeu {}",
                        arg.type_of()
                    )),
                }
            }
            "charAt" => {
                let (_, num_of_args) = self.get_method_info(method_name);

                if args.len() != num_of_args {
                    return Err(format!(
                        "Método nativo '{method_name}' esperava {num_of_args} argumentos, mas recebeu {}",
                        args.len()
                    ));
                }

                let arg = self.get_this();
                match arg {
                    Value::String(s) => {
                        let arg = args[0].clone();
                        match arg {
                            Value::Number(n) => {
                                let n = n as i32;
                                if n < 0 || n >= s.len() as i32 {
                                    return Ok(Value::Null)
                                }
                                Ok(Value::String(s.get_value().chars().nth(n as usize).unwrap().to_string().into()))
                            }
                            _ => Err(format!(
                                "Método nativo '{method_name}' esperava um segundo argumento do tipo Number, mas recebeu {}",
                                arg.type_of()
                            )),
                        }
                    }
                    _ => Err(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",arg.type_of())),
                }
            }
            "charCodeAt" => {
                let (_, num_of_args) = self.get_method_info(method_name);

                if args.len() != num_of_args {
                    return Err(format!(
                        "Método nativo '{method_name}' esperava {num_of_args} argumentos, mas recebeu {}",
                        args.len()
                    ));
                }

                let value = self.get_this();
                match value {
                    Value::String(s) => {
                        let s = s.get_value();
                        let arg = args[0].clone();
                        match arg {
                            Value::Number(n) => {
                                let n = n as i32;
                                if n < 0 || n >= s.len() as i32 {
                                    return Ok(Value::Null)
                                }
                                Ok(Value::Number(s.chars().nth(n as usize).unwrap() as i32 as f64))
                            }
                            _ => Err(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",arg.type_of())),
                        }
                    }
                    _ => Err(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",
                        value.type_of()
                    )),
                }
            }
            "valueOf" => {
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
        let methods = vec![
            "length",
            "toUpperCase",
            "toLowerCase",
            "charAt",
            "charCodeAt",
            "concat",
            "indexOf",
            "lastIndexOf",
            "localeCompare",
            "match",
            "replace",
            "search",
            "slice",
            "split",
            "substring",
            "toLocaleLowerCase",
            "toLocaleUpperCase",
            "toLowerCase",
            "toUpperCase",
            "trim",
            "trimLeft",
            "trimRight",
            "valueOf",
            "toString",
        ];

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
                "Class 'String' expected 1 argument but received {}",
                args.len()
            ));
        }
        let arg = args[0].clone();
        Ok(Value::String(arg.to_string().into()))
    }

    fn get_name(&self) -> String {
        "String".to_string()
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
        MethodInfo::new("length", 0, false),
        MethodInfo::new("toUpperCase", 0, false),
        MethodInfo::new("toLowerCase", 0, false),
        MethodInfo::new("charAt", 1, false),
        MethodInfo::new("charCodeAt", 0, false),
        MethodInfo::new("concat", usize::MAX, true),
        MethodInfo::new("indexOf", 1, false),
        MethodInfo::new("lastIndexOf", 1, false),
        MethodInfo::new("localeCompare", 1, false),
        MethodInfo::new("match", 1, false),
        MethodInfo::new("replace", 1, false),
        MethodInfo::new("search", 1, false),
        MethodInfo::new("slice", 2, false),
        MethodInfo::new("split", 1, false),
        MethodInfo::new("substring", 2, false),
        MethodInfo::new("toLocaleLowerCase", 0, false),
        MethodInfo::new("toLocaleUpperCase", 0, false),
        MethodInfo::new("toLowerCase", 0, false),
        MethodInfo::new("toUpperCase", 0, false),
        MethodInfo::new("trim", 0, false),
        MethodInfo::new("trimLeft", 0, false),
        MethodInfo::new("trimRight", 0, false),
        MethodInfo::new("valueOf", 1, true),
        MethodInfo::new("toString", 0, true),
    ];
}
