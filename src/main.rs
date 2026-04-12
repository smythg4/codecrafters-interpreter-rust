use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf};

use codecrafters_interpreter::Parser as LoxParser;
use codecrafters_interpreter::lex_file;
use codecrafters_interpreter::evaluate_expression;
use codecrafters_interpreter::LoxError;

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
            let val = evaluate_expression(exp);
            match val {
                Ok(val) => println!("{val}"),
                Err(e) if matches!(e,LoxError::NumberOperandRequired | LoxError::TwoNumberOperandsRequired | LoxError::TwoNumberOrStringOperandsRequired) => {
                    eprintln!("{e}");
                    std::process::exit(70);
                },
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                },
            }

        }
    };

    Ok(())
}
