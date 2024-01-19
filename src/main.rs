use amvm::*;

fn main() {
    let hello_msg_var = Command::DeclareVariable {
        name: Box::from("hello_msg"),
        value: Some(Box::from(CommandExpression::Value(Value::String(
            "Hello World!".into(),
        )))),
    };

    let a_var = Command::DeclareVariable {
        name: Box::from("a"),
        value: Some(Box::from(CommandExpression::Value(Value::U8(1)))),
    };
    let b_var = Command::DeclareVariable {
        name: Box::from("b"),
        value: Some(Box::from(CommandExpression::Value(Value::U8(2)))),
    };
    let c_var = Command::DeclareVariable {
        name: Box::from("c"),
        value: Some(Box::from(CommandExpression::Addition(
            Box::new(CommandExpression::Value(Value::U8(1))),
            Box::new(CommandExpression::Value(Value::U8(2))),
        ))),
    };
    let mut runtime = Runtime::new(vec![hello_msg_var, a_var, b_var, c_var]);
    runtime.step();
    runtime.step();
    runtime.step();
    runtime.step();
    println!("{runtime:#?}");
    // let hello_msg_var = hello_msg_var.compile_bytecode();
    // print!("{AMVM_HEADER}{COMMAND_SEPARATOR}{hello_msg_var}{COMMAND_SEPARATOR}");
}
