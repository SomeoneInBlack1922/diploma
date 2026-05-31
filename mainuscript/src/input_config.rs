use std::path::PathBuf;
use std::env::home_dir;

pub const SAVE_FOLDER_NAME: &'static str = ".mainuscript";
pub struct InputConfig{
    pub storage_folder: PathBuf
}
impl InputConfig{
    pub fn new() -> Result<Self, String>{
        let storgae_folder = match home_dir(){
            Some(data) => {data.join(SAVE_FOLDER_NAME)},
            None => return Err("Failed to get home directory".into())
        };
        return Ok(Self { storage_folder: storgae_folder })
    }
}