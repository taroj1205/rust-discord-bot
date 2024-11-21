use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue, CommandInteraction};
use serenity::prelude::*;
use crate::api::hiroyuki::get_hiroyuki_voice;

pub async fn run(ctx: &Context, command: &CommandInteraction, options: &[ResolvedOption<'_>]) -> Result<Vec<u8>, String> {
    // First, defer the response to show that we're processing
    if let Err(why) = command.defer(&ctx.http).await {
        println!("❌ Failed to defer response: {}", why);
        return Err(format!("Failed to defer response: {}", why));
    }
    println!("✅ Successfully deferred response");

    if let Some(ResolvedOption {
        value: ResolvedValue::String(text), ..
    }) = options.first()
    {
        println!("🎤 Processing voice command for text: {}", text);
        get_hiroyuki_voice(text)
            .await
            .map_err(|e| {
                println!("❌ Error generating voice: {}", e);
                format!("Failed to generate Hiroyuki's voice: {}", e)
            })
    } else {
        println!("❌ No valid text provided");
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