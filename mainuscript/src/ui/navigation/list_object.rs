use std::sync::Arc;

use relm4::ComponentSender;
use relm4::gtk;
use gtk::{Button, Label, ScrolledWindow, NoSelection, prelude::{ButtonExt, WidgetExt}};
use relm4::typed_view::list::RelmListItem;
use relm4::typed_view::list::TypedListView;

use crate::bus::Bus;
use crate::object::{FsObject, ObjectType};
use crate::ui::navigation::{NavigationView, NavigationOutput};
use crate::storage::Storage;
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
            sender_clone.output(NavigationOutput::Object(fs_object_ref.clone()));
        });
        match self.object.object_type{
            ObjectType::RegularChat => {
                _root.add_css_class("navigation-object-regular-chat");
            },
            ObjectType::TextScript => {
                _root.add_css_class("navigation-object-text-script");
            }
        }
    }
}
pub fn populate_object_view_wrapper(container: &ScrolledWindow, wrapper: &mut TypedListView<NavigationObjectButton, NoSelection>, bus: &Bus, sender: ComponentSender<NavigationView>){
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