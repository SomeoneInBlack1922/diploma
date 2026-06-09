use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::ai::AI;
use crate::config::{self, Config, SerializableConfig};
use crate::css::Css;
use crate::input_config::InputConfig;
use crate::storage::{Storage, CONFIG_FILE, SETTINGS_DATA_FILE};
use crate::ui::work_area::settings::OptionsData;
use colored::Colorize;
use relm4::tokio;
use tokio::runtime::{Runtime, Builder};

// #[derive(Default)]
pub struct Bus{
    pub css: Css,
    pub storage: Storage,
    pub config: Config,
    pub async_runtime: Runtime,
    pub ai: AI,
    pub options_data: Arc<RwLock<OptionsData>>
}
impl Bus {
    pub fn new(input_config: InputConfig) -> Result<Self, String>{
        let css = Css::new();
        let storage = Storage::new(input_config.storage_folder);
        let serializable_config: SerializableConfig = match storage.read_from_file(CONFIG_FILE){
            Ok(storage) => storage,
            Err(err) => {
                println!(
                    "{} {} {} {} {}",
                    "Failed to read config file".yellow(),
                    storage.join(PathBuf::from(CONFIG_FILE)).to_string_lossy().cyan(),
                    "due to".yellow(),
                    err.to_string().red(),
                    "falling back to default...".yellow()
                );
                SerializableConfig::default()
            }
        };
        let config = serializable_config.into_config();
        let async_runtime = match Builder::new_multi_thread().enable_all().build(){
            Ok(runtime) => runtime,
            Err(err) => return Err(format!("Failed to set up tokio runtime: {}", err.to_string()))
        };
        let ai = AI::new(&config);
        let options_data_raw = match storage.read_from_file::<OptionsData>(SETTINGS_DATA_FILE) {
            Ok(data) => data,
            Err(err) => {
                println!(
                    "{} {} {} {} {}",
                    "Failed to read data file".yellow(),
                    storage.join(PathBuf::from(CONFIG_FILE)).to_string_lossy().cyan(),
                    "due to".yellow(),
                    err.to_string().red(),
                    "falling back to default...".yellow()
                );
                OptionsData::default()
            }
        };
        let options_data: Arc<RwLock<OptionsData>> = Arc::new(RwLock::new(options_data_raw));
        Ok(Bus{
            css,
            storage,
            config,
            async_runtime,
            ai,
            options_data
        })
    }
    pub fn close(self){
        let storage_clone = self.storage.clone();
        let config_owned = self.config;
        // storage_clone.store_to_file::<SerializableConfig>(&config_owned.into_serializable_config(), CONFIG_FILE);
        self.async_runtime.spawn_blocking(move ||{
            println!("First write entered");
            storage_clone.store_to_file::<SerializableConfig>(&config_owned.into_serializable_config(), CONFIG_FILE);
            println!("Second write exited");
        });
        let storage_owned = self.storage;
        let options_data_owned = self.options_data;
        // storage_owned.store_to_file(&*options_data_owned.read().unwrap(), SETTINGS_DATA_FILE);
        self.async_runtime.spawn_blocking(move ||{
            println!("Second write entered");
            storage_owned.store_to_file(&*options_data_owned.read().unwrap(), SETTINGS_DATA_FILE);
            println!("Second write exited");
        });
        let runtime = self.async_runtime;
        runtime.shutdown_timeout(Duration::from_hours(2));
    }
}
// Store settings to files when program ends
// impl Drop for Bus{
//     fn drop(&mut self) {
//         let storage_clone = self.storage.clone();
//         let config_clone = self.config.clone();
//         self.async_runtime.spawn_blocking(move ||{
//             storage_clone.store_to_file::<SerializableConfig>(&config_clone.into_serializable_config(), CONFIG_FILE);
//         });
//         let storage_clone = self.storage.clone();
//         let options_data_clone = self.options_data.clone();
//         self.async_runtime.spawn_blocking(move ||{
//             storage_clone.store_to_file(&*options_data_clone.read().unwrap(), SETTINGS_DATA_FILE);
//         });
//         let runtime = &self.async_runtime;
//         runtime.shutdown_timeout(Duration::from_hours(2));
//     }
// }