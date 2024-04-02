use std::fmt;

use crate::parser::{self, ParserResult};
use crate::{compilable_enum, Compilable, Parser};
use crate::{AMVM_HEADER, COMMAND_SEPARATOR};

compilable_enum!(pub AmvmTypeCasting {
    /// Try to cast types, but throws if it can't cast.
    TypeCastingStrict = '\x01',

    /// Try to cast types, if it can't cast and has an
    /// `Serialize` implementation then serialize both
    /// sides.
    TypeCastingString = '\x02',

    /// Try to cast types, if it can't cast and has an
    /// `Serialize` implementation then serialize both
    /// sides, else just serialize it by default.
    TypeCastingStrictlessString = '\x03',

    /// Never cast types
    Strict = '\x04'
});

impl fmt::Display for AmvmTypeCasting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "0x{:02x}",
            self.compile_bytecode().chars().next().unwrap() as u8
        ))
    }
}

impl AmvmTypeCasting {
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, AmvmTypeCasting> {
        let (parser, kind) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected type casting kind"))?;

        let kind = AmvmTypeCasting::from_char(kind).ok_or_else(|| {
            parser::Err::Failure(parser::VerboseError {
                errors: vec![(parser, parser::VerboseErrorKind::Context("Unknown byte"))],
            })
        })?;

        Ok((parser, kind))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmvmHeader {
    pub sum_kind: AmvmTypeCasting,
}

impl Compilable for AmvmHeader {
    fn compile_bytecode(&self) -> Box<str> {
        let sum_kind = self.sum_kind.compile_bytecode();
        Box::from(format!("{AMVM_HEADER}{sum_kind}{COMMAND_SEPARATOR}"))
    }
}

impl AmvmHeader {
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        // Check header integrity
        let (header, parser) = parser::take(3usize)(parser)
            .map_err(parser.nom_err_with_context("Invalid bytecode header"))?;

        if header.value != AMVM_HEADER {
            return Err(parser.error(
                parser::VerboseErrorKind::Context("Invalid bytecode header"),
                true,
            ));
        }

        let (parser, sum_kind) = AmvmTypeCasting::visit(parser)?;

        Ok((parser, AmvmHeader { sum_kind }))
    }
}
