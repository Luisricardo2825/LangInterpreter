use std::io::Write;

use crate::environment::{native::native_callable::NativeCallable, Value};

#[derive(Debug, Clone)]
pub struct NativeFsClass {
    args: Vec<Value>,
}

impl NativeFsClass {
    pub fn new() -> Self {
        Self { args: vec![] }
    }

    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self { args }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }
}

impl NativeCallable for NativeFsClass {
    fn call(&self, method_name: &str) -> Result<Value, String> {
        let args = self.get_args();

        match method_name {
            "write" => {
                let mut file = std::fs::File::create(args[0].to_string()).unwrap();
                let ret = file.write_all(args[1].to_string().as_bytes()).is_ok();
                Ok(Value::Bool(ret))
            }

            "readFile" => {
                let file = std::fs::read_to_string(args[0].to_string()).unwrap();
                Ok(Value::String(file.into()))
            }
            _ => Err(format!("Método nativo desconhecido: {}", method_name)),
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
}
