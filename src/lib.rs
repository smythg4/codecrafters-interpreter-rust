use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

mod ast;
mod evaluator;
mod lexer;
mod parser;
mod resolver;
mod token;

pub use evaluator::{Interpreter, Value};
pub use parser::Parser;
pub use resolver::Resolver;
use token::TokenKind;

#[derive(Error, Debug, Clone)]
pub enum LoxError {
    #[error("[line {0}] Error: Unexpected character: {1}")]
    UnexpectedCharacter(usize, char), // line number and character read
    #[error("[line {0}] Error: Failed to parse number {1}.")]
    ParseNumberFailed(usize, String), // line number and str attempted to parse
    #[error("[line {0}] Error: Unterminated string.")]
    UnterminatedString(usize, String), // line number and str attempted to parse
    #[error("[line {0}] Error: Unexpected End of File.")]
    UnexpectedEof(usize), // line number
    #[error("[line {0}] Parse Error: Unexpected Token: expected {1:?}, got {2:?}.")]
    UnexpectedToken(usize, TokenKind, TokenKind), // (line#, expected, got)
    #[error("[line {0}] Error at end: Expect '{1}'.")]
    UnexpectedEofExpecting(usize, &'static str),
    #[error("[line {0}] Parse Error: Invalid Token for current operation: {1:?}.")]
    InvalidToken(usize, TokenKind), // (line#, tokentype)
    #[error("[line {0}] Type Error: Invalid Type for current operation: expected: {1}, got {2:?}")]
    InvalidType(usize, String, String),
    #[error("[line {0}] Type Mismatch: {1:?} , {2:?}")]
    TypeMismatch(usize, String, String),
    #[error("[line {0}] Operand must be a number.")]
    NumberOperandRequired(usize),
    #[error("[line {0}] Operands must be numbers.")]
    TwoNumberOperandsRequired(usize),
    #[error("[line {0}] Operands must be two numbers or two strings.")]
    TwoNumberOrStringOperandsRequired(usize),
    #[error("[line {0}] Operands must be booleans.")]
    TwoBooleanOperandsRequired(usize),
    #[error("[line {0}] Invalid Token for Binary Expression: {1:?}.")]
    BinaryInvalidToken(usize, TokenKind),
    #[error("[line {0}] Invalid Token for Unary Expression: {1:?}.")]
    UnaryInvalidToken(usize, TokenKind),
    #[error("[line {0}] Invalid Token for Literal Expression: {1:?}.")]
    LiteralInvalidToken(usize, TokenKind),
    #[error("[line {0}] Undefined variable '{1}'.")]
    UndefinedVariable(usize, String), // line, name
    #[error("[line {0}] Invalid assignment target.")]
    InvalidAssignment(usize), // line
    #[error("[line {0}] Can't have more than 255 arguments.")]
    TooManyArguments(usize), // line
    #[error("[line {0}] Can only call functions and classes.")]
    Uncallable(usize), // line
    #[error("[line {0}] Expected {1} arguments but got {2}.")]
    Arity(usize, usize, usize), // line, expected, got
    #[error("This is just a return value, not an actual error")]
    Return(Box<Value>),
    #[error("[line {0}] Already a variable with name '{1}' in this scope.")]
    DuplicateDeclaration(usize, String),
    #[error("[line {0}] Can't return from top-level code.")]
    TopLevelReturn(usize),
    #[error("[line {0}] can't read local variable '{1}' in its own initializer")]
    SelfInitialization(usize, String),
    #[error("[line {0}] Only instances have properties or fields. Found: {1}")]
    InvalidTypeProperties(usize, String),
    #[error("[line {0}] Undefined property '{2}' found for class '{1}'")]
    UndefinedProperty(usize, String, String),
}

impl LoxError {
    pub fn is_runtime_error(&self) -> bool {
        matches!(
            self,
            LoxError::NumberOperandRequired(_)
                | LoxError::TwoNumberOperandsRequired(_)
                | LoxError::TwoNumberOrStringOperandsRequired(_)
                | LoxError::TwoBooleanOperandsRequired(_)
                | LoxError::UndefinedVariable(_, _)
                | LoxError::InvalidAssignment(_)
                | LoxError::TooManyArguments(_)
                | LoxError::Uncallable(_)
                | LoxError::Arity(_, _, _)
        )
    }
}

pub fn lex_file(path: PathBuf) -> Result<()> {
    let mut f = File::open(path)?;
    let mut source_code = String::new();
    let mut output = std::io::stdout();

    f.read_to_string(&mut source_code)?;

    lex(&source_code, &mut output)?;
    Ok(())
}

// move this to its own module
pub fn run_repl() -> Result<()> {
    let input = BufReader::new(std::io::stdin());
    let mut output = std::io::stdout();

    write!(output, ">> ")?;
    output.flush()?;

    for line in input.lines() {
        let line = line?;
        lex(&line, &mut output)?;

        write!(output, "\n>> ")?;
        output.flush()?;
    }

    Ok(())
}

fn lex<W: Write>(input: &str, output: &mut W) -> Result<()> {
    let lexer = lexer::Lexer::new(input);
    let mut errors_detected = false;

    for result in lexer {
        match result {
            Ok(token) => writeln!(output, "{token}")?,
            Err(e) => {
                eprintln!("{e}");
                errors_detected = true;
            }
        }
    }
    writeln!(output, "EOF  null")?;

    if errors_detected {
        std::process::exit(65);
    }

    Ok(())
}
