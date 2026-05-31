
use std::sync::{Arc, RwLock};
use relm4::gtk::{ListItemFactory, NoSelection, ScrolledWindow};
use relm4::gtk::prelude::{BoxExt, WidgetExt};
use relm4::typed_view::list::{RelmListItem, TypedListView};
use relm4::{gtk::prelude::ButtonExt, prelude::*};
use gtk::{Box as GtkBox, Button, Separator, Label};
use gtk::glib::clone;
use crate::bus::Bus;
use crate::object::{self, FsObject};
use crate::storage::Storage;
use crate::ui::logo::LogoView;

pub const NAVIGATION_ELEMENT_SPACING: i32 = 5;
pub struct NavigationView{
    bus: Arc<RwLock<Bus>>
}
#[derive(Debug)]
pub enum NavigationEvent{
    Settings,
    Object(Arc<FsObject>)
}
impl SimpleComponent for NavigationView{
    type Input = ();
    type Output = NavigationEvent;
    type Init = Arc<RwLock<Bus>>;
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
        settings_button.add_css_class("navigation-settings");
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
        // let object_selector_container_scroll = ScrolledWindow::builder()
        //     .vexpand(true)
        //     .hexpand(false)
        //     .build();
        // let object_selector_container = GtkBox::builder()
        //     .orientation(Orientation::Vertical)
        //     .spacing(0)
        //     .vexpand(true)
        //     .hexpand(false)
        //     .build();
        // populate_object_selector_container(&object_selector_container, &init);
        // object_selector_container_scroll.set_child(Some(&object_selector_container));
        let scrollable_container = ScrolledWindow::builder()
            .vexpand(true)
            .build();
        let mut object_view_wrapper: TypedListView<NavigationObjectButton, NoSelection> = TypedListView::new();
        object_view_wrapper.view.add_css_class("navigation-list-view");
        populate_object_view_wrapper(&scrollable_container, &mut object_view_wrapper, &init.read().unwrap(), &sender);


        // Assembling root container
        root.append(logo.widget());
        root.append(&settings_button);
        root.append(&separator);
        root.append(&scrollable_container);
        ComponentParts { model: NavigationView {bus: init}, widgets: () }
    }
}
pub struct NavigationObjectButton{
    pub object: Arc<FsObject>,
    pub sender: ComponentSender<NavigationView>
}
impl NavigationObjectButton{
    fn new(object: FsObject, sender: ComponentSender<NavigationView>) -> Self{
        return NavigationObjectButton{object: Arc::new(object), sender};
    }
}
impl RelmListItem for NavigationObjectButton{
    type Root = Button;
    type Widgets = ();
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let object_button = Button::builder()
                .hexpand(true)
                .height_request(20)
                // .margin_bottom(0)
                // .margin_top(0)
                // .margin_start(0)
                // .margin_end(0)
                // .label()
                .build();
        return (object_button, ())
    }
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        _root.set_label(&self.object.name);
        let sender_clone = self.sender.clone();
        let fs_object_ref = self.object.clone();
        _root.connect_clicked(move |_| {
            sender_clone.output(NavigationEvent::Object(fs_object_ref.clone()));
        });
        match self.object.object_type{
            object::ObjectType::RegularChat => {
                _root.add_css_class("navigation-object-regular-chat");
            },
            object::ObjectType::TextScript => {
                _root.add_css_class("navigation-object-text-script");
            }
        }
    }
}
fn populate_object_view_wrapper(container: &ScrolledWindow, wrapper: &mut TypedListView<NavigationObjectButton, NoSelection>, bus: &Bus, sender: &ComponentSender<NavigationView>){
    // Get object list or generate empty on failure
    let mut object_list = bus.storage.get_object_list().unwrap_or(vec![]);
    Storage::sort_object_list(&mut object_list);
    if object_list.is_empty(){
        container.set_child(
            Some(&Label::new(Some("There are no chats")))
        );
        container.remove_css_class("navigation-container");
        container.add_css_class("navigation-empty-container");
    }
    else{
        container.remove_css_class("navigation-empty-container");
        container.add_css_class("navigation-container");
        for fs_object in object_list{
            // let object_button = Button::builder()
            //     .hexpand(true)
            //     .height_request(20)
            //     .label(&*fs_object.name)
            //     .build();
            // match fs_object.object_type{
            //     object::ObjectType::RegularChat => {
            //         object_button.add_css_class("navigation-object-regular-chat");
            //     },
            //     object::ObjectType::TextScript => {
            //         object_button.add_css_class("navigation-object-text-script");
            //     }
            // }
            wrapper.append(NavigationObjectButton::new(fs_object, sender.clone()));
        }
        container.set_child(Some(&wrapper.view));
    }
}