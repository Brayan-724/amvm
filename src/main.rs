use amvm::*;

fn main() {
    let commands = [
        Command::DeclareVariable {
            name: "hello_msg".into(),
            kind: VariableKind::Const,
            value: Value::String("Hello World!\n".into()).into(),
        },
        Command::Puts {
            value: CommandExpression::Var("hello_msg".into()),
        },
    ];

    let mut args = std::env::args().skip(1);
    let runit = args.next() == Some("run".into());
    if runit {
        let filepath = args.next().expect("Provide file path to the bytecode file");
        let mut parser = Parser::from_filepath(filepath).expect("Can't read file");
        let mut runtime = parser.runtime();
        println!("{runtime:#?}");
        runtime.run();
    } else {
        let hello_msg_var = commands.compile_bytecode();
        print!("{AMVM_HEADER}{COMMAND_SEPARATOR}{hello_msg_var}");
    }
}
