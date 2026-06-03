use std::sync::{Arc, RwLock};
use std::thread::spawn;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::models::{Model, ListModelResponse};
use relm4::gtk::{Label, NoSelection, Orientation, ScrolledWindow};
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::typed_view::list::{RelmListItem, TypedListView};
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent, gtk};

use gtk::Button as GtkButton;

use crate::ai::{ClientState, MyModelList, get_models, string_from_openai_error};
use crate::bus::Bus;
use crate::config_field::ConfigSubscription;
use crate::ui::navigation::{NavigationInput, NavigationView};

pub struct ModelButton{
    bus: Arc<RwLock<Bus>>,
    button: GtkButton,
    button_mode: ModelButtonMode,
    navigation_sender: ComponentSender<NavigationView>,
    scroll: ScrolledWindow
}
#[derive(Debug)]
pub enum ModelButtonInput{
    ClientUpdate,
    OpenModelList,
    CloseModelList,
    ModelSelected(String)
}
#[derive(Debug)]
pub enum ModelButtonOutput{
    OpenModelsList(ScrolledWindow),
    CloseModelList(ScrolledWindow)
}
#[derive(Debug)]
pub enum ModelButtonMode{
    Opened(ScrolledWindow),
    Closed
}
#[derive(Debug)]
pub enum ModelButtonCommandOutput{
    ModelsLoaded(MyModelList),
    Error(String)
}
// pub struct ModelButtonInit{
//     bus: Arc<RwLock<Bus>>,

// }
impl Component for ModelButton{
    type Input = ModelButtonInput;
    type Output = ModelButtonOutput;
    type Init = (Arc<RwLock<Bus>>, ComponentSender<NavigationView>);
    type Root = GtkButton;
    type Widgets = ();
    type CommandOutput = ModelButtonCommandOutput;
    fn init_root() -> Self::Root {
        GtkButton::builder()
            .label("...")
            .sensitive(true)
            .css_classes(["navigation-model-button-closed"])
            .build()
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self>
    {
        let (bus, navigation_sender) = init;
        let bus_read = bus.read().unwrap();


        let sender_clone = sender.clone();
        root.connect_clicked(move |_|{
            sender_clone.input(ModelButtonInput::OpenModelList);
        });

        // Handle current state of AI
        // let current_openai_option = bus_read.ai.get_value_rw_lock().read().unwrap();
        // handle_client_state(&*current_openai_option, sender);

        let selected_model_name_ref = bus_read.config.selected_model_name.get_value_rw_lock().read().unwrap();
        return ComponentParts{
            model: Self {
                bus: bus.clone(),
                button: root,
                button_mode: ModelButtonMode::Closed,
                navigation_sender,
                scroll: new_scroll()
            },
            widgets: ()
        }

    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            ModelButtonInput::OpenModelList => {
                let bus_ref = self.bus.read().unwrap();
                let ai_url_key_pair = bus_ref.config.ai_api_url_key_pair.get_value_rw_lock().read().unwrap();

                let message = Label::new(Some("Loading..."));
                self.scroll.set_child(Some(&message));
                let sender_clone = sender.clone();
                self.button.connect_clicked(move |_|{
                    sender_clone.input(ModelButtonInput::CloseModelList);
                });
                // self.navigation_sender.input(NavigationInput::ModelButtonEvent(
                //     ModelButtonOutput::CloseModelList(self.scroll.clone())
                // ));
                if let(Some(url), Some(key)) = (&ai_url_key_pair.url, &ai_url_key_pair.key){
                    let mut  config = OpenAIConfig::new();
                    config = config.with_api_base(url.to_string());
                    config = config.with_api_key(key);
                    let client = Client::with_config(config);

                    sender.oneshot_command(async move{
                        let model_list = get_models(&client).await;
                        match model_list{
                            Ok(model_list) => {
                                return ModelButtonCommandOutput::ModelsLoaded(model_list)
                            },
                            Err(err) => {
                                return ModelButtonCommandOutput::Error(string_from_openai_error(err))
                            }
                        }
                    });
                }
                else{
                    message.set_label("Failed");
                }
            },
            ModelButtonInput::CloseModelList => {
                let sender_clone = sender.clone();
                self.button.connect_clicked(move |_|{
                    sender_clone.input(ModelButtonInput::OpenModelList);
                });
                // self.navigation_sender.input(NavigationInput::ModelButtonEvent(
                //     ModelButtonOutput::CloseModelList(self.scroll.clone())
                // ));
            }
            ModelButtonInput::ModelSelected(model) => {
                println!("ModelSelected TRIGGERED");
            },
            ModelButtonInput::ClientUpdate => {
                println!("ClientUpdate TRIGGERED");
            }

        }
    }
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    )
    {
        match message{
            ModelButtonCommandOutput::ModelsLoaded(model_list) => {
                let mut model_typed_list: TypedListView<ModelEntry, NoSelection> = TypedListView::new();
                self.scroll.set_child(Some(&model_typed_list.view));
                for model in model_list.data{
                    let model_entry = ModelEntry::new(model.id, sender.clone());
                    model_typed_list.append(model_entry);
                }
            },
            ModelButtonCommandOutput::Error(err) => {
                self.scroll.set_child(Some(&Label::new(Some("Failed to load models"))));
            }
        }
    }
}
fn new_scroll() -> ScrolledWindow {
    ScrolledWindow::builder()
        .css_classes(["navigation-model-scroll"])
        .build()
}
struct ModelEntry{
    model_id: String,
    sender: ComponentSender<ModelButton>
}
impl ModelEntry{
    pub fn new(model_id: String, sender: ComponentSender<ModelButton>) -> Self{
        return Self { model_id, sender }
    }
}
impl RelmListItem for ModelEntry{
    type Root = GtkButton;
    type Widgets = ();
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        return (GtkButton::builder().css_classes(["model-list-entry"]).build(), ())
    }
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        _root.set_label(&self.model_id);
        let sender_clone = self.sender.clone();
        let model_id_clone = self.model_id.clone();
        _root.connect_clicked(move |_|{
            sender_clone.input(ModelButtonInput::ModelSelected(model_id_clone.clone()));
        });
    }
}