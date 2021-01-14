mod char_iter;
mod token;

use std::collections::VecDeque;
use std::str::CharIndices;

use char_iter::CharIter;
use token::SyntaxKind::{self, *};

pub struct Lexer<'a> {
    chars: CharIter<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer<'_> {
        Lexer {
            chars: CharIter::new(input),
        }
    }

    fn lex_main(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let c = self.chars.next()?;

        let res = match c {
            '=' => Assign,
            '\'' | '"' => self.string()?,
            '[' => LBracket,
            ']' => RBracket,
            ',' => Comma,
            '{' => LBrace,
            '}' => RBrace,
            '\n' => Newline,
            '#' => self.comment()?,
            _ if is_whitespace(c) => self.whitespace()?,
            _ if is_letter(c) => self.key_word()?,
            _ if is_number(c) => self.number()?,
            _ => todo!(),
        };
        let slice = self.chars.slice();
        self.chars.ignore();

        Some((res, slice))
    }

    fn whitespace(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_while(is_whitespace);
        Some(Whitespace)
    }

    fn string(&mut self) -> Option<SyntaxKind> {
        self.chars.find_char('"');
        Some(String)
    }

    fn key_word(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_while(is_letter);
        let res = match self.chars.slice() {
            "true" => True,
            "false" => False,
            _ => self.ident()?,
        };
        Some(res)
    }

    fn ident(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_while(is_letter);
        Some(Ident)
    }

    fn number(&mut self) -> Option<SyntaxKind> {
        Some(Number)
    }

    fn comment(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_until(|c| c == '\n');
        Some(Comment)
    }
}

fn is_letter(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

const fn is_whitespace(c: char) -> bool {
    c == '\t' || c == ' '
}

fn is_number(c: char) -> bool {
    c.is_digit(10)
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (SyntaxKind, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.lex_main()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lexer(input: &str, expected_tokens: &[(SyntaxKind, &str)]) {
        let lexer = Lexer::new(input);
        lexer
            .into_iter()
            .zip(expected_tokens.iter())
            .for_each(|(got, &expected)| {
                assert_eq!(got, expected);
            })
    }

    #[test]
    fn brackets() {
        test_lexer("[]", &[(LBracket, "["), (RBracket, "]")])
    }

    #[test]
    fn simple() {
        test_lexer(
            "[]={}",
            &[
                (LBracket, "["),
                (RBracket, "]"),
                (Assign, "="),
                (LBrace, "{"),
                (RBrace, "}"),
            ],
        )
    }

    #[test]
    fn test_peek() {
        let mut lexer = Lexer::new("[]");
        assert_eq!(lexer.chars.peek().unwrap(), '[');
        assert_eq!(lexer.chars.next().unwrap(), '[');
        assert_eq!(lexer.chars.peek().unwrap(), ']');
        assert_eq!(lexer.chars.next().unwrap(), ']');
        assert!(lexer.chars.next().is_none());
        assert!(lexer.chars.peek().is_none());
    }

    #[test]
    fn test_assign() {
        test_lexer(
            r#"# a comment
this_key = "a string""#,
            &[
                (Comment, " a comment"),
                (Newline, "\n"),
                (Ident, "this_key"),
                (Whitespace, " "),
                (Assign, "="),
                (Whitespace, " "),
                (String, "\"a string\""),
            ],
        )
    }
}
