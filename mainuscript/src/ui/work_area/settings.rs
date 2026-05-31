use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}, rc::Rc, sync::{Arc, RwLock}};

use relm4::{Sender, gtk::{BoolFilter, Button, Orientation::Horizontal, ScrolledWindow, glib::{SendWeakRef, StrV, object::ObjectExt, property::PropertyGet}, prelude::{BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, WidgetExt}}, prelude::*};
use gtk::{Box as GtkBox, Label, Entry};
use url::Url;

use borsh::{BorshSerialize, BorshDeserialize};

use crate::{bus::Bus, ui::work_area::settings::selection::{API_KEY, API_URL, Selection, SelectionFlag, SelectionTrait}};
mod selection;
mod set;
fn set_state_label_empty(label: &Label){
    label.set_text("");
    label.set_css_classes(&[]);
}
fn set_state_label_changed(label: &Label){
    label.set_text("Changed");
    label.set_css_classes(&["settings-popup-changed"]);
}
pub struct SettingsView{
    bus: Arc<RwLock<Bus>>,
    widgets: SettingsWidgets,
    changed_options: Arc<RwLock<Selection>>, // All options whose value have changed
    error_options: Arc<RwLock<Selection>>, // All options that contain error
    highlighted_options: Arc<RwLock<Selection>> // All options that are highlighted
}
#[derive(Default)]
#[derive(Debug)]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct OptionsData {
    inner: HashMap<SelectionFlag, OptionsDataKind>
}
impl OptionsData {
    pub fn get_previous_text(&self, flag: SelectionFlag) -> String{
        let ret = match self.get(&flag){
            Some(data_kind) => {
                match data_kind{
                    OptionsDataKind::String(data) => data
                }
            },
            None => &"".into()
        };
        return ret.clone();
    }
}
impl Deref for OptionsData{
    type Target = HashMap<SelectionFlag, OptionsDataKind>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for OptionsData{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
#[derive(Debug)]
#[derive(BorshSerialize, BorshDeserialize)]
pub enum OptionsDataKind{
    String(String)
}
pub struct SettingsWidgets{
    save_button: Button,
    clear_button: Button,
    option_widgets: OptionWidgets,
    scroll: ScrolledWindow,
    root: GtkBox,
}
pub struct OptionWidgets{
    api_url_entry: Entry,
    api_key_entry: Entry,
}


#[derive(Debug)]
pub enum SettingsInput{
    SetOptionAsChanged(Selection),
    SetOptionAsNotChanged(Selection),
    ClearAllOptions //Basically a signal to disable clear_button
}
pub struct SettingsInit{
    pub bus: Arc<RwLock<Bus>>,
    pub highlighted_options: Arc<RwLock<Selection>>,
    pub init_control_message: String
}
pub const SETTINGS_VERTICAL_SPACING: i32 = 10;
pub const SETTINGS_HORIZONTAL_SPACING: i32 = 10;

impl SimpleComponent for SettingsView {
    type Input = SettingsInput;
    type Output = ();
    type Init = SettingsInit;
    /// Top wrapper that allows to scroll
    type Root = GtkBox;
    /// Root box that holds the ui
    type Widgets = ();
    fn init_root() -> Self::Root {
        let root = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .css_classes(["settings-root"])
            .build();
        return root
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let bus = init.bus.read().unwrap();

        let control_container = GtkBox::builder()
            .hexpand(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(SETTINGS_HORIZONTAL_SPACING)
            .css_classes(["settings-control-container"])
            .build();

        let save_button = Button::builder()
            .label("Save")
            .sensitive(false)
            .css_classes(["settings-save-button"])
            .build();

        let clear_button = Button::builder()
            .label("Clear")
            .sensitive(false)
            .css_classes(["settings-clear-button"])
            .build();

        let control_label = Label::new(Some(&init.init_control_message[..]));

        control_container.append(&save_button);
        control_container.append(&clear_button);
        control_container.append(&control_label);

        let scroll = scroll();

        let changed_options = Arc::new(RwLock::new(0));
        let error_options = Arc::new(RwLock::new(0));
        let highlighted_options = init.highlighted_options;

        let option_widgets = create_entry_widgets(
            scroll.clone(),
            bus.options_data.clone(), 
            sender.clone(), 
            highlighted_options.clone()
            // init.init_control_message
        );

        // Implement clear_button
        {
            // let entries_box_clone = entries_box.clone();
            // let settings_data_clone = bus.settings_data.clone();
            let sender_clone = sender.clone();
            // let save_button_clone = save_button.clone();
            // let clear_button_clone = clear_button.clone();
            // let changed_options_clone = changed_options.clone();
            // let error_options_clone = error_options.clone();
            // let highlighted_options_clone = highlighted_options.clone();

            // let control_container = 
            clear_button.connect_clicked(move |_|{
                sender.input(SettingsInput::ClearAllOptions);
            });
        }
        
        
        

        root.append(&control_container);
        root.append(&scroll);
        drop(bus);
        ComponentParts {
            model: SettingsView{
                bus: init.bus,
                widgets: SettingsWidgets {
                    save_button,
                    clear_button,
                    option_widgets,
                    scroll,
                    root
                },
                changed_options,
                error_options,
                highlighted_options,
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message{
            SettingsInput::SetOptionAsChanged(flag) => {
                self.widgets.clear_button.set_sensitive(true);
                self.changed_options.write().unwrap().set(flag);
            },
            SettingsInput::SetOptionAsNotChanged(flag) => {
                let mut changed_options_ref = self.changed_options.write().unwrap();
                changed_options_ref.clear(flag);
                // If no changed settings remain
                if *changed_options_ref == 0{
                    self.widgets.clear_button.set_sensitive(false);
                }
            },
            // Basically just disable clear_button
            SettingsInput::ClearAllOptions => {
                // self.widgets.clear_button.set_sensitive(false);
                self.widgets.root.remove(&self.widgets.scroll);
                let new_scroll = scroll();
                let bus_ref = self.bus.read().unwrap();
                let new_option_widgets = create_entry_widgets(
                    new_scroll.clone(),
                    bus_ref.options_data.clone(),
                    sender,
                    self.highlighted_options.clone()
                );
                self.widgets.option_widgets = new_option_widgets;
                self.widgets.root.append(&new_scroll);
                self.widgets.scroll = new_scroll;
                self.widgets.clear_button.set_sensitive(false);
            }
        }
    }
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        let mut mut_bus = match self.bus.write(){
            Ok(lock) => lock,
            Err(_) => return
        };
        let mut mut_settings_data = match mut_bus.options_data.write(){
            Ok(lock) => lock,
            Err(_) => return
        };
        mut_settings_data.insert(API_URL, OptionsDataKind::String(self.widgets.option_widgets.api_url_entry.text().to_string()));
        mut_settings_data.insert(API_KEY, OptionsDataKind::String(self.widgets.option_widgets.api_key_entry.text().to_string()));
    }
}
fn create_entry_widgets(
        scroll: ScrolledWindow,
        options_data: Arc<RwLock<OptionsData>>,
        sender: ComponentSender<SettingsView>,
        highlighted_options: Arc<RwLock<Selection>>,
        // init_control_message: String
    ) -> OptionWidgets{
    
    let options_data_ref = options_data.read().unwrap();
    let highlighted_options_ref = highlighted_options.write().unwrap();

    let container = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(SETTINGS_VERTICAL_SPACING)
            .vexpand(true)
            .hexpand(true)
            .css_classes(["settings-entries-container"])
            .build();
    // +++++++++++++++++++
    // Connection Settings
    // +++++++++++++++++++
    let connection_settings_label = Label::builder()
        .label("OpenAPI connection")
        .halign(gtk::Align::Fill)
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["settings-section-label"])
        .build();
    // connection_settings_label.set_justify(gtk::Justification::Left);
    container.append(&connection_settings_label);


    // Row 1
    let api_url_entry: Entry;
    {
        let api_url_row = GtkBox::builder()
            .orientation(Horizontal)
            .spacing(SETTINGS_HORIZONTAL_SPACING)
            .css_classes(["settings-row"])
            .build();
        if highlighted_options_ref.test(selection::API_URL){
            api_url_row.set_css_classes(&["settings-row-highlighted"]);
        }

        let api_url_label = Label::new(Some("API URL"));

        let url_previous_text: String = options_data_ref.get_previous_text(API_URL);
        api_url_entry = Entry::builder()
            .text(url_previous_text)
            .hexpand(true)
            .build();

        let api_url_state = Label::new(None);

        let api_url_popup = Label::new(None);
        api_url_popup.add_css_class("settings-popup");

        // Follow when data in entry changes
        watch_entry_option(
            &api_url_entry,
            api_url_state.clone(),
            sender.clone(),
            options_data.clone(),
            API_URL
        );
        
        api_url_entry.emit_by_name::<()>("changed", &[]);

        api_url_row.append(&api_url_label);
        api_url_row.append(&api_url_entry);
        api_url_row.append(&api_url_state);
        container.append(&api_url_row);
        container.append(&api_url_popup);
    }
    


    // Row 2
    let api_key_row = GtkBox::builder()
            .orientation(Horizontal)
            .spacing(SETTINGS_HORIZONTAL_SPACING)
            .css_classes(["settings-row"])
            .build();
    if highlighted_options_ref.test(selection::API_KEY){
        api_key_row.set_css_classes(&["settings-row-highlighted"]);
    }

    let api_key_label = Label::new(Some("API Key"));

    let key_previous_text = options_data_ref.get_previous_text(API_KEY);
    let api_key_entry = Entry::builder()
            .text(key_previous_text)
            .hexpand(true)
            .build();

    let api_key_state = Label::new(None);

    let api_key_popup = Label::new(None);
    api_key_popup.add_css_class("settings-popup");

    // Follow when data in entry changes
    watch_entry_option(
        &api_key_entry,
        api_key_state.clone(),
        sender.clone(),
        options_data.clone(),
        API_KEY
    );
    
    api_key_entry.emit_by_name::<()>("changed", &[]);

    api_key_row.append(&api_key_label);
    api_key_row.append(&api_key_entry);
    container.append(&api_key_row);

    scroll.set_child(Some(&container));
    return OptionWidgets { api_url_entry, api_key_entry };
}
fn watch_entry_option(entry: &Entry, state_label: Label, sender: ComponentSender<SettingsView>,  settings_data: Arc<RwLock<OptionsData>>, flag: SelectionFlag) {
    entry.connect_changed(move |entry|{
        let settings_data_ref = settings_data.read().unwrap();
        let Some(previous_text_wrapped) = settings_data_ref.get(&flag) else {return;};
        let OptionsDataKind::String(previous_text) = previous_text_wrapped else {return;};
        if *previous_text == entry.text().to_string(){
            set_state_label_empty(&state_label);
            sender.input(SettingsInput::SetOptionAsNotChanged(API_URL));
        }
        else{
            set_state_label_changed(&state_label);
            sender.input(SettingsInput::SetOptionAsChanged(API_URL));
        }
    });
    
}
fn scroll() -> ScrolledWindow{
    ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build()
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