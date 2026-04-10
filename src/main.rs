use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// filename of lox file to run, ex. helloworld.lox
    script: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // You can check the value provided by positional arguments, or option arguments
    match cli.script {
        Some(path) => jrlox::run_file(path)?,
        None => jrlox::run_repl()?,
    };

    Ok(())
}
