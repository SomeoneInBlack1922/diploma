use relm4::{RELM_BLOCKING_THREADS, RELM_THREADS, RelmApp, gtk::{gdk::set_allowed_backends}};
use ui::top::TopView;
use colored::Colorize;
use std::process::exit;

mod ui{
    pub mod top;
    pub mod navigation;
    pub mod logo;
    pub mod work_area{
        pub mod settings;
    }
}
mod storage;
mod object;
mod helper_types;
mod config_field;
mod config;
mod bus;
mod css;
mod ai;
mod input_config;

use config::*;

use crate::{bus::Bus, input_config::InputConfig};
fn main() {
    set_allowed_backends("x11,*");
    RELM_THREADS.set(8).unwrap();
    let app = RelmApp::new("none.none.mainuscript");
    let input_config = match InputConfig::new(){
        Ok(input_config) => {input_config},
        Err(err_message) => {
            println!("{}\n{}", "Failed to load input configuration".yellow(), err_message.red());
            exit(-1)
        }
    };
    let bus = match Bus::new(input_config){
        Ok(bus) => {bus},
        Err(err_message) => {
            println!("{}\n{}", "Failed to initiate a bus".yellow(), err_message.red());
            exit(-1)
        }
    };
    app.run::<TopView>(bus);
}
