use std::io::Write;

use crate::environment::{native::native_callable::NativeCallable, Value};

#[derive(Debug, Clone)]
pub struct NativeIoClass {
    args: Vec<Value>,
}

impl NativeIoClass {
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

impl NativeCallable for NativeIoClass {
    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> Result<Value, String> {
        match method_name {
            "print" => match &args[..] {
                [Value::String(s)] => {
                    print!("{}", s);
                    std::io::stdout().flush().unwrap();
                    Ok(Value::Void)
                }
                _ => Err("print espera uma string".to_string()),
            },
            "println" => match &args[..] {
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
                    // Remove last space
                    s.pop();
                    println!("{}", s);
                    Ok(Value::Void)
                }
            },
            "readln" => match &args[..] {
                [Value::String(prompt)] => {
                    use std::io::{self, Write};

                    let prompt = prompt.to_string();
                    print!("{}", prompt);
                    io::stdout().flush().unwrap();

                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer).unwrap();
                    std::io::stdout().flush().unwrap();

                    Ok(Value::String(buffer.trim().to_string().into()))
                }
                [Value::String(msg), value] => {
                    print!("{}", msg);
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    std::io::stdout().flush().unwrap();
                    let input = input.trim();
                    if input.is_empty() {
                        return Ok(value.clone());
                    }
                    Ok(Value::String(input.trim().to_string().into()))
                }
                _ => {
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    Ok(Value::String(input.trim().to_string().into()))
                }
            },

            _ => Err(format!("Método nativo desconhecido: {}", method_name)),
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
        "Fs".to_string()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn set_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
        Ok(())
    }
}
