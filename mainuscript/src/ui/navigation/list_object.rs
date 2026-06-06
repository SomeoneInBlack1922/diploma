use std::sync::Arc;
use std::sync::RwLock;

use relm4::ComponentSender;
use relm4::gtk;
use gtk::{Button as GtkButton, Label as GtkLabel, ScrolledWindow, NoSelection, prelude::{ButtonExt, WidgetExt}};
use relm4::typed_view::list::RelmListItem;
use relm4::typed_view::list::TypedListView;

use crate::bus::Bus;
use crate::object::{FsObject, ObjectType};
use crate::ui::navigation::NavigationInput;
use crate::ui::navigation::{NavigationView, NavigationOutput};
use crate::storage::Storage;
pub type SharedClickFunction = Arc<RwLock<fn(&GtkButton, ComponentSender<NavigationView>, Arc<FsObject>)>>;
pub struct NavigationObjectButton{
    pub object: Arc<FsObject>,
    pub sender: ComponentSender<NavigationView>,
    pub click_function: SharedClickFunction
}
impl NavigationObjectButton{
    pub fn new(object: FsObject, sender: ComponentSender<NavigationView>, click_function: SharedClickFunction) -> Self{
        return NavigationObjectButton{object: Arc::new(object), sender, click_function};
    }
}
impl RelmListItem for NavigationObjectButton{
    type Root = GtkButton;
    type Widgets = ();
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let object_button = GtkButton::builder()
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
        let fs_object_clone = self.object.clone();
        let click_fucntion_ref = self.click_function.clone();
        _root.connect_clicked(move |button| {
            click_fucntion_ref.read().unwrap()(button, sender_clone.clone(), fs_object_clone.clone())
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
pub fn populate_object_view_wrapper(
    container: &ScrolledWindow,
    wrapper: &mut TypedListView<NavigationObjectButton, NoSelection>,
    bus: &Bus,
    sender: ComponentSender<NavigationView>,
    click_function: SharedClickFunction
){
    // Get object list or generate empty on failure
    let mut object_list = bus.storage.get_object_list().unwrap_or(vec![]);
    Storage::sort_object_list(&mut object_list);
    if object_list.is_empty(){
        container.set_child(
            Some(&GtkLabel::new(Some("There are no chats")))
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
            wrapper.append(NavigationObjectButton::new(fs_object, sender.clone(), click_function.clone()));
        }
        container.set_child(Some(&wrapper.view));
    }
}
// Click functions
pub fn open_object_click(button: &GtkButton, sender: ComponentSender<NavigationView>, fs_object: Arc<FsObject>){
    sender.output(NavigationOutput::Object(fs_object));
}
pub fn remove_object_click(button: &GtkButton, sender: ComponentSender<NavigationView>, fs_object: Arc<FsObject>){
    sender.input(NavigationInput::RemoveThisObject(fs_object.name.clone()));
}