use std::{cell::RefCell, rc::Rc};
macro_rules! declare_modules_with_reexport {
    ($($name:ident),*) => {
        $(
            pub mod $name;
            pub use $name::create_instance;
        )*
    };
}

macro_rules! create_instance_fn {
    ($struct_name:ident) => {
        pub fn create_instance() -> $struct_name {
            $struct_name::new()
        }
    };
}

macro_rules! declare_modules {
    ($($name:ident),*) => {
        $(
            pub mod $name;
        )*

        pub fn list_modules() -> &'static [&'static str] {
            &[$(stringify!($name)),*]
        }

        pub fn get_module_by_name(name: &str) -> Option<Rc<RefCell<dyn crate::environment::native::native_callable::NativeCallable>>> {
            match name.to_ascii_lowercase().as_str() {
                $(
                    stringify!($name) => Some(Rc::new(RefCell::new($name::create_instance()))),
                )*
                _ => None,
            }
        }
    };
}

// Use:
declare_modules!(fs, io, json, math, array, number);
