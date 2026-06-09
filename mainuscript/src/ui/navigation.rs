
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use relm4::component::Connector;
use relm4::gtk::glib::SignalHandlerId;
use relm4::gtk::glib::object::ObjectExt;
use relm4::gtk::pango::BidiType::L;
use relm4::gtk::{ListItemFactory, NoSelection, ScrolledWindow};
use relm4::gtk::prelude::{BoxExt, EditableExt, WidgetExt};
use relm4::typed_view::list::{RelmListItem, TypedListView};
use relm4::{gtk::prelude::ButtonExt, prelude::*};
use gtk::{Box as GtkBox, Button as GtkButton, Separator, Label as GtkLabel, Entry as GtkEntry};
use gtk::glib::clone;
use crate::bus::Bus;
use crate::helper_types::NameString;
use crate::object::regular_chat::RegularChat;
use crate::object::{self, FsObject};
use crate::storage::Storage;
use crate::ui::logo::LogoView;
use crate::ui::navigation::list_object::{NavigationObjectButton, SharedClickFunction, open_object_click, populate_object_view_wrapper, remove_object_click};
use crate::ui::navigation::model_button::{ModelButton, ModelButtonOutput};
use crate::ui::navigation::model_list::ModelList;

mod list_object;
mod model_button;
mod model_list;

pub const NAVIGATION_ELEMENT_SPACING: i32 = 5;
pub const NAVIGATION_WIDTH: i32 = 500;
pub struct NavigationView{
    bus: Arc<RwLock<Bus>>,
    list_place: GtkBox,
    objects_container: ScrolledWindow,
    model_list_button: GtkButton,
    model_list_button_handler: SignalHandlerId,
    model_list_controller: Option<Controller<ModelList>>,
    object_click_function: SharedClickFunction,
    object_list: TypedListView<NavigationObjectButton, NoSelection>,
    new_ojbect_button: GtkButton,
    new_ojbect_button_handler: SignalHandlerId,
    delete_object_button: GtkButton,
    delete_ojbect_button_handler: SignalHandlerId,
    root: GtkBox,
    new_object_widgets: Option<(GtkEntry, GtkLabel)>,
    new_object_entry_place: GtkBox
}
#[derive(Debug)]
pub enum NavigationOutput{
    Settings,
    Object(Arc<FsObject>)
}
#[derive(Debug)]
pub enum NavigationInput{
    ModelSelected(String),
    OpenModeList,
    CloseModelList,
    StartObjectRemoval,
    StopObjectRemoval,
    StartObjectAddition,
    StopObjectAddition,
    TryCommitAddition,
    RemoveThisObject(NameString)
}
impl SimpleComponent for NavigationView{
    type Input = NavigationInput;
    type Output = NavigationOutput;
    type Init = Arc<RwLock<Bus>>;
    type Root = GtkBox;
    type Widgets = ();
    fn init_root() -> Self::Root {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .width_request(NAVIGATION_WIDTH)
            .spacing(NAVIGATION_ELEMENT_SPACING)
            .vexpand(true)
            .hexpand(false)
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
        let bus_clone = init.clone();
        let bus_ref = bus_clone.read().unwrap();
        let logo = LogoView::builder().launch(());
        let settings_button = gtk::Button::builder()
            .label("Settings")
            .build();
        settings_button.add_css_class("navigation-settings");
        settings_button.connect_clicked(
            clone!(
                #[strong] sender,
                move |_| {
                    _ = sender.output(NavigationOutput::Settings);
                }
            )
        );

        // let model_button_builder = ModelButton::builder();

        

        let model_list_button = GtkButton::builder()
            .label("ListModels")
            .css_classes(["navigation-model-select-button-empty"])
            .build();
        let selected_model = bus_ref.config.selected_model_name.get_value_rw_lock().read().unwrap();
        if let Some(name) = &*selected_model {
            model_list_button.set_css_classes(&["navigation-model-select-button-selected"]);
            model_list_button.set_label(name);
        }
        let sender_clone = sender.clone();
        let model_list_button_handler = model_list_button.connect_clicked(move |_|{
            sender_clone.input(NavigationInput::OpenModeList);
        });

        drop(&bus_ref);

        let chat_control_button_container = GtkBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(0)
            // .width_request(NAVIGATION_WIDTH)
            .hexpand(true)
            .css_classes(["navigation-object-control-container"])
            .build();

        let new_object_entry_place = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .build();
        let control_label = GtkLabel::new(Some("Chat:"));
        let new_ojbect_button = GtkButton::builder()
            .label("New")
            .hexpand(true)
            .css_classes(["navigation-object-control-new-button"])
            .build();
        let delete_object_button =  GtkButton::builder()
            .label("Delete")
            .hexpand(true)
            .css_classes(["navigation-object-control-delete-button"])
            .build();
        let sender_clone = sender.clone();
        let new_ojbect_button_handler = new_ojbect_button.connect_clicked(move |_|{
            sender_clone.input(NavigationInput::StartObjectAddition);
        });
        let sender_clone = sender.clone();
        let delete_ojbect_button_handler= delete_object_button.connect_clicked(move|_|{
            sender_clone.input(NavigationInput::StartObjectRemoval);
        });

        chat_control_button_container.append(&control_label);
        chat_control_button_container.append(&new_ojbect_button);
        chat_control_button_container.append(&delete_object_button);

        let separator = Separator::new(gtk::Orientation::Horizontal);
        separator.add_css_class("settings-chats-separator");



        let list_place = GtkBox::new(gtk::Orientation::Vertical, 0);


        let scrollable_container = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        list_place.append(&scrollable_container);

        let mut object_list: TypedListView<NavigationObjectButton, NoSelection> = TypedListView::new();
        object_list.view.add_css_class("navigation-list-view");
        let object_click_function: SharedClickFunction = Arc::new(RwLock::new(open_object_click));
        populate_object_view_wrapper(
            &scrollable_container,
            &mut object_list,
            &init.clone().read().unwrap(),
            sender.clone(),
            object_click_function.clone()
        );

        // Assembling root container
        root.append(logo.widget());
        root.append(&settings_button);
        root.append(&model_list_button);
        root.append(&chat_control_button_container);
        root.append(&separator);
        root.append(&new_object_entry_place);
        root.append(&list_place);

        // let model_button_connector = model_button_builder.launch((init.clone(), sender.clone()));
        // let mut model_button_controller = model_button_connector.forward(sender.input_sender(), |message|{
        //     NavigationInput::ModelButtonEvent(message)
        // });
        ComponentParts { model: NavigationView {
            bus: init,
            list_place,
            model_list_button,
            model_list_button_handler,
            objects_container: scrollable_container,
            model_list_controller: None,
            object_click_function,
            object_list,
            new_ojbect_button,
            new_ojbect_button_handler,
            delete_object_button,
            delete_ojbect_button_handler,
            root,
            new_object_widgets: None,
            new_object_entry_place
        }, widgets: () }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NavigationInput::OpenModeList => {
                println!("OpenModeList");
                let model_list_builder = ModelList::builder();

                let child_option = self.list_place.first_child();
                if let Some(child) = child_option {
                    self.list_place.remove(&child);
                }

                self.list_place.append(&model_list_builder.root);

            
                self.model_list_controller = Some(model_list_builder.launch(self.bus.clone()).forward(
                    sender.input_sender(),
                    |model_id|{NavigationInput::ModelSelected(model_id)}
                ));

                // Chnage button handler
                let new_handler = self.model_list_button.connect_clicked(move |_|{
                    sender.input(NavigationInput::CloseModelList);
                });
                let old_handler = std::mem::replace(&mut self.model_list_button_handler, new_handler);
                self.model_list_button.disconnect(old_handler);
            },
            NavigationInput::CloseModelList => {
                println!("CloseModelList");
                let child_option = self.list_place.first_child();
                if let Some(child) = child_option {
                    self.list_place.remove(&child);
                }

                self.list_place.append(&self.objects_container);

                self.model_list_controller = None;

                // Chnage button handler
                let new_handler = self.model_list_button.connect_clicked(move |_| {
                    sender.input(NavigationInput::OpenModeList);
                });
                let old_handler = std::mem::replace(&mut self.model_list_button_handler, new_handler);
                self.model_list_button.disconnect(old_handler);
            },
            NavigationInput::ModelSelected(model_id) => {
                println!("ModelSelected");
                let bus_ref = self.bus.read().unwrap();
                self.model_list_button.set_css_classes(&["navigation-model-select-button-selected"]);
                self.model_list_button.set_label(&model_id);
                bus_ref.config.selected_model_name.update(Some(model_id));
                sender.input(NavigationInput::CloseModelList);
            },
            NavigationInput::StartObjectRemoval => {
                println!("StartObjectRemoval");
                *self.object_click_function.write().unwrap() = remove_object_click;
                self.new_ojbect_button.set_sensitive(false);
                self.delete_object_button.set_css_classes(&["navigation-object-control-delete-button-engaged"]);


                let new_handler = self.delete_object_button.connect_clicked(move|_|{
                    sender.input(NavigationInput::StopObjectRemoval);
                });
                let old_handler = std::mem::replace(&mut self.delete_ojbect_button_handler, new_handler);
                self.delete_object_button.disconnect(old_handler);
                self.delete_object_button.set_label("Cancel");

            },
            NavigationInput::StopObjectRemoval => {
                println!("StopObjectRemoval");
                *self.object_click_function.write().unwrap() = open_object_click;
                self.new_ojbect_button.set_sensitive(true);
                self.delete_object_button.set_css_classes(&["navigation-object-control-delete-button"]);

                let new_handler = self.delete_object_button.connect_clicked(move|_|{
                    sender.input(NavigationInput::StartObjectRemoval);
                });
                let old_handler = std::mem::replace(&mut self.delete_ojbect_button_handler, new_handler);
                self.delete_object_button.disconnect(old_handler);
                self.delete_object_button.set_label("Delete");
            },
            NavigationInput::RemoveThisObject(name) => {
                println!("RemoveThisObject");
                let mut to_delete_index: Option<u32> = None;
                let mut to_delete_path: Option<PathBuf> = None;
                for (object_index, object_button_wraped) in self.object_list.iter().enumerate(){
                    let object_button = object_button_wraped.borrow();
                    if object_button.object.name.inner == name.inner{
                        to_delete_index = Some(object_index as u32);
                        to_delete_path = Some(object_button.object.path.clone())
                    }
                }
                if let (Some(index), Some(path)) = (to_delete_index, to_delete_path){
                    self.object_list.remove(index);

                    if self.object_list.is_empty(){
                        self.objects_container.set_child(Some(&GtkLabel::new(Some("There are no chats"))));
                    }

                    std::fs::remove_file(path);
                    sender.input(NavigationInput::StopObjectRemoval);
                }
            },
            NavigationInput::StartObjectAddition => {
                println!("StartObjectAddition");
                // Create the entry
                let new_object_entry = GtkEntry::builder()
                    .css_classes(["navigation-object-control-entry"])
                    .hexpand(true)
                    .build();
                let status_label = GtkLabel::builder()
                    .css_classes(["navigation-object-control-label"])
                    .build();
                self.new_object_entry_place.append(&new_object_entry);
                self.new_object_entry_place.append(&status_label);
                self.new_object_widgets = Some((new_object_entry, status_label));
                
                // Set up control buttons
                self.new_ojbect_button.set_label("Save");
                let sender_clone = sender.clone();
                let new_new_handler = self.new_ojbect_button.connect_clicked(move |_|{
                    sender_clone.input(NavigationInput::TryCommitAddition);
                });
                let old_new_handler = std::mem::replace(&mut self.new_ojbect_button_handler, new_new_handler);
                self.new_ojbect_button.disconnect(old_new_handler);
                
                self.delete_object_button.set_label("Cancel");
                let new_delete_handler: SignalHandlerId = self.delete_object_button.connect_clicked(move |_|{
                    sender.input(NavigationInput::StopObjectAddition);
                });
                let old_delete_handler = std::mem::replace(&mut self.delete_ojbect_button_handler, new_delete_handler);
                self.delete_object_button.disconnect(old_delete_handler);
            },
            NavigationInput::StopObjectAddition => {
                println!("StopObjectAddition");
                // Remove widgets
                if let Some((entry, label)) = &self.new_object_widgets{
                    self.new_object_entry_place.remove(entry);
                    self.new_object_entry_place.remove(label);
                }

                //Reset buttons
                self.new_ojbect_button.set_label("New");
                let sender_clone = sender.clone();
                let new_new_handler = self.new_ojbect_button.connect_clicked(move |_|{
                    sender_clone.input(NavigationInput::StartObjectAddition);
                });
                let old_new_handler = std::mem::replace(&mut self.new_ojbect_button_handler, new_new_handler);
                self.new_ojbect_button.disconnect(old_new_handler);

                self.delete_object_button.set_label("Delete");
                let new_delete_handler = self.delete_object_button.connect_clicked(move|_|{
                    sender.input(NavigationInput::StartObjectRemoval);
                });
                let old_delete_handler = std::mem::replace(&mut self.delete_ojbect_button_handler, new_delete_handler);
                self.delete_object_button.disconnect(old_delete_handler);
            },
            NavigationInput::TryCommitAddition => {
                println!("TryCommitAddition");
                if let Some((new_object_entry, status_label)) = &self.new_object_widgets{
                    let name_text = new_object_entry.text().to_string();

                    // Check if name is valid

                    let valid_name = match NameString::try_from(name_text.clone()){
                        Ok(name) => name,
                        Err(err) => {
                            match err{
                                Some(invalid_char) => status_label.set_label(&format!("Character '{}' is invalid", invalid_char)[..]),
                                None => status_label.set_label("Name ccan not be empty"),
                            }
                            return;
                        }
                    };

                    // Check if such does not exist
                    let mut exists = false;
                    status_label.set_label("Checking for mathces");
                    for object_button_wrapped in self.object_list.iter(){
                        let object_button = object_button_wrapped.borrow();
                        if object_button.object.name.inner == name_text{
                            exists = true;
                        }
                    }
                    if exists {
                        status_label.set_label("Chat with this name exists");
                        return;
                    }

                    // Create new chat
                    let bus_ref = self.bus.read().unwrap();
                    match bus_ref.storage.new_chat(valid_name) {
                        Ok((_, fs_object)) => {
                            let object_list_was_empty = self.object_list.is_empty();
                            self.object_list.append(NavigationObjectButton::new(
                                fs_object,
                                sender.clone(),
                                self.object_click_function.clone()
                            ));
                            if object_list_was_empty{
                                self.objects_container.set_child(Some(&self.object_list.view));
                            }
                            sender.input(NavigationInput::StopObjectAddition);
                        },
                        Err(_) => {
                            status_label.set_label("I/O error happened");
                        }
                    }
                }
            }
        }
    }
}
