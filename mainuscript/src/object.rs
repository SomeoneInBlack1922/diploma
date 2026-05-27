use std::{fmt::Display, fs::{Metadata, DirEntry}, path::{Path, PathBuf}};
use std::fs::read_dir;

use crate::config::{self, Config};
use crate::storage::OBJECT_DIR_PATH;

pub mod regular_chat;

pub struct FsObject{
    object_type: ObjectType,
    metadata: Metadata,
    path: PathBuf
}
impl TryFrom<DirEntry> for FsObject{
    type Error = ();
    fn try_from(dir_entry: DirEntry) -> Result<Self, Self::Error> {
        let object_type = match ObjectType::try_from(dir_entry.path().as_ref()){
            Ok(object_type) => {object_type},
            Err(_) => {return Err(())}
        };
        todo!()
    }
}
#[derive(Debug)]
pub enum ObjectType{
    RegularChat,
    TextScript
}
impl TryFrom<&Path> for ObjectType {
    type Error = ();
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file_extension = match path.extension(){
            Some(extension_os_str) => {extension_os_str.to_string_lossy()},
            None => {return Err(());}
        };
        match &*file_extension{
            "regular" => {return Ok(ObjectType::RegularChat)},
            "textscript" => {return Ok(ObjectType::TextScript)}
            _ => {return Err(());}
        }
    }
}
// impl ObjectType{
//     /// Insert full file name get it's tipe. None if file is not valid
//     fn try_from_file_path(path: &Path) -> Option<Self>{
//         dbg!(name);
//         None
//     }
// }
#[test]
fn type_try_from() {
    let config: Config = config::config_init().unwrap();
    let read_dir_result = read_dir(&*config.storage_folder.get_value().join(OBJECT_DIR_PATH)).unwrap();
    for read_result in read_dir_result{
        if let Ok(read) = read_result {
            let object_tye = ObjectType::try_from(read.path().as_ref());
            #[cfg(test)] dbg!(object_tye);
        }
    }
}