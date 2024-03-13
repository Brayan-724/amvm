use amvm::aml3;

fn main() {
    let mut args = std::env::args().skip(1);

    let source = args.next().expect("Provide source file");
    let source = std::fs::read_to_string(source).expect("Cannot read source file");

    let mut source = aml3::Aml3Parser::new(Box::from(source.as_bytes()));

    let a = aml3::Aml3Scope::visit(&mut source, false);

    match a {
        Ok(a) => println!("{a:#?}"),
        Err(a) => eprintln!("{a}"),
    }
}
