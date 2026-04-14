use crate::Interpreter;
use crate::ast::{Expression, Statement};
use std::collections::HashMap;
use std::rc::Rc;

use crate::LoxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    Function,
    None,
}

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn finish(self) -> Interpreter {
        self.interpreter
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), LoxError> {
        match statement {
            Statement::Block(stmts) => {
                self.begin_scope();
                self.resolve_statements(stmts)?;
                self.end_scope();
            }
            Statement::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(init) = initializer {
                    self.resolve_expression(init)?;
                }
                self.define(name);
            }
            Statement::Function { name, params, body } => {
                self.declare(name)?;
                self.define(name);

                self.resolve_function(params, body, FunctionType::Function)?;
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch)?;
                }
            }
            Statement::Print(expression) => self.resolve_expression(expression)?,
            Statement::Expression(expression) => self.resolve_expression(expression)?,
            Statement::Return(expression) => {
                if self.current_function == FunctionType::None {
                    return Err(LoxError::TopLevelReturn(0));
                }
                if let Some(expression) = expression {
                    self.resolve_expression(expression)?;
                }
            }
            Statement::While {
                condition,
                statement,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(statement)?;
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, params: &Vec<Rc<str>>, body: &[Statement], f_type: FunctionType) -> Result<(), LoxError> {
        let enclosing_function = self.current_function;
        self.current_function = f_type;

        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), LoxError> {
        match expression {
            Expression::Variable { expr_id, name, .. } => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(name.as_ref()) == Some(&false)
                {
                    return Err(LoxError::SelfInitialization(0, name.as_ref().into()));
                }
                self.resolve_local(*expr_id, name);
            }
            Expression::Assign {
                expr_id,
                name,
                value,
                ..
            } => {
                self.resolve_expression(value)?;
                self.resolve_local(*expr_id, name);
            }
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::Call { callee, args, .. } => {
                self.resolve_expression(callee)?;
                args.iter().try_for_each(|a| self.resolve_expression(a))?;
            }
            Expression::Grouping(expression) => self.resolve_expression(expression)?,
            Expression::Literal(_) => {}
            Expression::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::Unary { right, .. } => {
                self.resolve_expression(right)?;
            }
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr_id: usize, name: &str) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(name) {
                self.interpreter.resolve(expr_id, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) -> Result<(), LoxError> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.get(name).is_some() {
            return Err(LoxError::DuplicateDeclaration(0, name.into()));
        }
        scope.insert(name.into(), false);
        Ok(())
    }

    fn define(&mut self, name: &str) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.into(), true);
    }
}
