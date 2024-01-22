use amvm::*;

#[cfg(feature = "useron")]
fn compile(mut args: impl Iterator<Item = String>) {
    let source = args.next().expect("Provide file path to the source file");
    let output = args.next().expect("Provide file path to the bytecode file");
    let content = std::fs::read_to_string(source).expect("Cannot read source file");
    let commands: Vec<Command> = ron::from_str(&content).unwrap();

    println!("{commands:#?}");

    let compiled = commands.compile_bytecode();
    let content = format!("{AMVM_HEADER}{COMMAND_SEPARATOR}{compiled}");

    std::fs::write(output, content).expect("Cannot write file");
}

fn run(mut args: impl Iterator<Item = String>) {
    let filepath = args.next().expect("Provide file path to the bytecode file");
    let mut parser = Parser::from_filepath(filepath).expect("Can't read file");
    let mut runtime = parser.runtime();
    runtime.run();
}

fn jit(mut args: impl Iterator<Item = String>) {
    let source = args.next().expect("Provide file path to the source file");
    let content = std::fs::read_to_string(source).expect("Cannot read source file");
    let commands: Vec<Command> = ron::from_str(&content).unwrap();

    let compiled = commands.compile_bytecode();
    let content = format!("{AMVM_HEADER}{COMMAND_SEPARATOR}{compiled}");
    
    let mut parser = Parser::from_string(content);
    let mut runtime = parser.runtime();
    runtime.run();
}

fn help() {
    let mut args = std::env::args();
    let cli = args.next().unwrap();
    println!("Usage: {cli} <command>\n");
    println!("Commands:");
    println!("  run [filepath]             Execute the bytecode file at filepath");
    println!("  compile [source] [output]  Compile source commands to bytecode at output (requires `useron` feature)");
    println!("  jit [source]               Compile source commands and run it (requires `useron` feature)");
}

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("run") => run(args),
        #[cfg(feature = "useron")]
        Some("compile") => compile(args),
        #[cfg(not(feature = "useron"))]
        Some("compile") => eprintln!("This compilation doesn't have `useron` feature. Use `cargo build --features useron` or contact with provider."),

        #[cfg(feature = "useron")]
        Some("jit") => jit(args),
        #[cfg(not(feature = "useron"))]
        Some("jit") => eprintln!("This compilation doesn't have `useron` feature. Use `cargo build --features useron` or contact with provider."),

        _ => help(),
    }
}
