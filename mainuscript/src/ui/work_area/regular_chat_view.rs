use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::chat::ChatCompletionRequestAssistantMessage;
use async_openai::types::chat::ChatCompletionRequestAssistantMessageArgs;

use async_openai::types::chat::CreateChatCompletionResponse;
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
use gtk::Separator;
use relm4::gtk::Button as GtkButton;
use relm4::gtk::Entry;
use relm4::gtk::NoSelection;
use relm4::gtk::ScrolledWindow;
use relm4::gtk::glib::SignalHandlerId;
use relm4::gtk::glib::object::ObjectExt;
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
    control_button_handler: SignalHandlerId,
    bus: Arc<RwLock<Bus>>,
    // ai_client_subscription: ConfigSubscription<Option<Client<OpenAIConfig>>>,
    selected_model_subscription: ConfigSubscription<Option<String>>,
    selected_model: Option<String>,
    ai_client: Option<Client<OpenAIConfig>>,
    chat_completition_request: CreateChatCompletionRequest,
    message_is_read_from_stream: bool, // If this is set to fasle listening to AI response will stop
    model_is_unselected: bool, // This will be set to true is model was unselected while response was listened in
    messages_list: TypedListView<Message, NoSelection>,
    chat_data: RegularChat,
    chat_fs_object: Arc<FsObject>,
    current_model_name: String // Name of model that the response in process now used
}
pub struct RegularChatView{
    data: Option<RegularChatViewData>,
    is_uninit: bool
}
enum MessagesHolder{
    Messagess(TypedListView<Message, NoSelection>),
    Placeholder(GtkLabel)
}
pub struct RegularChatInit{
    pub bus: Arc<RwLock<Bus>>,
    pub chat_fs_object: Arc<FsObject>
}
#[derive(Debug)]
pub enum RegularChatInput{
    ResponseOver,
    SendMessage,
    StopMessage,
    // AiClientUpdated,
    SelectedModelUpdated
}
// enum RegularChatReadiness{
//     ModelNotSelected,
//     ClientNotInitiated,
//     Ready
// }
pub enum RegularChatCommandOutput{
    GotResponse(CreateChatCompletionResponse),
    // GotStream(ClientStreamPair),
    // GotMessageChank(String, ClientStreamPair),
    // ContinueListening(ClientStreamPair),
    Failure(OpenAIError),
    ClientEmpty,
    ResponseOver
}
struct ClientStreamPair{
    client: Client<OpenAIConfig>,
    stream: ChatCompletionResponseStream
}
impl Debug for RegularChatCommandOutput{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegularChatCommandOutput")
    }
}
impl Component for RegularChatView{
    type Input = RegularChatInput;
    type Output = ();
    type Init = RegularChatInit;
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
                    model: RegularChatView { data: None, is_uninit:true },
                    widgets: ()
                }
            }
        }

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
            .spacing(10)
            .css_classes(["regular_chat-header"])
            .build();
        let name_label = GtkLabel::builder()
            .label(init_chat.name.inner.clone())
            .css_classes(["regular_chat-name_label"])
            .build();
        let header_separator = Separator::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["regular_chat-header_separator"])
            .build();
        let status_label = GtkLabel::builder()
            .css_classes(["regular_chat-status_label"])
            .build();

        header.append(&name_label);
        header.append(&header_separator);
        header.append(&status_label);
        root.append(&header);

        // Scrollable list of messages or placeholder
        let messages_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();
        let mut chat_completition_request = CreateChatCompletionRequest::default();
        chat_completition_request.max_completion_tokens = Some(512u32);

            
        // Deal with messages
        let mut messages_list = TypedListView::new();
        if init_chat.contents.is_empty(){
            status_label.set_label("Type message bellow to start");
        }
        // Populate struct for prompting
        messages_list.view.set_css_classes(&["regular_chat-list"]);
        for message in init_chat.contents.iter(){
        // let message = message_ref.borrow();
            chat_completition_request.messages.push(match &message.author{
                MessageAuthor::User => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default().content(message.content.clone()).build().expect("ChatCompletionRequestUserMessageArgs")
                ),
                MessageAuthor::AI(name) => ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessageArgs::default().content(message.content.clone()).name(name).build().expect("ChatCompletionRequestAssistantMessageArgs")
                )
            });
            let message_clone: Message = message.clone();
            messages_list.append(message_clone);
        }
        messages_scroll.set_child(Some(&messages_list.view));
        root.append(&messages_scroll);
        
        // Bottom chat controls with entry ffield and button
        let chat_controls_box = GtkBox::builder()
            .hexpand(true)
            .height_request(50)
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["regular_chat-controls_box"])
            .build();

        let chat_entry = Entry::builder()
            .css_classes(["regular_chat-entry"])
            .hexpand(true)
            .build();
        let control_button = GtkButton::builder()
            .label("=>")
            .vexpand(false)
            .css_classes(["regular_chat-control_button"])
            .build();
        let sender_clone = sender.clone();
        let control_button_handler = control_button.connect_clicked(move |_|{
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
                data: Some(RegularChatViewData{
                    status_label,
                    messages_scroll,
                    chat_entry,
                    control_button,
                    control_button_handler,
                    bus: init.bus,
                    selected_model_subscription,
                    selected_model: current_selected_model.clone(),
                    chat_completition_request,
                    message_is_read_from_stream: false,
                    ai_client: ai_client_option.clone(),
                    model_is_unselected: false,
                    messages_list,
                    chat_data: init_chat,
                    chat_fs_object: init.chat_fs_object,
                    current_model_name: String::new()
                }),
                is_uninit: false
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, sender: relm4::prelude::ComponentSender<Self>, root: &Self::Root) {
        match message {
            RegularChatInput::SendMessage => {
                if let Some(data) = &mut self.data{
                let entry_text: String = data.chat_entry.text().into();
                // If not text is input => stop
                if entry_text.is_empty(){
                    // data.chat_entry.set_css_classes(&[]);
                    data.chat_entry.set_css_classes(&["regular_chat-entry-hightlight"]);
                    return;
                }
                println!("SendMessage beyond empty");
                let bus_ref = data.bus.read().unwrap();

                // Create user's message and AI's response
                let new_user_message = Message {
                    author: MessageAuthor::User,
                    content: entry_text.clone()
                };
                data.messages_list.append(new_user_message.clone());
                data.chat_data.contents.push(new_user_message);

                // Push only User message to completition request
                data.chat_completition_request.messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(entry_text)
                        .build().unwrap()
                ));

                // Put selected model to completion message
                data.chat_completition_request.model = data.selected_model.clone().unwrap();
                data.current_model_name = data.selected_model.clone().unwrap();

                // Change control button function to stopping the ai message
                let sender_clone = sender.clone();
                let new_handler = data.control_button.connect_clicked(move|_|{
                    sender_clone.input(RegularChatInput::StopMessage);
                });
                let old_handler = std::mem::replace(&mut data.control_button_handler, new_handler);
                data.control_button.disconnect(old_handler);
                data.control_button.set_label("=");

                // Save to state the information that the message is supposed to be streamed
                data.message_is_read_from_stream = true;

                // Start the command to get the stream
                let client_clone = data.ai_client.clone().unwrap();
                let chat_completion_request_clone = data.chat_completition_request.clone();
                sender.oneshot_command(async move{
                    let chat_api = client_clone.chat();
                    let response_attempt = chat_api.create(chat_completion_request_clone).await;
                    match response_attempt{
                        Ok(response) => return RegularChatCommandOutput::GotResponse(response),
                        Err(err) => return RegularChatCommandOutput::Failure(err)
                    }
                });
            }},
            RegularChatInput::StopMessage => {
                if let Some(data) = &mut self.data{
                data.message_is_read_from_stream = false;
            }},
            RegularChatInput::SelectedModelUpdated => {
                if let Some(data) = &mut self.data{
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
            }},
            RegularChatInput::ResponseOver => {
                if let Some(data) = &mut self.data{
                // Reset cutton and label
                reset_controls_to_ready(data, sender);

                // Set last update to now
                data.chat_data.last_update_date = chrono::Utc::now().timestamp_micros();
                
                
                // If model was unselected while previous response was listened in disable controls now
                if data.model_is_unselected{
                    data.control_button.set_sensitive(false);
                    data.status_label.set_label("Model not selected");
                }

                data.message_is_read_from_stream = false;
            }}
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
            RegularChatCommandOutput::GotResponse(response) => {
                if let Some(data) = &mut self.data{
                if data.message_is_read_from_stream{
                    for choise in response.choices{
                        if let Some(content) = choise.message.content{
                            write_last_message_to_all_places(data, content);
                        }
                    }
                    sender.input(RegularChatInput::ResponseOver);
                }
                // When user hit stop button while waiting for response
                else{
                    delete_working_messages(data);
                    data.status_label.set_label("Stopped listening");
                    reset_controls_to_ready(data, sender);
                }
                
            }},
            // RegularChatCommandOutput::GotStream(pair) => {
            //     if let Some(data) = &mut self.data{
            //     println!("GotStream");
            //     if !data.message_is_read_from_stream{ // Stop if command to stop was send
            //         data.status_label.set_label("Stopped listening");
            //         // Delete user's and model's messages
            //         delete_last_two_messages(data);
            //         return;
            //     }
            //     data.status_label.set_label("Streaming...");
            //     // let last_message_wrapped = Arc::new(RwLock::new(last_message));
            //     sender.oneshot_command(async move{
            //         work_on_stream(pair).await
            //     });
            // }},
            // RegularChatCommandOutput::GotMessageChank(message, pair) => {
            //     if let Some(data) = &mut self.data{
            //     println!("GotMessageChank");
            //     if !data.message_is_read_from_stream{ // Stop if command to stop was send
            //         data.status_label.set_label("Stopped listening");
            //         delete_last_two_messages(data);
                    
            //         return;
            //     }
            //     let list_len = data.chat_data.contents.len();
            //     let mut last_message_option: Option<&mut Message> = data.chat_data.contents.get_mut(list_len - 1);
            //     let last_message = match last_message_option{
            //         Some(message) => message,
            //         None => {
            //             dbg!("Last message empty return triggered");
            //             return
            //         }
            //     };
            //     last_message.content.push_str(&message);
            //     sender.oneshot_command(async move{
            //         work_on_stream(pair).await
            //     });
            // }},
            // RegularChatCommandOutput::ContinueListening(pair) => {
            //     if let Some(data) = &mut self.data{
            //     println!("ContinueListening");
            //     if !data.message_is_read_from_stream{ // Stop if command to stop was send
            //         data.status_label.set_label("Stopped listening");
            //         delete_last_two_messages(data);
            //         return;
            //     }
            //     sender.oneshot_command(async move{
            //         work_on_stream(pair).await
            //     });
            // }},
            RegularChatCommandOutput::Failure(err) => {
                if let Some(data) = &mut self.data{
                data.status_label.set_label("Error happened");
                delete_working_messages(data);
                dbg!(err);

                // Reset cutton and label
                data.control_button.set_label("=>");
                data.status_label.set_label("");
                let sender_clone = sender.clone();
                let new_handler = data.control_button.connect_clicked(move |_|{
                    sender.input(RegularChatInput::SendMessage);
                });
                let old_handler = std::mem::replace(&mut data.control_button_handler, new_handler);
                data.control_button.disconnect(old_handler);
            }},
            RegularChatCommandOutput::ResponseOver => {
                sender.input(RegularChatInput::ResponseOver);
            },
            RegularChatCommandOutput::ClientEmpty => {
                println!("ClientEmpty not implemented");
            }
        }
    }
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        if self.is_uninit{
            return;
        }
        if let Some(data) = &mut self.data {
            // If was in the process of listning on new message forget it
            if data.message_is_read_from_stream{
                // Delete two last messages
                delete_working_messages(data);
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
fn delete_working_messages(data: &mut RegularChatViewData){
    let list_len = data.messages_list.len();
    if list_len > 0{
        data.messages_list.remove(list_len - 1);
    }
    // let list_len = data.messages_list.len();
    // if list_len > 0{
    //     data.messages_list.remove(list_len - 1);
    // }
    data.chat_data.contents.pop();
    // data.chat_data.contents.pop();

    // Assistant message is not added to it at this point
    data.chat_completition_request.messages.pop();
}
fn write_last_message_to_all_places(data: &mut RegularChatViewData, content: String){
    let new_ai_message = Message{
        author: MessageAuthor::AI(data.selected_model.clone().unwrap()),
        content: content.clone()
    };
    data.messages_list.append(new_ai_message.clone());
    data.chat_data.contents.push(new_ai_message.clone());
    data.chat_completition_request.messages.push(
        ChatCompletionRequestMessage::Assistant(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .name(data.current_model_name.clone())
                .build().unwrap()
        )
    );
}
fn reset_controls_to_ready(data: &mut RegularChatViewData, sender: ComponentSender<RegularChatView>){
    data.control_button.set_label("=>");
    data.status_label.set_label("");
    let new_handler = data.control_button.connect_clicked(move |_|{
        sender.input(RegularChatInput::SendMessage);
    });
    let old_handler = std::mem::replace(&mut data.control_button_handler, new_handler);
    data.control_button.disconnect(old_handler);
}
// async fn work_on_stream(mut pair: ClientStreamPair) -> RegularChatCommandOutput{
//     let listen_result = pair.stream.next().await;
//     dbg!(&listen_result);
//     if let Some(result) = listen_result{
//         match result {
//             Ok(response) => {
//                 for chat_choise in response.choices.iter(){
//                     match chat_choise.delta.content{
//                         Some(ref content) => {
//                             return RegularChatCommandOutput::GotMessageChank(content.clone(), pair)
//                         },
//                         None => return RegularChatCommandOutput::ContinueListening(pair)
//                     };
//                 };
//                 return RegularChatCommandOutput::ContinueListening(pair)
//             },
//             Err(err) => {
//                 return RegularChatCommandOutput::Failure(err);
//             }
//         }
//     };
//     return RegularChatCommandOutput::ResponseOver;
// }