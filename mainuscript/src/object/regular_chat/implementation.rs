use std::ops::{Deref, DerefMut};

use relm4::gtk::TextView;
use relm4::gtk::prelude::BoxExt;
use relm4::gtk::prelude::TextBufferExt;
use relm4::gtk::prelude::TextViewExt;
use relm4::typed_view::list::RelmListItem;
use relm4::gtk;
use gtk::Box as GtkBox;
use gtk::Label as GtkLabel;

use crate::object::regular_chat::*;
impl RegularChat {
    
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
        let sener_label = GtkLabel::builder()
            .label(match &self.author{
                MessageAuthor::AI(model_id) => model_id.clone(),
                MessageAuthor::User => "User".into()
            })
            .hexpand(true)
            .xalign(0.0)
            .css_classes(["regular_chat-name_label"])
            .build();

        let message_text_view = TextView::builder()
            .hexpand(true)
            .css_classes(["regular_chat-text_view"])
            .build();
        message_text_view.buffer().set_text(&self.content);

        _root.append(&sener_label);
        _root.append(&message_text_view);
    }
}