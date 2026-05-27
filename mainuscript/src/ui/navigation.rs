use std::sync::Arc;

use relm4::gtk::prelude::{BoxExt, WidgetExt};
use relm4::{gtk::prelude::ButtonExt, prelude::*};
use gtk::{Box as GtkBox, Button, Separator, Label, Orientation};
use gtk::glib::clone;
use crate::bus::Bus;
use crate::ui::logo::LogoView;

pub const NAVIGATION_ELEMENT_SPACING: i32 = 5;
pub struct NavigationWidgets{

}
pub struct NavigationView{
    bus: Arc<Bus>
}
#[derive(Debug)]
pub enum NavigationEvent{
    Settings,
    Chat
}
impl SimpleComponent for NavigationView{
    type Input = ();
    type Output = NavigationEvent;
    type Init = Arc<Bus>;
    type Root = GtkBox;
    type Widgets = ();
    fn init_root() -> Self::Root {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .width_request(300)
            .spacing(NAVIGATION_ELEMENT_SPACING)
            .vexpand(true)
            .build();
        root.add_css_class("navigation-view");
        return root;
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let logo = LogoView::builder().launch(());
        let settings_button = gtk::Button::builder()
            .label("Settings")
            .build();
        settings_button.connect_clicked(
            clone!(
                #[strong] sender,
                move |_| {
                    _ = sender.output(NavigationEvent::Settings);
                }
            )
        );
        let separator = Separator::new(gtk::Orientation::Horizontal);
        separator.add_css_class("settings-chats-separator");

        // ObjectSelector
        let object_selector_container = GtkBox::new(Orientation::Vertical, NAVIGATION_ELEMENT_SPACING);
        populate_object_selector_container(&object_selector_container, &init);


        // Assembling root container
        root.append(logo.widget());
        root.append(&settings_button);
        root.append(&separator);
        ComponentParts { model: NavigationView {bus: init}, widgets: () }
    }
}
fn populate_object_selector_container(container: &GtkBox, bus: &Bus){
    let object_list = bus.storage.get_object_list(&bus.config);
    todo!()
}