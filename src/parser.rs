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
            let maybe_stmt = self.statement();
            match maybe_stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    errs.push(e);
                }
            }
        }

        (stmts, errs)
    }

    fn statement(&mut self) -> Result<Statement<'de>, LoxError> {
        if self.match_any(&[TokenKind::Print])?.is_some() {
            return self.print_statement();
        }
        self.expression_statement()
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

    // Recursive descent methods.
    pub fn expression(&mut self) -> Result<Expression<'de>, LoxError> {
        self.equality()
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
