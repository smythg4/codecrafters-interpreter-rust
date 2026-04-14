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

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> Vec<LoxError> {
        let mut errors = Vec::new();
        for statement in statements {
            let errs = self.resolve_statement(statement);
            errors.extend_from_slice(&errs);
        }
        errors
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Vec<LoxError> {
        let mut errors = Vec::new();
        match statement {
            Statement::Block(stmts) => {
                self.begin_scope();
                let mut errs = self.resolve_statements(stmts);
                if let Some(e) = errs.pop() {
                    errors.push(e);
                }
                self.end_scope();
            }
            Statement::Class { name, methods } => {
                if let Err(e) = self.declare(name) {
                    errors.push(e);
                }
                self.define(name);
            }
            Statement::Var { name, initializer } => {
                if let Err(e) = self.declare(name) {
                    errors.push(e);
                }
                if let Some(init) = initializer
                    && let Err(e) = self.resolve_expression(init)
                {
                    errors.push(e);
                }
                self.define(name);
            }
            Statement::Function { name, params, body } => {
                if let Err(e) = self.declare(name) {
                    errors.push(e);
                }
                self.define(name);

                let errs = self.resolve_function(params, body, FunctionType::Function);
                errors.extend_from_slice(&errs);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Err(e) = self.resolve_expression(condition) {
                    errors.push(e);
                }
                let errs = self.resolve_statement(then_branch);
                errors.extend_from_slice(&errs);

                if let Some(else_branch) = else_branch {
                    let errs = self.resolve_statement(else_branch);
                    errors.extend_from_slice(&errs);
                }
            }
            Statement::Print(expression) => {
                if let Err(e) = self.resolve_expression(expression) {
                    errors.push(e);
                }
            }
            Statement::Expression(expression) => {
                if let Err(e) = self.resolve_expression(expression) {
                    errors.push(e);
                }
            }
            Statement::Return(expression) => {
                if self.current_function == FunctionType::None {
                    errors.push(LoxError::TopLevelReturn(0));
                }
                if let Some(expression) = expression {
                    if let Err(e) = self.resolve_expression(expression) {
                        errors.push(e);
                    }
                }
            }
            Statement::While {
                condition,
                statement,
            } => {
                if let Err(e) = self.resolve_expression(condition) {
                    errors.push(e);
                }
                let errs = self.resolve_statement(statement);
                errors.extend_from_slice(&errs);
            }
        }
        errors
    }

    fn resolve_function(
        &mut self,
        params: &[Rc<str>],
        body: &[Statement],
        f_type: FunctionType,
    ) -> Vec<LoxError> {
        let mut errors = Vec::new();
        let enclosing_function = self.current_function;
        self.current_function = f_type;

        self.begin_scope();
        for param in params {
            if let Err(e) = self.declare(param) {
                errors.push(e);
            }
            self.define(param);
        }
        let errs = self.resolve_statements(body);
        for e in errs {
            errors.push(e);
        }
        self.end_scope();
        self.current_function = enclosing_function;
        errors
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
