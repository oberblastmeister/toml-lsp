mod char_iter;
mod token;

use std::collections::VecDeque;
use std::str::CharIndices;

use char_iter::CharIter;
pub use token::SyntaxKind;
use SyntaxKind::*;

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: CharIter<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer<'_> {
        Lexer {
            chars: CharIter::new(input),
        }
    }

    pub fn slice(&self) -> &'a str {
        self.chars.slice()
    }

    fn lex_main(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let c = self.chars.next()?;

        let res = match c {
            '=' => Equal,
            '\'' | '"' => self.string()?,
            '[' => LBracket,
            ']' => RBracket,
            ',' => Comma,
            '.' => Dot,
            '{' => LBrace,
            '}' => RBrace,
            '\n' => Newline,
            '#' => self.comment()?,
            _ if is_whitespace(c) => self.whitespace()?,
            _ if is_letter(c) => self.key_word()?,
            _ if is_number(c) => self.number()?,
            _ => Error,
        };
        let slice = self.slice();
        self.chars.ignore();

        Some((res, slice))
    }

    fn whitespace(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_while(is_whitespace);
        Some(Whitespace)
    }

    fn string(&mut self) -> Option<SyntaxKind> {
        let op = self.chars.find_char('"');
        if op.is_some() {
            Some(String)
        } else {
            Some(Error)
        }
    }

    fn key_word(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_while(is_letter);
        let res = match self.slice() {
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
        self.chars.accept_while(is_number);
        Some(Number)
    }

    fn comment(&mut self) -> Option<SyntaxKind> {
        self.chars.accept_until(|c| c == '\n');
        Some(Comment)
    }
}

fn is_letter(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-'
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
                (Equal, "="),
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
                (Comment, "# a comment"),
                (Newline, "\n"),
                (Ident, "this_key"),
                (Whitespace, " "),
                (Equal, "="),
                (Whitespace, " "),
                (String, "\"a string\""),
            ],
        )
    }

    #[test]
    fn test_number() {
        test_lexer(
            r#"# another comment
another_key = 12345"#,
            &[
                (Comment, "# another comment"),
                (Newline, "\n"),
                (Ident, "another_key"),
                (Whitespace, " "),
                (Equal, "="),
                (Whitespace, " "),
                (Number, "12345"),
            ],
        )
    }

    #[test]
    fn test_assign_again() {
        test_lexer(
            r#"hello = 12345"#,
            &[
                (Ident, "hello"),
                (Whitespace, " "),
                (Equal, "="),
                (Whitespace, " "),
                (Number, "12345"),
            ],
        )
    }

    #[test]
    fn test_cannot_find() {
        test_lexer(
            r#"" adfasdf"#,
            &[
                (Error, "\" adfasdf")
            ]
        );
        test_lexer(
            r#"""#,
            &[
                (Error, "\"")
            ]
        );
    }
}
