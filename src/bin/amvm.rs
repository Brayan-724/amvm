use amvm::*;

fn compile(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let source = args.next().expect("Provide file path to the source file");
    let output = args.next().expect("Provide file path to the bytecode file");
    let content = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let commands: Vec<Command> =
        aml3::from_str(&content).map_err(|err| format!("Can't parse file {source}\n{err}"))?;

    println!("{commands:#?}");

    let header = AmvmHeader {
        sum_kind: AmvmTypeCasting::TypeCastingStrictlessString,
    };
    let program = Program::new(header, commands);
    let content = program.compile_bytecode();

    std::fs::write(output, content.as_bytes()).expect("Cannot write file");
    Ok(())
}

fn run(mut args: impl Iterator<Item = String>) {
    let filepath = args.next().expect("Provide file path to the bytecode file");
    let mut parser = Parser::from_filepath(filepath).expect("Can't read file");
    let program = match parser.program() {
        Ok(a) => a,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1)
        }
    };
    program.runtime().run();
}

fn jit(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let Some(source) = args.next() else {
        return Err(String::from("Provide source file."));
    };
    let content = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let commands: Vec<Command> =
        aml3::from_str(&content).map_err(|err| format!("Can't parse file {source}\n{err}"))?;

    let header = AmvmHeader {
        sum_kind: AmvmTypeCasting::TypeCastingStrictlessString,
    };
    let program = Program::new(header, commands);
    let content = program.compile_bytecode();

    let mut parser = Parser::from_string(content);
    let program = parser.program().map_err(|err| format!("{err}"))?;
    program.runtime().run();

    Ok(())
}

fn inspect(mut args: impl Iterator<Item = String>) {
    let filepath = args.next().expect("Provide file path to the bytecode file");
    let mut parser = Parser::from_filepath(filepath).expect("Can't read file");
    let program = parser.program().expect("Cannot parse bytecode");
    let commands = program.body;

    for (i, cmd) in commands.iter().enumerate() {
        let i = format!("\x1b[32m{i:03x}\x1b[0m");
        let cmd = format!("{cmd}");
        let cmd = cmd
            .split('\n')
            .map(|c| format!("{i}{c}\n"))
            .collect::<String>();
        print!("{cmd}");
    }
}

fn aml3(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let Some(source) = args.next() else {
        return Err(String::from("Provide source file."));
    };
    let content = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let commands: Vec<Command> =
        aml3::from_str(&content).map_err(|err| format!("Can't parse file {source}\n{err}"))?;

    println!("{commands:#?}");

    Ok(())
}

fn help() {
    let mut args = std::env::args();
    let cli = args.next().unwrap();
    println!("Usage: {cli} <command>\n");
    println!("Commands:");
    println!("  compile [source] [output]  Compile aml3 to bytecode at output");
    println!("  inspect [filepath]         Read bytecode and show all commands");
    println!("  jit [source]               Compile aml3 and run it");
    println!("  aml3 [source]              Parse and show info about aml3");
    println!("  run [filepath]             Execute the bytecode file at filepath");
}

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("run") => run(args),
        Some("compile") => {
            let Err(err) = compile(args) else {
                return;
            };

            eprintln!("\x1b[31m{err}\x1b[0m");
        }
        Some("inspect") => inspect(args),

        Some("aml3") => {
            let Err(err) = aml3(args) else {
                return;
            };

            eprintln!("\x1b[31m{err}\x1b[0m");
        }
        Some("jit") => {
            let Err(err) = jit(args) else {
                return;
            };

            eprintln!("\x1b[31m{err}\x1b[0m");
        }

        _ => help(),
    }
}
