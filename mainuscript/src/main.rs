use relm4::{RELM_BLOCKING_THREADS, RELM_THREADS, RelmApp, gtk::{gdk::set_allowed_backends}};
use ui::top::TopView;
use colored::Colorize;
use std::{process::exit, time::Duration};

use std::sync::Arc;
use std::sync::RwLock;

mod ui{
    pub mod top;
    pub mod navigation;
    pub mod logo;
    pub mod work_area{
        pub mod settings;
        pub mod regular_chat_view;
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
    let bus_shared = Arc::new(RwLock::new(bus));
    app.run::<TopView>(bus_shared.clone());
    match Arc::into_inner(bus_shared){
        Some(bus_rwlock) => {
            match bus_rwlock.into_inner(){
                Ok(bus) => {
                    bus.close();
                },
                Err(err) => {
                    println!("Bus got poisoned!\n{}", err.to_string());
                }
            }
        },
        None => {
            println!("Unexpected error happened when attempting to get bus shared refference to wrap things up")
        }
    }
}
