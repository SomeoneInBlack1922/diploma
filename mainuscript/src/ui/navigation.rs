
use std::sync::Arc;
use relm4::gtk::prelude::{BoxExt, WidgetExt};
use relm4::{gtk::prelude::ButtonExt, prelude::*};
use gtk::{Box as GtkBox, Button, Separator, Label, Orientation, ScrolledWindow};
use gtk::glib::clone;
use crate::bus::Bus;
use crate::object::{self, FsObject};
use crate::storage::Storage;
use crate::ui::logo::LogoView;

pub const NAVIGATION_ELEMENT_SPACING: i32 = 5;
pub struct NavigationView{
    bus: Arc<Bus>
}
#[derive(Debug)]
pub enum NavigationEvent{
    Settings,
    Object(FsObject)
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
        let object_selector_container_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(false)
            .build();
        let object_selector_container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(false)
            .build();
        populate_object_selector_container(&object_selector_container, &init);
        object_selector_container_scroll.set_child(Some(&object_selector_container));


        // Assembling root container
        root.append(logo.widget());
        root.append(&settings_button);
        root.append(&separator);
        root.append(&object_selector_container_scroll);
        ComponentParts { model: NavigationView {bus: init}, widgets: () }
    }
}
fn populate_object_selector_container(container: &GtkBox, bus: &Bus){
    // Get object list or generate empty on failure
    let mut object_list = bus.storage.get_object_list(&bus.config).unwrap_or(vec![]);
    Storage::sort_object_list(&mut object_list);
    if object_list.is_empty(){
        container.append(
            &Label::new(Some("There are no chats"))
        );
        container.remove_css_class("navigation-container");
        container.add_css_class("navigation-empty-container");
    }
    else{
        container.remove_css_class("navigation-empty-container");
        container.add_css_class("navigation-container");
        for fs_object in object_list{
            let object_button = Button::builder()
                .hexpand(true)
                .height_request(20)
                .label(&*fs_object.name)
                .build();
            match fs_object.object_type{
                object::ObjectType::RegularChat => {
                    object_button.add_css_class("navigation-object-regular-chat");
                },
                object::ObjectType::TextScript => {
                    object_button.add_css_class("navigation-object-text-script");
                }
            }
            container.append(&object_button);
        }
    }
}