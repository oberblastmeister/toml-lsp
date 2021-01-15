use super::{
    Parser,
    SyntaxKind::{self, *},
    AST,
};

impl<'a> Parser<'a> {
    pub(super) fn eat_trivia(&mut self) {
        while self
            .peek_token_raw()
            .map(|t| t == Whitespace || t == Comment)
            .unwrap_or(false)
        {
            self.bump_raw()
        }
    }

    pub(super) fn peek_token_raw(&self) -> Option<SyntaxKind> {
        self.peek_raw().map(|(tok, s)| tok)
    }

    pub(super) fn peek_raw(&self) -> Option<(SyntaxKind, &'a str)> {
        self.buffer.get(0).copied()
    }

    pub(super) fn peek(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.eat_trivia();
        self.peek_raw()
    }

    pub(super) fn peek_token(&mut self) -> Option<SyntaxKind> {
        self.peek().map(|(tok, s)| tok)
    }

    pub(super) fn next(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let res = self.buffer.pop_front();
        if let Some(next) = self.lexer.next() {
            self.buffer.push_back(next);
        }
        res
    }

    pub(super) fn next_token(&mut self) -> Option<SyntaxKind> {
        self.next().map(|(tok, _)| tok)
    }

    pub(super) fn parse(mut self) -> AST {
        self.start_node(Root);
        while self.peek().is_some() {
            self.parse_main();
        }
        self.finish_node();
        AST {
            node: self.builder.finish(),
            errors: self.errors,
        }
    }
}
