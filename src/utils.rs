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
        }

        impl $crate::Compilable for $name {
            fn compile_bytecode(&self) -> Box<str> {
                Box::from(match self {
                    $(Self::$id => $val.to_string()),*
                })
            }
        }
    };
}

pub trait Compilable {
    fn compile_bytecode(&self) -> Box<str>;
}

impl Compilable for [crate::Command] {
    fn compile_bytecode(&self) -> Box<str> {
        self.iter()
            .map(|c| c.compile_bytecode().to_string())
            .collect::<Vec<String>>()
            .join("")
            .into()
    }
}
