use crate::LoxError;
use crate::token::{Token, TokenKind};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Var {
        name: Rc<str>,
        initializer: Option<Expression>,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        statement: Box<Statement>,
    },
    Function {
        name: Rc<str>,
        params: Vec<Rc<str>>,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
    String(Rc<str>),
    Nil,
}

impl std::fmt::Display for Literal {
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

impl TryFrom<Token<'_>> for Literal {
    type Error = LoxError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::True => Literal::Boolean(true),
            TokenKind::False => Literal::Boolean(false),
            TokenKind::Number(n) => Literal::Number(n),
            TokenKind::String => {
                //let msg = Token::unescape(value.origin); // Cow -> &str wasn't behaving
                Literal::String(Rc::from(value.origin.trim_matches('"')))
            }
            TokenKind::Nil => Literal::Nil,
            _ => return Err(LoxError::LiteralInvalidToken(value.line, value.kind)), // calculate line count
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add(usize),
    Minus(usize),
    Times(usize),
    Divide(usize),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Unary {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Logical {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Variable {
        expr_id: usize,
        line: usize,
        name: Rc<str>,
    },
    Assign {
        expr_id: usize,
        line: usize,
        name: Rc<str>,
        value: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Call {
        line: usize,
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
}

impl std::fmt::Display for Expression {
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
            Expression::Logical {
                left,
                operator,
                right,
            } => {
                write!(f, "({operator} {left} {right})")
            }
            Expression::Assign { name, value, .. } => {
                write!(f, "{} = {}", name, value)
            }
            Expression::Variable { name, .. } => {
                write!(f, "{}", name)
            }
            Expression::Grouping(exp) => write!(f, "(group {exp})"),
            Expression::Call { callee, args, .. } => {
                write!(
                    f,
                    "{callee}({})",
                    args.iter()
                        .map(|a| format!("{a}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}
