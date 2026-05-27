use crate::config::{Config, config_init};
use crate::css::Css;
use crate::storage::Storage;
pub struct Bus{
    pub config: Config,
    pub css: Css,
    pub storage: Storage
}
impl Bus {
    pub fn init() ->Result<Self, String>{
        let config = config_init()?;
        let css = Css::new(&config);
        let storage = Storage::new();
        Ok(Bus{
            config,
            css,
            storage
        })
    }
}