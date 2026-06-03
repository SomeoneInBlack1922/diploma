use std::sync::{Arc, RwLock};

use crate::ai::AI;
use crate::config::{self, Config, SerializableConfig};
use crate::css::Css;
use crate::input_config::InputConfig;
use crate::storage::{Storage, CONFIG_FILE, SETTINGS_DATA_FILE};
use crate::ui::work_area::settings::OptionsData;
use relm4::tokio;
use tokio::runtime::{Runtime, Builder};

// #[derive(Default)]
pub struct Bus{
    pub css: Css,
    pub storage: Storage,
    pub config: Config,
    // pub async_runtime: Runtime,
    pub ai: AI,
    pub options_data: Arc<RwLock<OptionsData>>
}
impl Bus {
    pub fn new(input_config: InputConfig) -> Result<Self, String>{
        let css = Css::new();
        let storage = Storage::new(input_config.storage_folder);
        let serializable_config: SerializableConfig = storage.read_from_file(CONFIG_FILE);
        let config = serializable_config.into_config();
        let async_runtime = match Builder::new_multi_thread().enable_all().build(){
            Ok(runtime) => runtime,
            Err(err) => return Err(format!("Failed to set up tokio runtime: {}", err.to_string()))
        };
        let ai = AI::new(&config, async_runtime);
        let options_data: Arc<RwLock<OptionsData>> = Arc::new(RwLock::new(storage.read_from_file(SETTINGS_DATA_FILE)));
        Ok(Bus{
            css,
            storage,
            config,
            ai,
            options_data
        })
    }
}
/// Store settings to files when program ends
impl Drop for Bus{
    fn drop(&mut self) {
        self.storage.store_to_file::<SerializableConfig>(&self.config.into_serializable_config(), CONFIG_FILE);
        self.storage.store_to_file(&*self.options_data.read().unwrap(), SETTINGS_DATA_FILE);
    }
}