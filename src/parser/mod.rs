mod error;
mod syntax;
mod utils;
mod next;

use std::{collections::VecDeque, convert::TryFrom};

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, Language, SyntaxNode, TextRange, TextSize};

use crate::lexer::{
    Lexer,
    SyntaxKind::{self, *},
};
pub use error::ParseError;

use syntax::Toml;

macro_rules! expect_match {
    ($p:expr, $($token:ident => $do:expr,)+) => {
        {
            let p = &mut *$p;
            if let Some(next) = p.expect_peek_any(&[$( $token ),+]) {
                match next {
                    $( $token => {$do} ),+,
                    _ => unreachable!(),
                }
            }
        }
    };

    // be able to accept optional comma at the end
    ($p:expr, $($token:ident => $do:expr),+) => {
        expect_match!($p, $($token => $do,)+);
    }
}

pub fn parse(input: &str) -> AST {
    Parser::new(input).parse()
}

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static>,
    buffer: VecDeque<(SyntaxKind, &'a str)>,
    index: TextSize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &str) -> Parser<'_> {
        let mut buffer = VecDeque::with_capacity(1);
        let mut lexer = Lexer::new(input);
        if let Some(tok) = lexer.next() {
            println!("yes");
            buffer.push_back(tok);
        }
        let errors: Vec<ParseError> = Vec::new();
        Parser {
            lexer,
            builder: GreenNodeBuilder::new(),
            buffer,
            errors,
            index: TextSize::from(0),
        }
    }

    fn parse_main(&mut self) {
        expect_match!(self,
            Ident => self.parse_assign(),
            RBracket => self.parse_header(),
        );
    }

    fn parse_assign(&mut self) {
        self.start_node(Assign);
        self.expect(Ident);
        self.expect(Assign);
        self.parse_rhs();
        self.finish_node();
    }

    fn parse_header(&mut self) {
        self.bump(); // consume first [ token
        // if self.accept(RBrace)
    }

    fn parse_table_header(&mut self) {}

    fn parse_array_header(&mut self) {
        self.start_node(ArrayHeader);
        expect_match!(self, LBracket => self.bump());
    }


    fn parse_rhs(&mut self) {
        expect_match!(self,
            Number => self.bump(),
            String => self.bump(),
            LBrace => self.parse_table(),
            LBracket => self.parse_array(),
        )
    }

    fn parse_table(&mut self) {}

    fn parse_array(&mut self) {}

    fn start_node(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        self.builder.start_node(kind.into())
    }

    fn finish_node(&mut self) {
        self.builder.finish_node()
    }

    fn token(&mut self, token: SyntaxKind, s: &str) {
        self.builder.token(token.into(), s.into());
    }

    fn bump_raw(&mut self) {
        let next = self.next();
        match next {
            Some((tok, s)) => {
                self.index.checked_add(TextSize::of(s));
                self.token(tok, s);
            }
            None => self.errors.push(ParseError::UnexpectedEof),
        }
    }

    fn bump(&mut self) {
        self.eat_trivia();
        self.bump_raw();
    }

    fn bump_error(&mut self) {
        let next = self.next();
        match next {
            Some((tok, s)) => self.token(Error, s),
            None => self.errors.push(ParseError::UnexpectedEof),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AST {
    node: GreenNode,
    errors: Vec<ParseError>,
}

impl AST {
    pub(crate) fn node(&self) -> SyntaxNode<Toml> {
        SyntaxNode::new_root(self.node.clone())
    }

    pub fn debug(&self) -> std::string::String {
        let formatted = format!("{:#?}", self.node());

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        formatted[0..formatted.len() - 1].to_string()
    }

    pub fn errors(&self) -> Vec<ParseError> {
        self.errors.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};

    fn check(input: &str, expected: Expect) {
        let ast = Parser::new(input).parse();
        expected.assert_eq(&ast.debug());
    }

    #[test]
    fn test_assign() {
        check(
            "hello = 12435",
            expect![[r#"Root@0..13
  Assign@0..13
    Ident@0..5 "hello"
    Whitespace@5..6 " "
    Assign@6..7 "="
    Whitespace@7..8 " "
    Number@8..13 "12435""#]],
        )
    }

    #[test]
    fn test_whitespace() {
        check(
            "name \t = \"hello\"",
            expect![["Root@0..16
  Assign@0..16
    Ident@0..4 \"name\"
    Whitespace@4..7 \" \\t \"
    Assign@7..8 \"=\"
    Whitespace@8..9 \" \"
    String@9..16 \"\\\"hello\\\"\""]],
        )
    }
}
