use crate::Interpreter;
use crate::ast::{Expression, Statement};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
        }
    }

    pub fn finish(self) -> Interpreter {
        self.interpreter
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Block(stmts) => {
                self.begin_scope();
                self.resolve_statements(stmts);
                self.end_scope();
            }
            Statement::Var { name, initializer } => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expression(init);
                }
                self.define(name);
            }
            Statement::Function { name, params, body } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(params, body);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Statement::Print(expression) => self.resolve_expression(expression),
            Statement::Expression(expression) => self.resolve_expression(expression),
            Statement::Return(expression) => {
                if let Some(expression) = expression {
                    self.resolve_expression(expression);
                }
            }
            Statement::While {
                condition,
                statement,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(statement);
            }
        }
    }

    fn resolve_function(&mut self, params: &Vec<Rc<str>>, body: &[Statement]) {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_statements(body);
        self.end_scope();
    }

    fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Variable { expr_id, name, .. } => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(name.as_ref()) != Some(&true)
                {
                    // ERROR: "can't read local variable in its own initializer"
                }
                self.resolve_local(*expr_id, name);
            }
            Expression::Assign {
                expr_id,
                name,
                value,
                ..
            } => {
                self.resolve_expression(value);
                self.resolve_local(*expr_id, name);
            }
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expression::Call { callee, args, .. } => {
                self.resolve_expression(callee);
                args.iter().for_each(|a| self.resolve_expression(a));
            }
            Expression::Grouping(expression) => self.resolve_expression(expression),
            Expression::Literal(_) => {}
            Expression::Logical { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expression::Unary { right, .. } => {
                self.resolve_expression(right);
            }
        }
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

    fn declare(&mut self, name: &str) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.into(), false);
    }

    fn define(&mut self, name: &str) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.into(), true);
    }
}
