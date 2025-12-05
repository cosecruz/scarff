use std::{error::Error};

pub fn run(path: String)-> Result<(), Box<dyn Error>>{

  // if let Some(full_path) = (!path.is_empty()).then(|| path) {
  //       println!("`init` {}", full_path);
  //       Ok(())
  //   } else {
  //       Err("project name is required".into())
  //   }

  println!("initializing scarff in {}", path);
  Ok(())
}
