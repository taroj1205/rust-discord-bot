use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::input::Input;

use crate::api::hiroyuki;
use crate::db;

pub async fn handle_message(ctx: &Context, msg: &Message) -> Result<(), String> {
    // Ignore messages from bots to prevent potential loops
    if msg.author.bot {
        println!("🤖 Ignoring bot message");
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or("Not in a guild")?;
    let channel_id = msg.channel_id;

    println!("📝 Message received in guild {} channel {}", guild_id, channel_id);

    // Check if bot should be listening in this channel
    match db::is_listening(guild_id.get(), channel_id.get()) {
        Ok(is_listening) => {
            println!("🎧 Listening status for channel {}: {}", channel_id, is_listening);
            if !is_listening {
                println!("❌ Not listening in this channel, ignoring message");
                return Ok(());
            }

            // Get the voice manager to check if we're actually in a voice channel
            println!("🎤 Getting voice manager");
            let manager = match songbird::get(ctx).await {
                Some(manager) => manager.clone(),
                None => {
                    println!("❌ Voice client not available");
                    return Err("Failed to get voice client".to_string());
                }
            };

            // If we're supposed to be listening but not in a voice channel, update the database
            if !manager.get(guild_id).is_some() {
                println!("⚠️ Database says listening but not in voice channel, updating status");
                if let Err(e) = db::set_listening_status(guild_id.get(), channel_id.get(), false) {
                    println!("❌ Failed to update listening status: {}", e);
                }
                return Ok(());
            }
        },
        Err(e) => {
            println!("❌ Error checking listening status: {}", e);
            return Err(format!("Failed to check listening status: {}", e));
        }
    }

    println!("🎤 Getting voice manager for playback");
    // Get the voice manager
    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to get voice client")?
        .clone();

    // Check if bot is connected to a voice channel in this guild
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => {
            println!("✅ Found voice handler for guild {}", guild_id);
            handler
        },
        None => {
            println!("❌ Bot is not in a voice channel");
            return Ok(());  // Bot is not in a voice channel, ignore message
        }
    };

    println!("🔊 Generating voice for message: {}", msg.content);
    // Get audio data from Hiroyuki API
    let audio_data = hiroyuki::get_hiroyuki_voice(&msg.content)
        .await
        .map_err(|e| format!("Failed to get Hiroyuki voice: {}", e))?;

    // Create input from the audio bytes
    let input = Input::try_from(audio_data)
        .map_err(|e| format!("Failed to create input source: {}", e))?;

    // Play the audio
    let mut handler = handler_lock.lock().await;
    println!("🎵 Playing audio");
    handler.play_input(input);

    Ok(())
}
