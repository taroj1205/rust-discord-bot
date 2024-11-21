use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use std::error::Error;
use crate::api::hiroyuki::get_hiroyuki_voice;

pub async fn run(options: &[ResolvedOption<'_>]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(text), ..
    }) = options.first()
    {
        get_hiroyuki_voice(text).await
    } else {
        Err("Please provide valid text".into())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("hiroyuki")
        .description("Convert text to Hiroyuki's voice")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "text", "The text to convert to speech")
                .required(true),
        )
}