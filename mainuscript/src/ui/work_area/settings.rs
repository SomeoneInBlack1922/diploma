use relm4::{gtk::prelude::{BoxExt, WidgetExt}, prelude::*};
use gtk::{Box as GtkBox, Label};
pub struct SettingsView;

pub const SETTINGS_ELEMENT_SPACING: i32 = 10;

impl SimpleComponent for SettingsView {
    type Input = ();
    type Output = ();
    type Init = ();
    type Root = GtkBox;
    type Widgets = ();
    fn init_root() -> Self::Root {
        let root = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .build();
        root.add_css_class("settings-root");
        root
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let settings_temp_label = Label::new(Some("SETTINGS"));
        root.append(&settings_temp_label);
        ComponentParts { model: SettingsView{}, widgets: () }
    }
}