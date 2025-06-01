#[macro_export]
macro_rules! impl_from_for_class {
    // Recursão para múltiplos tipos
    ([$t:ty, $($rest:ty),+], $y:tt, $c:ty) => {
        impl_from_for_class!($t, $y, $c);
        impl_from_for_class!([$($rest),+], $y, $c);
    };
    // Caso base: um único tipo
    ([$t:ty], $y:tt, $c:ty) => {
        impl_from_for_class!($t, $y, $c);
    };

    // Caso especial para f64
    ($t:ty, f64, $c:ty) => {
        impl From<$t> for $c {
            fn from(value: $t) -> Self {
                Self::new_with_value(value as f64)
            }
        }

        impl From<&$t> for $c {
            fn from(value: &$t) -> Self {
                Self::new_with_value(*value as f64)
            }
        }

        impl From<$c> for $t {
            fn from(value: $c) -> Self {
                value.get_value() as $t
            }
        }

        impl From<&$c> for $t {
            fn from(value: &$c) -> Self {
                value.get_value() as $t
            }
        }
    };

    // Caso especial para String
    ($t:ty, String, $c:ty) => {
        impl From<$t> for $c {
            fn from(value: $t) -> Self {
                Self::new_with_value(value.to_string())
            }
        }

        impl From<&$t> for $c {
            fn from(value: &$t) -> Self {
                Self::new_with_value(value.to_string())
            }
        }

        impl From<$c> for $t {
            fn from(value: $c) -> Self {
                value.get_value().clone()
            }
        }

        impl From<&$c> for $t {
            fn from(value: &$c) -> Self {
                value.get_value().clone()
            }
        }
    };

}

#[macro_export]
macro_rules! impl_logical_operations {
    ($Lhs:ty, $Rhs:ty) => {
        impl PartialEq<$Rhs> for $Lhs {
            fn eq(&self, rhs: &$Rhs) -> bool {
                self.get_value() == (&rhs).get_value()
            }
        }
        impl PartialOrd<$Rhs> for $Lhs {
            fn partial_cmp(&self, rhs: &$Rhs) -> Option<std::cmp::Ordering> {
                self.get_value().partial_cmp(&(&rhs).get_value())
            }
        }
        impl PartialEq<$Rhs> for &$Lhs {
            fn eq(&self, rhs: &$Rhs) -> bool {
                self.get_value() == (&rhs).get_value()
            }
        }
        impl PartialOrd<$Rhs> for &$Lhs {
            fn partial_cmp(&self, rhs: &$Rhs) -> Option<std::cmp::Ordering> {
                self.get_value().partial_cmp(&(&rhs).get_value())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_math_operations {
    ($T:ty) => {
        impl_math_operations!(@impl $T, $T);
        impl_math_operations!(@impl &$T, $T);
        impl_math_operations!(@impl $T, &$T);
        impl_math_operations!(@impl &$T, &$T);
    };

    (@impl $Lhs:ty, $Rhs:ty) => {
        impl std::ops::Div<$Rhs> for $Lhs {
            type Output = NativeNumberClass;
            fn div(self, rhs: $Rhs) -> Self::Output {
                let lhs_val = (&self).get_value();
                let rhs_val = (&rhs).get_value();
                NativeNumberClass::new_with_value(lhs_val / rhs_val)
            }
        }

        impl std::ops::Mul<$Rhs> for $Lhs {
            type Output = NativeNumberClass;
            fn mul(self, rhs: $Rhs) -> Self::Output {
                let mut value = (&self).get_value();
                value *= (&rhs).get_value();
                NativeNumberClass::new_with_value(value)
            }
        }

        impl std::ops::Sub<$Rhs> for $Lhs {
            type Output = NativeNumberClass;
            fn sub(self, rhs: $Rhs) -> Self::Output {
                let mut value = (&self).get_value();
                value -= (&rhs).get_value();
                NativeNumberClass::new_with_value(value)
            }
        }

        impl std::ops::Add<$Rhs> for $Lhs {
            type Output = NativeNumberClass;
            fn add(self, rhs: $Rhs) -> Self::Output {
                let mut value = (&self).get_value();
                value += (&rhs).get_value();
                NativeNumberClass::new_with_value(value)
            }
        }

        impl std::ops::Rem<$Rhs> for $Lhs {
            type Output = NativeNumberClass;
            fn rem(self, rhs: $Rhs) -> Self::Output {
                let mut value = (&self).get_value();
                value %= (&rhs).get_value();
                NativeNumberClass::new_with_value(value)
            }
        }
    };
}
