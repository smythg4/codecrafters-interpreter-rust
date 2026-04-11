use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{io::Read, path::PathBuf};

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
            let exp = parser.expression()?;

            println!("{}", exp);
        }
        Commands::Run { filename: _ } => {
            unimplemented!();
        }
    };

    Ok(())
}
