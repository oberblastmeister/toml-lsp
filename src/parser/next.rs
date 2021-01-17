use super::{
    ParseError, Parser,
    SyntaxKind::{self, *},
    AST,
};

impl<'a> Parser<'a> {
    pub(super) fn eat_trivia(&mut self) {
        while self
            .peek_token_raw()
            .map(|k| k.is_trivia())
            .unwrap_or(false)
        {
            self.bump_raw();
        }
    }

    pub(super) fn eat_trivia_back(&mut self) -> Option<()> {
        while self
            .peek_back_raw_token()
            .map(|k| k.is_trivia())
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

    pub(super) fn peek_until(&mut self, predicate: impl Fn(SyntaxKind) -> bool) -> Option<(SyntaxKind, &'a str)> {
        self.bump_until(predicate);
        self.peek_raw()
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

    pub(super) fn peek_n(&mut self, mut n: usize) -> Option<(SyntaxKind, &'a str)> {
        loop {
            dbg!(n);
            dbg!(&self.buffer);
            if let Some((tok, s)) = dbg!(self.buffer.get(n)) {
                let tok = *tok;
                if tok.is_trivia() {
                    self.buffer.push_back(dbg!(self.lexer.next()?));
                    n += 1;
                } else {
                    println!("return");
                    return Some((tok, s))
                }
            }
        }
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
