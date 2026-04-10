use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

mod lexer;
mod token;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("[line {0}] Error: Unexpected character: {1}")]
    UnexpectedCharacter(usize, char), // line number and character read
    #[error("[line {0}] Error: Failed to parse number {1}.")]
    ParseNumberFailed(usize, String), // line number and str attempted to parse
    #[error("[line {0}] Error: Unterminated string.")]
    UnterminatedString(usize, String), // line number and str attempted to parse
    #[error("[line {0}] Error: Unexpected End of File.")]
    UnexpectedEof(usize), // line number
}

pub fn run_file(path: PathBuf) -> Result<()> {
    let mut f = File::open(path)?;
    let mut source_code = String::new();
    let mut output = std::io::stdout();
    
    f.read_to_string(&mut source_code)?;

    run(&source_code, &mut output)?;
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
        run(&line, &mut output)?;

        write!(output, "\n>> ")?;
        output.flush()?;
    }

    Ok(())
}

fn run<W: Write>(input: &str, output: &mut W) -> Result<()> {
    let mut lexer = lexer::Lexer::new(input.into());
    let mut errors_detected = false;
    
    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => writeln!(output, "{token}")?,
            Err(e) => {
                eprintln!("{e}");
                errors_detected = true;
            },
        }
    }
    writeln!(output, "EOF  null")?;

    if errors_detected {
        std::process::exit(65);
    }

    Ok(())
}
