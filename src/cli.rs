mod commands;

use std::error::Error;

use clap::{Parser};
use commands::{Commands, new, init};


#[derive(Parser)]
#[command(name = "scarff")]
#[command(version = "0.1.0")]
#[command(about = "Intelligent project scaffolding tool")]
pub struct Cli{
  //command contains the subcommands of scarf.
  #[command(subcommand)]
  pub command: Commands,

  //verbose: with --verbose / -v commands will show more info
  #[arg(short, long, global=true)]
  verbose: bool,

}


impl Commands{
  pub fn run(self)->Result<(), Box<dyn Error>>{
    println!("running scarff cli");

  match self{
    Commands::New { project_name }=>{
      new::run(project_name)

    },
    Commands::Init { path }=>{
      init::run(path)
    },
  }
}
}

// pub fn run() -> Result<(), Box<dyn Error>>{
//   let cli = Cli::parse();
// }

