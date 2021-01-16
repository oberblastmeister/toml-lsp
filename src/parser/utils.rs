use rowan::{TextRange, TextSize};

use super::{
    Error, ParseError, Parser,
    SyntaxKind::{self, *},
};

impl<'a> Parser<'a> {
    pub fn get_text_position(&self) -> TextSize {
        self.index
    }

    pub(super) fn bump_until<F>(&mut self, predicate: F)
    where
        F: Fn(SyntaxKind) -> bool,
    {
        loop {
            if self
                .peek_token()
                .map(|kind| predicate(kind))
                .unwrap_or(true)
            {
                break;
            }

            self.bump();
        }
    }

    pub(super) fn expect_peek_any(&mut self, allowed_slice: &[SyntaxKind]) -> Option<SyntaxKind> {
        let next = match self.peek_token() {
            None => None,
            Some(kind) if allowed_slice.contains(&kind) => Some(kind),
            Some(kind) => {
                let start = self.start_error_node();
                self.bump_until(|k| allowed_slice.contains(&k));
                let end = self.finish_error_node();

                self.errors.push(ParseError::Expected {
                    expected: allowed_slice.to_vec().into_boxed_slice(),
                    got: kind,
                    range: TextRange::new(start, end),
                });

                self.peek_token()
            }
        };

        if next.is_none() {
            self.errors.push(ParseError::UnexpectedEofWanted(
                allowed_slice.to_vec().into_boxed_slice(),
            ));
        }
        next
    }

    pub(super) fn expect_peek(&mut self, expected: SyntaxKind) -> Option<SyntaxKind> {
        self.expect_peek_any(&[expected])
    }

    pub(super) fn expect_bump(&mut self, expected: SyntaxKind) {
        if self.expect_peek_any(&[expected]).is_some() {
            self.bump();
        }
    }

    pub(super) fn expect_sequential(&mut self, expected_slice: &[SyntaxKind]) -> Option<()> {
        for expected in expected_slice {
            self.expect_bump(*expected);
        }
        Some(())
    }

    pub(super) fn accept(&mut self, accept: SyntaxKind) -> bool {
        if self.peek_token().map(|tok| tok == accept).unwrap_or(false) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(super) fn accept_all(&mut self, accept: SyntaxKind) {
        while self.accept(accept) {}
    }
}
