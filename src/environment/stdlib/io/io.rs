use std::io::Write;

use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, Value},
};

#[derive(Debug, Clone)]
pub struct NativeIoClass {
    args: Vec<Value>,
}

impl NativeIoClass {
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self { args }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }

    fn format_primitive_with_color(&self, val: &Value) -> String {
        match val {
            Value::String(s) => s.to_string(),      // verde com aspas
            other => self.format_with_color(other), // fallback sem cor
        }
    }

    fn format_with_color(&self, val: &Value) -> String {
        match val {
            Value::String(s) => format!("\x1b[32m\"{}\"\x1b[0m", s), // verde com aspas
            Value::Number(n) => {
                format!("\x1b[33m{}\x1b[0m", n.get_value())
            } // amarelo
            Value::Bool(b) => format!("\x1b[36m{}\x1b[0m", b),       // ciano
            Value::Null => format!("\x1b[90mnull\x1b[0m"),           // cinza
            Value::Void => String::new(),

            Value::Array(arr) => {
                let elements = arr
                    .get_value()
                    .borrow()
                    .iter()
                    .map(|item| self.format_with_color(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements)
            }

            Value::Object(obj) => {
                let props = obj
                    .borrow()
                    .iter()
                    .map(|(k, v)| {
                        let key_colored = format!("\x1b[35m\"{}\"\x1b[0m", k); // roxo para chaves
                        let val_colored = self.format_with_color(v);
                        format!("{}: {}", key_colored, val_colored)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", props)
            }
            Value::Instance(instance) => {
                let value_of_method = instance.borrow().get_value_of(); // Usa o valueOf se existir

                if value_of_method.is_some() {
                    let value_of_method = value_of_method.unwrap().unwrap();
                    let value = value_of_method.call(vec![val.clone()]);
                    if !value.is_err() {
                        let value = value.unwrap();

                        return self.format_primitive_with_color(&value);
                    }
                }
                let obj = val.convert_class_to_object();

                self.format_primitive_with_color(&obj)
            }
            Value::Class(_) | Value::InternalClass(_) => {
                format!("\x1b[34m{}\x1b[0m", val.to_string())
            } // azul para classes
            Value::Function(_) | Value::InternalFunction(_) => {
                format!("\x1b[31m{}\x1b[0m", val.to_string())
            } // vermelho para funções
            other => format!("{}", other.to_string()), // fallback sem cor
        }
    }
}

impl NativeCallable for NativeIoClass {
    fn new() -> Self {
        Self { args: vec![] }
    }
    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> ControlFlow<Value> {
        match method_name {
            "print" => match &args[..] {
                _ => {
                    let mut s = String::new();
                    for arg in args {
                        if arg.is_void() {
                            continue;
                        }
                        s += &arg.to_string();
                        // Add space
                        s += " ";
                    }
                    if !s.is_empty() {
                        s.pop(); // Remove last space
                        print!("{}", s);
                    }
                    ControlFlow::None
                }
            },
            "println" => {
                let s = args
                    .iter()
                    .filter(|arg| !arg.is_void())
                    .map(|arg| {
                        if arg.is_primitive() {
                            self.format_primitive_with_color(arg)
                        } else {
                            self.format_with_color(arg)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                if !s.is_empty() {
                    println!("{}", s);
                }

                ControlFlow::None
            }
            "readln" => match &args[..] {
                [Value::String(prompt)] => {
                    use std::io::{self, Write};

                    let prompt = prompt.to_string();
                    print!("{}", prompt);
                    io::stdout().flush().unwrap();

                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer).unwrap();
                    std::io::stdout().flush().unwrap();

                    ControlFlow::Return(Value::String(buffer.trim().to_string().into()))
                }
                [Value::String(msg), value] => {
                    print!("{}", msg);
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    std::io::stdout().flush().unwrap();
                    let input = input.trim();
                    if input.is_empty() {
                        return ControlFlow::Return(value.clone());
                    }
                    ControlFlow::Return(Value::String(input.trim().to_string().into()))
                }
                _ => {
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    ControlFlow::Return(Value::String(input.trim().to_string().into()))
                }
            },

            _ => ControlFlow::Error(Value::String(
                format!("Método nativo desconhecido: {}", method_name).into(),
            )),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        let methods = vec!["print", "println", "readln"];
        methods.iter().map(|s| s.to_string()).collect()
    }

    fn get_args(&self) -> Vec<Value> {
        self.args.clone()
    }

    fn add_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
        Ok(())
    }

    fn get_name(&self) -> String {
        "Io".to_string()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn set_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
        Ok(())
    }
}

pub fn create_instance() -> NativeIoClass {
    NativeIoClass::new()
}
