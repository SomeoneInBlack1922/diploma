use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::chat::ChatCompletionRequestAssistantMessage;
use async_openai::types::chat::ChatCompletionRequestAssistantMessageArgs;

use futures::StreamExt;

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
use relm4::gtk::prelude::BoxExt;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::EditableExt;
use relm4::gtk::prelude::WidgetExt;
use relm4::tokio::sync::watch::Ref;
use relm4::typed_view::TypedListItem;
use relm4::typed_view::list::TypedListView;

use crate::bus::Bus;
use crate::config_field::ConfigSubscription;
use crate::helper_types::AdequateDateTime;
use crate::helper_types::NameString;
use crate::object::FsObject;
use crate::object::regular_chat;
use crate::object::regular_chat::Message;
use crate::object::regular_chat::MessageAuthor;
use crate::object::regular_chat::RegularChat;
use crate::storage;
mod regular_message_view;

// struct ChatCompletionRequestMessage {inner: ChatCompletionRequestMessageOriginal}
// impl Deref for ChatCompletionRequestMessage{
//     type Target = ChatCompletionRequestMessageOriginal;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

pub struct RegularChatViewData{
    status_label: GtkLabel,
    messages_scroll: ScrolledWindow,
    chat_entry: Entry,
    control_button: GtkButton,
    bus: Arc<RwLock<Bus>>,
    // ai_client_subscription: ConfigSubscription<Option<Client<OpenAIConfig>>>,
    selected_model_subscription: ConfigSubscription<Option<String>>,
    selected_model: Option<String>,
    ai_client: Option<Client<OpenAIConfig>>,
    chat_completition_request: CreateChatCompletionRequest,
    readiness: RegularChatReadiness,
    message_is_read_from_stream: bool, // If this is set to fasle listening to AI response will stop
    model_is_unselected: bool, // This will be set to true is model was unselected while response was listened in
    messages_list: TypedListView<Message, NoSelection>,
    chat_data: RegularChat,
    chat_fs_object: FsObject
}
pub struct RegularChatView{
    data: MaybeUninit<RegularChatViewData>,
    is_uninit: bool
}
enum MessagesHolder{
    Messagess(TypedListView<Message, NoSelection>),
    Placeholder(GtkLabel)
}
pub struct ReularChatInit{
    bus: Arc<RwLock<Bus>>,
    chat_fs_object: FsObject
}
#[derive(Debug)]
pub enum RegularChatInput{
    SendMessage,
    StopMessage,
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
    GotMessageChank(String, ChatCompletionResponseStream),
    ContinueListening(ChatCompletionResponseStream),
    Failure(OpenAIError),
    ClientEmpty,
    ResponseOver
}
impl Debug for RegularChatCommandOutput{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegularChatCommandOutput")
    }
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
        let bus_clone = init.bus.clone();
        let bus_ref = bus_clone.read().unwrap();
        // Try to read chat from file
        let mut init_chat: RegularChat;
        match bus_ref.storage.read_from_file::<RegularChat>(&init.chat_fs_object.path.to_string_lossy()){
            Ok(chat_data) => {init_chat = chat_data},
            Err(err) => {
                let failed_to_load_label = GtkLabel::new(Some("Failed to load chat file"));
                root.append(&failed_to_load_label);
                return ComponentParts {
                    model: RegularChatView { data: MaybeUninit::uninit(), is_uninit:true },
                    widgets: ()
                }
            }
        }

        let mut chat_readiness = RegularChatReadiness::Ready;
        let sender_clone = sender.clone();

        let selected_model_subscription = ConfigSubscription::subscribe(
            bus_ref.config.selected_model_name.clone(),
            Box::new(move|_|{
                sender_clone.input(RegularChatInput::SelectedModelUpdated);
            })
        );

        // Header
        // Chat name and status
        let header = GtkBox::builder()
            .hexpand(true)
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["regular_chat-header"])
            .build();
        let name_label = GtkLabel::builder()
            .label(init_chat.name.inner.clone())
            .css_classes(["regular_chat-name_label"])
            .build();
        let status_label = GtkLabel::builder()
            .css_classes(["regular_chat-name_label"])
            .build();

        header.append(&name_label);
        header.append(&status_label);
        root.append(&header);

        // Scrollable list of messages or placeholder
        let messages_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let mut chat_completition_request = CreateChatCompletionRequest::default();
        chat_completition_request.max_completion_tokens = Some(u16::MAX as u32);

            
        // Deal with messages
        let mut messages_list = TypedListView::new();
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
            // Populate struct for prompting
            messages_list.view.set_css_classes(&["regular_chat-list"]);
            for message in init_chat.contents.iter(){
                // let message = message_ref.borrow();
                chat_completition_request.messages.push(match message.author{
                    MessageAuthor::User => ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default().content(message.content.clone()).build().unwrap()
                    ),
                    MessageAuthor::AI(_) => ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessageArgs::default().content(message.content.clone()).build().unwrap()
                    )
                });
                let message_clone: Message = message.clone();
                messages_list.append(message_clone);
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

        // Two conditions to not let RegularChatInput::SendMessage be sent if either
        // selected_model or ai_client is not set
        let current_selected_model = bus_ref.config.selected_model_name.get_value_rw_lock().read().unwrap();
        if let None = &*current_selected_model{
            control_button.set_sensitive(false);
            status_label.set_label("Model not selected");
        };

        let ai_client_option = bus_ref.ai.get_value_rw_lock().read().unwrap();
        if let None = &*ai_client_option{
            control_button.set_sensitive(false);
            status_label.set_label("OpenAI settings are not set correctly");
        }
        // drop(buf_ref);
        ComponentParts{
            model: RegularChatView {
                data: MaybeUninit::new(RegularChatViewData{
                    status_label,
                    messages_scroll,
                    chat_entry,
                    control_button,
                    bus: init.bus,
                    selected_model_subscription,
                    selected_model: current_selected_model.clone(),
                    chat_completition_request,
                    readiness: chat_readiness,
                    message_is_read_from_stream: false,
                    ai_client: ai_client_option.clone(),
                    model_is_unselected: false,
                    messages_list,
                    chat_data: init_chat,
                    chat_fs_object: init.chat_fs_object
                }),
                is_uninit: false
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, sender: relm4::prelude::ComponentSender<Self>, root: &Self::Root) {
        match message {
            RegularChatInput::SendMessage => unsafe{
                let data = &mut self.data.assume_init_read();
                let entry_text: String = data.chat_entry.text().into();
                // If not text is input => stop
                if entry_text.is_empty(){
                    data.chat_entry.set_css_classes(&["regular_chat-entry-hightlight"]);
                    return;
                }
                let bus_ref = data.bus.read().unwrap();
                
                // Clone request and put user's message into chat
                let mut local_completition_request = data.chat_completition_request.clone();
                local_completition_request.model = data.selected_model.clone().unwrap(); // Should be set at this point
                local_completition_request.messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default().content(entry_text.clone()).build().unwrap()
                ));
                let new_message = Message {
                    author: MessageAuthor::User,
                    content: entry_text
                };
                data.messages_list.append(new_message.clone());
                data.chat_data.contents.push(new_message);

                // Change control button function to stopping the ai message
                let sender_clone = sender.clone();
                data.control_button.connect_clicked(move|_|{
                    sender_clone.input(RegularChatInput::StopMessage);
                });
                data.control_button.set_label("=");

                // Save to state the information that the message is supposed to be streamed
                data.message_is_read_from_stream = true;

                // Start the command to get the stream
                let client_clone = data.ai_client.clone().unwrap();
                sender.oneshot_command(async move{
                    let stream_attempt: Result<ChatCompletionResponseStream, OpenAIError> = client_clone.chat().create_stream(local_completition_request).await;
                    match stream_attempt{
                        Ok(stream) => {
                            RegularChatCommandOutput::GotStream(stream)
                        },
                        Err(err) => {
                            RegularChatCommandOutput::Failure(err)
                        }
                    }
                });
            },
            RegularChatInput::StopMessage => unsafe {
                self.data.assume_init_read().message_is_read_from_stream = false;
            }
            RegularChatInput::SelectedModelUpdated => unsafe {
                let data = &mut self.data.assume_init_read();
                let bus_ref = data.bus.read().unwrap();
                match &*bus_ref.config.selected_model_name.get_value_rw_lock().read().unwrap(){
                    Some(_) => {
                        data.control_button.set_sensitive(true);
                        data.status_label.set_label("");
                    },
                    None => {
                        if data.message_is_read_from_stream{
                            data.model_is_unselected = true;
                        }
                        else {
                            data.control_button.set_sensitive(false);
                            data.status_label.set_label("Model not selected");
                        }
                    }
                }
                todo!("RegularChatInput::SelectedModelUpdated")
            }
            // RegularChatInput::AiClientUpdated => {
            //     todo!()
            // },
            // Reg
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
            RegularChatCommandOutput::GotStream(stream) => unsafe{
                let data = &mut self.data.assume_init_read();
                if !data.message_is_read_from_stream{ // Stop if command to stop was send
                    data.status_label.set_label("Stopped listening");
                    let list_len = data.chat_data.contents.len();
                    data.chat_data.contents.remove(list_len);
                    return;
                }
                data.status_label.set_label("Streaming...");
                // let last_message_wrapped = Arc::new(RwLock::new(last_message));
                sender.oneshot_command(async move{
                    get_stream_chunk(stream).await
                });
            },
            RegularChatCommandOutput::GotMessageChank(message, stream) => unsafe{
                let data = &mut self.data.assume_init_read();
                if !data.message_is_read_from_stream{ // Stop if command to stop was send
                    data.status_label.set_label("Stopped listening");
                    let list_len = data.chat_data.contents.len();
                    data.chat_data.contents.remove(list_len);
                    
                    return;
                }
                let list_len = data.chat_data.contents.len();
                let mut last_message_option: Option<&mut Message> = data.chat_data.contents.get_mut(list_len);
                let last_message = match last_message_option{
                    Some(message) => message,
                    None => {
                        dbg!("Last message empty return triggered");
                        return
                    }
                };
                last_message.content.push_str(&message);
                sender.oneshot_command(async move{
                    get_stream_chunk(stream).await
                });
            },
            RegularChatCommandOutput::ContinueListening(stream) => unsafe{
                let data = &mut self.data.assume_init_read();
                if !data.message_is_read_from_stream{ // Stop if command to stop was send
                    data.status_label.set_label("Stopped listening");
                    let list_len = data.chat_data.contents.len();
                    data.chat_data.contents.remove(list_len);
                    return;
                }
                sender.oneshot_command(async move{
                    get_stream_chunk(stream).await
                });
            },
            RegularChatCommandOutput::Failure(err) => unsafe{
                let data = &mut self.data.assume_init_read();
                data.status_label.set_label(&err.to_string());
                let list_len = data.chat_data.contents.len();
                data.chat_data.contents.remove(list_len);

                let list_len = data.messages_list.len();
                data.messages_list.remove(list_len);
            },
            RegularChatCommandOutput::ResponseOver => unsafe{
                let data = &mut self.data.assume_init_read();
                // Reset cutton and label
                data.control_button.set_label("=>");
                data.status_label.set_label("");
                let sender_clone = sender.clone();
                data.control_button.connect_clicked(move |_|{
                    sender.input(RegularChatInput::SendMessage);
                });

                data.chat_data.last_update_date = chrono::Utc::now().timestamp_micros();
                
                
                // If model was unselected while previous response was listened in disable controls now
                if data.model_is_unselected{
                    data.control_button.set_sensitive(false);
                    data.status_label.set_label("Model not selected");
                }
            }
            _ => {todo!()}
        }
    }
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        if self.is_uninit{
            return;
        }
        unsafe {
            let data = &mut self.data.assume_init_read();
            // If was in the process of listning on new message forget it
            if data.message_is_read_from_stream{
                // Delete two last messages
                let list_len = data.chat_data.contents.len();
                data.chat_data.contents.remove(list_len);

                let list_len = data.chat_data.contents.len();
                data.chat_data.contents.remove(list_len);
            }
            let bus_ref = data.bus.read().unwrap();
            let storage_clone = bus_ref.storage.clone();
            let chat_data_replacement = RegularChat::empty();
            let chat_data = mem::replace(&mut data.chat_data, chat_data_replacement);
            let fs_object_clone = data.chat_fs_object.clone();
            bus_ref.async_runtime.spawn_blocking( move||{
                storage_clone.store_to_file(&chat_data, fs_object_clone.path.to_str().unwrap());
            });

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
async fn get_stream_chunk(mut stream: ChatCompletionResponseStream) -> RegularChatCommandOutput{
    if let Some(result) = stream.next().await{
        match result {
            Ok(response) => {
                for chat_choise in response.choices.iter(){
                    match chat_choise.delta.content{
                        Some(ref content) => {
                            return RegularChatCommandOutput::GotMessageChank(content.clone(), stream)
                        },
                        None => return RegularChatCommandOutput::ContinueListening(stream)
                    };
                };
            },
            Err(err) => {
                return RegularChatCommandOutput::Failure(err);
            }
        }
        };
        return RegularChatCommandOutput::ResponseOver;
}