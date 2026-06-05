use std::{io::Write, ops::Deref, sync::{Arc, RwLock}, time::Duration};

use async_openai::{Client, config::OpenAIConfig, error::{ApiError, ApiErrorResponse, OpenAIError}, types::models::Model};

use std::thread::sleep;
use std::fs::File;

use relm4::tokio::{self};
use tokio::runtime::{Runtime, Builder};
use url::Url;

use crate::{config::{AiUrlKeyPair, Config}, config_field::{ConfigField, ConfigSubscription}};

pub type GlobalClientConfig = Arc<ConfigField<Option<Client<OpenAIConfig>>>>;

#[derive(Debug)]
pub struct AI{
    openai: GlobalClientConfig,
    async_runtime: Arc<Runtime>,
    ai_url_key_pair_subscriber: ConfigSubscription<AiUrlKeyPair>
}
impl AI {
    pub fn new(config: &Config, async_runtime: Runtime) -> Self{

        let global_config:GlobalClientConfig = Arc::new(ConfigField::new(None));
        // Handle changes in future
        let runtine_arc = Arc::new(async_runtime);
        let runtime_clone = runtine_arc.clone();
        let global_config_clone = global_config.clone();
        let ai_url_key_pair_subscriber = ConfigSubscription::subscribe(config.ai_api_url_key_pair.clone(), Box::new(move |new_pair|{
            if let (Some(url), Some(key)) = (&new_pair.url, &new_pair.key){
                let mut  config = OpenAIConfig::new();
                config = config.with_api_base(url.to_string());
                config = config.with_api_key(key);
                let client = Client::with_config(config);
                global_config_clone.update(Some(client));
            }
            else {
                global_config_clone.update(None);
            }
        }
        ));


        let current_url_key_pair = config.ai_api_url_key_pair.get_value_rw_lock().read().unwrap();
        if let (Some(url), Some(key)) = (&current_url_key_pair.url, &current_url_key_pair.key){
            let mut  config = OpenAIConfig::new();
            config = config.with_api_base(url.to_string());
            config = config.with_api_key(key);
            let client = Client::with_config(config);
            global_config.ignorant_update(Some(client));
        }
        else {
            global_config.ignorant_update(None);
        }
        
        
        return Self{
            openai: global_config,
            async_runtime: runtine_arc,
            ai_url_key_pair_subscriber: ai_url_key_pair_subscriber
        }
    }
    

}
pub fn string_from_openai_error(err: OpenAIError) -> String{
    match err{
        OpenAIError::ApiError(response) => {
            return response.to_string();
        },
        OpenAIError::Reqwest(err) =>{
            return err.to_string()
        },
        OpenAIError::FileReadError(err) => {
            return format!("FileReadError: {}",err)
        },
        OpenAIError::FileSaveError(err) => {
            return format!("FileSaveError: {}",err)
        },
        OpenAIError::InvalidArgument(err) => {
            return format!("InvalidArgument: {}",err)
        },
        OpenAIError::JSONDeserialize(err, message) => {
            return format!("JSONDeserialize: {}\n{}", err, message)
        },
        _ => {
            return "OTHER".into()
        }
    }
}
pub async fn get_models(client: &Client<OpenAIConfig>) -> Result<MyModelList, OpenAIError>{
    let model_api = client.models();
    let models_response: Result<MyModelList, OpenAIError> = model_api.list_byot().await;
    match models_response{
        Ok(models) => {
            return Ok(models)
        },
        Err(api_err) => {
            return Err(api_err)
        }
    }
}
impl Deref for AI{
    type Target = GlobalClientConfig;
    fn deref(&self) -> &Self::Target {
        &self.openai
    }
}
#[derive(Debug)]
#[derive(Clone)]
pub enum ClientState{
    Works(Client<OpenAIConfig>),
    Fails(String),
    Empty
}
// impl Clone for ClientState{
//     fn clone(&self) -> Self {
//         match &*self {
//             ClientState::Empty => {
//                 return Self::Empty
//             },
//             ClientState::Fails(err) => {
//                 return ClientState::Fails(OpenAIError::FileReadError("CUSTOM".into()))
//             }
//             ClientState::Works(client) => {
//                 return Self::Works(client.clone())
//             }
//         }
//     }
// }
#[cfg(test)]
impl PartialEq for ClientState {
    fn eq(&self, other: &Self) -> bool {
        match *self {
            ClientState::Empty => {
                if let ClientState::Empty = other {return true}
                else {return false;}
            },
            ClientState::Fails(_) => {
                if let ClientState::Fails(_) = other {return true}
                else {return false;}
            }
            ClientState::Works(_) => {
                if let ClientState::Works(_) = other {return true}
                else {return false;}
            }
        }
    }
}
// unsafe impl Send for ClientState{}

// #[test]
// fn ai_new_test(){
//     let config = Config{
//         ai_api_url_key_pair: Arc::new(ConfigField::new(AiUrlKeyPair{
//             url: Some(Url::parse("https://api.com").unwrap()),
//             key: Some("sk-or-v1-2588449ed871e381e5a98e7cbcdvfb5442gg59c28d0476e88dd6c2a733f2d44".into())
//         })),
//         selected_model_name: Arc::new(ConfigField::new(None))
//     };
//     let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
//     let ai = AI::new(&config, runtime);
//     assert_eq!(*ai.get_value_rw_lock().read().unwrap(), ClientState::Empty);
//     sleep(Duration::from_secs(5));
//     match &*ai.get_value_rw_lock().read().unwrap(){
//         ClientState::Empty => {println!("EMPTY")},
//         ClientState::Fails(err) => {
//             File::create("./error.txt").unwrap().write_all(&err.as_bytes()).unwrap();
//         },
//         ClientState::Works(_) => {println!("WORKS")}
//     }
//     // assert_eq!(*ai.get_value_rw_lock().read().unwrap(), ClientState::Works(Client::new()));
// }
use serde::Deserialize;
#[derive(Deserialize)]
#[derive(Debug)]
pub struct MyModelList{
    pub data: Vec<MyModel>
}
#[derive(Deserialize)]
#[derive(Debug)]
pub struct MyModel{
    pub id: String
}