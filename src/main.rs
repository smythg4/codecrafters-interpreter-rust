use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf};

use codecrafters_interpreter::LoxError;
use codecrafters_interpreter::Parser as LoxParser;
use codecrafters_interpreter::Intepreter;
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
            let (statements, errors) = parser.parse_program();
            let mut i = Intepreter::new();
            let val = i.interpret(statements);
            match val {
                Ok(_) => return Ok(()),
                Err(e)
                    if matches!(
                        e,
                        LoxError::NumberOperandRequired(_)
                            | LoxError::TwoNumberOperandsRequired(_)
                            | LoxError::TwoNumberOrStringOperandsRequired(_)
                            | LoxError::TwoBooleanOperandsRequired(_)
                    ) =>
                {
                    eprintln!("{e}");
                    std::process::exit(70);
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            }
        }
        Commands::Run { filename } => {
            let mut f = std::fs::File::open(filename)?;
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let mut parser = LoxParser::new(&contents);
            let (statements, errors) = parser.parse_program();
            let mut i = Intepreter::new();
            let val = i.interpret(statements);
            match val {
                Ok(_) => return Ok(()),
                Err(e)
                    if matches!(
                        e,
                        LoxError::NumberOperandRequired(_)
                            | LoxError::TwoNumberOperandsRequired(_)
                            | LoxError::TwoNumberOrStringOperandsRequired(_)
                            | LoxError::TwoBooleanOperandsRequired(_)
                    ) =>
                {
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
