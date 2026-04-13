use crate::LoxError;
use crate::ast::{BinaryOperator, Expression, Literal, Statement, UnaryOperator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// creates a child scope from a parent
    pub fn from(parent: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }
    }

    /// used for variable declaration (and initialization) only
    pub fn define(&mut self, key: String, value: Value) -> Option<Value> {
        self.values.insert(key, value)
    }

    /// used for mutable assignments, must first find the scope in which the variable was declared
    pub fn assign(&mut self, key: &str, value: Value) -> Option<Value> {
        if self.values.contains_key(key) {
            return self.values.insert(key.to_string(), value);
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(key, value);
        }
        None
    }

    /// first looks in 'local' scope, then checks the parent scope
    /// this will cascade all the way up the environment stack before returning `None`
    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(val) = self.values.get(key) {
            return Some(val.clone());
        }
        self.parent.as_ref().and_then(|p| p.borrow().get(key))
    }

    /// pop off the return stack
    pub fn parent_env(self) -> Option<Rc<RefCell<Environment>>> {
        self.parent
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String), // owned, not &'de str
    Nil,
    NativeFunction {
        arity: usize,
        func: fn(&[Value]) -> Result<Value, LoxError>,
    },
  LoxFunction {
      name: Rc<str>,
      params: Vec<Rc<str>>,
      body: Vec<Statement>,
      closure: Rc<RefCell<Environment>>,
  }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn arity(&self) -> usize {
        match self {
            Self::NativeFunction { arity, .. } => *arity,
            Self::LoxFunction { params, .. } => params.len(),
            _ => unimplemented!(),
        }
    }

    pub fn call(
        self,
        line: usize,
        interpreter: &mut Intepreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxError> {
        if arguments.len() != self.arity() {
            return Err(LoxError::Arity(line, self.arity(), arguments.len()));
        }
        match self {
            Self::NativeFunction { func, .. } => func(&arguments),
            Self::LoxFunction { params, body, closure, .. } => {
                let old_env = std::mem::replace(&mut interpreter.environment, Rc::new(RefCell::new(Environment::default())));
                interpreter.environment = Rc::new(RefCell::new(Environment::from(&old_env)));
                for (param, arg) in params.iter().zip(arguments.into_iter()) {
                    interpreter.environment.borrow_mut().define(param.to_string(), arg);
                }

                interpreter.interpret(body)?;
                interpreter.environment = old_env;
                return Ok(Value::Nil);
            }
            _ => Err(LoxError::Uncallable(line)),
        }
    }
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
            Value::NativeFunction { .. } => write!(f, "<native fn>"),
            Value::LoxFunction { name, .. } => {
                write!(f, "<fn {name}>")
            },
            //_ => todo!(),
        }
    }
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Nil => Value::Nil,
            Literal::String(s) => Value::String(s.to_string()),
            Literal::Number(n) => Value::Number(n),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Intepreter {
    environment: Rc<RefCell<Environment>>,
}

impl Intepreter {
    pub fn new() -> Self {
        let mut env = Environment::default();
        env.define(
            "clock".into(),
            Value::NativeFunction {
                arity: 0,
                func: |_args| {
                    let secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    Ok(Value::Number(secs))
                },
            },
        );
        Intepreter {
            environment: Rc::new(RefCell::new(env)),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: Statement) -> Result<(), LoxError> {
        match stmt {
            Statement::Expression(exp) => {
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
                self.environment.borrow_mut().define(name.to_string(), value);
            }
            Statement::Block(statements) => {
                self.environment = Rc::new(RefCell::new(Environment::from(&self.environment)));
                for statement in statements {
                    self.execute_statement(statement)?;
                }
                let env = self.environment.take();
                let parent_env = env.parent_env().unwrap();
                self.environment = parent_env;
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if Self::is_truthy(&self.evaluate_expression(condition)?) {
                    self.execute_statement(*then_branch)?;
                } else if let Some(branch) = else_branch {
                    self.execute_statement(*branch)?;
                }
            }
            Statement::While {
                condition,
                statement,
            } => {
                // TODO: this cloning seems silly here!
                while Self::is_truthy(&self.evaluate_expression(condition.clone())?) {
                    self.execute_statement(*statement.clone())?;
                }
            },
            Statement::Function { name, params, body } => {
                let function = Value::LoxFunction { name: Rc::clone(&name), params, body, closure: Rc::clone(&self.environment) };
                self.environment.borrow_mut().define(name.to_string(), function);
            },
            Statement::Return(_) => todo!(),
        }
        Ok(())
    }

    fn evaluate_print(&mut self, exp: Expression) -> Result<(), LoxError> {
        let value = self.evaluate_expression(exp)?;
        println!("{value}");
        Ok(())
    }

    pub fn evaluate_expression(&mut self, expr: Expression) -> Result<Value, LoxError> {
        match expr {
            Expression::Literal(l) => Ok(Value::from(l)),
            Expression::Unary { operator, right } => self.eval_unary(operator, *right),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(operator, *left, *right),
            Expression::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(operator, *left, *right),
            Expression::Grouping(expr) => self.evaluate_expression(*expr),
            Expression::Assign { line, name, value } => {
                let result = self.evaluate_expression(*value)?;
                if self
                    .environment
                    .borrow_mut()
                    .assign(name.as_ref(), result.clone())
                    .is_none()
                {
                    return Err(LoxError::UndefinedVariable(line, name.to_string()));
                }
                Ok(result)
            }
            Expression::Variable(line, name) => match self.environment.borrow().get(name.as_ref()) {
                Some(value) => Ok(value.clone()),
                None => Err(LoxError::UndefinedVariable(line, name.to_string())),
            },
            Expression::Call { line, callee, args } => {
                let callee = self.evaluate_expression(*callee)?;
                let mut arguments = Vec::new();
                for arg in args {
                    let value = self.evaluate_expression(arg)?;
                    arguments.push(value);
                }
                callee.call(line, self, arguments)
            }
        }
    }

    fn eval_unary(
        &mut self,
        operator: UnaryOperator,
        right: Expression,
    ) -> Result<Value, LoxError> {
        let right = self.evaluate_expression(right)?;
        match (operator, right) {
            (UnaryOperator::Minus(_), Value::Number(n)) => Ok(Value::Number(-n)),
            (UnaryOperator::Minus(line), _) => Err(LoxError::NumberOperandRequired(line)),
            (UnaryOperator::Not(_), val) => Ok(Value::Boolean(!Self::is_truthy(&val))),
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    fn eval_binary(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
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

    fn eval_logical(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
    ) -> Result<Value, LoxError> {
        let left = self.evaluate_expression(left)?;
        if matches!(operator, BinaryOperator::Or(_)) {
            if Self::is_truthy(&left) {
                return Ok(left);
            }
        } else if !Self::is_truthy(&left) {
            return Ok(left.clone());
        }

        self.evaluate_expression(right)
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
            //(left, right)=> Err(LoxError::TypeMismatch(0, left, right)),
        }
    }
}
