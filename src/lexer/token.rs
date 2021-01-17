use num_derive::{FromPrimitive, ToPrimitive};

use SyntaxKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum SyntaxKind {
    Root,

    ArrayHeader,
    TableHeader,

    Array,
    Table,

    Assign,

    Ident,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Number,
    True,
    False,
    Date,
    Equal,
    String,
    Newline,
    Comma,
    Whitespace,
    Comment,
    Error,
    Dot,
}

impl SyntaxKind {
    pub fn is_trivia(&self) -> bool {
        match self {
            Comment | Whitespace => true,
            _ => false,
        }
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}
