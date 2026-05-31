use crate::config::{Config};
use crate::css::Css;
use crate::input_config::InputConfig;
use crate::storage::{Storage, CONFIG_FILE, SETTINGS_DATA_FILE};
use crate::ui::work_area::settings::SettingsData;
use borsh::{BorshSerialize, BorshDeserialize};

// #[derive(Default)]
pub struct Bus{
    pub css: Css,
    pub storage: Storage,
    pub config: Config,
    pub settings_data: SettingsData
}
impl Bus {
    pub fn new(input_config: InputConfig) -> Result<Self, String>{
        let css = Css::new();
        let storage = Storage::new(input_config.storage_folder);
        let config: Config = storage.read_from_file(CONFIG_FILE);
        let settings_data: SettingsData = storage.read_from_file(SETTINGS_DATA_FILE);
        Ok(Bus{
            css,
            storage,
            config,
            settings_data
        })
    }
}
/// Store settings to files when program ends
impl Drop for Bus{
    fn drop(&mut self) {
        self.storage.store_to_file(&self.config, CONFIG_FILE);
        self.storage.store_to_file(&self.settings_data, SETTINGS_DATA_FILE);
    }
}
#[test]
fn bus(){
    let bus = Bus::new(InputConfig::new().unwrap()).unwrap();
    dbg!(&bus.config);
    dbg!(&bus.settings_data);
}