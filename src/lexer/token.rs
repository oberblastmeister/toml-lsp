#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
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