use crate::LoxError;
use crate::ast::{BinaryOperator, Expression, Literal, Statement, UnaryOperator};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use std::iter::Peekable;

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

    pub fn parse_program(&mut self) -> (Vec<Statement<'de>>, Vec<LoxError>) {
        let mut stmts = Vec::new();
        let mut errs = Vec::new();
        loop {
            if self.lexer.peek().is_none() {
                break;
            }
            let maybe_dec = self.declaration();
            match maybe_dec {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    errs.push(e);
                    self.synchronize(); // recover from the errors to continue parsing
                }
            }
        }

        (stmts, errs)
    }

    fn synchronize(&mut self) {
        loop {
            let kind = match self.lexer.peek() {
                None => return,
                Some(Ok(t)) => t.kind,
                Some(Err(_)) => {
                    self.lexer.next();
                    continue;
                }
            };

            match kind {
                TokenKind::Semicolon => {
                    self.lexer.next(); // consume the ';' then stop
                    return;
                }
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return, // don't consume — let the next parse own it
                _ => {
                    self.lexer.next();
                }
            }
        }
    }

    fn block(&mut self) -> Result<Vec<Statement<'de>>, LoxError> {
        let mut statements = Vec::new();

        while !self.check_peek(TokenKind::RightBrace)? {
            if self.lexer.peek().is_none() {
                let last_line = self.whole.lines().count();
                return Err(LoxError::UnexpectedEofExpecting(last_line, "}"));
            }
            statements.push(self.declaration()?);
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement<'de>, LoxError> {
        if self.match_any(&[TokenKind::Var])?.is_some() {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Statement<'de>, LoxError> {
        let name = self.expect(TokenKind::Ident)?.origin;
        let mut initializer = None;
        if self.match_any(&[TokenKind::Equal])?.is_some() {
            // if we have `var x = 5;` we evaluate the right hand side.
            // if we have `var x;` we toss a `None` into the `initializer` section
            initializer = Some(self.expression()?);
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Statement<'de>, LoxError> {
        // if self.match_any(&[TokenKind::LeftBrace])?.is_some() {
        //     return Ok(Statement::Block(self.block()?));
        // }

        // if self.match_any(&[TokenKind::Print])?.is_some() {
        //     return self.print_statement();
        // }

        match self.lexer.peek() {
            Some(Ok(t)) => match t.kind {
                TokenKind::LeftBrace => {
                    self.advance()?;
                    Ok(Statement::Block(self.block()?))
                }
                TokenKind::If => {
                    self.advance()?;
                    self.if_statement()
                },
                TokenKind::While => {
                    self.advance()?;
                    self.while_statement()
                },
                TokenKind::For => {
                    self.advance()?;
                    self.for_statement()
                }
                TokenKind::Print => {
                    self.advance()?;
                    self.print_statement()
                }
                _ => self.expression_statement(),
            },
            Some(Err(e)) => return Err(e.clone()),
            None => return Err(LoxError::UnexpectedEof(self.whole.lines().count())),
        }
    }

    fn if_statement(&mut self) -> Result<Statement<'de>, LoxError> {
        self.expect(TokenKind::LeftParen)?; // consume the '('
        let condition = self.expression()?;
        self.expect(TokenKind::RightParen)?; // consume the ')'

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;
        if self.match_any(&[TokenKind::Else])?.is_some() {
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement<'de>, LoxError> {
        self.expect(TokenKind::LeftParen)?; // consume the '('
        let condition = self.expression()?;
        self.expect(TokenKind::RightParen)?; // consume the ')'

        let statement = Box::new(self.statement()?);

        Ok(Statement::While {
            condition,
            statement,
        })
    }

    fn for_statement(&mut self) -> Result<Statement<'de>, LoxError> {
        // desugar the for loop into a while loop
        self.expect(TokenKind::LeftParen)?; // consume the '('
        let mut initializer= None;
        if self.match_any(&[TokenKind::Var])?.is_some() {
            initializer = Some(self.var_declaration()?);
        } else if self.check_peek(TokenKind::Semicolon)? {
            self.advance()?; // consume the ';', no initializer
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if !self.check_peek(TokenKind::Semicolon)? {
            condition = Some(self.expression()?);
        }
        self.expect(TokenKind::Semicolon)?;

        let mut increment = None;
        if !self.check_peek(TokenKind::RightParen)? {
            increment = Some(self.expression()?);
        }
        self.expect(TokenKind::RightParen)?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Statement::Block(vec![body, Statement::ExpressionStatement(increment)]);
        }

        let condition = condition.unwrap_or(Expression::Literal(Literal::Boolean(true)));

        body = Statement::While { condition, statement: Box::new(body) };

        if let Some(initializer) = initializer {
            body = Statement::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Statement<'de>, LoxError> {
        let expr = self.expression()?;
        self.expect(TokenKind::Semicolon)?; // consume the ';'
        Ok(Statement::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Statement<'de>, LoxError> {
        let expr = self.expression()?;
        self.expect(TokenKind::Semicolon)?; // consume the ';'
        Ok(Statement::ExpressionStatement(expr))
    }

    pub fn expression(&mut self) -> Result<Expression<'de>, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression<'de>, LoxError> {
        let expr = self.or()?;

        if let Some(op) = self.match_any(&[TokenKind::Equal])? {
            let value = Box::new(self.assignment()?);

            // makes sure that the LHS of the `=` is a valid thing to assign a value
            // to
            match expr {
                Expression::Variable(line, name) => {
                    return Ok(Expression::Assign { line, name, value });
                }
                _ => {
                    return Err(LoxError::InvalidAssignment(op.line));
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expression<'de>, LoxError> {
        let mut expr = self.and()?;

        while let Some(op_token) = self.match_any(&[TokenKind::Or])? {
            let right = self.and()?;
            let operator = BinaryOperator::try_from(op_token)?;
            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression<'de>, LoxError> {
        let mut expr = self.equality()?;

        while let Some(op_token) = self.match_any(&[TokenKind::And])? {
            let right = self.equality()?;
            let operator = BinaryOperator::try_from(op_token)?;
            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
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

        if token.kind == TokenKind::Ident {
            return Ok(Expression::Variable(token.line, token.origin));
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
