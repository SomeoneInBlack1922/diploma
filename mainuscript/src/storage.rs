use std::fs::{File, ReadDir, create_dir_all, read_dir};
use std::path::{Path, PathBuf};
use std::io::ErrorKind::{self, NotFound};
use crate::bus::Bus;
use crate::helper_types::{OnFailure};
use crate::helper_types::NameString;
use crate::config::{self, Config};
use crate::input_config::InputConfig;
use crate::ui::work_area::settings::SettingsData;
use crate::object::FsObject;
use borsh::{BorshDeserialize, BorshSerialize};
use colored::Colorize;

pub const OBJECT_DIR_PATH: &'static str = "objects";
pub const CONFIG_FILE: &'static str = "config.data";
pub const SETTINGS_DATA_FILE: &'static str = "settings.data";
#[derive(Default)]
pub struct Storage{
    storage_folder: PathBuf
}
impl Storage{
    pub fn new(storage_folder: PathBuf) -> Self{
        return Self{
            storage_folder
        }
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
    pub fn read_from_file<T: BorshDeserialize + Default>(&self, file_path_input: &str) -> T{
        let file_path = &self.storage_folder.join(file_path_input);

        let file_result = File::open(file_path);
        let output: T = match file_result{
            // If can open file
            Ok(mut file) => {
                match borsh::from_reader(&mut file) {
                    Ok(config) => config,
                    Err(err) => {
                        println!(
                            "{} {} {}",
                            "Failed to deserialize config from save file".yellow() ,
                            file_path.to_string_lossy().cyan(), 
                            "Loading default cconfig".yellow()
                        );
                        T::default()
                    }
                }
            },
            // If failed to open a file
            Err(err) => {
                // If error was something other then file not existing print about it
                if err.kind() != NotFound{
                    println!(
                        "{} {} {} {} {}",
                        "Failed to open save file".yellow() ,
                        file_path.to_string_lossy().cyan(), 
                        "due to".yellow(),
                        err.to_string().red(),
                        "falling back to default".yellow()
                    );
                }
                T::default()
            }
        };
        return output
    }
    pub fn store_to_file<T: BorshSerialize + Default>(&self, data: &T, file_path_input: &str){
        let file_path = &self.storage_folder.join(file_path_input);
        let file: File = match File::create(file_path){
            Ok(file) => {file},
            Err(err) => {
                println!(
                    "{} {} {} {} {}",
                    "Failed to save data to".yellow(),
                    file_path.to_string_lossy().cyan(),
                    "due to".yellow(),
                    err.to_string().red(),
                    "some program settings may be lost".yellow()
                );
                return;
            }
        };
        match borsh::to_writer(file, &data){
            Ok(_) => return,
            Err(err) => {
                println!(
                    "{} {} {} {} {}",
                    "Failed to save data to".yellow(),
                    file_path.to_string_lossy().cyan(),
                    "due to".yellow(),
                    err.to_string().red(),
                    "some program settings may be lost".yellow()
                );
                return;
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