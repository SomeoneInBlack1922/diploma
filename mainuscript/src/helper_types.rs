use borsh::{BorshDeserialize, BorshSerialize};

pub use implementation::*;
pub mod implementation;

/// A string with strict constrains on allowed characters:
/// Unicode letters, numeric, '_', ' '
#[derive(BorshDeserialize, BorshSerialize, Debug)]
#[derive(Clone)]
pub struct NameString{
    pub inner: String
}
pub type TextString = String;
pub type AdequateDateTime = i64; // Microseconds since epoch on UTC
pub type OnFailure<T> = Option<T>;