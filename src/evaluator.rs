use crate::parser::{BinaryOperator, Expression, Literal, UnaryOperator};
use crate::LoxError;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String), // owned, not &'de str
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Number(n) => {
                write!(f,"{n}")
            }
        }
    }
}

impl From<Literal<'_>> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Nil => Value::Nil,
            Literal::String(s) => Value::String(s.to_string()),
            Literal::Number(n) => Value::Number(n),
        }
    }
}

pub fn evaluate_expression(expr: Expression<'_>) -> Result<Value, LoxError> {
    match expr {
        Expression::Literal(l) => Ok(Value::from(l)),
        Expression::Unary { operator, right } => eval_unary(operator, *right),
        Expression::Binary { left, operator, right } => eval_binary(operator, *left, *right),
        Expression::Grouping(expr) => evaluate_expression(*expr),
    }
}

fn eval_unary(operator: UnaryOperator, right: Expression<'_>) -> Result<Value, LoxError> {
    let right = evaluate_expression(right)?;
    match (operator, right) {
        (UnaryOperator::Minus, Value::Number(n)) => Ok(Value::Number(-n)),
        (UnaryOperator::Minus, val) => Err(LoxError::InvalidType("NUMBER".into(), val)),
        (UnaryOperator::Not, val) => Ok(Value::Boolean(!is_truthy(val))),
    }
}

fn is_truthy(value: Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Boolean(b) => b,
        _ => true,
    }
}

fn eval_binary(operator: BinaryOperator, left: Expression<'_>, right: Expression<'_>) -> Result<Value, LoxError> {
    let left = evaluate_expression(left)?;
    let right = evaluate_expression(right)?;
    match (operator, left, right) {
        (BinaryOperator::Add, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x+y)),
        (BinaryOperator::Add, Value::String(str1), Value::String(str2)) => Ok(Value::String(format!("{}{}", str1, str2))),
        (BinaryOperator::Minus, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x-y)),
        (BinaryOperator::Times, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x*y)),
        (BinaryOperator::Divide, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x/y)),
        (BinaryOperator::GreaterThan, Value::Number(x), Value::Number(y)) => Ok(Value::Boolean(x>y)),
        (BinaryOperator::GreaterEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Boolean(x>=y)),
        (BinaryOperator::LessThan, Value::Number(x), Value::Number(y)) => Ok(Value::Boolean(x<y)),
        (BinaryOperator::LessEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Boolean(x<=y)),
        (BinaryOperator::Equal, left, right) => Ok(Value::Boolean(is_equal(left, right)?)),
        (BinaryOperator::NotEqual, left, right) => Ok(Value::Boolean(!is_equal(left, right)?)),
        (_, left, right)=> Err(LoxError::TypeMismatch(left, right)),
    }
}

fn is_equal(left: Value, right: Value) -> Result<bool, LoxError> {
    match (left, right) {
        (Value::Nil, Value::Nil) => Ok(true),
        (Value::Nil, _) => Ok(false),
        (Value::Number(x), Value::Number(y)) => Ok(x==y),
        (Value::String(x), Value::String(y)) => Ok(x==y),
        (Value::Boolean(x), Value::Boolean(y)) => Ok(x==y),
        (left, right)=> Ok(left == right),
        //(left, right)=> Err(LoxError::TypeMismatch(left, right)),
    }
}