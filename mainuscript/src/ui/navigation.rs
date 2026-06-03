
use std::sync::{Arc, RwLock};
use relm4::component::Connector;
use relm4::gtk::{ListItemFactory, NoSelection, ScrolledWindow};
use relm4::gtk::prelude::{BoxExt, WidgetExt};
use relm4::typed_view::list::{RelmListItem, TypedListView};
use relm4::{gtk::prelude::ButtonExt, prelude::*};
use gtk::{Box as GtkBox, Button as GtkButton, Separator, Label};
use gtk::glib::clone;
use crate::bus::Bus;
use crate::object::{self, FsObject};
use crate::storage::Storage;
use crate::ui::logo::LogoView;
use crate::ui::navigation::list_object::{NavigationObjectButton, populate_object_view_wrapper};
use crate::ui::navigation::model_button::{ModelButton, ModelButtonOutput};
use crate::ui::navigation::model_list::ModelList;

mod list_object;
mod model_button;
mod model_list;

pub const NAVIGATION_ELEMENT_SPACING: i32 = 5;
pub struct NavigationView{
    bus: Arc<RwLock<Bus>>,
    list_place: GtkBox,
    objects_container: ScrolledWindow,
    model_list_button: GtkButton,
    model_list_controller: Option<Controller<ModelList>>
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
    CloseModelList
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
            .width_request(500)
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
        model_list_button.connect_clicked(move |_|{
            sender_clone.input(NavigationInput::OpenModeList);
        });

        drop(&bus_ref);


        let separator = Separator::new(gtk::Orientation::Horizontal);
        separator.add_css_class("settings-chats-separator");



        let list_place = GtkBox::new(gtk::Orientation::Vertical, 0);


        let scrollable_container = ScrolledWindow::builder()
            .vexpand(true)
            .build();

        list_place.append(&scrollable_container);

        let mut object_view_wrapper: TypedListView<NavigationObjectButton, NoSelection> = TypedListView::new();
        object_view_wrapper.view.add_css_class("navigation-list-view");
        populate_object_view_wrapper(&scrollable_container, &mut object_view_wrapper, &init.clone().read().unwrap(), sender.clone());


        // Assembling root container
        root.append(logo.widget());
        root.append(&settings_button);
        root.append(&model_list_button);
        root.append(&separator);
        root.append(&list_place);

        // let model_button_connector = model_button_builder.launch((init.clone(), sender.clone()));
        // let mut model_button_controller = model_button_connector.forward(sender.input_sender(), |message|{
        //     NavigationInput::ModelButtonEvent(message)
        // });
        ComponentParts { model: NavigationView {
            bus: init,
            list_place,
            model_list_button,
            objects_container: scrollable_container,
            model_list_controller: None
        }, widgets: () }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NavigationInput::OpenModeList => {
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

                self.model_list_button.connect_clicked(move |_|{
                    sender.input(NavigationInput::CloseModelList);
                });
            },
            NavigationInput::CloseModelList => {
                let child_option = self.list_place.first_child();
                if let Some(child) = child_option {
                    self.list_place.remove(&child);
                }

                self.list_place.append(&self.objects_container);

                self.model_list_controller = None;

                self.model_list_button.connect_clicked(move |_| {
                    sender.input(NavigationInput::OpenModeList);
                });
            },
            NavigationInput::ModelSelected(model_id) => {
                let bus_ref = self.bus.read().unwrap();
                self.model_list_button.set_css_classes(&["navigation-model-select-button-selected"]);
                self.model_list_button.set_label(&model_id);
                bus_ref.config.selected_model_name.update(Some(model_id));
                sender.input(NavigationInput::CloseModelList);
            }
        }
    }
}
