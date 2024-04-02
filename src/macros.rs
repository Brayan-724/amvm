#[macro_export(local_inner_macros)]
macro_rules! amvm_expr {
    (@value u8 $v:expr) => {
        ::amvm::Value::U8($v)
    };
    (@value i16 $v:expr) => {
        ::amvm::Value::I16($v)
    };
    (@value f32 $v:expr) => {
        ::amvm::Value::F32($v)
    };
    (@value string $v:expr) => {
        $crate::Value::String($v)
    };

    ($v:literal $ty:ident) => {
        $crate::CommandExpression::Value(amvm_expr!(@value $ty $v))
    };

    (+ ($($a:tt)*) ($($b:tt)*)) => {
        $crate::CommandExpression::Addition(Box::from(amvm_expr!($($a)*)), Box::from(amvm_expr!($($b)*)))
    };

    ($t:tt $name:ident) => {
        if ::std::stringify!($t) != "$" {
            ::std::panic!(::std::concat!("Unexpected token \"", ::std::stringify!($t), "\""))
        } else {
            $crate::CommandExpression::Var($crate::Value::String(String::from(::std::stringify!($name))))
        }
    };
}
