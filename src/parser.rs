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
            }
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
            }
            TokenKind::Nil => Literal::Nil,
            _ => return Err(LoxError::LiteralInvalidToken(value.line, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus(usize),
    Not(usize),
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Minus(_) => write!(f, "-"),
            UnaryOperator::Not(_) => write!(f, "!"),
        }
    }
}

impl TryFrom<Token<'_>> for UnaryOperator {
    type Error = LoxError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::Minus => UnaryOperator::Minus(value.line),
            TokenKind::Bang => UnaryOperator::Not(value.line),
            _ => return Err(LoxError::UnaryInvalidToken(value.line, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add(usize),
    Minus(usize),
    Times(usize),
    Divide(usize),
    Assign(usize),
    Equal(usize),
    NotEqual(usize),
    GreaterThan(usize),
    LessThan(usize),
    GreaterEqual(usize),
    LessEqual(usize),
    And(usize),
    Or(usize),
}

impl BinaryOperator {
    pub fn get_line(&self) -> usize {
        match self {
            BinaryOperator::Add(line) => *line,
            BinaryOperator::Minus(line) => *line,
            BinaryOperator::Times(line) => *line,
            BinaryOperator::Divide(line) => *line,
            BinaryOperator::Assign(line) => *line,
            BinaryOperator::Equal(line) => *line,
            BinaryOperator::NotEqual(line) => *line,
            BinaryOperator::GreaterThan(line) => *line,
            BinaryOperator::LessThan(line) => *line,
            BinaryOperator::GreaterEqual(line) => *line,
            BinaryOperator::LessEqual(line) => *line,
            BinaryOperator::And(line) => *line,
            BinaryOperator::Or(line) => *line,
        }
    }
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add(_) => write!(f, "+"),
            BinaryOperator::Minus(_) => write!(f, "-"),
            BinaryOperator::Times(_) => write!(f, "*"),
            BinaryOperator::Divide(_) => write!(f, "/"),
            BinaryOperator::Assign(_) => write!(f, "="),
            BinaryOperator::NotEqual(_) => write!(f, "!="),
            BinaryOperator::GreaterThan(_) => write!(f, ">"),
            BinaryOperator::GreaterEqual(_) => write!(f, ">="),
            BinaryOperator::LessThan(_) => write!(f, "<"),
            BinaryOperator::Equal(_) => write!(f, "=="),
            BinaryOperator::LessEqual(_) => write!(f, "<="),
            BinaryOperator::And(_) => write!(f, "and"),
            BinaryOperator::Or(_) => write!(f, "or"),
        }
    }
}

impl TryFrom<Token<'_>> for BinaryOperator {
    type Error = LoxError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::Plus => BinaryOperator::Add(value.line),
            TokenKind::Minus => BinaryOperator::Minus(value.line),
            TokenKind::Star => BinaryOperator::Times(value.line),
            TokenKind::Slash => BinaryOperator::Divide(value.line),
            TokenKind::Equal => BinaryOperator::Assign(value.line),
            TokenKind::EqualEqual => BinaryOperator::Equal(value.line),
            TokenKind::BangEqual => BinaryOperator::NotEqual(value.line),
            TokenKind::Greater => BinaryOperator::GreaterThan(value.line),
            TokenKind::Less => BinaryOperator::LessThan(value.line),
            TokenKind::GreaterEqual => BinaryOperator::GreaterEqual(value.line),
            TokenKind::LessEqual => BinaryOperator::LessEqual(value.line),
            TokenKind::And => BinaryOperator::And(value.line),
            TokenKind::Or => BinaryOperator::Or(value.line),
            _ => return Err(LoxError::BinaryInvalidToken(value.line, value.kind)), // calculate line count
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
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({operator} {left} {right})")
            }
            Expression::Grouping(exp) => write!(f, "(group {exp})"),
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
            Err(LoxError::UnexpectedToken(token.line, kind, token.kind))
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

        Err(LoxError::UnexpectedToken(
            token.line,
            TokenKind::LeftParen,
            token.kind,
        ))
    }
}
