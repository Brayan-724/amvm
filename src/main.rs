use amvm::*;

fn main() {
    let hello_msg_var = Command::DeclareVariable {
        name: Box::from("hello_msg"),
        value: Some(Box::from(CommandExpression::Value(Value::String(
            "Hello World!".into(),
        )))),
    };
    let hello_msg_var = hello_msg_var.compile_bytecode();
    print!("{AMVM_HEADER}{COMMAND_SEPARATOR}{hello_msg_var}{COMMAND_SEPARATOR}");
}
