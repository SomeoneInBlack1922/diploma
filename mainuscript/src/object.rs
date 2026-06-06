use std::{fmt::Display, fs::{Metadata, DirEntry}, path::{Path, PathBuf}};
use std::fs::read_dir;

use crate::{config::{self, Config}, helper_types::NameString};
use crate::storage::OBJECT_DIR_PATH;

pub use implementation::*;

pub mod regular_chat;
pub mod implementation;

#[derive(Debug)]
#[derive(Clone)]
// Data about object not loaded in memory but present on disk
pub struct FsObject{
    pub object_type: ObjectType,
    pub name: NameString,
    pub metadata: Metadata,
    pub path: PathBuf // relative to storage
}
#[derive(Clone)]
#[derive(Debug)]
pub enum ObjectType{
    RegularChat,
    TextScript
}
// #[test]
// fn type_try_from() {
//     let config: Config = config::config_init().unwrap();
//     let read_dir_result = read_dir(&*config.storage_folder.get_value().join(OBJECT_DIR_PATH)).unwrap();
//     for read_result in read_dir_result{
//         if let Ok(read) = read_result {
//             let object_tye = ObjectType::try_from(read.path().as_ref());
//             #[cfg(test)] dbg!(object_tye);
//         }
//     }
// }