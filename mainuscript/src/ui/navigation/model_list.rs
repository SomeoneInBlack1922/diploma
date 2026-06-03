use std::sync::{Arc, RwLock};

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use relm4::gtk::NoSelection;
use relm4::gtk::prelude::ButtonExt;
use relm4::typed_view::list::{RelmListItem, TypedListView};
use relm4::{Component, gtk::ScrolledWindow};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};
use gtk::Label as GtkLabel;
use gtk::Button as GtkButton;

use crate::ai::{MyModelList, get_models, string_from_openai_error};
use crate::bus::Bus;

use relm4::tokio;

use tokio::runtime::{Builder, Runtime};

pub struct ModelList{
    root: ScrolledWindow
}
impl Component for ModelList{
    type Input = ();
    type Output = ();
    type Init = Arc<RwLock<Bus>>;
    type Root = ScrolledWindow;
    type Widgets = ();
    type CommandOutput = ModelListInput;
    fn init_root() -> Self::Root {
        new_scroll()
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self>
    {

        let bus_ref = init.read().unwrap();
        let ai_args = bus_ref.config.ai_api_url_key_pair.get_value_rw_lock().read().unwrap();
        
        let default_message = GtkLabel::new(None);
        root.set_child(Some(&default_message));

        if let (Some(url), Some(key)) = (&ai_args.url, &ai_args.key){
            let config = OpenAIConfig::new().with_api_base(url.to_string()).with_api_key(key);
            let client = Client::with_config(config);
            default_message.set_label("Loading...");
            let sender_clone = sender.clone();
            sender.oneshot_command(async move{
                let model_list = get_models(&client).await;
                match model_list{
                    Ok(model_list) => {
                        ModelListInput::ModelsLoaded(model_list)
                    },
                    Err(err) => {
                        ModelListInput::Error(string_from_openai_error(err))
                    }
                }
            });
            
        }
        else{
            default_message.set_label("Set settings");
        }
        root.set_child(Some(&default_message));
        ComponentParts{
            model: Self{root: root},
            widgets: ()
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
            ModelListInput::ModelsLoaded(model_list) => {
                let mut model_typed_list: TypedListView<ModelEntry, NoSelection> = TypedListView::new();
                for model in model_list.data.iter(){
                    let model_entry = ModelEntry::new(&model.id, sender.clone());
                    model_typed_list.append(model_entry);
                }
                self.root.set_child(Some(&model_typed_list.view));
            },
            ModelListInput::Error(err) => {
                self.root.set_child(Some(&GtkLabel::new(Some("Failed to load models"))));
            }
        }
    }
}
#[derive(Debug)]
pub enum ModelListInput{
    ModelsLoaded(MyModelList),
    Error(String)
}
fn new_scroll() -> ScrolledWindow {
    ScrolledWindow::builder()
        .vexpand(true)
        .css_classes(["navigation-model-scroll"])
        .build()
}
struct ModelEntry{
    model_id: String,
    sender: ComponentSender<ModelList>
}
impl ModelEntry{
    pub fn new(model_id: &String, sender: ComponentSender<ModelList>) -> Self{
        return Self { model_id: model_id.clone(), sender }
    }
}
impl RelmListItem for ModelEntry{
    type Root = GtkButton;
    type Widgets = ();
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        return (GtkButton::builder().hexpand(true).height_request(20).css_classes(["model-list-entry"]).build(), ())
    }
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        _root.set_label(&self.model_id);
        let sender_clone = self.sender.clone();
        let model_id_clone = self.model_id.clone();
        // _root.connect_clicked(move |_|{
        //     sender_clone.input(ModelButtonInput::ModelSelected(model_id_clone.clone()));
        // });
    }
}