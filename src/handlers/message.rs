use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::input::Input;
use regex::Regex;
use lazy_static::lazy_static;
use serenity::model::id::UserId;

use crate::api::hiroyuki;
use crate::db;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://[^\s]+").unwrap();
    static ref MENTION_REGEX: Regex = Regex::new(r"<@!?(\d+)>").unwrap();
}

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

    // Process message content
    let mut processed_content = msg.content.clone();

    // Replace user mentions with usernames using regex
    if let Some(guild) = msg.guild(&ctx.cache) {
        processed_content = MENTION_REGEX.replace_all(&processed_content, |caps: &regex::Captures| {
            if let Ok(user_id) = caps[1].parse::<u64>() {
                let user_id = UserId::new(user_id);
                if let Some(member) = guild.members.get(&user_id) {
                    return member.display_name().to_string();
                }
            }
            caps[0].to_string()
        }).to_string();
    }

    // Replace URLs with リンク省略
    processed_content = URL_REGEX.replace_all(&processed_content, "リンク省略").to_string();
    println!("🔊 Generating voice for message: {}", processed_content);
    
    // Get audio data from Hiroyuki API
    let audio_data = hiroyuki::get_hiroyuki_voice(&processed_content)
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
