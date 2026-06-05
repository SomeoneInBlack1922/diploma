use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::chat::ChatCompletionRequestAssistantMessage;
use async_openai::types::chat::ChatCompletionRequestAssistantMessageArgs;

pub use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestUserMessage;
use async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use async_openai::types::chat::ChatCompletionResponseStream;
use async_openai::types::chat::CreateChatCompletionRequest;
use async_openai::types::chat::CreateChatCompletionRequestArgs;
use relm4::Component;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::gtk;
use gtk::Box as GtkBox;
use gtk::Label as GtkLabel;
use relm4::gtk::Button as GtkButton;
use relm4::gtk::Entry;
use relm4::gtk::NoSelection;
use relm4::gtk::ScrolledWindow;
use relm4::gtk::gdk::EventType::PadStrip;
use relm4::gtk::prelude::BoxExt;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::EditableExt;
use relm4::gtk::prelude::WidgetExt;
use relm4::typed_view::list::RelmListItem;
use relm4::typed_view::list::TypedListView;

use crate::ai::GlobalClientConfig;
use crate::bus::Bus;
use crate::config_field::ConfigSubscription;
use crate::object::FsObject;
use crate::object::regular_chat::Message;
use crate::object::regular_chat::MessageAuthor;
use crate::object::regular_chat::RegularChat;

mod regular_message_view;

// struct ChatCompletionRequestMessage {inner: ChatCompletionRequestMessageOriginal}
// impl Deref for ChatCompletionRequestMessage{
//     type Target = ChatCompletionRequestMessageOriginal;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

pub struct RegularChatView{
    status_label: GtkLabel,
    messages_scroll: ScrolledWindow,
    chat_entry: Entry,
    control_button: GtkButton,
    messages_list: TypedListView<Message, NoSelection>,
    bus: Arc<RwLock<Bus>>,
    // ai_client_subscription: ConfigSubscription<Option<Client<OpenAIConfig>>>,
    selected_model_subscription: ConfigSubscription<Option<String>>,
    chat_completition_request: CreateChatCompletionRequest,
    readiness: RegularChatReadiness
}
enum MessagesHolder{
    Messagess(TypedListView<Message, NoSelection>),
    Placeholder(GtkLabel)
}
pub struct ReularChatInit{
    bus: Arc<RwLock<Bus>>,
    chat_data: RegularChat
}
#[derive(Debug)]
pub enum RegularChatInput{
    SendMessage,
    // AiClientUpdated,
    SelectedModelUpdated
}
enum RegularChatReadiness{
    ModelNotSelected,
    ClientNotInitiated,
    Ready
}
pub enum RegularChatCommandOutput{
    GotStream(ChatCompletionResponseStream),
    Failure(OpenAIError),
    ClientEmpty
}
impl Component for RegularChatView{
    type Input = RegularChatInput;
    type Output = ();
    type Init = ReularChatInit;
    type Root = GtkBox;
    type Widgets = ();
    type CommandOutput = RegularChatCommandOutput;
    fn init_root() -> Self::Root {
        GtkBox::builder()
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["regular_chat-root"])
            .build()
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self>
    {
        // Initial chat is going to be moved
        let init_chat = init.chat_data;

        let mut chat_readiness = RegularChatReadiness::Ready;
        let bus_clone = init.bus.clone();
        let buf_ref = bus_clone.read().unwrap();
        let sender_clone = sender.clone();
        let selected_model_subscription = ConfigSubscription::subscribe(
            buf_ref.config.selected_model_name.clone(),
            Box::new(move|_|{
                sender_clone.input(RegularChatInput::SelectedModelUpdated);
            })
        );
        let current_selected_model = buf_ref.config.selected_model_name.get_value_rw_lock().read().unwrap();
        // Header
        // Chat name and status
        let header = GtkBox::builder()
            .hexpand(true)
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["regular_chat-header"])
            .build();
        let name_label = GtkLabel::builder()
            .label(init_chat.name.inner)
            .css_classes(["regular_chat-name_label"])
            .build();
        let status_label = GtkLabel::builder()
            .css_classes(["regular_chat-name_label"])
            .build();
        if let None = &*current_selected_model{
            status_label.set_label("Model not selected");
            chat_readiness = RegularChatReadiness::ModelNotSelected;
        }

        header.append(&name_label);
        header.append(&status_label);
        root.append(&header);

        // Scrollable list of messages or placeholder
        let messages_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let mut messages_list: TypedListView<Message, NoSelection> = TypedListView::new();
        let mut chat_completition_request = CreateChatCompletionRequest::default();
        chat_completition_request.max_completion_tokens = Some(u16::MAX as u32);
        if let Some(model_name) = &*current_selected_model{
            chat_completition_request.model = model_name.clone();
        };
        let mut chat_completition_messages: Vec<ChatCompletionRequestMessage> = vec![];
            
        if init_chat.contents.is_empty(){
            let empty_chat_placeholder = GtkLabel::builder()
                .label("Type message bellow to start")
                .vexpand(true)
                .hexpand(true)
                .yalign(0.5)
                .xalign(0.5)
                .build();
            messages_scroll.set_child(Some(&empty_chat_placeholder));
        }
        else {
            let messages_vector = init_chat.contents.messages;
            messages_list.view.set_css_classes(&["regular_chat-list"]);
            for message in messages_vector.into_iter(){
                chat_completition_request.messages.push(match message.author{
                    MessageAuthor::User => ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default().content(message.content.clone()).build().unwrap()
                    ),
                    MessageAuthor::AI(_) => ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessageArgs::default().content(message.content.clone()).build().unwrap()
                    )
                });
                messages_list.append(message);
            }
            messages_scroll.set_child(Some(&messages_list.view));
        }
        root.append(&messages_scroll);
        
        // Bottom chat controls with entry ffield and button
        let chat_controls_box = GtkBox::builder()
            .hexpand(true)
            .height_request(150)
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["regular_chat-controls_box"])
            .build();

        let chat_entry = Entry::builder()
            .css_classes(["regular_chat-entry"])
            .hexpand(true)
            .build();
        let control_button = GtkButton::builder()
            .label("=>")
            .css_classes(["regular_chat-control_button"])
            .build();
        let sender_clone = sender.clone();
        control_button.connect_clicked(move |_|{
            sender_clone.input(RegularChatInput::SendMessage);
        });
        chat_controls_box.append(&chat_entry);
        chat_controls_box.append(&control_button);
        root.append(&chat_controls_box);
        // drop(buf_ref);
        ComponentParts{
            model: Self {
                status_label,
                messages_scroll,
                chat_entry,
                control_button,
                messages_list,
                bus: init.bus,
                selected_model_subscription,
                chat_completition_request,
                readiness: chat_readiness
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, sender: relm4::prelude::ComponentSender<Self>, root: &Self::Root) {
        match message {
            RegularChatInput::SendMessage => {
                let entry_text: String = self.chat_entry.text().into();
                if entry_text.is_empty(){
                    self.chat_entry.set_css_classes(&["regular_chat-entry-hightlight"]);
                    return;
                }
                let bus_ref = self.bus.read().unwrap();
                let selected_model_option = bus_ref.config.selected_model_name.get_value_rw_lock().read().unwrap();
                let ai_client_option = bus_ref.ai.get_value_rw_lock().read().unwrap();
                match (&*selected_model_option, &*ai_client_option) {
                    (Some(model_name), Some(client)) => {
                        let mut local_completition_request = self.chat_completition_request.clone();
                        local_completition_request.model = model_name.clone();
                        local_completition_request.messages.push(ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default().content(entry_text).build().unwrap()
                        ));
                        let bus_clone = self.bus.clone();
                        drop(bus_ref);
                        drop(selected_model_option);
                        drop(ai_client_option);
                        sender.oneshot_command(async move{
                            let local_bus_ref = bus_clone.read().unwrap();
                            let local_client_option = local_bus_ref.ai.get_value_rw_lock().read().unwrap();
                            match &*local_client_option{
                                Some(local_client)  => {
                                    let stream_attempt: Result<ChatCompletionResponseStream, OpenAIError> = local_client.chat().create_stream(local_completition_request).await;
                                    match stream_attempt{
                                        Ok(stream) => {
                                            return RegularChatCommandOutput::GotStream(stream)
                                        },
                                        Err(err) => {
                                            return RegularChatCommandOutput::Failure(err);
                                        }
                                    };
                                },
                                None => {
                                    return RegularChatCommandOutput::ClientEmpty
                                }
                            }
                            
                        });
                    },
                    (None, Some(_)) => {
                        self.status_label.set_label("Select model first");
                    },
                    (_, None) => {
                        self.status_label.set_label("OpenAI API settings are not set correctly");
                    }
                }
            },
            RegularChatInput::SelectedModelUpdated => {
                todo!("RegularChatInput::SelectedModelUpdated")
            }
            // RegularChatInput::AiClientUpdated => {
            //     todo!()
            // },
            // Reg
        }
    }
}
fn set_control_button(
    control_button: GtkButton,
    chat_readiness: RegularChatReadiness,
    sender: ComponentSender<RegularChatView>
){
    match chat_readiness{
        RegularChatReadiness::Ready => {
            control_button.set_sensitive(true);
            control_button.connect_clicked(move |_| {
                sender.input(RegularChatInput::SendMessage)
            });
        },
        _ => {
            control_button.set_sensitive(false);
        }
    }
}
// impl RelmListItem for ChatCompletionRequestMessage{
//     type Root = GtkBox;
//     type Widgets = ();
//     fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
//         let chat_box = GtkBox::builder()
//             .hexpand(true)
//             .orientation(gtk::Orientation::Vertical)
//             .spacing(0)
//             .css_classes(["regular_chat-chat_box"])
//             .build();
//         return (chat_box, ())
//     }
//     fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
//         let sener_label = GtkLabel::builder()
//             .label(match &self.author{
//                 MessageAuthor::AI(model_id) => model_id.clone(),
//                 MessageAuthor::User => "User".into()
//             })
//             .hexpand(true)
//             .xalign(0.0)
//             .css_classes(["regular_chat-name_label"])
//             .build();

//         let message_text_view = TextView::builder()
//             .hexpand(true)
//             .css_classes(["regular_chat-text_view"])
//             .build();
//         message_text_view.buffer().set_text(&self.content);

//         _root.append(&sener_label);
//         _root.append(&message_text_view);
//     }
// }