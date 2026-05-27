use relm4::gtk::Text;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::helper_types::{AdequateDateTime, NameString, TextString};
// pub mod implementations;
pub struct Chat{
    //To be able to distinguish between files of different versions
    api_version: u64,
    name: NameString,
    creating_date: AdequateDateTime,
    last_update_date: AdequateDateTime,
    contents: ChatCollection,
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ChatCollection{
    messages: Vec<ResponseMessageCollection>
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ResponseMessageCollection{
    messages: Vec<MessageInCollection>
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MessageInCollection{
    body: Message,
    //Id of position of previous message in it's vector in ResponseMessageCollection
    previous: usize,
    //Id of position of next message in it's vector in ResponseMessageCollection
    next: usize
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message{
    author: NameString,
    date: AdequateDateTime,
    content: TextString
}