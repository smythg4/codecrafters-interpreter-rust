use anyhow::Result;
use clap::{Parser, Subcommand};
use codecrafters_interpreter::Resolver;
use std::{io::Read, path::PathBuf};

use codecrafters_interpreter::Interpreter;
use codecrafters_interpreter::Parser as LoxParser;
use codecrafters_interpreter::lex_file;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Tokenize { filename: PathBuf },
    Parse { filename: PathBuf },
    Evaluate { filename: PathBuf },
    Run { filename: PathBuf },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Tokenize { filename } => {
            lex_file(filename)?;
        }
        Commands::Parse { filename } => {
            let mut f = std::fs::File::open(filename)?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let mut parser = LoxParser::new(&contents);
            match parser.expression() {
                Ok(exp) => println!("{}", exp),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65);
                }
            }
        }
        Commands::Evaluate { filename } => {
            let mut f = std::fs::File::open(filename)?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let mut parser = LoxParser::new(&contents);
            let exp = match parser.expression() {
                Ok(exp) => exp,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65);
                }
            };

            let mut interpreter = Interpreter::new();
            match interpreter.evaluate_expression(&exp) {
                Ok(val) => println!("{val}"),
                Err(e) if e.is_runtime_error() => {
                    // only this arm should ever trigger
                    eprintln!("{e}");
                    std::process::exit(70);
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65);
                }
            }
        }
        Commands::Run { filename } => {
            let mut f = std::fs::File::open(filename)?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let mut parser = LoxParser::new(&contents);
            let (statements, errors) = parser.parse_program();
            if !errors.is_empty() {
                for e in &errors {
                    eprintln!("{e}");
                }
                std::process::exit(65);
            }
            let interpreter = Interpreter::new();
            let mut resolver = Resolver::new(interpreter);
            let errs = resolver.resolve_statements(&statements);
            for resolution_error in &errs {
                eprintln!("{resolution_error}");
            }
            if !errs.is_empty() {
                std::process::exit(65);
            }
            let mut interpreter = resolver.finish();
            let val = interpreter.interpret(statements);
            match val {
                Ok(_) => return Ok(()),
                Err(e) if e.is_runtime_error() => {
                    // only this arm should ever trigger
                    eprintln!("{e}");
                    std::process::exit(70);
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            }
        }
    };

    Ok(())
}
