use amvm::{parser, Parser, ParserResult};

fn visit_var<'a>(parser: Parser<'a>) -> ParserResult<'a, Parser<'a>> {
    let (parser, varname) = parser::take_until_space(parser)?;

    Ok((parser, varname))
}

fn visit_let<'a>(parser: Parser<'a>) -> ParserResult<'a, ()> {
    let (parser, _) = parser::char(' ')(parser)?;
    let (parser, varname) = visit_var(parser)?;

    println!("Varname: {varname:?}");

    Ok((parser, ()))
}

fn main() {
    let parser = Parser::new("@let ");

    let (parser, _) = parser::char('@')(parser)
        .map_err(Parser::map_nom_err)
        .unwrap();

    let (parser, v) = parser::map(parser::take_until_space, |v: Parser<'_>| v.value)(parser)
        .map_err(Parser::map_nom_err_with_context(
            "Expected command (@KIND ...)",
        ))
        .unwrap();

    let (parser, v) = match v {
        "let" => visit_let(parser).map_err(Parser::map_nom_err).unwrap(),
        _ => unimplemented!("{v}"),
    };

    println!("Found: {v:#?}");

    let offset = parser.pointer_position();

    println!("parser: {parser:#?}");
    println!("Offset: {offset}");
}
