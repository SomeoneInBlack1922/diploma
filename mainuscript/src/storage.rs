use std::fs::{File, ReadDir, create_dir_all, read_dir};
use std::path::{PathBuf};
use std::io::ErrorKind;
use crate::bus::Bus;
use crate::helper_types::{OnFailure};
use crate::helper_types::NameString;
use crate::config::{self, Config, config_init};
use crate::object::FsObject;

pub const OBJECT_DIR_PATH: &'static str = "objects";
pub struct Storage{
}
impl Storage{
    pub fn new() -> Storage{
        Storage {}
    }
    /// Returns collection of names of FsObjects
    /// If fails to load returns Err(String)
    pub fn get_object_list(&self, config: &Config) -> Result<Vec<FsObject>, String>{
        self.create_and_populate_storage_dir(config);
        let mut out_vec: Vec<FsObject> = Vec::with_capacity(16);
        let object_folder_iterator = match read_dir(&*config.storage_folder.get_value().join(OBJECT_DIR_PATH)){
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
    pub fn create_and_populate_storage_dir(&self, config: &Config) -> OnFailure<String>{
        let storage_folder_path = &*config.storage_folder.get_value();
        let objects_dir_path = PathBuf::from(storage_folder_path).join(OBJECT_DIR_PATH);

        #[cfg(test)] println!("{:?}", &storage_folder_path);
        #[cfg(test)] println!("{:?}", &objects_dir_path);

        if let Err(err) = create_dir_all(storage_folder_path){
            #[cfg(test)] println!("{:?}", &err);
            return Some(err.to_string());
        }
        if let Err(err) = create_dir_all(objects_dir_path){
            #[cfg(test)] println!("{:?}", &err);
            return Some(err.to_string());
        }
        return None;
    }
}
#[test]
fn create_and_populate_storage_dir_test(){
    let storage = Storage::new();
    let config = config_init().unwrap();
    storage.create_and_populate_storage_dir(&config);
}
#[test]
#[ignore]
fn populate_storage_dir_with_test_data(){
    let config = config_init().unwrap();
    let path = config.storage_folder.get_value().join(OBJECT_DIR_PATH);
    File::create(path.clone().join("chat1.regular"));
    File::create(path.clone().join("chat2.regular"));
    File::create(path.clone().join("chat3.regular"));
}