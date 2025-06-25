use std::any::Any;
use std::{cell::RefCell, rc::Rc};

use crate::environment::values::Value;

pub trait Environment: std::fmt::Debug + EnvironmentClone + Any {
    fn exist_in_current_scope(&self, name: &str) -> bool;

    fn get(&self, name: &str) -> Option<Value>;

    fn clear(&mut self);

    // const RESERVED: &[&str] = &["String", "Boolean", "Array", "Object", "Function"];

    fn is_reserved(&self, name: &str) -> bool;

    fn define(&mut self, name: String, value: Value);

    fn assign(&mut self, name: &str, value: Value) -> Result<(), String>;

    fn exist(&self, name: &str) -> bool;

    fn get_vars(&self) -> Vec<(String, Value)>;

    fn get_vars_name_value(&self) -> String;
    fn get_vars_from_parent(&self) -> Vec<(String, Value)>;

    fn get_vars_name_value_from_parent(&self) -> String;
    fn get_vars_string(&self) -> String;
}

pub trait GetNative {
    fn get_self(&self) -> dyn Environment;
    fn with_parent(&self, parent: Rc<RefCell<dyn Environment>>) -> dyn Environment;
    fn to_rc(&self) -> Rc<RefCell<dyn Environment>>;
    fn merge(&mut self, other: dyn Environment);
    fn new_rc_merged(parent: Rc<RefCell<dyn Environment>>) -> Rc<RefCell<dyn Environment>>;
    fn rc_enclosed(&self, parent: Rc<RefCell<dyn Environment>>) -> Rc<RefCell<dyn Environment>>;
    fn copy_from(&mut self, other: Rc<RefCell<dyn Environment>>);
    fn get_parent(&self) -> Option<Rc<RefCell<dyn Environment>>>;
    fn new() -> dyn Environment;

    fn new_rc() -> Rc<RefCell<dyn Environment>>;

    fn merge_environments(&mut self, other: dyn Environment);

    fn new_enclosed(parent: &mut Rc<RefCell<dyn Environment>>) -> dyn Environment;

    fn new_rc_enclosed(parent: &mut Rc<RefCell<dyn Environment>>) -> Rc<RefCell<dyn Environment>>;
}
pub trait EnvironmentClone {
    fn clone_box(&self) -> Box<dyn Environment>;
}

impl<T> EnvironmentClone for T
where
    T: 'static + Environment + Clone,
{
    fn clone_box(&self) -> Box<dyn Environment> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Environment> {
    fn clone(&self) -> Box<dyn Environment> {
        self.clone_box()
    }
}
