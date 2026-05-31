use std::path::PathBuf;
use std::env::home_dir;
use borsh::{BorshSerialize, BorshDeserialize};
use url::Url;

use crate::{ai::AI, config_field::ConfigField, helper_types::OnFailure};


#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Debug)]
pub struct Config{
    #[borsh(skip)]
    pub ai: ConfigField<Option<AI>>,
    // pub ai_api_url: ConfigField<Option<Url>>,
    // pub ai_api_key: ConfigField<Option<String>>
}
impl Default for Config{
    fn default() -> Self {
        return Self {
            ai: ConfigField::new(Some(AI::new()))
        }
    }
}
// impl Config {
//     /// This fucntuion is supposed to be called on a deserialized config to populate values that had been
//     /// initialized to default values
//     pub fn finish(&mut self) -> OnFailure<String>{
//         let home_dir = match home_dir(){
//             Some(path) => {path},
//             None => {return Some(String::from("Failed to identify home directory, fucntion std::env::home_dir failed"));}
//         };
//         // self.storage_folder = ConfigField::new(home_dir.join(".mainuscript"));
//         self.ai = ConfigField::new(None);
//         return None;
//         // return Ok(Config {
//         //     storage_folder: ConfigField::new(home_dir.join(".mainuscript")),
//         //     ai: ConfigField::new(None),
//         //     // ai_api_url: ConfigField::new(None),
//         //     // ai_api_key: ConfigField::new(None)
//         // })
//     }
// }