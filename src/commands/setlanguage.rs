use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use crate::db::language::Language;

pub fn run(options: &[ResolvedOption]) -> String {
    if let (Some(ResolvedOption {
        value: ResolvedValue::String(lang), ..
    }), Some(ResolvedOption {
        value: ResolvedValue::Integer(guild_id), ..
    })) = (options.first(), options.get(1)) {
        let language = Language::from(&**lang);
        match crate::db::set_guild_language(*guild_id as u64, language) {
            Ok(_) => match language {
                Language::English => "Language has been set to English".to_string(),
                Language::Japanese => "言語が日本語に設定されました".to_string(),
            },
            Err(_) => "Failed to set language".to_string(),
        }
    } else {
        "Please provide a valid language option (english/japanese) and guild ID".to_string()
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
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "guild_id", "Guild ID")
                .required(true),
        )
}
