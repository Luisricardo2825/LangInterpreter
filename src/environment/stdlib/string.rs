use std::{collections::HashMap, fmt::Display, ops::Add};

use serde::{Deserialize, Serialize};

use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, values::Value},
    impl_from_for_class, impl_logical_operations,
};

create_instance_fn!(NativeStringClass);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeStringClass {
    pub args: Vec<Value>,
    pub value: Option<String>,
    pub is_static: bool,
    pub custom_methods: HashMap<String, Value>,
}

impl NativeStringClass {
    pub fn new_with_value(value: String) -> Self {
        Self {
            args: vec![],
            value: Some(value),
            is_static: false,
            custom_methods: HashMap::new(),
        }
    }
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self {
            args,
            value: None,
            is_static: true,
            custom_methods: HashMap::new(),
        }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }
    pub fn len(&self) -> usize {
        self.get_value().chars().count()
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

    pub fn contains(&self, key: &NativeStringClass) -> bool {
        self.get_value().contains(&key.get_value())
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

impl Add<&NativeStringClass> for NativeStringClass {
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

impl_from_for_class!(String, String, NativeStringClass);

impl Display for NativeStringClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_value())
    }
}

impl_logical_operations!(NativeStringClass, NativeStringClass);
impl NativeCallable for NativeStringClass {
    fn new() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: true,
            custom_methods: HashMap::new(),
        }
    }
    fn call(&self, method_name: &str) -> ControlFlow<Value> {
        let mut args = self.get_args();
        if args.len() < 1 && self.args.len() > 0 {
            args = self.args.clone();
        }
        match method_name {
            "length" => {
                let this = self.get_value();

                ControlFlow::Return(Value::Number((this.len() as f64).into()))
            }
            "toUpperCase" => {
                let Value::String(s) = self.get_this() else {
                    unreachable!("Expected Value::String");
                };

                ControlFlow::Return(Value::String(s.get_value().to_uppercase().into()))
            }
            "toLowerCase" => {
                let arg = self.get_this();

                match arg {
                    Value::String(s) => ControlFlow::Return(Value::String(s.get_value().to_lowercase().into())),
                    _ => ControlFlow::Error(format!(
                        "Método nativo 'toLowerCase' esperava um argumento do tipo String, mas recebeu {}",
                        arg.type_of()
                    ).into()),
                }
            }
            "charAt" => {
                let arg = self.get_this();
                match arg {
                    Value::String(s) => {
                        let arg = args[0].clone();
                        match arg {
                            Value::Number(n) => {
                                let n = n.get_value() as i32;
                                if n < 0 || n >= s.len() as i32 {
                                    return ControlFlow::Return(Value::Null)
                                }
                                ControlFlow::Return(Value::String(s.get_value().chars().nth(n as usize).unwrap().to_string().into()))
                            }
                            _ => ControlFlow::Error(format!(
                                "Método nativo '{method_name}' esperava um segundo argumento do tipo Number, mas recebeu {}",
                                arg.type_of()
                            ).into()),
                        }
                    }
                    _ => ControlFlow::Error(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",arg.type_of()).into()),
                }
            }
            "charCodeAt" => {
                let value = self.get_this();
                match value {
                    Value::String(s) => {
                        let s = s.get_value();
                        let arg = args[0].clone();
                        match arg {
                            Value::Number(n) => {
                                let n = n.get_value() as i32;
                                if n < 0 || n >= s.len() as i32 {
                                    return ControlFlow::Return(Value::Null)
                                }
                                ControlFlow::Return(Value::Number((s.chars().nth(n as usize).unwrap() as i32 as f64).into()))
                            }
                            _ => ControlFlow::Error(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",arg.type_of()).into()),
                        }
                    }
                    _ => ControlFlow::Error(format!("Método nativo '{method_name}' esperava um argumento do tipo String, mas recebeu {}",
                        value.type_of()
                    ).into()),
                }
            }
            "slice" => {
                let Value::Number(start) = args[0].clone() else {
                    return ControlFlow::Error(
                        format!("Expected a number, got {}", args[0].type_of()).into(),
                    );
                };

                let end = args.get(1).unwrap_or(&Value::Null).clone();

                let v = self.get_value();
                let vec = v.chars().collect::<Vec<char>>();
                let start = start.get_value() as usize;
                let max_size = vec.len();

                if end.is_null() {
                    let slice = vec.get(start..).unwrap_or(&[]).to_vec();
                    // slice to string
                    let slice = slice.iter().collect::<String>();
                    return ControlFlow::Return(Value::String(slice.into()));
                }
                let Value::Number(end) = end else {
                    return ControlFlow::Error(
                        format!("Expected a number, got {}", end.type_of()).into(),
                    );
                };
                let mut end = end.get_value() as usize;

                if end > max_size {
                    end = max_size;
                }
                let slice = vec.get(start..end).unwrap_or(&[]).to_vec();
                let slice = slice.iter().collect::<String>();

                ControlFlow::Return(Value::String(slice.into()))
            }
            "valueOf" => {
                let arg = self.get_this();

                ControlFlow::Return(Value::String(arg.to_string().into()))
            }
            _ => ControlFlow::Error(format!("Método nativo desconhecido: {}", method_name).into()),
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

    fn get_custom_method(&self, _method_name: &str) -> Option<Value> {
        self.custom_methods.get(_method_name).cloned()
    }
    fn add_custom_method(&mut self, _method_name: String, _method: Value) -> Result<(), String> {
        self.custom_methods.insert(_method_name, _method);
        Ok(())
    }
}
