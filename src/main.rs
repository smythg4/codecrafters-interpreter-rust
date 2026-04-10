use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use codecrafters_interpreter::{run_file, run_repl};

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
            run_file(filename)?;
        },
        _ => unimplemented!("Haven't done that yet!"),
    };

    Ok(())
}
