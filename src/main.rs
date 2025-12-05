use clap::Parser;
use scarff::*;

fn main(){
     println!("Welcome to Scarff");
    let cli = cli::Cli::parse();
    if let Err(e) = cli.command.run(){
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

}
