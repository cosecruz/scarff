use std::error::Error;

/// command: new
///   parameter =  project_name:Option<String>
///   if project_name is added then use it
///   else load prompt engine which use dialoguer to get the other info
/// but for now just return error
pub fn run(project_name: Option<String>)-> Result<(), Box<dyn Error>>{
  let name = project_name.ok_or("project_name is requires")?;

  println!("creating new project {}", name);
  Ok(())
}
