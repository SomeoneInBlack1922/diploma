use std::path::PathBuf;
use std::env::home_dir;

use crate::{config_field::ConfigField, helper_types::OnFailure};

pub struct Config{
    pub storage_folder: ConfigField<PathBuf>
}
pub fn config_init() -> Result<Config, String>{
    let home_dir = match home_dir(){
        Some(path) => {path},
        None => {return Err(String::from("Failed to identify home directory, fucntion std::env::home_dir failed"));}
    };
    return Ok(Config { storage_folder: ConfigField::new(home_dir.join(".mainuscript")) })
}