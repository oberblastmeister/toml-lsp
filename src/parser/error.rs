use rowan::{Checkpoint, TextRange, TextSize};
use thiserror::Error;

use crate::lexer::SyntaxKind::{self, *};
use crate::parser::Parser;

use super::ParseResult;

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Expected token {expected:?}, got {got:?}")]
    Expected {
        expected: Box<[SyntaxKind]>,
        got: SyntaxKind,
        range: Option<TextRange>,
    },

    #[error("Unexpected end of file, wanted: {0:?}")]
    UnexpectedEofWanted(Box<[SyntaxKind]>),
    // Stra
}

impl<'a> Parser<'a> {
    pub fn start_error_node(&mut self) -> TextSize {
        self.start_node(Error);
        self.get_text_position()
    }

    pub fn start_error_node_at(&mut self, at: Checkpoint) -> TextSize {
        self.start_node_at(at, Error);
        self.get_text_position()
    }

    pub fn error_node_until(&mut self, predicate: impl Fn(SyntaxKind) -> bool) -> TextRange {
        let start = self.start_error_node();
        self.bump_until(predicate);
        let end = self.finish_error_node();
        TextRange::new(start, end)
    }

    pub fn finish_error_node(&mut self) -> TextSize {
        self.finish_node();
        self.get_text_position()
    }

    pub fn add_error_until(&mut self, mut e: ParseError, predicate: impl Fn(SyntaxKind) -> bool) {
        if let ParseError::Expected { ref mut range, .. } = e {
            *range = Some(self.error_node_until(predicate));
            self.errors.push(e);
        } else {
            self.errors.push(e);
            self.finish_node();
        }
    }
}
