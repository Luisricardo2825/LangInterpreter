use dyn_clone::DynClone;
use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{ast::ast::ControlFlow, environment::values::Value};

dyn_clone::clone_trait_object!(NativeCallable);

pub trait NativeCallable: std::fmt::Debug + NativeCallableClone + Any + DynClone {
    fn new() -> Self
    where
        Self: Sized;

    fn create() -> Rc<RefCell<dyn NativeCallable>>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(Self::new()))
    }
    fn is_static(&self) -> bool {
        false
    }
    fn get_name(&self) -> String;

    fn get_args(&self) -> Vec<Value>;

    fn set_args(&mut self, args: Vec<Value>) -> Result<(), String> {
        self.get_args().clear();
        self.get_args().append(&mut args.clone());
        Ok(())
    }
    fn add_args(&mut self, args: Vec<Value>) -> Result<(), String>;
    fn add_arg(&mut self, arg: Value) -> Result<(), String> {
        self.get_args().push(arg);
        Ok(())
    }

    fn methods_names(&self) -> Vec<String>;
    fn call(&self, method_name: &str) -> ControlFlow<Value> {
        todo!("Method called {method_name}")
    }
    fn call_with_args(&self, method_name: &str, args: Vec<Value>) -> ControlFlow<Value> {
        todo!("Method called {method_name} {args:?}")
    }
    fn instantiate(&self, _args: Vec<Value>) -> Result<Value, String> {
        Err(format!("Class {} cannot be instantiated", self.get_name()))
    }

    fn get_custom_method(&self, _method_name: &str) -> Option<Value> {
        None
    }
    fn add_custom_method(&mut self, _method_name: String, _method: Value) -> Result<(), String> {
        Err(format!(
            "Class {} cannot have custom methods",
            self.get_name()
        ))
    }
}

pub trait NativeCallableClone {
    fn clone_box(&self) -> Box<dyn NativeCallable>;
}

impl<T> NativeCallableClone for T
where
    T: 'static + NativeCallable + Clone,
{
    fn clone_box(&self) -> Box<dyn NativeCallable> {
        Box::new(self.clone())
    }
}
