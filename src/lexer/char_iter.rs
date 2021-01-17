use std::collections::VecDeque;
use std::str::CharIndices;

#[derive(Debug)]
pub struct CharIter<'a> {
    start: usize,
    end: usize,
    iter: CharIndices<'a>,
    input: &'a str,
    buffer: VecDeque<(usize, char)>,
}

impl<'a> CharIter<'a> {
    pub fn new(input: &str) -> CharIter<'_> {
        let mut iter = input.char_indices();
        let mut buffer = VecDeque::with_capacity(1);
        if let Some(c) = iter.next() {
            buffer.push_back(c);
        }

        CharIter {
            iter,
            input,
            buffer,
            start: 0,
            end: 0,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        let (_, c) = self.buffer.get(0)?;
        Some(*c)
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn slice(&self) -> &'a str {
        &self.input[self.start..self.end]
    }

    #[inline]
    pub fn ignore(&mut self) {
        self.start = self.end;
    }

    pub fn accept(&mut self, valid: char) -> bool {
        let peeked = self.peek();
        match peeked {
            Some(c) if c == valid => {
                self.next();
                true
            }
            _ => false,
        }
    }

    pub fn accept_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            } else {
                self.next();
            }
        }
    }

    pub fn accept_until(&mut self, predicate: impl Fn(char) -> bool) {
        self.accept_while(|c| !predicate(c))
    }

    pub fn find_char(&mut self, c: char) -> Option<char> {
        self.find(|candidate| *candidate == c)
    }
}

impl Iterator for CharIter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let (end, c) = self.buffer.pop_front()?;

        self.end = end + 1;

        if let Some((idx, next_c)) = self.iter.next() {
            self.buffer.push_back((idx, next_c));
        }
        Some(c)
    }
}
