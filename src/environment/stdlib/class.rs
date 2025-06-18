use std::fmt::Debug;

use crate::environment::values::Value;

pub struct InternalClass<T: Debug + Clone> {
    pub args: Vec<Value>,
    pub value: Option<T>,
    pub is_static: bool,
}

impl<T: Debug + Clone> InternalClass<T> {
    pub fn new(args: Vec<Value>, value: Option<T>, is_static: bool) -> Self {
        Self {
            args,
            value,
            is_static,
        }
    }
}

impl<T: Debug + Clone> Default for InternalClass<T> {
    fn default() -> Self {
        Self {
            args: vec![],
            value: None,
            is_static: false,
        }
    }
}

impl<T: Debug + Clone> Clone for InternalClass<T> {
    fn clone(&self) -> Self {
        Self {
            args: self.args.clone(),
            value: self.value.clone(),
            is_static: self.is_static,
        }
    }
}

impl<T: Debug + Clone> Debug for InternalClass<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalClass")
            .field("args", &self.args)
            .field("value", &self.value)
            .field("is_static", &self.is_static)
            .finish()
    }
}
