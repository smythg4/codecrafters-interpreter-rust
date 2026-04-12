use crate::LoxError;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Statement<'de> {
    ExpressionStatement(Expression<'de>),
    Print(Expression<'de>),
    Var {
        name: &'de str,
        initializer: Option<Expression<'de>>,
    },
    Block(Vec<Statement<'de>>),
    If{condition: Expression<'de>, then_branch: Box<Statement<'de>>, else_branch: Option<Box<Statement<'de>>> },
}

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
    Variable(usize, &'de str), // line, name
    Assign {
        line: usize,
        name: &'de str,
        value: Box<Expression<'de>>,
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
            Expression::Assign { line, name, value } => {
                todo!();
            }
            Expression::Variable(line, name) => {
                todo!();
            }
            Expression::Grouping(exp) => write!(f, "(group {exp})"),
        }
    }
}
