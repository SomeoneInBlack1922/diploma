use std::fs::{File, create_dir_all, read_dir};
use std::path::PathBuf;
use std::io::ErrorKind::NotFound;
use crate::helper_types::{NameString, OnFailure};
use crate::object::FsObject;
use crate::object::regular_chat::RegularChat;
use borsh::{BorshDeserialize, BorshSerialize};
use colored::Colorize;

pub const OBJECT_DIR_PATH: &'static str = "objects";
pub const CONFIG_FILE: &'static str = "config.data";
pub const SETTINGS_DATA_FILE: &'static str = "settings.data";
#[derive(Default)]
#[derive(Clone)]
pub struct Storage{
    storage_folder: PathBuf
}
impl Storage{
    pub fn new(storage_folder: PathBuf) -> Self{
        return Self{
            storage_folder
        }
    }
    /// Join given path with the path of storage folder
    pub fn join(&self, path: PathBuf) -> PathBuf{
        return self.storage_folder.join(path);
    }
    /// Returns collection of names of FsObjects
    /// If fails to load returns Err(String)
    pub fn get_object_list(&self) -> Result<Vec<FsObject>, String>{
        self.create_and_populate_storage_dir();
        let mut out_vec: Vec<FsObject> = Vec::with_capacity(16);
        let object_folder_iterator = match read_dir(&self.storage_folder.join(OBJECT_DIR_PATH)){
            Ok(iterator) => {iterator},
            Err(err) => {return Err(format!("Error while opening object storage folder:\n{}", err.to_string()))}
        };
        for entry in object_folder_iterator{
            // Ingore Errors
            if let Ok(dir_entry) = entry{
                // Ignore those errors too
                if let Ok(fs_object) = FsObject::try_from(dir_entry){
                    out_vec.push(fs_object);
                }
            }
        }
        return Ok(out_vec);
    }
    pub fn sort_object_list(input: &mut Vec<FsObject>){
        input.sort_by(|currennt, next|{
            currennt.name.cmp(&next.name)
        });
    }
    /// This function should not fail if they already exist and other stuff is fine
    pub fn create_and_populate_storage_dir(&self) -> OnFailure<String>{
        let objects_dir_path = PathBuf::from(&self.storage_folder).join(OBJECT_DIR_PATH);

        #[cfg(test)] println!("{:?}", &self.storage_folder);
        #[cfg(test)] println!("{:?}", &objects_dir_path);

        if let Err(err) = create_dir_all(&self.storage_folder){
            #[cfg(test)] println!("{:?}", &err);
            return Some(err.to_string());
        }
        if let Err(err) = create_dir_all(objects_dir_path){
            #[cfg(test)] println!("{:?}", &err);
            return Some(err.to_string());
        }
        return None;
    }
    pub fn read_from_file<T: BorshDeserialize>(&self, file_path_input: &str) -> Result<T, std::io::Error>{
        let file_path = &self.storage_folder.join(file_path_input);

        let file_result = File::open(file_path);
        let output: T = match file_result{
            // If can open file
            Ok(mut file) => {
                match borsh::from_reader(&mut file) {
                    Ok(config) => config,
                    Err(err) => {
                        return Err(err)
                    }
                }
            },
            // If failed to open a file
            Err(err) => {return Err(err)}
        };
        return Ok(output)
    }
    pub fn store_to_file<T: BorshSerialize>(&self, data: &T, file_path_input: &str) -> OnFailure<std::io::Error>{
        let file_path = &self.storage_folder.join(file_path_input);
        let file: File = match File::create(file_path){
            Ok(file) => {file},
            Err(err) => {
                return Some(err);
            }
        };
        match borsh::to_writer(file, &data){
            Ok(_) => return None,
            Err(err) => {
                return Some(err);
            }
        }
    }
    pub fn new_chat(&self, name: NameString) -> Result<(RegularChat, FsObject), std::io::Error>{
        let chat_path = self.storage_folder.join(OBJECT_DIR_PATH).join::<String>(format!("{}.regular", name.inner).into());
        let new_chat = RegularChat::new(name.clone());
        // IO part
        let file_path = &self.storage_folder.join(chat_path.clone());
        let file: File = match File::create(file_path){
            Ok(file) => {file},
            Err(err) => {
                return Err(err);
            }
        };
        if let Err(err) = borsh::to_writer(&file, &new_chat){
            return Err(err);
        }

        match self.store_to_file(&new_chat, &chat_path.to_string_lossy()){
            Some(err) => return Err(err),
            None => {
                let out_fs_object = FsObject{
                    object_type: crate::object::ObjectType::RegularChat,
                    name: name,
                    metadata: match file.metadata(){
                        Ok(metadata) => metadata,
                        Err(err) => return Err(err)
                    },
                    path: chat_path
                };
                return Ok((new_chat, out_fs_object))
            }
        }
    }
}
// #[test]
// fn create_and_populate_storage_dir_test(){
//     let storage = Storage::new();
//     let mut config = Config::default();
//     config.finish();
//     storage.create_and_populate_storage_dir(&config);
// }
// #[test]
// #[ignore]
// fn populate_storage_dir_with_test_data(){
//     let mut config = Config::default();
//     config.finish();
//     let path = config.storage_folder.get_value().join(OBJECT_DIR_PATH);
//     File::create(path.clone().join("chat1.regular"));
//     File::create(path.clone().join("chat2.regular"));
//     File::create(path.clone().join("chat3.regular"));
// }