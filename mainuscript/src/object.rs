use std::{fmt::Display, fs::{Metadata, DirEntry}, path::{Path, PathBuf}};

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
pub enum ObjectType{
    RegularChat,
    TextScript
}
impl TryFrom<&Path> for ObjectType {
    type Error = ();
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        dbg!(path);
        todo!()
    }
}
// impl ObjectType{
//     /// Insert full file name get it's tipe. None if file is not valid
//     fn try_from_file_path(path: &Path) -> Option<Self>{
//         dbg!(name);
//         None
//     }
// }