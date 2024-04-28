#[macro_export(local_inner_macros)]
macro_rules! amvm_expr {
    (@value u8 $v:expr) => {
        $crate::tokens::Value::U8($v)
    };
    (@value i16 $v:expr) => {
        $crate::tokens::Value::I16($v)
    };
    (@value f32 $v:expr) => {
        $crate::tokens::Value::F32($v)
    };
    (@value string $v:expr) => {
        $crate::Value::String($v)
    };

    ($v:literal $ty:ident) => {
        $crate::tokens::CommandExpression::Value(amvm_expr!(@value $ty $v))
    };

    (+ ($($a:tt)*) ($($b:tt)*)) => {
        $crate::tokens::CommandExpression::Binary(
            $crate::tokens::BinaryKind::Add,
            Box::from(amvm_expr!($($a)*)),
            Box::from(amvm_expr!($($b)*))
        )
    };

    ($t:tt $name:ident) => {
        if ::std::stringify!($t) != "$" {
            ::std::panic!(::std::concat!("Unexpected token \"", ::std::stringify!($t), "\""))
        } else {
            $crate::tokens::CommandExpression::Var(String::from(::std::stringify!($name)))
        }
    };
}
