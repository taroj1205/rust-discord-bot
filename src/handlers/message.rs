use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::input::Input;

use crate::api::hiroyuki;

pub async fn handle_message(ctx: &Context, msg: &Message) -> Result<(), String> {
    // Ignore messages from bots to prevent potential loops
    if msg.author.bot {
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or("Not in a guild")?;

    // Get the voice manager
    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to get voice client")?
        .clone();

    // Check if bot is connected to a voice channel in this guild
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            return Ok(());  // Bot is not in a voice channel, ignore message
        }
    };

    // Get audio data from Hiroyuki API
    let audio_data = hiroyuki::get_hiroyuki_voice(&msg.content)
        .await
        .map_err(|e| format!("Failed to get Hiroyuki voice: {}", e))?;

    // Create input from the audio bytes
    let input = Input::try_from(audio_data)
        .map_err(|e| format!("Failed to create input source: {}", e))?;

    // Play the audio
    let mut handler = handler_lock.lock().await;
    handler.play_input(input);

    Ok(())
}
