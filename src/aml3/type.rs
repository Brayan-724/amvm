use crate::parser::{self, Parser, ParserResult};
use crate::tokens::{AmvmPrimitiveType, AmvmType};

use super::Aml3Variable;

pub struct Aml3Type;

impl Aml3Type {
    pub fn visit_name(parser: Parser<'_>) -> ParserResult<'_, Option<&str>> {
        let (parser, _) = parser::char('#')(parser)?;

        let Ok((parser, name)) = parser::take_until_space::<_, ()>(parser) else {
            return Ok((parser, None));
        };

        Ok((parser, Some(name.value)))
    }

    pub fn visit_tuple(parser: Parser<'_>) -> ParserResult<'_, AmvmType> {
        // (#A, #B + #C,#D)
        let (parser, types) = parser::delimited(
            parser::char('('),
            parser::separated_list1(
                parser::char(','),
                parser::preceded(parser::opt(parser::char(' ')), Self::visit),
            ),
            parser::char(')'),
        )(parser)?;

        Ok((parser, AmvmType::Tuple(types)))
    }

    pub fn visit(parser: Parser<'_>) -> ParserResult<'_, AmvmType> {
        if let Ok((parser, _)) = parser::char::<_, ()>('+')(parser) {
            let (parser, _) = parser::char(' ')(parser)?;
            let (parser, a) = Self::visit(parser)?;
            let (parser, _) = parser::char(' ')(parser)?;
            let (parser, b) = Self::visit(parser)?;

            return Ok((parser, AmvmType::Union(Box::new(a), Box::new(b))));
        }

        let (_parser, c) = parser::anychar(parser)?;

        let (parser, curr_type) = match c {
            '(' => Self::visit_tuple(parser)?,
            _ => {
                let (parser, name) = Aml3Variable::visit_ident(parser)
                    .map_or_else(|_| (parser, None), |v| (v.0, Some(v.1)));

                if let Some(name) = name {
                    let type_ = match name {
                        "string" => AmvmType::Primitive(AmvmPrimitiveType::String),
                        "u8" => AmvmType::Primitive(AmvmPrimitiveType::U8),
                        _ => AmvmType::Named(Box::from(name)),
                    };

                    (parser, type_)
                } else {
                    (parser, AmvmType::Anonymous)
                }
            }
        };

        Ok((parser, curr_type))
    }
}
