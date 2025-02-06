use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;
use crate::db::language::Language;

pub async fn run(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in servers")?;
    println!("🔄 Setting language for guild: {}", guild_id);

    if let Some(option) = command.data.options.first() {
        if let Some(lang) = option.value.as_str() {
            println!("🌐 Language option provided: {}", lang);
            let language = Language::from(lang);
            println!("🔍 Parsed language: {:?}", language);
            match crate::db::set_guild_language(guild_id.get(), language) {
                Ok(_) => {
                    println!("✅ Successfully set language to {:?} for guild {}", language, guild_id);
                    Ok(match language {
                        Language::English => "Language has been set to English".to_string(),
                        Language::Japanese => "言語が日本語に設定されました".to_string(),
                    })
                },
                Err(e) => {
                    println!("❌ Failed to set language: {:?}", e);
                    Err("Failed to set language".to_string())
                }
            }
        } else {
            println!("❌ Invalid language value provided");
            Err("Invalid language value provided".to_string())
        }
    } else {
        println!("❌ No language option provided");
        Err("Please provide a valid language option (english/japanese)".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("setlanguage")
        .description("Set the server's language (English/Japanese)")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "language", "The language to use (english/japanese)")
                .required(true)
                .add_string_choice("English", "english")
                .add_string_choice("Japanese", "japanese"),
        )
}
