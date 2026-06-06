use std::{borrow::Cow, ffi::{OsStr, OsString}, fmt::Write, ops::{Deref, DerefMut}};


use crate::helper_types::*;

impl NameString{
    pub fn new() -> Self{
        NameString { inner: String::new() }
    }
    pub fn set_to(&mut self, target: &str) -> OnFailure<char>{
        for a_char in target.chars(){
            if !(a_char.is_alphanumeric() || a_char.eq(&'_')){
                return Some(a_char);
            };
        };
        self.inner = target.into();
        return None;
    }
    pub fn from_full_file_name(name: &str) -> Result<Self, char>{
        let mut out_string = String::new();
        for a_char in name.chars(){
            if a_char.eq(&'.'){
                break
            }
            else if !Self::valid_char(a_char){
                return Err(a_char)
            }
            else{
                out_string.push(a_char);
            }
        };
        return Ok(NameString{
            inner: out_string
        })
    }
    fn valid_str(target: &str) -> OnFailure<char>{
        for a_char in target.chars(){
            if !Self::valid_char(a_char){
                return Some(a_char);
            };
        };
        return None;
    }
    fn valid_char(a_char: char) -> bool{
        return (a_char.is_alphanumeric() || a_char.eq(&'_') || a_char.eq(&' '))
    }
}
impl TryFrom<String> for NameString{
    type Error = Option<char>;
    /// If validation fails first invalid chat is returned
    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.is_empty() {
            return Err(None);
        }
        match Self::valid_str(&text[..]){
            Some(char) => {return Err(Some(char))},
            None => {return Ok(Self{inner: text})}
        }
    }
}
impl TryFrom<&OsStr> for NameString{
    type Error = Option<char>;
    /// If validation fails first invalid chat is returned
    /// If conversion from OsString to String fails returns Err(None)
    fn try_from(text_os: &OsStr) -> Result<Self, Self::Error> {
        let text = match text_os.to_str(){
            Some(string) => {string},
            None => {return Err(None)}
        };
        match Self::valid_str(&text[..]){
            Some(char) => {return Err(Some(char))},
            None => {return Ok(Self{inner: text.to_string()})}
        }
    }
}

impl Deref for NameString{
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}