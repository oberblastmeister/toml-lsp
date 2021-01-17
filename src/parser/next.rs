use super::{
    ParseError, Parser,
    SyntaxKind::{self, *},
    AST,
};

impl<'a> Parser<'a> {
    pub(super) fn eat_trivia(&mut self) -> Option<()> {
        while self
            .peek_token_raw()
            .map(|t| t == Whitespace || t == Comment)
            .unwrap_or(false)
        {
            self.bump_raw();
        }
        Some(())
    }

    pub(super) fn eat_trivia_back(&mut self) -> Option<()> {
        while self
            .peek_back_raw_token()
            .map(|t| t == Whitespace || t == Comment)
            .unwrap_or(false)
        {
            self.bump_raw_back();
        }
        Some(())
    }

    pub(super) fn peek_token_raw(&self) -> Option<SyntaxKind> {
        self.peek_raw().map(|(tok, _s)| tok)
    }

    pub(super) fn peek_raw(&self) -> Option<(SyntaxKind, &'a str)> {
        self.buffer.get(0).copied()
    }

    pub(super) fn peek(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.eat_trivia();
        self.peek_raw()
    }

    pub(super) fn peek_back_raw(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.buffer.get(1).copied()
    }

    pub(super) fn peek_back_raw_token(&mut self) -> Option<SyntaxKind> {
        self.peek_back_raw().map(|(tok, _s)| tok)
    }

    pub(super) fn peek_back(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.eat_trivia_back();
        self.peek_back_raw()
    }

    pub(super) fn peek_back_token(&mut self) -> Option<SyntaxKind> {
        self.peek_back().map(|(tok, _s)| tok)
    }

    pub(super) fn peek_token(&mut self) -> Option<SyntaxKind> {
        self.peek().map(|(tok, _s)| tok)
    }

    pub(super) fn peek_is_any(&mut self, is: &[SyntaxKind]) -> Option<SyntaxKind> {
        self.peek_token()
            .or_else(|| {
                self.errors.push(ParseError::UnexpectedEofWanted(
                    is.to_vec().into_boxed_slice(),
                ));
                None
            })
            .and_then(|tok| if is.contains(&tok) { Some(tok) } else { None })
    }

    pub(super) fn peek_is(&mut self, is: SyntaxKind) -> Option<SyntaxKind> {
        self.peek_is_any(&[is])
    }

    pub(super) fn next(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let res = self.buffer.pop_front();
        if let Some(next) = self.lexer.next() {
            self.buffer.push_back(next);
        }
        res
    }

    pub(super) fn next_back(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let res = self.buffer.pop_back();
        if let Some(next) = self.lexer.next() {
            self.buffer.push_front(next);
        }
        res
    }

    pub(super) fn next_token(&mut self) -> Option<SyntaxKind> {
        self.next().map(|(tok, _)| tok)
    }
}
