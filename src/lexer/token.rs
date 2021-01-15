use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum SyntaxKind {
    Root,
    ArrayHeader,
    TableHeader,

    Ident,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Number,
    True,
    False,
    Date,
    Assign,
    String,
    Newline,
    Comma,
    Whitespace,
    Comment,
    Error,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}
