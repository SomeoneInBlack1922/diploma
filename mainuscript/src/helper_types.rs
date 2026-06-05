use borsh::{BorshDeserialize, BorshSerialize};

pub use implementation::*;
pub mod implementation;

/// A string with strict constrains on allowed characters:
/// Unicode letters, numeric, '_', ' '
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct NameString{
    pub inner: String
}
pub type TextString = String;
pub type AdequateDateTime = u64;
pub type OnFailure<T> = Option<T>;