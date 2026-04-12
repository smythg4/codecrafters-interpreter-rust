use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

mod ast;
mod evaluator;
mod lexer;
mod parser;
mod token;

pub use evaluator::{Intepreter, Value};
pub use parser::Parser;
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
    #[error("[line {0}] Parse Error: Invalid Token for current operation: {1:?}.")]
    InvalidToken(usize, TokenKind), // (line#, tokentype)
    #[error("[line {0}] Type Error: Invalid Type for current operation: expected: {1}, got {2:?}")]
    InvalidType(usize, String, Value),
    #[error("[line {0}] Type Mismatch: {1:?} , {2:?}")]
    TypeMismatch(usize, Value, Value),
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
