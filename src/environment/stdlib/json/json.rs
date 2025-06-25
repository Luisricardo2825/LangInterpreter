use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, Value},
};

#[derive(Debug, Clone)]
pub struct NativeJsonClass {
    args: Vec<Value>,
}

impl NativeJsonClass {
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self { args }
    }
    pub fn set_args(&mut self, args: Vec<Value>) {
        self.args = args;
    }
}

impl NativeCallable for NativeJsonClass {
    fn new() -> Self {
        Self { args: vec![] }
    }
    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> ControlFlow<Value> {
        match method_name {
            "parse" => {
                let json_string = args[0].to_string();
                let json: serde_json::Value = serde_json::from_str(&json_string).unwrap();
                ControlFlow::Return(Value::from(json))
            }

            "stringify" => {
                let json = args[0].stringfy();
                // let json_string = serde_json::to_string(&json).unwrap();
                ControlFlow::Return(Value::String(json.into()))
            }

            _ => ControlFlow::Error(format!("Método nativo desconhecido: {}", method_name).into()),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        let methods = vec!["parse", "stringfy"];
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
        "JSON".to_string()
    }

    fn is_static(&self) -> bool {
        true
    }
}

pub fn create_instance() -> NativeJsonClass {
    NativeJsonClass::new()
}
