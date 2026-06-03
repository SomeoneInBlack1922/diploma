use std::sync::{Arc, RwLock};

use regex::Regex;
use relm4::{ComponentSender, gtk::prelude::EditableExt};
use url::Url;

use crate::{bus::Bus, config::AiUrlKeyPair, ui::work_area::settings::{SettingsInput, SettingsView, save_options_data}};

pub fn attempt_read_and_save_settings(settings_view: &mut SettingsView, sender: ComponentSender<SettingsView>){
    // Read raw values from widgets
    let api_url_raw = settings_view.widgets.option_widgets.api_url_entry.text().to_string();
    let api_key_raw = settings_view.widgets.option_widgets.api_key_entry.text().to_string();
    // Validate and convert values
    let api_url = match validate_api_url(api_url_raw){
        Ok(data) => data,
        Err(err) => {
            handle_invalid(settings_view, err);
            return;
        },
    };
    let api_key = match validate_api_key(api_key_raw){
        Ok(data) => data,
        Err(err) => {
            handle_invalid(settings_view, err);
            return;
        },
    };
    settings_view.widgets.control_label.set_text("");
    // Save values to config
    let mut mut_bus = settings_view.bus.write().unwrap();
    mut_bus.config.ai_api_url_key_pair.update(AiUrlKeyPair{
        url: Some(api_url),
        key: Some(api_key)
    });
    drop(mut_bus);
    // Save options data
    save_options_data(settings_view.bus.clone(), &settings_view.widgets);
    // Reload widgets
    sender.input(SettingsInput::ClearAllOptions);

}
fn handle_invalid(settings_view: &mut SettingsView, error: String){
    settings_view.widgets.control_label.set_text(&error);
}
/// Returns empty string if text is empty
fn validate_api_url(text: String) -> Result<Url, String>{
        if text.is_empty(){
            return Err("".into());
        }
        match Url::parse(&text){
            Ok(url) => {
                return Ok(url)
            },
            Err(err) => {
                return Err(
                    format!("Error: {}", err.to_string())
                );
            }
        };
}
/// TODO
fn validate_api_key(text: String) -> Result<String, String>{
    let re = Regex::new("^sk-[a-zA-Z0-9_-]{32,}$").unwrap();
    if re.is_match(&text){
        return Ok(text);
    }
    else{
        return Err("Openai api key seems to be invalid".into());
    }
}