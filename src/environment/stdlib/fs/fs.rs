use std::io::Write;

use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, Value},
};

create_instance_fn!(NativeFsClass);

#[derive(Debug, Clone)]
pub struct NativeFsClass {
    args: Vec<Value>,
}

impl NativeFsClass {
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self { args }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }
}

impl NativeCallable for NativeFsClass {
    fn new() -> Self {
        Self { args: vec![] }
    }
    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> ControlFlow<Value> {
        match method_name {
            "write" => {
                let mut file = std::fs::File::create(args[0].to_string()).unwrap();
                let ret = file.write_all(args[1].to_string().as_bytes()).is_ok();
                ControlFlow::Return(Value::Bool(ret))
            }
            "writeLine" => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(args[0].to_string())
                    .unwrap();

                let line = args[1].to_string() + "\n";
                let ret = file.write_all(line.as_bytes()).is_ok();
                ControlFlow::Return(Value::Bool(ret))
            }
            "readFile" => {
                let file = std::fs::read_to_string(args[0].to_string()).unwrap();
                ControlFlow::Return(Value::String(file.into()))
            }
            _ => ControlFlow::Error(format!("Método nativo desconhecido: {}", method_name).into()),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        let methods = vec!["write", "readFile"];
        methods.iter().map(|s| s.to_string()).collect()
    }

    fn get_args(&self) -> Vec<Value> {
        self.args.clone()
    }
    fn add_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args.extend(args);
        Ok(())
    }
    fn get_name(&self) -> String {
        "Fs".to_string()
    }
    fn is_static(&self) -> bool {
        true
    }
}
