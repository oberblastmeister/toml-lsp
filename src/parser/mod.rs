mod error;
mod next;
mod syntax;
mod utils;

use std::{collections::VecDeque, convert::TryFrom, fmt};

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, Language, SyntaxNode, TextRange, TextSize};

use crate::lexer::{
    Lexer,
    SyntaxKind::{self, *},
};
pub use error::ParseError;

use syntax::Toml;

type ParseResult<T, E = ParseError> = Result<T, E>;

macro_rules! expect_match {
    ($p:expr, $( $token:ident => $do:expr ),+ $(,)?) => {
        // expect_match!($p, $( $token => { $do } ),+, _ => ())
        {
            let p = &mut *$p;
            if let Some(tok) = p.expect_peek_any(&[$( $token ),+]) {
                match tok {
                    $( $token => {
                        $do;

                        #[allow(unreachable_code)]
                        Some(tok)
                    } ),+
                    _ => panic!("BUG: should not happen"),
                }
            } else {
                None
            }
        }
    };

    ($p:expr, $( $token:ident => $do:expr ),+, _ => $else:expr $(,)?) => {
        {
            let p = &mut *$p;
            if let Some(tok) = p.expect_peek_any(&[$( $token ),+]) {
                match tok {
                    $( $token => {
                        $do
                    } ),+
                    _ => panic!("BUG: should not happen"),
                }
            } else {
                $else;

                #[allow(unreachable_code)]
                None
            }
        }
    };
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
        let mut buffer = VecDeque::new();
        let mut lexer = Lexer::new(input);
        for _ in 0..2 {
            if let Some(tok) = lexer.next() {
                buffer.push_back(tok);
            }
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

    pub fn parse(mut self) -> AST {
        self.start_node(Root);
        loop {
            self.accept_all(Newline);

            if self.peek().is_none() {
                break;
            }

            if self.peek_token().unwrap() == LBracket {
                self.parse_all_headers();
                break;
            }

            self.parse_contents();
        }
        self.finish_node();

        AST {
            node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn parse_contents(&mut self) {
        expect_match!(self,
            Ident => self.parse_assign(),
        );
    }

    fn parse_assign(&mut self) {
        self.start_node(Assign);
        self.expect_sequential(&[Ident, Equal]);
        self.parse_rhs();
        self.finish_node();
    }

    fn parse_all_headers(&mut self) {
        loop {
            self.accept_all(Newline);

            if self.peek().is_none() {
                return;
            }

            self.parse_header();
        }
    }

    #[inline]
    fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }

    #[inline]
    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }

    fn parse_header(&mut self) {
        let checkpoint = self.checkpoint();
        expect_match!(self,
            LBracket => {
                self.bump();
                if self.accept(LBracket) {
                    self.start_node_at(checkpoint, ArrayHeader);
                    self.parse_table_ident();
                    self.expect_sequential(&[RBracket, RBracket, Newline]);
                    self.parse_table_contents();
                    self.finish_node();
                } else {
                    self.start_node_at(checkpoint, TableHeader);
                    self.parse_table_ident();
                    self.expect_sequential(&[RBracket, Newline]);
                    self.parse_table_contents();
                    self.finish_node();
                }
            },
        );
    }

    fn parse_table_ident(&mut self) {
        self.expect_bump(Ident);
        if self.accept(Dot) {
            self.parse_table_ident()
        }
    }

    fn parse_table_contents(&mut self) {
        loop {
            self.accept_all(Newline);

            if self.peek_token().map(|k| k == LBracket).unwrap_or(true) {
                return;
            }

            if self.peek().is_none() {
                return;
            }

            expect_match!(self,
                Ident => self.parse_assign(),
                RBracket => {
                    break;
                },
            );
        }
    }

    fn parse_rhs(&mut self) -> Option<SyntaxKind> {
        expect_match!(self,
            Number => self.bump(),
            String => self.bump(),
            LBrace => self.parse_table(),
            LBracket => self.parse_array(),
        )
    }

    fn parse_rhs_no_advance(&mut self) -> ParseResult<SyntaxKind> {
        let expected = [Number, String, LBrace, LBracket];
        let peek_tok = self
            .peek_token()
            .ok_or_else(|| ParseError::UnexpectedEofWanted(expected.to_vec().into_boxed_slice()))?;
        match peek_tok {
            Number => self.bump(),
            String => self.bump(),
            LBrace => self.parse_table(),
            LBracket => self.parse_array(),
            got => {
                return Err(ParseError::Expected {
                    expected: expected.to_vec().into_boxed_slice(),
                    got,
                    range: None,
                })
            }
        }
        Ok(peek_tok)
    }

    fn parse_array(&mut self) {
        self.start_node(Array);
        self.expect_bump(LBracket);

        loop {
            self.accept_all_if(|k| k.is_trivia() || k == Newline);

            if self.peek().is_none() {
                break;
            }

            if let Some(RBracket) = self.peek_token() {
                break;
            }

            if let Err(e) = self.parse_rhs_no_advance() {
                if let ParseError::Expected { .. } = e {
                    self.add_error_until(e, |k| k == RBracket);
                    break;
                } else {
                    self.errors.push(e);
                    self.finish_node();
                    return;
                }
            }

            if let Some(RBracket) = self
                .peek_until(|k| !k.is_trivia() && k != Newline)
                .map(|(tok, _s)| tok)
            {
                self.accept(Comma);
            } else {
                self.expect_bump(Comma);
            }

            self.accept_all_if(|k| k.is_trivia() || k == Newline);
        }

        self.expect_bump(RBracket);
        self.finish_node();
    }

    fn parse_table(&mut self) {}

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into())
    }

    fn finish_node(&mut self) {
        self.builder.finish_node()
    }

    fn token(&mut self, token: SyntaxKind, s: &str) {
        self.index = self.index.checked_add(TextSize::of(s)).expect("Overflow");
        self.builder.token(token.into(), s.into());
    }

    fn bump_raw(&mut self) {
        let next = self.next();
        match next {
            Some((tok, s)) => {
                self.token(tok, s);
            }
            None => {
                self.errors.push(ParseError::UnexpectedEof);
            }
        }
    }

    fn bump_raw_back(&mut self) {
        match self.next_back() {
            Some((tok, s)) => {
                self.token(tok, s);
            }
            None => {
                self.errors.push(ParseError::UnexpectedEof);
            }
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

    pub fn errors(&self) -> Vec<ParseError> {
        self.errors.clone()
    }

    pub fn debug(&self) -> std::string::String {
        let formatted = format!("{:#?}", self.node());

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        format!("{}", &formatted[0..formatted.len() - 1])
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, fmt::Write, fs, path::PathBuf};

    use super::*;
    use expect_test::{expect, expect_file, Expect};

    fn check(input: &str, expected: Expect) {
        let ast = Parser::new(input).parse();
        expected.assert_eq(&ast.debug());
    }

    fn test_dir(name: &str) {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "test_data", name]
            .iter()
            .collect();

        dir.read_dir()
            .expect("Failed to read dir")
            .map(|p| p.expect("Failed to read entry").path())
            .filter(|p| p.extension() == Some(OsStr::new("toml")))
            .for_each(|actual_path| {
                let mut toml = fs::read_to_string(&actual_path).expect("Failed to read to string");
                if toml.ends_with('\n') {
                    toml.truncate(toml.len() - 1);
                }
                let ast = parse(&toml);

                let mut actual = std::string::String::new();
                for error in ast.errors() {
                    writeln!(actual, "error: {}", error).unwrap();
                }
                writeln!(actual, "{}", ast.debug()).unwrap();

                let expect_path = actual_path.with_extension("expect");
                expect_file![expect_path].assert_eq(&actual);
            })
    }

    #[rustfmt::skip]
    mod dir_tests {
        use super::test_dir;
        #[test] fn let_test() { test_dir("parser/let") }
        #[test] fn array() { test_dir("parser/array") }
    }
}
