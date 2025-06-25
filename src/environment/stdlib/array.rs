use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, values::Value},
};

create_instance_fn!(NativeArrayClass);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeArrayClass {
    pub args: Vec<Value>,
    pub value: Option<Rc<RefCell<Vec<Value>>>>,
    pub is_static: bool,
}

impl NativeArrayClass {
    pub fn new_with_value(value: Rc<RefCell<Vec<Value>>>) -> Self {
        Self {
            args: vec![],
            value: Some(value),
            is_static: false,
        }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }

    pub fn get_value(&self) -> Rc<RefCell<Vec<Value>>> {
        if self.is_static {
            let val = self.args[0].clone();
            return Rc::new(RefCell::new(vec![val]));
        }
        self.value.clone().unwrap()
    }

    pub fn set_value(&mut self, value: Rc<RefCell<Vec<Value>>>) {
        self.value = Some(value);
    }

    pub fn get_this(&self) -> Value {
        Value::Array(self.get_value().into())
    }
}

impl From<Rc<RefCell<Vec<Value>>>> for NativeArrayClass {
    fn from(value: Rc<RefCell<Vec<Value>>>) -> Self {
        Self::new_with_value(value)
    }
}

impl PartialEq for NativeArrayClass {
    fn eq(&self, other: &Self) -> bool {
        if self.is_static && other.is_static {
            return Rc::ptr_eq(&self.get_value(), &other.get_value());
        } else if !self.is_static && !other.is_static {
            return Rc::ptr_eq(&self.get_value(), &other.get_value());
        } else {
            return false;
        }
    }
}

impl NativeCallable for NativeArrayClass {
    fn new() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: true,
        }
    }
    // TODO: Modificar metodos para static e instancia
    fn call(&self, method_name: &str) -> ControlFlow<Value> {
        let mut args = self.get_args();
        if args.len() < 1 && self.args.len() > 0 {
            args = self.args.clone();
        }
        match method_name {
            "push" => {
                let arg = args[0].clone();
                self.get_value().borrow_mut().push(arg);
                ControlFlow::None
            }
            "pop" => {
                let value = self.get_value().borrow_mut().pop();
                let value = value.unwrap();
                ControlFlow::Return(value)
            }
            "shift" => {
                let v = self.get_value();
                let mut vec = v.borrow_mut();
                if !vec.is_empty() {
                    ControlFlow::Return(vec.remove(0))
                } else {
                    ControlFlow::None
                }
            }
            "unshift" => {
                let value = args[0].clone();
                self.get_value().borrow_mut().insert(0, value.clone());
                ControlFlow::None
            }
            "slice" => {
                let Value::Number(start) = args[0].clone() else {
                    return ControlFlow::Error(
                        format!("Expected a number, got {}", args[0].type_of()).into(),
                    );
                };

                let end = args.get(1).unwrap_or(&Value::Null).clone();

                let v = self.get_value();
                let vec = v.borrow();
                let start = start.get_value() as usize;
                let max_size = vec.len();

                if end.is_null() {
                    let slice = vec.get(start..).unwrap_or(&[]).to_vec();
                    return ControlFlow::Return(Value::Array(Rc::new(RefCell::new(slice)).into()));
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
                ControlFlow::Return(Value::Array(Rc::new(RefCell::new(slice)).into()))
            }
            "concat" => {
                let Value::Array(arr2) = args[0].clone() else {
                    return ControlFlow::Error(
                        format!("Expected a array, got {}", args[0].type_of()).into(),
                    );
                };
                let mut result = self.get_value().borrow().clone();
                result.extend_from_slice(&arr2.get_value().borrow());
                ControlFlow::Return(Value::Array(Rc::new(RefCell::new(result)).into()))
            }
            "join" => {
                let Value::String(sep) = args[0].clone() else {
                    return ControlFlow::Error(
                        format!("Expected a String, got {}", args[0].type_of()).into(),
                    );
                };
                let sep = sep.get_value();
                let joined = self
                    .get_value()
                    .borrow()
                    .iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join(&sep);
                ControlFlow::Return(Value::String(joined.into()))
            }
            "reverse" => {
                self.get_value().borrow_mut().reverse();
                ControlFlow::None
            }
            "indexOf" => {
                let value = &args.get(0).unwrap_or(&Value::Void);
                let v = self.get_value();
                let vec = v.borrow();
                let index = vec
                    .iter()
                    .position(|v| v.equal(value))
                    .map(|i| i as f64)
                    .unwrap_or(-1.0);
                ControlFlow::Return(Value::Number(index.into()))
            }
            "lastIndexOf" => {
                let value = &args.get(0).unwrap_or(&Value::Void);

                let v = self.get_value();
                let vec = v.borrow();
                let index = vec
                    .iter()
                    .rposition(|v| v.equal(value))
                    .map(|i| i as f64)
                    .unwrap_or(-1.0);
                ControlFlow::Return(Value::Number(index.into()))
            }
            "includes" => {
                let value = &args.get(0).unwrap_or(&Value::Void);

                let found = self.get_value().borrow().iter().any(|v| v.equal(value));
                ControlFlow::Return(Value::Bool(found))
            }
            "toString" => {
                let this = self.get_this().to_string();
                ControlFlow::Return(Value::String(this.into()))
            }
            "isArray" => match &args[..] {
                [Value::Array(_)] => ControlFlow::Return(Value::Bool(true)),
                [_] => ControlFlow::Return(Value::Bool(false)),
                _ => ControlFlow::Error("isArray espera um único argumento".to_string().into()),
            },
            "length" => ControlFlow::Return(Value::Number(
                (self.get_value().borrow().len() as f64).into(),
            )),
            "sort" => {
                let vec = self.get_value();
                let mut array = vec.borrow_mut();
                array.sort_by(|a, b| a.to_string().partial_cmp(&b.to_string()).unwrap());
                ControlFlow::None
            }
            "of" => match &args[..] {
                values => ControlFlow::Return(Value::Array(
                    Rc::new(RefCell::new(values.iter().map(|v| v.clone()).collect())).into(),
                )),
            },

            "valueOf" => ControlFlow::Return(Value::Array(self.get_value().into())),
            _ => ControlFlow::Error(format!("Método nativo desconhecido: {}", method_name).into()),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        let methods = vec![
            "push",
            "pop",
            "shift",
            "unshift",
            "slice",
            "splice",
            "concat",
            "join",
            "reverse",
            "sort",
            "filter",
            "map",
            "reduce",
            "reduceRight",
            "every",
            "some",
            "indexOf",
            "lastIndexOf",
            "includes",
            "find",
            "findIndex",
            "fill",
            "copyWithin",
            "entries",
            "keys",
            "values",
            "forEach",
            "toString",
            "toLocaleString",
            "toSource",
            "isArray",
            "of",
            "length",
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
        Ok(Value::Array(Rc::new(RefCell::new(args.into())).into()))
    }
    fn get_name(&self) -> String {
        "Array".to_string()
    }

    fn is_static(&self) -> bool {
        false
    }
}
