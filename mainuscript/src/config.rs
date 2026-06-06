use std::sync::{Arc, RwLock};

use borsh::{BorshSerialize, BorshDeserialize};
use url::Url;

use crate::{ai::AI, config_field::ConfigField};

#[derive(Debug)]
#[derive(Clone)]
pub struct AiUrlKeyPair{
    pub url: Option<Url>,
    pub key: Option<String>
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Config{
    pub ai_api_url_key_pair: Arc<ConfigField<AiUrlKeyPair>>,
    pub selected_model_name: Arc<ConfigField<Option<String>>>
}
#[derive(BorshSerialize, BorshDeserialize)]

pub struct SerializableConfig{
    pub ai_api_url: Option<String>,
    pub ai_api_key: Option<String>,
    pub selected_model_name: Option<String>
}
impl SerializableConfig{
    pub fn into_config(self) -> Config {
        let ai_api_url = match &self.ai_api_url{
            Some(url_string) => Some(Url::parse(&url_string).unwrap()),
            None => None
        };
        Config {
            ai_api_url_key_pair: Arc::new(ConfigField::new(AiUrlKeyPair{
                url: ai_api_url,
                key: self.ai_api_key
            })),
            selected_model_name: Arc::new(ConfigField::new(self.selected_model_name))
        }
    }
}
impl Config{
    pub fn into_serializable_config(&self) -> SerializableConfig {
        let ai_api_url_key_pair = self.ai_api_url_key_pair.get_value_rw_lock().read().unwrap();
        SerializableConfig {
            ai_api_url: match &ai_api_url_key_pair.url {
                Some(url) => Some(url.to_string()),
                None => None
            },
            ai_api_key: ai_api_url_key_pair.key.clone(),
            selected_model_name: self.selected_model_name.get_value_rw_lock().read().unwrap().clone()
        }
    }
}
impl Default for SerializableConfig{
    fn default() -> Self {
        return Self {
            ai_api_url: None,
            ai_api_key: None,
            selected_model_name: None
        };
    }
}