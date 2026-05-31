use std::sync::{Arc, RwLock};

use crate::config::{Config};
use crate::css::Css;
use crate::input_config::InputConfig;
use crate::storage::{Storage, CONFIG_FILE, SETTINGS_DATA_FILE};
use crate::ui::work_area::settings::OptionsData;

// #[derive(Default)]
pub struct Bus{
    pub css: Css,
    pub storage: Storage,
    pub config: Config,
    pub options_data: Arc<RwLock<OptionsData>>
}
impl Bus {
    pub fn new(input_config: InputConfig) -> Result<Self, String>{
        let css = Css::new();
        let storage = Storage::new(input_config.storage_folder);
        let config: Config = storage.read_from_file(CONFIG_FILE);
        let options_data: Arc<RwLock<OptionsData>> = Arc::new(RwLock::new(storage.read_from_file(SETTINGS_DATA_FILE)));
        Ok(Bus{
            css,
            storage,
            config,
            options_data
        })
    }
}
/// Store settings to files when program ends
impl Drop for Bus{
    fn drop(&mut self) {
        self.storage.store_to_file(&self.config, CONFIG_FILE);
        self.storage.store_to_file(&*self.options_data.read().unwrap(), SETTINGS_DATA_FILE);
    }
}