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

macro_rules! expect_match {
    ($p:expr, $($token:ident => $do:expr,)+) => {
        {
            let p = &mut *$p;
            if p.expect_peek_any(&[$( $token ),+]) {
                match p.peek_token().unwrap() {
                    $( $token => {
                        {$do}
                    } ),+,
                    _ => None,
                }
            } else {
                None
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

    fn parse_contents(&mut self) -> Option<()> {
        expect_match!(self,
            Ident => self.parse_assign(),
        )
    }

    fn parse_assign(&mut self) -> Option<()> {
        self.start_node(Assign);
        self.expect_sequential(&[Ident, Equal])?;
        self.parse_rhs()?;
        self.finish_node();
        Some(())
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

    fn parse_header(&mut self) -> Option<()> {
        let checkpoint = self.checkpoint();
        expect_match!(self,
            LBracket => {
                self.bump()?;
                if self.accept(LBracket) {
                    self.start_node_at(checkpoint, ArrayHeader);
                    self.parse_table_ident()?;
                    self.expect_sequential(&[RBracket, RBracket, Newline])?;
                    self.parse_table_contents()?;
                    self.finish_node();
                } else {
                    self.start_node_at(checkpoint, TableHeader);
                    self.parse_table_ident()?;
                    self.expect_sequential(&[RBracket, Newline])?;
                    self.parse_table_contents()?;
                    self.finish_node();
                }
                Some(())
            },
        )
    }

    fn parse_table_ident(&mut self) -> Option<()> {
        self.expect_bump(Ident)?;
        if self.accept(Dot) {
            self.parse_table_ident()?;
        }
        Some(())
    }

    fn parse_table_contents(&mut self) -> Option<()> {
        loop {
            self.accept_all(Newline);

            dbg!(self.peek());

            if self.peek_token().map(|k| k == LBracket).unwrap_or(true) {
                return Some(());
            }

            if self.peek().is_none() {
                return Some(());
            }

            expect_match!(self,
                Ident => self.parse_assign(),
                RBracket => {
                    break;
                },
            )?;
        }
        Some(())
    }

    fn parse_rhs(&mut self) -> Option<()> {
        expect_match!(self,
            Number => self.bump(),
            String => self.bump(),
            LBrace => self.parse_table(),
            LBracket => self.parse_array(),
        )
    }

    fn parse_array(&mut self) -> Option<()> {
        self.start_node(Array);
        self.bump()?;
        self.parse_rhs()?;
        loop {
            if self.accept(Comma) {
                self.parse_rhs()?;
            } else if self.accept(Newline) {
                return None;
            } else if self.accept(RBracket) {
                break;
            }
        }
        // self.expect_bump(RBracket)?;
        self.finish_node();
        Some(())
    }

    fn parse_table(&mut self) -> Option<()> {
        Some(())
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into())
    }

    fn finish_node(&mut self) {
        self.builder.finish_node()
    }

    fn token(&mut self, token: SyntaxKind, s: &str) {
        self.builder.token(token.into(), s.into());
    }

    fn bump_raw(&mut self) -> Option<()> {
        let next = self.next();
        match next {
            Some((tok, s)) => {
                self.index.checked_add(TextSize::of(s));
                self.token(tok, s);
                Some(())
            }
            None => {
                self.errors.push(ParseError::UnexpectedEof);
                None
            }
        }
    }

    fn bump(&mut self) -> Option<()> {
        self.eat_trivia()?;
        self.bump_raw()?;
        Some(())
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
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = format!("{:#?}", self.node());

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        write!(f, "{}", &formatted[0..formatted.len() - 1])
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, fmt::Write, fs, path::PathBuf};

    use super::*;
    use expect_test::{expect, expect_file, Expect};
    use pretty_assertions::assert_eq;

    fn check(input: &str, expected: Expect) {
        let ast = Parser::new(input).parse();
        expected.assert_eq(&format!("{}", ast));
    }

    fn test_dir(name: &str) {
        let dir: PathBuf = ["test_data", name].iter().collect();

        dir.read_dir()
            .expect("Failed to read dir")
            .map(|p| p.expect("Failed to read entry").path())
            .filter(|p| p.extension() == Some(OsStr::new("toml")))
            .for_each(|mut p| {
                let mut code = fs::read_to_string(&p).expect("Failed to read to string");
                if code.ends_with('\n') {
                    code.truncate(code.len() - 1);
                }
                println!("code: {}\n", code);

                let ast = parse(&code);

                p.set_extension("expect");
                let expected = fs::read_to_string(&p).unwrap();

                let mut actual = std::string::String::new();
                for error in ast.errors() {
                    writeln!(actual, "error: {}", error).unwrap();
                }
                writeln!(actual, "{}", ast).unwrap();

                if actual != expected {
                    p.set_extension("toml");
                    eprintln!("In {}:", p.display());
                    eprintln!("--- Actual ---");
                    eprintln!("{}", actual);
                    eprintln!("-- Expected ---");
                    eprintln!("{}", expected);
                    eprintln!("--- End ---");
                    panic!("Tests did not match");
                }
            })
    }

    #[rustfmt::skip]
    mod dir_tests {
        use super::test_dir;
        #[test] fn let_test() { test_dir("parser/let") }
        #[test] fn header() { test_dir("parser/header") }
    }
}
