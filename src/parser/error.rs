use rowan::{Checkpoint, TextRange, TextSize};
use thiserror::Error;

use crate::lexer::SyntaxKind::{self, *};
use crate::parser::Parser;

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Expected token {expected:?}, got {got:?}")]
    Expected {
        expected: Box<[SyntaxKind]>,
        got: SyntaxKind,
        range: TextRange,
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

    pub fn finish_error_node(&mut self) -> TextSize {
        self.finish_node();
        self.get_text_position()
    }
}
