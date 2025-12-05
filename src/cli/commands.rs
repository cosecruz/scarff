use clap::{Subcommand};

pub mod new;
pub mod init;

#[derive(Subcommand, Debug)]
#[command(about="commands")]
pub enum Commands{
  //new: command to scaffold a new project
  New{
    //project_name: can be a <project_name> or a ./path_to/<project_name>
    project_name: Option<String>,

    //todo: //language --lang:
    //todo: //--interactive --i:
    //todo: more

  },

  //init: command to initialize scaffold in an already created project; if none scarff will ask to create.
  Init {
    //path: default "." but user can enter valid path
    #[arg(short, long, default_value=".")]
    path : String,

  },

  //command to manage and view documentations of user or libraries.
  //todo: Doc,

  //command to rewind, reverse changes to their specified or previous state.
  //todo: RollBack,
}
