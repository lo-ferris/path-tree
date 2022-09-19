use std::{iter::Peekable, str::CharIndices};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// `:` 58
    /// `:name`
    Normal,
    /// `?` 63
    /// Optional: `:name?-`
    Optional,
    /// Optional Segment: `/:name?/` or `/:name?`
    OptionalSegment,
    // Optional,
    /// `+` 43
    OneOrMore,
    /// `*` 42
    /// ZeroOrMore: `*-`
    ZeroOrMore,
    /// ZeroOrMore Segment: `/*/` or `/*`
    ZeroOrMoreSegment,
    // TODO: regexp
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece<'a> {
    String(&'a [u8]),
    Parameter(Position<'a>, Kind),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Position<'a> {
    Index(usize),
    Named(&'a str),
}

pub struct Parser<'a> {
    pos: usize,
    count: usize,
    input: &'a str,
    cursor: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            count: 0,
            cursor: input.char_indices().peekable(),
        }
    }

    fn string(&mut self) -> &'a [u8] {
        let mut start = self.pos;
        while let Some(&(i, c)) = self.cursor.peek() {
            match c {
                '\\' => {
                    if start < i {
                        self.pos = i;
                        return self.input[start..i].as_bytes();
                    }

                    self.cursor.next();
                    if let Some(&(j, c)) = self.cursor.peek() {
                        // removes `\`
                        if c == '\\' {
                            start = j;
                        } else {
                            self.cursor.next();
                            self.pos = j + 1;
                            return self.input[j..self.pos].as_bytes();
                        }
                    }
                }
                ':' | '+' | '*' => {
                    self.pos = i + 1;
                    return self.input[start..i].as_bytes();
                }
                _ => {
                    self.cursor.next();
                }
            }
        }

        self.input[start..self.input.len()].as_bytes()
    }

    fn parameter(&mut self) -> (Position<'a>, Kind) {
        let start = self.pos;
        while let Some(&(i, c)) = self.cursor.peek() {
            match c {
                '-' | '.' | '_' | '~' | '/' | '\\' | ':' => {
                    self.pos = i;
                    return (Position::Named(&self.input[start..i]), Kind::Normal);
                }
                '?' | '+' | '*' => {
                    self.cursor.next();
                    self.pos = i + 1;
                    return (
                        Position::Named(&self.input[start..i]),
                        if c == '+' {
                            Kind::OneOrMore
                        } else {
                            let f = {
                                let prefix = self
                                    .input
                                    .get(start - 2..start - 1)
                                    .map(|s| s == "/")
                                    .unwrap_or(false);
                                let suffix =
                                    self.cursor.peek().map(|(_, c)| *c == '/').unwrap_or(true);
                                prefix && suffix
                            };
                            if c == '?' {
                                if f {
                                    Kind::OptionalSegment
                                } else {
                                    Kind::Optional
                                }
                            } else if f {
                                Kind::ZeroOrMoreSegment
                            } else {
                                Kind::ZeroOrMore
                            }
                        },
                    );
                }
                _ => {
                    self.cursor.next();
                }
            }
        }

        (
            Position::Named(&self.input[start..self.input.len()]),
            Kind::Normal,
        )
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Piece<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.peek() {
            Some(&(i, c)) => match c {
                ':' => {
                    self.cursor.next();
                    self.pos = i + 1;
                    let (position, kind) = self.parameter();
                    Some(Piece::Parameter(position, kind))
                }
                '+' | '*' => {
                    self.cursor.next();
                    self.count += 1;
                    self.pos = i + 1;
                    Some(Piece::Parameter(
                        Position::Index(self.count),
                        if c == '+' {
                            Kind::OneOrMore
                        } else {
                            let f = {
                                let prefix =
                                    self.input.get(i - 1..i).map(|s| s == "/").unwrap_or(false);
                                let suffix =
                                    self.cursor.peek().map(|(_, c)| *c == '/').unwrap_or(true);
                                prefix && suffix
                            };
                            if f {
                                Kind::ZeroOrMoreSegment
                            } else {
                                Kind::ZeroOrMore
                            }
                        },
                    ))
                }
                _ => Some(Piece::String(self.string())),
            },
            None => None,
        }
    }
}