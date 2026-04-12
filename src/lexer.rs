use crate::LoxError;
use crate::token::{Token, TokenKind};

pub struct Lexer<'de> {
    whole: &'de str,
    rest: &'de str,
    byte_index: usize,
    peeked: Option<Result<Token<'de>, LoxError>>,
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            rest: input,
            byte_index: 0,
            peeked: None,
        }
    }
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, LoxError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.peeked.take() {
            return Some(next);
        }

        loop {
            let mut chars = self.rest.chars();

            let c = chars.next()?;
            let c_at = self.byte_index;
            let c_str = &self.rest[..c.len_utf8()];
            let c_onwards = self.rest;
            self.rest = chars.as_str();
            self.byte_index += c.len_utf8();

            enum Started {
                Slash,
                String,
                Number,
                Ident,
                IfEqualElse(TokenKind, TokenKind),
            }

            let just = |kind: TokenKind| {
                Some(Ok(Token {
                    kind,
                    offset: c_at,
                    line: self.whole[..c_at].lines().count(),
                    origin: c_str,
                }))
            };

            let started = match c {
                '(' => return just(TokenKind::LeftParen),
                ')' => return just(TokenKind::RightParen),
                '{' => return just(TokenKind::LeftBrace),
                '}' => return just(TokenKind::RightBrace),
                ',' => return just(TokenKind::Comma),
                ';' => return just(TokenKind::Semicolon),
                '.' => return just(TokenKind::Dot),
                '+' => return just(TokenKind::Plus),
                '-' => return just(TokenKind::Minus),
                '*' => return just(TokenKind::Star),
                '/' => Started::Slash,
                '<' => Started::IfEqualElse(TokenKind::LessEqual, TokenKind::Less),
                '>' => Started::IfEqualElse(TokenKind::GreaterEqual, TokenKind::Greater),
                '!' => Started::IfEqualElse(TokenKind::BangEqual, TokenKind::Bang),
                '=' => Started::IfEqualElse(TokenKind::EqualEqual, TokenKind::Equal),
                '"' => Started::String,
                '0'..='9' => Started::Number,
                'a'..='z' | 'A'..='Z' | '_' => Started::Ident,
                c if c.is_ascii_whitespace() => continue,
                c => {
                    let lines = self.whole[..self.byte_index].lines().count();
                    return Some(Err(LoxError::UnexpectedCharacter(lines, c)));
                }
            };

            break match started {
                Started::String => {
                    if let Some(end) = self.rest.find('"') {
                        let literal = &c_onwards[..end + 1 + 1];
                        self.byte_index += end + 1;
                        self.rest = &self.rest[end + 1..];
                        Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            line: self.whole[..c_at].lines().count(),
                            kind: TokenKind::String,
                        }))
                    } else {
                        let lines = self.whole[..self.byte_index - c.len_utf8()].lines().count();
                        let msg = &self.whole[self.byte_index - c.len_utf8()..self.whole.len()];
                        let err = LoxError::UnterminatedString(lines, msg.into());
                        // swallow the remainder of input as being a string
                        self.byte_index += self.rest.len();
                        self.rest = &self.rest[self.rest.len()..];
                        return Some(Err(err));
                    }
                }
                Started::Slash => {
                    if self.rest.starts_with('/') {
                        // this is a comment!
                        let line_end = self.rest.find('\n').unwrap_or(self.rest.len());
                        self.byte_index += line_end;
                        self.rest = &self.rest[line_end..];
                        continue;
                    } else {
                        Some(Ok(Token {
                            origin: c_str,
                            offset: c_at,
                            line: self.whole[..c_at].lines().count(),
                            kind: TokenKind::Slash,
                        }))
                    }
                }
                Started::Ident => {
                    let first_non_ident = c_onwards
                        .find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
                        .unwrap_or(c_onwards.len());
                    let literal = &c_onwards[..first_non_ident];
                    let extra_bytes = literal.len() - c.len_utf8();
                    self.byte_index += extra_bytes;
                    self.rest = &self.rest[extra_bytes..];

                    let kind = match literal {
                        "and" => TokenKind::And,
                        "class" => TokenKind::Class,
                        "else" => TokenKind::Else,
                        "false" => TokenKind::False,
                        "for" => TokenKind::For,
                        "fun" => TokenKind::Fun,
                        "if" => TokenKind::If,
                        "nil" => TokenKind::Nil,
                        "or" => TokenKind::Or,
                        "print" => TokenKind::Print,
                        "return" => TokenKind::Return,
                        "super" => TokenKind::Super,
                        "this" => TokenKind::This,
                        "true" => TokenKind::True,
                        "var" => TokenKind::Var,
                        "while" => TokenKind::While,
                        _ => TokenKind::Ident,
                    };

                    return Some(Ok(Token {
                        origin: literal,
                        offset: c_at,
                        line: self.whole[..c_at].lines().count(),
                        kind,
                    }));
                }
                Started::Number => {
                    let first_non_digit = c_onwards
                        .find(|c| !matches!(c, '.' | '0'..='9'))
                        .unwrap_or(c_onwards.len());

                    let mut literal = &c_onwards[..first_non_digit];
                    let mut dotted = literal.splitn(3, '.');
                    match (dotted.next(), dotted.next(), dotted.next()) {
                        (Some(one), Some(two), Some(_)) => {
                            literal = &literal[..one.len() + 1 + two.len()];
                        }
                        (Some(one), Some(""), None) => {
                            literal = &literal[..one.len()];
                        }
                        _ => {} // leave as is - no dots
                    }

                    let extra_bytes = literal.len() - c.len_utf8();
                    self.byte_index += extra_bytes;
                    self.rest = &self.rest[extra_bytes..];

                    let n = match literal.parse() {
                        Ok(n) => n,
                        Err(_) => {
                            let lines = self.whole[..self.byte_index].lines().count();
                            return Some(Err(LoxError::ParseNumberFailed(lines, literal.into())));
                        }
                    };

                    return Some(Ok(Token {
                        origin: literal,
                        offset: c_at,
                        line: self.whole[..c_at].lines().count(),
                        kind: TokenKind::Number(n),
                    }));
                }
                Started::IfEqualElse(yes, no) => {
                    //self.rest = self.rest.trim_start();
                    //let trimmed = c_onwards.len() - self.rest.len() - 1;
                    //self.byte_index += trimmed;
                    if self.rest.starts_with('=') {
                        //let span = &c_onwards[..c.len_utf8() + trimmed + 1];
                        let span = &c_onwards[..c.len_utf8() + 1];
                        self.rest = &self.rest[1..];
                        self.byte_index += 1;
                        Some(Ok(Token {
                            origin: span,
                            offset: c_at,
                            line: self.whole[..c_at].lines().count(),
                            kind: yes,
                        }))
                    } else {
                        Some(Ok(Token {
                            origin: c_str,
                            offset: c_at,
                            line: self.whole[..c_at].lines().count(),
                            kind: no,
                        }))
                    }
                }
            };
        }
    }
}
