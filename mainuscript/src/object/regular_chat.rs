use borsh::{BorshDeserialize, BorshSerialize};
pub use implementation::*;
use relm4::{gtk::{NoSelection, TextBuffer}, typed_view::list::TypedListView};
/// Each regular chat has a thread of messages.
/// There can be multiple threads: at each message user can reprompt model or edit own message
/// wich creates alternative branch

use crate::helper_types::{AdequateDateTime, NameString, TextString};
pub mod implementation;

pub const API_VERSION: u64 = 0;
/// Holds all messages and metadata
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RegularChat{
    // To be able to distinguish between files of different versions
    pub api_version: u64,
    pub name: NameString,
    pub creating_date: AdequateDateTime,
    pub last_update_date: AdequateDateTime,
    pub contents: RegularChatBrench,
}
/// Holds all messages in a branch
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RegularChatBrench{
    pub messages: Vec<Message>
}
/// Holds the message, and all branches that branch off of it.
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct Message{
    pub author: MessageAuthor,
    // pub date: AdequateDateTime,
    pub content: TextString
}
unsafe impl Send for Message{}
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum MessageAuthor{
    User,
    AI(String)
}
// pub struct DisplayableMessage{
//     pub author: NameString,
//     pub date: AdequateDateTime,
//     pub content: TextBuffer
// }