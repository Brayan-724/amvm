use amvm::*;

macro_rules! test_numeric_it {
    ($val:expr; $i:ident; $ty:ident) => {
        assert_eq!(
            amvm_expr!($val $i),
            CommandExpression::Value(Value::$ty($val))
        )
    };
}

macro_rules! test_numeric {
    ($i:ident; $ty:ident) => {
        test_numeric_it!(1; $i; $ty);
        test_numeric_it!(10; $i; $ty);
        test_numeric_it!(100; $i; $ty);
        test_numeric_it!(2_34; $i; $ty);
    }
}

fn main() {
    assert_eq!(
        amvm_expr!($var),
        CommandExpression::Var(Value::String(String::from("var")))
    );

    test_numeric!(u8; U8);
    test_numeric!(i16; I16);

    assert_eq!(
        amvm_expr!(+(1 u8) (2 u8)),
        CommandExpression::Addition(
            Box::new(CommandExpression::Value(Value::U8(1))),
            Box::new(CommandExpression::Value(Value::U8(2)))
        )
    );
    assert_eq!(
        amvm_expr!(+($var) (2 u8)),
        CommandExpression::Addition(
            Box::new(CommandExpression::Var(Value::String(String::from("var")))),
            Box::new(CommandExpression::Value(Value::U8(2)))
        )
    );
}
