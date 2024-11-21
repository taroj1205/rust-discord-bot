use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub async fn run(options: &[ResolvedOption<'_>]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(text), ..
    }) = options.first()
    {
        let client = Client::new();
        
        // First API call to get the audio URL
        let res = client
            .post("https://plbwpbyme3.execute-api.ap-northeast-1.amazonaws.com/production/coefonts/19d55439-312d-4a1d-a27b-28f0f31bedc5/try")
            .json(&json!({
                "text": text
            }))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err("Failed to create Hiroyuki voice".into());
        }

        let json: serde_json::Value = res.json().await?;
        let location = json["location"].as_str().ok_or("No location in response")?;

        // Second API call to get the audio data
        let audio_res = client.get(location).send().await?;
        let audio_data = audio_res.bytes().await?;

        Ok(audio_data.to_vec())
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