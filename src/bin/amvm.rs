use amvm::{parser::Parser, runtime::*, tokens::*, *};

fn compile(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let source = args.next().expect("Provide file path to the source file");
    let output = args.next().expect("Provide file path to the bytecode file");
    let content = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let commands: Vec<Command> =
        aml3::from_str(&content).map_err(|err| format!("Can't parse file {source}\n{err}"))?;

    // println!("{commands:#?}");

    let header = AmvmHeader {
        sum_kind: AmvmTypeCasting::TypeCastingStrictlessString,
    };
    let program = Program::new(header, commands);
    let content = program.compile_bytecode();

    std::fs::write(output, content.as_bytes()).expect("Cannot write file");
    Ok(())
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let source = args.next().expect("Provide file path to the bytecode file");
    let source = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let parser = Parser::new(&source, &true);
    let (_, program) = Program::visit(parser).map_err(Parser::flat_errors)?;

    program.runtime().run().unwrap();

    Ok(())
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
    let mut runtime = program.runtime();
    runtime.run().map_err(|err| match err {
        AmvmPropagate::Err(err) => err.to_string(),
        AmvmPropagate::Return(_) => "Returning outside function scope".to_owned(),
        AmvmPropagate::Break => "Breaking outside loop scope".to_owned(),
    })?;

    Ok(())
}

fn inspect(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let source = args.next().expect("Provide file path to the bytecode file");
    let source = std::fs::read_to_string(&source)
        .map_err(|err| format!("Can't read file {source}\nCause by: {err}"))?;
    let parser = Parser::new(&source, &true);
    let (_, program) = Program::visit(parser).map_err(Parser::flat_errors)?;
    let commands = program.body;

    for (i, cmd) in commands.iter().enumerate() {
        let i = format!("\x1b[32m{i:03x}\x1b[0m");
        let cmd = format!("{cmd}");
        let cmd = cmd
            .split('\n')
            .fold(String::new(), |prev, c| format!("{prev}{i}{c}\n"));
        print!("{cmd}");
    }

    Ok(())
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

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::Uptime::default())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let res = match args.next().as_deref() {
        Some("run") => run(args),
        Some("compile") => compile(args),
        Some("inspect") => inspect(args),
        Some("aml3") => aml3(args),
        Some("jit") => jit(args),

        Some(cmd) => {
            help();
            Err(format!("Unknown command: {cmd}"))
        }

        None => Err("No command".to_owned()),
    };

    if let Err(err) = res {
        eprintln!("\x1b[31m{err}\x1b[0m");
        std::process::exit(1);
    }
}
