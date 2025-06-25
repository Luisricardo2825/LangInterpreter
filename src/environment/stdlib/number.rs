use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    ast::ast::{ControlFlow, Expr, MethodDecl, Stmt},
    environment::{
        helpers::class::ClassGenerator, native::native_callable::NativeCallable, values::Value,
    },
    impl_from_for_class, impl_logical_operations, impl_math_operations,
};

create_instance_fn!(NativeNumberClass);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeNumberClass {
    pub args: Vec<Value>,
    pub value: Option<f64>,
    pub is_static: bool,
}

impl NativeNumberClass {
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

    pub fn create_class() -> Stmt {
        let mut instance_fields = HashMap::new();
        let static_fields = HashMap::new();

        instance_fields.insert(
            "value".to_string(),
            Expr::Literal(crate::ast::ast::Literal::Null),
        );

        let method_value_of = MethodDecl {
            name: "valueOf".to_string(),
            params: vec![],
            body: vec![Stmt::Return(Some(crate::ast::ast::Expr::GetProperty {
                object: Box::new(Expr::This),
                property: Box::new(Expr::Identifier("value".to_string())),
            }))],
            modifiers: vec![],
            vararg: None,
        };

        let constructor = MethodDecl {
            name: "constructor".to_string(),
            params: vec!["value".to_string()],
            body: vec![Stmt::ExprStmt(ClassGenerator::set_prop_from_this(
                "value".to_string(),
                Expr::Identifier("value".to_string()),
            ))],
            modifiers: vec![],
            vararg: None,
        };

        let class_stmt = Stmt::ClassDecl {
            name: "Number".to_string(),
            superclass: None,
            methods: vec![method_value_of, constructor],
            static_fields: static_fields,
            instance_fields: instance_fields,
        };

        return class_stmt;
    }
}

impl_math_operations!(NativeNumberClass);

impl_from_for_class!(
    [f64, i32, i64, u32, u64, isize, usize],
    f64,
    NativeNumberClass
);

impl_logical_operations!(NativeNumberClass, NativeNumberClass);

impl Display for NativeNumberClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_value())
    }
}

#[allow(unused_assignments)]
impl NativeCallable for NativeNumberClass {
    fn new() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: true,
        }
    }
    fn call(&self, method_name: &str) -> ControlFlow<Value> {
        let mut args = self.get_args();
        if args.len() < 1 && self.args.len() > 0 {
            args = self.args.clone();
        }
        match method_name {
            "valueOf" => {
                let arg = self.get_this();

                ControlFlow::Return(Value::Number(arg.to_number().into()))
            }
            "toString" => {
                let arg = self.get_this();

                ControlFlow::Return(Value::String(arg.to_string().into()))
            }
            "isNaN" => {
                let arg = self.get_this();

                ControlFlow::Return(Value::Bool(arg.to_number().is_nan()))
            }
            _ => ControlFlow::Error(format!("Método nativo desconhecido: {}", method_name).into()),
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
    fn add_arg(&mut self, arg: Value) -> Result<(), String> {
        self.args.push(arg);
        Ok(())
    }

    fn set_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.args = args;
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
