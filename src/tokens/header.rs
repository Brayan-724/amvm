use std::fmt;

use crate::error::error_msgs;
use crate::{compilable_enum, Compilable, Parser, ParserError};
use crate::{AMVM_HEADER, COMMAND_SEPARATOR};

compilable_enum!(pub AmvmTypeCasting {
    /// Try to cast types, but throws if it can't cast.
    TypeCastingStrict = "\x01",

    /// Try to cast types, if it can't cast and has an
    /// `Serialize` implementation then serialize both
    /// sides.
    TypeCastingString = "\x02",

    /// Try to cast types, if it can't cast and has an
    /// `Serialize` implementation then serialize both
    /// sides, else just serialize it by default.
    TypeCastingStrictlessString = "\x03",

    /// Never cast types
    Strict = "\x04"
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
    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        let kind = parser
            .consume()
            .ok_or_else(|| {
                parser.error_corrupt(
                    error_msgs::ERROR_INVALID_HEADER_DECL,
                    "Can't get type casting kind",
                    1,
                )
            })?
            .to_string();

        let buffer = kind.chars().next().unwrap() as u8;

        let kind = AmvmTypeCasting::from_str(&kind).ok_or_else(|| {
            parser.error(
                error_msgs::ERROR_UNKNOWN_TYPE_CASTING_KIND,
                format!(
                    "Unknown byte {buffer:?}. Expected [{}, {}, {}, {}]",
                    AmvmTypeCasting::TypeCastingStrict,
                    AmvmTypeCasting::TypeCastingString,
                    AmvmTypeCasting::TypeCastingStrictlessString,
                    AmvmTypeCasting::Strict
                ),
                1,
            )
        })?;

        Ok(kind)
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
    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        // Check header integrity
        {
            if parser.bytes.len() < 4 {
                panic!("Invalid bytecode header");
            }

            let a = &parser.bytes[0..3];
            if a != AMVM_HEADER.as_bytes() {
                panic!("Invalid bytecode header");
            }
        }

        parser.pointer = 3;

        let sum_kind = AmvmTypeCasting::visit(parser)?;

        parser.pointer += 1;

        Ok(AmvmHeader { sum_kind })
    }
}
