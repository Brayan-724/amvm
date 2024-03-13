use crate::Value;

use super::{Aml3Error, Aml3Parser};

pub struct Aml3Value;

impl Aml3Value {
    fn visit_number(parser: &mut Aml3Parser) -> Result<Value, Aml3Error> {
        let first = parser
            .consume()
            .ok_or_else(|| unreachable!("This is verified by the visit root"))?;

        let mut str = first.to_string();

        loop {
            let a = parser
                .consume_oneline()
                .ok_or_else(|| parser.error("Untermited number"))?;

            match a {
                b @ 'u' | b @ 'i' => {
                    let size = parser
                        .consume_until(' ')
                        .ok_or_else(|| parser.error_expected("number size", None))?;

                    return match &size as &str {
                        "8" => {
                            if b == 'u' {
                                Ok(Value::U8(str.parse::<u8>().map_err(|e| {
                                    parser.error(format!("Cannot parse u8: {e}"))
                                })?))
                            } else {
                                todo!()
                            }
                        }

                        _ => Err(parser.error("Unknown number size: {size}")),
                    };
                }
                _ => todo!("{a:?}"),
            }
        }
    }

    fn visit_string(parser: &mut Aml3Parser) -> Result<Value, Aml3Error> {
        parser.consume().expect("Already verified");

        // let str = parser
        //     .consume_until_oneline('"')
        //     .ok_or_else(|| parser.error("Malformed string"))?;

        let mut str = String::new();
        let mut escaping = false;
        loop {
            let c = parser
                .consume()
                .ok_or_else(|| parser.error("Untermited string"))?;

            match c {
                '\n' => return Err(parser.error("Unterminated string")),
                '\\' if escaping => str.push('\\'),
                '"' if escaping => str.push('"'),
                'n' if escaping => str.push('\n'),
                '"' => break,
                '\\' => {}

                _ => str.push(c),
            }

            if c == '\\' && !escaping {
                escaping = true;
            } else {
                escaping = false;
            }
        }

        // Consume following character expect for new line, supposed to be space
        let sp = parser.consume_oneline();
        if !matches!(sp, Some(' ') | None) {
            return Err(parser.error_expected("space", Some(format!("{:?}", sp.unwrap()))));
        }

        Ok(Value::String(str))
    }

    fn visit_bool(parser: &mut Aml3Parser) -> Result<Value, Aml3Error> {
        let v = parser
            .peek_until(' ')
            .ok_or_else(|| parser.error_expected("boolean value: true or false", None))?;

        match &v as &str {
            "true" => {
                parser.consume_until('\n');
                Ok(Value::Bool(true))
            }
            "false" => {
                parser.consume_until('\n');
                Ok(Value::Bool(false))
            }

            _ => Err(parser.error_expected("boolean value: true or false", Some(v))),
        }
    }

    pub fn visit(parser: &mut Aml3Parser) -> Result<Value, Aml3Error> {
        let first = parser
            .peek(0)
            .ok_or_else(|| parser.error_expected("value", None))?;

        match first {
            b if b.is_digit(10) => Self::visit_number(parser),
            '"' => Self::visit_string(parser),

            't' | 'f' => Self::visit_bool(parser),

            _ => Err(parser.error(format!("Unknown token: {first:?}"))),
        }
    }
}
