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

    /// assigns a variable at the appropriate depth based on lexical scoping
    pub fn assign_at(&mut self, depth: usize, key: &str, value: Value) -> Option<Value> {
        if depth == 0 {
            self.values.insert(key.to_string(), value)
        } else {
            self.parent
                .as_mut()?
                .borrow_mut()
                .assign_at(depth - 1, key, value)
        }
    }

    /// first looks in 'local' scope, then checks the parent scope
    /// this will cascade all the way up the environment stack before returning `None`
    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(val) = self.values.get(key) {
            return Some(val.clone());
        }
        self.parent.as_ref().and_then(|p| p.borrow().get(key))
    }

    /// looks for a variable at the appropriate depth based on lexical scoping
    pub fn get_at(&self, depth: usize, key: &str) -> Option<Value> {
        if depth == 0 {
            self.values.get(key).cloned()
        } else {
            self.parent.as_ref()?.borrow().get_at(depth - 1, key)
        }
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
    },
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
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxError> {
        match self {
            Self::NativeFunction { func, .. } => {
                if arguments.len() != self.arity() {
                    return Err(LoxError::Arity(line, self.arity(), arguments.len()));
                }
                func(&arguments)
            }
            Self::LoxFunction {
                params,
                body,
                closure,
                ..
            } => {
                if arguments.len() != params.len() {
                    // silly arity check here to get around move semantics
                    return Err(LoxError::Arity(line, params.len(), arguments.len()));
                }
                let old_env = std::mem::replace(
                    &mut interpreter.environment,
                    Rc::new(RefCell::new(Environment::default())),
                );
                interpreter.environment = Rc::new(RefCell::new(Environment::from(&closure)));
                for (param, arg) in params.iter().zip(arguments.into_iter()) {
                    interpreter
                        .environment
                        .borrow_mut()
                        .define(param.to_string(), arg);
                }

                let result = interpreter.interpret(body);
                // make sure we restore the environment in the event of error propogation with `result?;`
                interpreter.environment = old_env;
                match result {
                    Ok(()) => Ok(Value::Nil),
                    Err(LoxError::Return(val)) => Ok(val),
                    Err(e) => return Err(e),
                }
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
            } //_ => todo!(),
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
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
    globals: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));
        globals.as_ref().borrow_mut().define(
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
        Interpreter {
            globals: Rc::clone(&globals),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute_statement(&statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<(), LoxError> {
        match stmt {
            Statement::Expression(exp) => {
                self.evaluate_expression(&exp)?;
            }
            Statement::Print(exp) => {
                self.evaluate_print(exp)?;
            }
            Statement::Var { name, initializer } => {
                let value = match initializer {
                    None => Value::Nil,
                    Some(v) => self.evaluate_expression(&v)?,
                };
                self.environment
                    .borrow_mut()
                    .define(name.to_string(), value);
            }
            Statement::Block(statements) => {
                let old_env = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(Environment::from(&old_env)));

                let result = statements
                    .iter()
                    .try_for_each(|s| self.execute_statement(s));
                // restore unconditionally
                self.environment = old_env;

                result?;
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if Self::is_truthy(&self.evaluate_expression(&condition)?) {
                    self.execute_statement(then_branch)?;
                } else if let Some(branch) = else_branch {
                    self.execute_statement(branch)?;
                }
            }
            Statement::While {
                condition,
                statement,
            } => {
                while Self::is_truthy(&self.evaluate_expression(&condition)?) {
                    self.execute_statement(statement)?;
                }
            }
            Statement::Function { name, params, body } => {
                let function = Value::LoxFunction {
                    name: Rc::clone(&name),
                    params: params.clone(),
                    body: body.clone(),
                    closure: Rc::clone(&self.environment),
                };
                self.environment
                    .borrow_mut()
                    .define(name.to_string(), function);
            }
            Statement::Return(value) => {
                if let Some(val) = value {
                    return Err(LoxError::Return(self.evaluate_expression(&val)?));
                } else {
                    return Err(LoxError::Return(Value::Nil));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn resolve(&mut self, exp_id: usize, depth: usize) {
        self.locals.insert(exp_id, depth);
    }

    fn evaluate_print(&mut self, exp: &Expression) -> Result<(), LoxError> {
        let value = self.evaluate_expression(exp)?;
        println!("{value}");
        Ok(())
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, LoxError> {
        match expr {
            Expression::Literal(l) => Ok(Value::from(l.clone())),
            Expression::Unary { operator, right } => self.eval_unary(operator, right),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(operator, left, right),
            Expression::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(operator, left, right),
            Expression::Grouping(expr) => self.evaluate_expression(expr),
            Expression::Assign {
                expr_id,
                line,
                name,
                value,
                ..
            } => {
                let result = self.evaluate_expression(value)?;
                let assigned = if let Some(depth) = self.locals.get(expr_id) {
                    self.environment.borrow_mut().assign_at(*depth, name.as_ref(), result.clone())
                } else {
                    self.globals.borrow_mut().assign(name.as_ref(), result.clone())  // global fallback
                };
                if assigned.is_none() {
                    return Err(LoxError::UndefinedVariable(*line, name.to_string()));
                }
                Ok(result)
            }
            Expression::Variable {
                expr_id,
                line,
                name,
            } => {
                let value = if let Some(depth) = self.locals.get(expr_id) {
                    self.environment.borrow().get_at(*depth, name)
                } else {
                    self.globals.borrow().get(name.as_ref())  // global fallback
                };
                value.ok_or_else(|| LoxError::UndefinedVariable(*line, name.to_string()))
            }
            Expression::Call { line, callee, args } => {
                let callee = self.evaluate_expression(callee)?;
                let mut arguments = Vec::new();
                for arg in args {
                    let value = self.evaluate_expression(arg)?;
                    arguments.push(value);
                }
                callee.call(*line, self, arguments)
            }
        }
    }

    fn eval_unary(
        &mut self,
        operator: &UnaryOperator,
        right: &Expression,
    ) -> Result<Value, LoxError> {
        let right = self.evaluate_expression(right)?;
        match (operator, right) {
            (UnaryOperator::Minus(_), Value::Number(n)) => Ok(Value::Number(-n)),
            (UnaryOperator::Minus(line), _) => Err(LoxError::NumberOperandRequired(*line)),
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
        operator: &BinaryOperator,
        left: &Expression,
        right: &Expression,
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
                Err(LoxError::TwoNumberOrStringOperandsRequired(*line))
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
            (op, _, _) => Err(LoxError::TwoNumberOperandsRequired(op.get_line())),
        }
    }

    fn eval_logical(
        &mut self,
        operator: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<Value, LoxError> {
        let left = self.evaluate_expression(left)?;
        if matches!(operator, BinaryOperator::Or(_)) {
            if Self::is_truthy(&left) {
                return Ok(left);
            }
        } else if !Self::is_truthy(&left) {
            return Ok(left);
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
