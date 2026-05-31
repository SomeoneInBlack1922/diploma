use std::{cell::RefCell, ops::Deref, rc::Rc, sync::{Arc, RwLock}};

use relm4::{gtk::{BoolFilter, Button, ScrolledWindow, glib::{SendWeakRef, StrV, object::ObjectExt}, prelude::{BoxExt, EditableExt, EntryBufferExtManual, EntryExt, WidgetExt}}, prelude::*};
use gtk::{Box as GtkBox, Label, Entry};
use url::Url;

use borsh::{BorshSerialize, BorshDeserialize};

use crate::bus::Bus;

#[derive(Debug)]
#[derive(BorshSerialize, BorshDeserialize)]
pub enum SettingsFieldState{
    Set,
    Ok,
    Err,
    Empty
}
impl std::fmt::Display for SettingsFieldState{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Ok => {
                write!(f, "{}", "Ok");
                return Ok(());
            },
            Self::Set => {
                write!(f, "{}", "Set");
                return Ok(());
            },
            Self::Err => {
                write!(f, "{}", "Error")
            },
            Self::Empty => {
                write!(f, "{}", "Empty")
            }
        }
    }
}
pub struct SettingsView{
    bus: Arc<RwLock<Bus>>
}

#[derive(Debug)]
#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Default)]
pub struct SettingsData{
    api_url_text: String,
    api_key_text: String
}
pub struct SettingsWidgets{
    api_url_entry: Entry,
    api_key_entry: Entry
}
pub const SETTINGS_VERTICAL_SPACING: i32 = 10;
pub const SETTINGS_HORIZONTAL_SPACING: i32 = 10;

impl SimpleComponent for SettingsView {
    type Input = ();
    type Output = ();
    type Init = Arc<RwLock<Bus>>;
    /// Top wrapper that allows to scroll
    type Root = GtkBox;
    /// Root box that holds the ui
    type Widgets = SettingsWidgets;
    fn init_root() -> Self::Root {
        let root = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .build();
        root.add_css_class("settings-root");
        return root
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let bus = init.read().unwrap();

        let control_container = GtkBox::builder()
            .hexpand(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(SETTINGS_HORIZONTAL_SPACING)
            .build();
        control_container.add_css_class("settings-control-container");

        let save_button = Button::builder()
            .label("Save")
            .sensitive(false)
            .build();
        save_button.add_css_class("settings-save-button");

        let clear_button = Button::builder()
            .label("Clear")
            .sensitive(false)
            .build();
        clear_button.add_css_class("settings-clear-button");

        let state_label = Label::new(None);

        control_container.append(&save_button);
        control_container.append(&clear_button);
        control_container.append(&state_label);

        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let entries_box = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(SETTINGS_VERTICAL_SPACING)
            .vexpand(true)
            .hexpand(true)
            .build();
        entries_box.add_css_class("settings-entries-container");

        let widgets = create_entry_widgets(&entries_box, &bus.settings_data);
        
        scroll.set_child(Some(&entries_box));

        root.append(&control_container);
        root.append(&scroll);
        drop(bus);
        ComponentParts {
            model: SettingsView{
                bus: init
            },
            widgets
        }
    }
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        let mut mut_bus = match self.bus.write(){
            Ok(lock) => lock,
            Err(_) => return
        };
        mut_bus.settings_data.api_url_text = widgets.api_url_entry.text().to_string();
    }
}
fn create_entry_widgets(container: &GtkBox, settings_data: &SettingsData) -> SettingsWidgets{
    // +++++++++++++++++++
    // Connection Settings
    // +++++++++++++++++++
    let connection_settings_label = Label::builder()
        .label("OpenAPI connection")
        .halign(gtk::Align::Fill)
        .xalign(0.0)
        .hexpand(true)
        .build();
    connection_settings_label.add_css_class("settings-section-label");
    // connection_settings_label.set_justify(gtk::Justification::Left);
    container.append(&connection_settings_label);


    // Row 1
    let api_url_entry: Entry;
    {
        let api_url_row = GtkBox::new(gtk::Orientation::Horizontal, SETTINGS_HORIZONTAL_SPACING);
        api_url_row.add_css_class("settings-row");

        let api_url_label = Label::new(Some("API URL"));

        api_url_entry = Entry::builder()
            .text(&settings_data.api_url_text)
            .hexpand(true)
            .build();

        let api_url_state = Label::new(Some(&SettingsFieldState::Empty.to_string()));

        let api_url_popup = Label::new(None);
        api_url_popup.add_css_class("settings-popup");

        // Validation on change
        {
            let api_url_popup_clone = api_url_popup.clone();
            let api_url_state_clone = api_url_state.clone();
            api_url_entry.connect_changed(move |entry|{
                if let Err((text, state)) = validate_api_url(entry.text().to_string()){
                    api_url_popup_clone.set_label(&text);
                    api_url_state_clone.set_label(&state.to_string());
                }
                else{
                    api_url_popup_clone.set_label(&"");
                    api_url_state_clone.set_label(&SettingsFieldState::Ok.to_string());
                }
            });
        }
        
        api_url_entry.emit_by_name::<()>("changed", &[]);

        api_url_row.append(&api_url_label);
        api_url_row.append(&api_url_entry);
        api_url_row.append(&api_url_state);
        container.append(&api_url_row);
        container.append(&api_url_popup);
    }
    


    // Row 2
    let api_key_row = GtkBox::new(gtk::Orientation::Horizontal, SETTINGS_HORIZONTAL_SPACING);
    api_key_row.add_css_class("settings-row");

    let api_key_label = Label::new(Some("API Key"));
    let api_key_entry = Entry::builder()
        .hexpand(true)
        .build();

    api_key_row.append(&api_key_label);
    api_key_row.append(&api_key_entry);
    container.append(&api_key_row);

    return SettingsWidgets {
        api_url_entry,
        api_key_entry
    }
}
fn validate_api_url(text: String) -> Result<Url, (String, SettingsFieldState)>{
        if text.is_empty(){
            return Err(("".into(), SettingsFieldState::Empty));
        }
        match Url::parse(&text){
            Ok(url) => {
                return Ok(url)
            },
            Err(err) => {
                return Err((
                    format!("Error: {}", err.to_string()),
                    SettingsFieldState::Err
                ));
            }
        };
}