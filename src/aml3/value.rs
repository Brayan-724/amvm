use crate::{parser, Parser, ParserResult, Value};

pub struct Aml3Value;

impl Aml3Value {
    fn visit_number<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let (parser, first) = parser::anychar(parser)
            .map_err(Parser::map_nom_err)
            .expect("This is verified by the visit root");

        let str = first.to_string();

        let mut parser = parser;
        loop {
            let (_parser, b) =
                parser::anychar(parser).map_err(parser.nom_err_with_context("No number size"))?;
            parser = _parser;

            match b {
                'u' | 'i' => {
                    let (parser, size) = parser::take_until_space(parser)
                        .map_err(parser.nom_err_with_context("Expected number size"))?;

                    return match size.value {
                        "8" => {
                            if b == 'u' {
                                let value = str.parse::<u8>().map_err(|_| {
                                    parser.error(
                                        parser::VerboseErrorKind::Context("Can't parse number"),
                                        true,
                                    )
                                })?;

                                Ok((parser, Value::U8(value)))
                            } else {
                                todo!("Number: Integer 8")
                            }
                        }

                        _ => Err(parser.error(
                            parser::VerboseErrorKind::Context("Unknown number size"),
                            true,
                        )),
                    };
                }
                _ => todo!("{b:?}"),
            }
        }
    }

    fn visit_char<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let (parser, _) = parser::anychar(parser)
            .map_err(Parser::map_nom_err)
            .expect("Already verified");

        let (parser, c) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Unterminated char"))?;

        let (parser, c) = match c {
            '\n' | '\r' | '\t' => {
                return Err(
                    parser.error(parser::VerboseErrorKind::Context("Unterminated char"), true)
                )
            }
            // Escaping
            '\\' => {
                let (parser, c2) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Unterminated char"))?;

                match c2 {
                    '\n' | '\r' | '\t' => {
                        return Err(parser
                            .error(parser::VerboseErrorKind::Context("Unterminated char"), true))
                    }

                    '\\' => (parser, c2),
                    'n' => (parser, '\n'),
                    '\'' => (parser, '\''),

                    _ => {
                        return Err(parser.error(
                            parser::VerboseErrorKind::Context("Invalid escaped character"),
                            true,
                        ))
                    }
                }
            }

            _ => (parser, c),
        };

        Ok((parser, Value::Char(c)))
    }

    fn visit_string<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let (parser, _) = parser::anychar(parser)
            .map_err(Parser::map_nom_err)
            .expect("Already verified");

        let mut str = String::new();
        let mut escaping = false;
        let mut parser = parser;
        loop {
            let (_parser, c) = parser::anychar(parser)
                .map_err(parser.nom_err_with_context("Unterminated string"))?;
            parser = _parser;

            let mut next_escaping = false;
            match c {
                '\n' => {
                    return Err(parser.error(
                        parser::VerboseErrorKind::Context("Unterminated string"),
                        true,
                    ))
                }
                '"' if escaping => str.push('"'),
                '"' => break,
                '\\' if escaping => str.push('\\'),
                '\\' => next_escaping = true,

                'n' if escaping => str.push('\n'),
                _ if escaping => {
                    return Err(parser.error(
                        parser::VerboseErrorKind::Context("Invalid escaped character"),
                        true,
                    ))
                }
                _ => str.push(c),
            }

            escaping = next_escaping;
        }

        Ok((parser, Value::String(str)))
    }

    fn visit_bool<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let (parser, value) = parser::take_until_space(parser)
            .map_err(parser.nom_err_with_context("Unexpected EOF"))?;

        match value.value {
            "true" => Ok((parser, Value::Bool(true))),
            "false" => Ok((parser, Value::Bool(false))),

            _ => Err(parser.error(
                parser::VerboseErrorKind::Context("Expected true or false"),
                true,
            )),
        }
    }

    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let first = parser.peek(0).ok_or_else(|| {
            parser.error(parser::VerboseErrorKind::Context("Unexpected EOF"), false)
        })?;

        match first {
            b if b.is_digit(10) => Self::visit_number(parser),
            '"' => Self::visit_string(parser),
            '\'' => Self::visit_char(parser),

            't' | 'f' => Self::visit_bool(parser),

            _ => Err(parser.error(parser::VerboseErrorKind::Context("Unknown value"), true)),
        }
    }
}
