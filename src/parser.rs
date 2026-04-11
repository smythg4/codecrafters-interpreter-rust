use crate::LoxError;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Literal<'de> {
    Boolean(bool),
    Number(f64),
    String(&'de str),
    Nil,
}

impl<'de> std::fmt::Display for Literal<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Boolean(b) => write!(f, "{b}"),
            Literal::Number(n) => {
                if n.trunc() == *n {
                    write!(f, "{n:.1}")
                } else {
                    write!(f, "{n}")
                }
            },
            Literal::String(s) => write!(f, "{s}"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl<'de> TryFrom<Token<'de>> for Literal<'de> {
    type Error = LoxError;
    fn try_from(value: Token<'de>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::True => Literal::Boolean(true),
            TokenKind::False => Literal::Boolean(false),
            TokenKind::Number(n) => Literal::Number(n),
            TokenKind::String => {
                //let msg = Token::unescape(value.origin); // Cow -> &str wasn't behaving
                Literal::String(value.origin.trim_matches('"'))
            },
            TokenKind::Nil => Literal::Nil,
            _ => return Err(LoxError::InvalidToken(0, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus,
    Not,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Not => write!(f, "!"),
        }
    }
}

impl TryFrom<Token<'_>> for UnaryOperator {
    type Error = LoxError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::Minus => UnaryOperator::Minus,
            TokenKind::Bang => UnaryOperator::Not,
            _ => return Err(LoxError::InvalidToken(0, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Minus,
    Times,
    Divide,
    Assign,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Times => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Assign => write!(f, "="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
        }
    }
}

impl TryFrom<Token<'_>> for BinaryOperator {
    type Error = LoxError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::Plus => BinaryOperator::Add,
            TokenKind::Minus => BinaryOperator::Minus,
            TokenKind::Star => BinaryOperator::Times,
            TokenKind::Slash => BinaryOperator::Divide,
            TokenKind::Equal => BinaryOperator::Assign,
            TokenKind::EqualEqual => BinaryOperator::Equal,
            TokenKind::BangEqual => BinaryOperator::NotEqual,
            TokenKind::Greater => BinaryOperator::GreaterThan,
            TokenKind::Less => BinaryOperator::LessThan,
            TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
            TokenKind::LessEqual => BinaryOperator::LessEqual,
            TokenKind::And => BinaryOperator::And,
            TokenKind::Or => BinaryOperator::Or,
            _ => return Err(LoxError::InvalidToken(0, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone)]
pub enum Expression<'de> {
    Literal(Literal<'de>),
    Unary {
        operator: UnaryOperator,
        right: Box<Expression<'de>>,
    },
    Binary {
        left: Box<Expression<'de>>,
        operator: BinaryOperator,
        right: Box<Expression<'de>>,
    },
    Grouping(Box<Expression<'de>>),
}

impl<'de> std::fmt::Display for Expression<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(lit) => write!(f, "{lit}"),
            Expression::Unary { operator, right } => write!(f, "({operator} {right})"),
            Expression::Binary { left, operator, right } => {
                write!(f, "({operator} {left} {right})")
            },
            Expression::Grouping(exp) => write!(f,"(group {exp})"),
        }
    }
}

pub struct Parser<'de> {
    whole: &'de str,
    lexer: Peekable<Lexer<'de>>,
}

impl<'de> Parser<'de> {
    pub fn new(input: &'de str) -> Self {
        let lexer = Lexer::new(input).peekable();
        Parser {
            whole: input,
            lexer,
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> Result<Option<Token<'de>>, LoxError> {
        for kind in kinds {
            if self.check_peek(*kind)? {
                return self.advance().map(Some);
            }
        }

        Ok(None)
    }

    fn check_peek(&mut self, kind: TokenKind) -> Result<bool, LoxError> {
        match self.lexer.peek() {
            Some(Ok(t)) => Ok(t.kind == kind),
            Some(Err(e)) => Err(e.clone()),
            None => Ok(false),
        }
    }

    fn advance(&mut self) -> Result<Token<'de>, LoxError> {
        match self.lexer.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(e)) => Err(e),
            None => Err(LoxError::UnexpectedEof(0)),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'de>, LoxError> {
        let token = self.advance()?;
        if token.kind == kind {
            Ok(token)
        } else {
            let line_count = self.whole[..token.offset].lines().count();
            Err(LoxError::UnexpectedToken(line_count, kind, token.kind))
        }
    }

    // Recursive descent methods.
    pub fn expression(&mut self) -> Result<Expression<'de>, LoxError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression<'de>, LoxError> {
        self.parse_binary_op(&[TokenKind::EqualEqual, TokenKind::BangEqual], |parser| {
            parser.comparison()
        })
    }

    fn parse_binary_op<F>(
        &mut self,
        operators: &[TokenKind],
        mut parse_next_level: F,
    ) -> Result<Expression<'de>, LoxError>
    where
        F: FnMut(&mut Self) -> Result<Expression<'de>, LoxError>,
    {
        let mut expr = parse_next_level(self)?;

        while let Some(op_token) = self.match_any(operators)? {
            let operator = BinaryOperator::try_from(op_token)?;
            let right = parse_next_level(self)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression<'de>, LoxError> {
        self.parse_binary_op(
            &[
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
            ],
            |parser| parser.term(),
        )
    }

    fn term(&mut self) -> Result<Expression<'de>, LoxError> {
        self.parse_binary_op(&[TokenKind::Minus, TokenKind::Plus], |parser| {
            parser.factor()
        })
    }

    fn factor(&mut self) -> Result<Expression<'de>, LoxError> {
        self.parse_binary_op(&[TokenKind::Star, TokenKind::Slash], |parser| {
            parser.unary()
        })
    }

    fn unary(&mut self) -> Result<Expression<'de>, LoxError> {
        if let Some(op_token) = self.match_any(&[TokenKind::Bang, TokenKind::Minus])? {
            let operator = UnaryOperator::try_from(op_token)?;
            let right = self.unary()?;
            return Ok(Expression::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expression<'de>, LoxError> {
        let token = self.advance()?;

        if let Ok(literal) = Literal::try_from(token) {
            return Ok(Expression::Literal(literal));
        }

        if token.kind == TokenKind::LeftParen {
            let expression = self.expression()?;
            self.expect(TokenKind::RightParen)?; // consume the trailing ')' 
            return Ok(Expression::Grouping(Box::new(expression)));
        }

        let line_count = self.whole[..token.offset].lines().count();
        Err(LoxError::UnexpectedToken(line_count, TokenKind::LeftParen, token.kind))
    }
}
