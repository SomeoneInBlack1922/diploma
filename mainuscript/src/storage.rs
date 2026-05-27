use std::fs::{File, ReadDir, create_dir_all, read_dir};
use std::path::{PathBuf};
use std::io::ErrorKind;
use crate::bus::Bus;
use crate::helper_types::{OnFailure};
use crate::helper_types::NameString;
use crate::config::{self, Config, config_init};
use crate::object::FsObject;

const OBJECT_DIR_PATH: &'static str = "objects";
pub struct Storage{
}
impl Storage{
    pub fn new() -> Storage{
        Storage {}
    }
    //Returns collection of names of objects. u64 is for sorting
    //If fails to load returns Err(String)
    pub fn get_object_list(&self, config: &Config) -> Result<Vec<(NameString, u64)>, String>{
        let mut object_folder_iterator: ReadDir;
        self.create_and_populate_storage_dir(config);
        loop{
            let read_dir_result = read_dir(&*config.storage_folder.get_value().join(OBJECT_DIR_PATH));
            if let Ok(read) = read_dir_result {
                object_folder_iterator = read;
                break;
            }
            // match read_dir_result {
            //     Ok(read) => {
                    
            //     },
            //     Err(err) => {
            //         //If storage dir does not eist attempt to crete it 
            //         if let ErrorKind::NotFound = err.kind() {
            //             self.create_and_populate_storage_dir(config);
            //             continue;
            //         }
            //         else {
            //             return Err(err.to_string());
            //         }
            //     }
            // }
        }
        for entry in object_folder_iterator{
            // Ingore Errors
            if let Ok(dir_entry) = entry{
                let fs_object = FsObject::try_from(dir_entry);
            }
        }
        todo!()
    }
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