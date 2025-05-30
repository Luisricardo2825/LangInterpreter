use std::fmt::Debug;

use crate::environment::values::Value;

pub struct NativeClass<T: Debug + Clone> {
    pub args: Vec<Value>,
    pub value: Option<T>,
    pub is_static: bool,
}

impl<T: Debug + Clone> NativeClass<T> {
    pub fn new(args: Vec<Value>, value: Option<T>, is_static: bool) -> Self {
        Self {
            args,
            value,
            is_static,
        }
    }
}

impl<T: Debug + Clone> Default for NativeClass<T> {
    fn default() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: false,
        }
    }
}

impl<T: Debug + Clone> Clone for NativeClass<T> {
    fn clone(&self) -> Self {
        Self {
            args: self.args.clone(),
            value: self.value.clone(),
            is_static: self.is_static,
        }
    }
}

impl<T: Debug + Clone> Debug for NativeClass<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeClass")
            .field("args", &self.args)
            .field("value", &self.value)
            .field("is_static", &self.is_static)
            .finish()
    }
}
