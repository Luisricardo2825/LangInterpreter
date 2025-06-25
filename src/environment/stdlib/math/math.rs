use crate::{
    ast::ast::ControlFlow,
    environment::{native::native_callable::NativeCallable, Value},
};

#[derive(Debug, Clone)]
pub struct NativeMathClass {
    args: Vec<Value>,
}

create_instance_fn!(NativeMathClass);

impl NativeMathClass {
    pub fn new_with_args(args: Vec<Value>) -> Self {
        Self { args }
    }
}

impl NativeCallable for NativeMathClass {
    fn new() -> Self {
        Self { args: vec![] }
    }

    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> ControlFlow<Value> {
        match method_name {
            "add" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    ControlFlow::Return(Value::Number((a.get_value() + b.get_value()).into()))
                }
                _ => ControlFlow::Error("Math.add espera dois números".to_string().into()),
            },
            "subtract" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    ControlFlow::Return(Value::Number((a.get_value() - b.get_value()).into()))
                }
                _ => ControlFlow::Error("Math.subtract espera dois números".to_string().into()),
            },
            "multiply" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    ControlFlow::Return(Value::Number((a.get_value() * b.get_value()).into()))
                }
                _ => ControlFlow::Error("Math.multiply espera dois números".to_string().into()),
            },
            "divide" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    let divisor = b.get_value();
                    if divisor == 0.0 {
                        ControlFlow::Error("Divisão por zero".to_string().into())
                    } else {
                        ControlFlow::Return(Value::Number((a.get_value() / divisor).into()))
                    }
                }
                _ => ControlFlow::Error("Math.divide espera dois números".to_string().into()),
            },
            "pow" => match &args[..] {
                [Value::Number(base), Value::Number(exp)] => ControlFlow::Return(Value::Number(
                    base.get_value().powf(exp.get_value()).into(),
                )),
                _ => ControlFlow::Error("Math.pow espera dois números".to_string().into()),
            },
            "abs" => match &args[..] {
                [Value::Number(x)] => {
                    ControlFlow::Return(Value::Number(x.get_value().abs().into()))
                }
                _ => ControlFlow::Error("Math.abs espera um número".to_string().into()),
            },
            "max" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    ControlFlow::Return(Value::Number(a.get_value().max(b.get_value()).into()))
                }
                _ => ControlFlow::Error("Math.max espera dois números".to_string().into()),
            },
            "min" => match &args[..] {
                [Value::Number(a), Value::Number(b)] => {
                    ControlFlow::Return(Value::Number(a.get_value().min(b.get_value()).into()))
                }
                _ => ControlFlow::Error("Math.min espera dois números".to_string().into()),
            },
            "ceil" => match &args[..] {
                [Value::Number(x)] => {
                    ControlFlow::Return(Value::Number(x.get_value().ceil().into()))
                }
                _ => ControlFlow::Error("Math.ceil espera um número".to_string().into()),
            },
            "floor" => match &args[..] {
                [Value::Number(x)] => {
                    ControlFlow::Return(Value::Number(x.get_value().floor().into()))
                }
                _ => ControlFlow::Error("Math.floor espera um número".to_string().into()),
            },
            "round" => match &args[..] {
                [Value::Number(x)] => {
                    ControlFlow::Return(Value::Number(x.get_value().round().into()))
                }
                _ => ControlFlow::Error("Math.round espera um número".to_string().into()),
            },
            "sqrt" => match &args[..] {
                [Value::Number(x)] => {
                    let value = x.get_value();
                    if value < 0.0 {
                        ControlFlow::Error("Raiz quadrada de número negativo".to_string().into())
                    } else {
                        ControlFlow::Return(Value::Number(value.sqrt().into()))
                    }
                }
                _ => ControlFlow::Error("Math.sqrt espera um número".to_string().into()),
            },
            _ => ControlFlow::Error(
                format!("Método nativo desconhecido: Math.{}", method_name).into(),
            ),
        }
    }

    fn methods_names(&self) -> Vec<String> {
        vec![
            "add", "subtract", "multiply", "divide", "pow", "abs", "max", "min", "ceil", "floor",
            "round", "sqrt",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn get_args(&self) -> Vec<Value> {
        self.args.clone()
    }

    fn add_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
        Ok(())
    }

    fn set_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
        Ok(())
    }

    fn get_name(&self) -> String {
        "Math".to_string()
    }

    fn is_static(&self) -> bool {
        true
    }
}
