use std::ops::{Deref, DerefMut};

use chrono::TimeZone;
use relm4::gtk::TextView;
use relm4::gtk::gdk::Cursor;
use relm4::gtk::prelude::BoxExt;
use relm4::gtk::prelude::TextBufferExt;
use relm4::gtk::prelude::TextViewExt;
use relm4::gtk::prelude::WidgetExt;
use relm4::typed_view::list::RelmListItem;
use relm4::gtk;
use gtk::Box as GtkBox;
use gtk::Label as GtkLabel;

use crate::object::regular_chat::*;
impl RegularChat {
    pub fn empty() -> Self{
        RegularChat {
            api_version: 0,
            name: NameString::new(),
            contents: RegularChatBrench{messages: vec![]},
            creating_date: 0,
            last_update_date: 0

        }
    }
    pub fn new(name: NameString) -> Self{
        let now_date = chrono::Utc::now().timestamp_micros();
        RegularChat {
            api_version: API_VERSION,
            name: name,
            contents: RegularChatBrench { messages: vec![] },
            creating_date: now_date,
            last_update_date: now_date
        }
    }
}
impl Deref for RegularChatBrench{
    type Target = Vec<Message>;
    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}
impl DerefMut for RegularChatBrench{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}
// impl DisplayableMessage{
//     pub fn from_message(message: Message) -> Self{
//         return Self{
//             author: message.author,
//             date: message.date,
//             content: TextBuffer::default().ins
//         }
//     }
// }
impl RelmListItem for Message{
    type Root = GtkBox;
    type Widgets = ();
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let chat_box = GtkBox::builder()
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .css_classes(["regular_chat-chat_box"])
            .build();
        return (chat_box, ())
    }
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let sender_label = GtkLabel::builder()
            .label(match &self.author{
                MessageAuthor::AI(model_id) => model_id.clone(),
                MessageAuthor::User => "User".into()
            })
            .selectable(true)
            .hexpand(true)
            .xalign(0.0)
            .css_classes(["regular_chat-sender_name_label"])
            .build();

        let message_text_view = GtkLabel::builder()
            .label(&self.content)
            .css_classes(["regular_chat-message_text"])
            .selectable(true)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::Word)
            .xalign(0.0)
            .build();

        _root.append(&sender_label);
        _root.append(&message_text_view);
    }
}
// Manual serialization and deserialiation so that i can use the TypedListView type in my structs

// For when i tried using TypedListView
// impl BorshDeserialize for RegularChatBrench{
//     fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
//         let len = u32::deserialize_reader(reader)?;
//         let mut out_list: TypedListView<Message, NoSelection> = TypedListView::new();
//         for _ in 0..len{
//             out_list.append(Message::deserialize_reader(reader)?);
//         }
//         return Ok(RegularChatBrench{
//             messages: out_list
//         });
//     }
// }
// impl BorshSerialize for RegularChatBrench{
//     fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         u32::serialize(&self.messages.len(), writer)?;
//         for message in self.messages.iter(){
//             Message::serialize(&*message.borrow(), writer)?;
//         };
//         return Ok(());
//     }
// }