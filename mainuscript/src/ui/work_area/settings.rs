use std::sync::Arc;

use relm4::{gtk::{ScrolledWindow, glib::StrV, prelude::{BoxExt, EditableExt, EntryBufferExtManual, EntryExt, WidgetExt}}, prelude::*};
use gtk::{Box as GtkBox, Label, Entry};
use url::Url;

use crate::bus::Bus;
pub struct SettingsView{
    bus: Arc<Bus>
}

pub const SETTINGS_VERTICAL_SPACING: i32 = 10;
pub const SETTINGS_HORIZONTAL_SPACING: i32 = 10;

impl SimpleComponent for SettingsView {
    type Input = ();
    type Output = ();
    type Init = Arc<Bus>;
    /// Top wrapper that allows to scroll
    type Root = ScrolledWindow;
    /// Root box that holds the ui
    type Widgets = GtkBox;
    fn init_root() -> Self::Root {
        let root = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        // root.set_child(Some(&root_box));
        return root
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let bus = init;
        let root_box = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(SETTINGS_VERTICAL_SPACING)
            .vexpand(true)
            .hexpand(true)
            .build();
        root_box.add_css_class("settings-root-box");
        // Connection Settings
        let connection_settings_label = Label::builder()
            .label("OpenAPI connection")
            .halign(gtk::Align::Fill)
            .xalign(0.0)
            .hexpand(true)
            .build();
        connection_settings_label.add_css_class("settings-section-label");
        // connection_settings_label.set_justify(gtk::Justification::Left);

        root_box.append(&connection_settings_label);


        // Row 1
        let api_url_row = GtkBox::new(gtk::Orientation::Horizontal, SETTINGS_HORIZONTAL_SPACING);
        api_url_row.add_css_class("settings-row");

        let api_url_label = Label::new(Some("API URL"));
        let api_url_entry_box = GtkBox::builder()
            .spacing(0)
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .build();
        let api_url_entry = Entry::builder()
            .hexpand(true)
            .build();
        let api_url_popup = Label::new(None);
        api_url_popup.add_css_class("settings-popup");

        // Validation and read
        let bus_clone = bus.clone();
        let api_url_popup_clone = api_url_popup.clone();
        api_url_entry.connect_changed(move |entry|{
            let text = entry.buffer().text().to_string();
            if text.is_empty(){
                api_url_popup_clone.set_label("");
                return;
            }
            let url = match Url::parse(&text){
                Ok(url) => {url},
                Err(err) => {
                    api_url_popup_clone.set_label(&format!("Error: {}", err.to_string())[..]);
                    return;
                }
            };

        });

        api_url_entry_box.append(&api_url_entry);
        api_url_entry_box.append(&api_url_popup);
        api_url_row.append(&api_url_label);
        api_url_row.append(&api_url_entry_box);
        root_box.append(&api_url_row);


        // Row 2
        let api_key_row = GtkBox::new(gtk::Orientation::Horizontal, SETTINGS_HORIZONTAL_SPACING);
        api_key_row.add_css_class("settings-row");

        let api_key_label = Label::new(Some("API Key"));
        let api_key_entry = Entry::builder()
            .hexpand(true)
            .build();

        api_key_row.append(&api_key_label);
        api_key_row.append(&api_key_entry);
        root_box.append(&api_key_row);
        
        
        root.set_child(Some(&root_box));
        ComponentParts { model: SettingsView{bus}, widgets: root_box }
    }
}