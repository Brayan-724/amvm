use std::fmt;

use crate::CompileResult;
use crate::{
    compilable_enum,
    parser::{self, Parser, ParserResult},
    tokens::AMVM_HEADER,
    Compilable,
};

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
        f.write_fmt(format_args!("0x{:02x}", self.to_char() as u8))
    }
}

impl AmvmTypeCasting {
    pub fn visit(parser: Parser<'_>) -> ParserResult<'_, AmvmTypeCasting> {
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
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        _ = buffer.write_str(AMVM_HEADER);
        buffer = self.sum_kind.compile_bytecode(buffer)?;

        Ok(buffer)
    }
}

impl AmvmHeader {
    pub fn visit(parser: Parser<'_>) -> ParserResult<'_, Self> {
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
