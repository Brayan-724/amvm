use crate::tokens::Value;

#[macro_export]
macro_rules! compilable_enum {
    ($vis:vis $name:ident { $($(#[$meta:meta])* $id:ident = $val:expr),* }) => {
        #[derive(Debug, Clone, PartialEq)]
        $vis enum $name {
            $(
                $(#[$meta])*
                $id
            ),*
        }

        impl $name {
            pub fn from_char(v: char) -> Option<Self> {
                match v {
                    $($val => Some(Self::$id),)*
                    _ => None
                }
            }

            pub fn to_char(&self) -> char {
                match self {
                    $(Self::$id => $val),*
                }
            }
        }

        impl $crate::Compilable for $name {
            fn compile_bytecode(&self, mut buffer: String) -> $crate::CompileResult {
                use std::fmt::Write;
                _ = buffer.write_char(match self {
                    $(Self::$id => $val),*
                });

                Ok(buffer)
            }
        }
    };
}

pub trait Compilable {
    fn compile_bytecode(&self, buffer: String) -> CompileResult;
}

pub type CompileResult = Result<String, ()>;

impl<T, F> Compilable for (&Vec<T>, F)
where
    F: Fn(String, &T) -> CompileResult,
{
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        let len = self.0.len() as u8 as char;
        _ = buffer.write_char(len);

        for i in self.0 {
            buffer = self.1(buffer, i)?;
        }

        Ok(buffer)
    }
}

impl<T: Compilable> Compilable for [T] {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_slice(buffer, self)
    }
}

impl<A: Compilable, B: Compilable> Compilable for (A, B) {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        buffer = self.0.compile_bytecode(buffer)?;
        self.1.compile_bytecode(buffer)
    }
}

impl Compilable for u16 {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;
        _ = buffer.write_str(&String::from_utf8_lossy(&[(self >> 8) as u8, *self as u8]));
        Ok(buffer)
    }
}

impl Compilable for String {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_string(buffer, self)
    }
}

impl Compilable for &String {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_string(buffer, self)
    }
}

impl Compilable for &str {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_string(buffer, self)
    }
}

impl Compilable for Box<str> {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_string(buffer, self)
    }
}

impl Compilable for &Box<str> {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        Value::compile_string(buffer, self)
    }
}
