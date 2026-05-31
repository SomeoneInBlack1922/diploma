use crate::config::Config;
use openai_api_rust::OpenAI;

#[derive(Debug)]
pub struct AI{
    openai: Option<OpenAI>
}
impl AI {
    pub fn new() -> Self{
        return Self{
            openai: None
        }
    }
}