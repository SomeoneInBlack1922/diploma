use relm4::{RelmApp, gtk::FilterMatch::Some};
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

use config::*;

use crate::bus::Bus;
fn main() {
    let app = RelmApp::new("none.none.mainuscript");
    let bus = match Bus::init(){
        Ok(bus) => {bus},
        Err(err_message) => {
            println!("{}\n{}", "Failed to initiate a bus".yellow(), err_message.red());
            exit(-1)
        }
    };
    app.run::<TopView>(bus);
}
