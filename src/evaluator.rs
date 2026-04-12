use crate::LoxError;
use crate::ast::{BinaryOperator, Expression, Literal, Statement, UnaryOperator};
use std::collections::HashMap;

pub type Environment = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String), // owned, not &'de str
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Number(n) => {
                write!(f, "{n}")
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

pub struct Intepreter {
    environment: Environment,
}

impl Intepreter {
    pub fn new() -> Self {
        Intepreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement<'_>>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: Statement<'_>) -> Result<(), LoxError> {
        match stmt {
            Statement::ExpressionStatement(exp) => {
                self.evaluate_expression(exp)?;
            }
            Statement::Print(exp) => {
                self.evaluate_print(exp)?;
            }
            Statement::Var { name, initializer } => {
                let value = match initializer {
                    None => Value::Nil,
                    Some(v) => self.evaluate_expression(v)?,
                };
                self.environment.insert(name.into(), value);
            }
        }
        Ok(())
    }

    fn evaluate_print(&mut self, exp: Expression) -> Result<Value, LoxError> {
        let value = self.evaluate_expression(exp)?;
        println!("{value}");
        Ok(value)
    }

    pub fn evaluate_expression(&mut self, expr: Expression<'_>) -> Result<Value, LoxError> {
        match expr {
            Expression::Literal(l) => Ok(Value::from(l)),
            Expression::Unary { operator, right } => self.eval_unary(operator, *right),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(operator, *left, *right),
            Expression::Grouping(expr) => self.evaluate_expression(*expr),
            Expression::Assign { line, name, value } => {
                if !self.environment.contains_key(name) {
                    return Err(LoxError::UndefinedVariable(line, name.into()));
                }
                let result = self.evaluate_expression(*value)?;
                self.environment.insert(name.into(), result.clone());
                Ok(result)
            }
            Expression::Variable(line, name) => match self.environment.get(name) {
                Some(value) => Ok(value.clone()),
                None => Err(LoxError::UndefinedVariable(line, name.into())),
            },
        }
    }

    fn eval_unary(
        &mut self,
        operator: UnaryOperator,
        right: Expression<'_>,
    ) -> Result<Value, LoxError> {
        let right = self.evaluate_expression(right)?;
        match (operator, right) {
            (UnaryOperator::Minus(_), Value::Number(n)) => Ok(Value::Number(-n)),
            (UnaryOperator::Minus(line), _) => Err(LoxError::NumberOperandRequired(line)),
            (UnaryOperator::Not(_), val) => Ok(Value::Boolean(!Self::is_truthy(val))),
        }
    }

    fn is_truthy(value: Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => b,
            _ => true,
        }
    }

    fn eval_binary(
        &mut self,
        operator: BinaryOperator,
        left: Expression<'_>,
        right: Expression<'_>,
    ) -> Result<Value, LoxError> {
        let left = self.evaluate_expression(left)?;
        let right = self.evaluate_expression(right)?;
        match (operator, left, right) {
            (BinaryOperator::Add(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Number(x + y))
            }
            (BinaryOperator::Add(_), Value::String(str1), Value::String(str2)) => {
                Ok(Value::String(format!("{}{}", str1, str2)))
            }
            (BinaryOperator::Add(line), _, _) => {
                Err(LoxError::TwoNumberOrStringOperandsRequired(line))
            }
            (BinaryOperator::Minus(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Number(x - y))
            }
            (BinaryOperator::Times(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Number(x * y))
            }
            (BinaryOperator::Divide(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Number(x / y))
            }
            (BinaryOperator::GreaterThan(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Boolean(x > y))
            }
            (BinaryOperator::GreaterEqual(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Boolean(x >= y))
            }
            (BinaryOperator::LessThan(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Boolean(x < y))
            }
            (BinaryOperator::LessEqual(_), Value::Number(x), Value::Number(y)) => {
                Ok(Value::Boolean(x <= y))
            }
            (BinaryOperator::Equal(_), left, right) => {
                Ok(Value::Boolean(Self::is_equal(left, right)?))
            }
            (BinaryOperator::NotEqual(_), left, right) => {
                Ok(Value::Boolean(!Self::is_equal(left, right)?))
            }
            (BinaryOperator::And(_), Value::Boolean(b1), Value::Boolean(b2)) => {
                Ok(Value::Boolean(b1 && b2))
            }
            (BinaryOperator::Or(_), Value::Boolean(b1), Value::Boolean(b2)) => {
                Ok(Value::Boolean(b1 || b2))
            }
            (BinaryOperator::And(line) | BinaryOperator::Or(line), _, _) => {
                Err(LoxError::TwoBooleanOperandsRequired(line))
            }
            (op, _, _) => Err(LoxError::TwoNumberOperandsRequired(op.get_line())),
        }
    }

    fn is_equal(left: Value, right: Value) -> Result<bool, LoxError> {
        match (left, right) {
            (Value::Nil, Value::Nil) => Ok(true),
            (Value::Nil, _) => Ok(false),
            (Value::Number(x), Value::Number(y)) => Ok(x == y),
            (Value::String(x), Value::String(y)) => Ok(x == y),
            (Value::Boolean(x), Value::Boolean(y)) => Ok(x == y),
            (left, right) => Ok(left == right),
            // I think a TypeMismatch is appropriate here, but the tests want 65.0 == "65" to return Ok(`false`)
            //(left, right)=> Err(LoxError::TypeMismatch(left, right)),
        }
    }
}
