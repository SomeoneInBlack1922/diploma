use borsh::{BorshDeserialize, BorshSerialize};
pub use implementation::*;
/// Each regular chat has a thread of messages.
/// There can be multiple threads: at each message user can reprompt model or edit own message
/// wich creates alternative branch

use crate::helper_types::{AdequateDateTime, NameString, TextString};
pub mod implementation;
/// Holds all messages and metadata
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RegularChat{
    // To be able to distinguish between files of different versions
    api_version: u64,
    name: NameString,
    creating_date: AdequateDateTime,
    last_update_date: AdequateDateTime,
    contents: RegularChatBrench,
}
/// Holds all messages in a branch
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RegularChatBrench{
    messages: Vec<Message>
}
/// Holds the message, and all branches that branch off of it.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message{
    author: NameString,
    date: AdequateDateTime,
    content: TextString,
    branches: Vec<RegularChatBrench>
}