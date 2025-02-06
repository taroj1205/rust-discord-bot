use crate::audio;
use crate::db;
use serenity::builder::CreateCommand;
use serenity::builder::EditInteractionResponse;
use serenity::model::application::CommandInteraction;
use serenity::prelude::*;
use songbird::get;
use std::fs;

pub fn register() -> CreateCommand {
    CreateCommand::new("connect").description("Connect to your voice channel")
}

pub fn register_disconnect() -> CreateCommand {
    CreateCommand::new("disconnect").description("Disconnect from the voice channel")
}

pub async fn run(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
    // Defer the response to show we're processing
    if let Err(why) = command.defer(&ctx.http).await {
        println!("❌ Failed to defer response: {}", why);
        return Err(format!("Failed to defer response: {}", why));
    }
    println!("✅ Successfully deferred response");

    let guild_id = command
        .guild_id
        .ok_or("This command can only be used in servers")?;
    let user_id = command.user.id;
    let command_channel_id = command.channel_id;

    // Get the channel_id before getting the manager
    let voice_channel_id = {
        let guild = guild_id
            .to_guild_cached(&ctx.cache)
            .ok_or("Failed to get guild")?;

        let voice_state = guild
            .voice_states
            .get(&user_id)
            .ok_or("You must be in a voice channel")?;

        voice_state
            .channel_id
            .ok_or("You must be in a voice channel")?
    };

    let manager = get(ctx).await.ok_or("Failed to get voice client")?.clone();

    // Check if already connected
    if let Some(handler_lock) = manager.get(guild_id) {
        // Check if bot is in the same channel as the user
        let mut handler = handler_lock.lock().await;
        if let Some(call) = handler.current_connection() {
            if call.channel_id.map(|id| id.0.get()) == Some(voice_channel_id.get()) {
                // Bot is already in the same channel, just update the listening status
                if let Err(e) =
                    db::set_listening_status(guild_id.get(), command_channel_id.get(), true)
                {
                    println!("❌ Failed to store channel in database: {}", e);
                    return Err(format!("Failed to store channel in database: {}", e));
                }
                println!("✅ Successfully updated listening status");

                let builder = EditInteractionResponse::new()
                    .content("Updated listening status for this channel!");
                command
                    .edit_response(&ctx.http, builder)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok("".to_string());
            } else {
                // Bot is in a different channel, need to move it
                handler.leave().await.map_err(|e| e.to_string())?;
            }
        }
    }

    // Connect to the voice channel
    let _handler = manager.join(guild_id, voice_channel_id).await;

    // Store the command's text channel in the database
    if let Err(e) = db::set_listening_status(guild_id.get(), command_channel_id.get(), true) {
        println!("❌ Failed to store channel in database: {}", e);
        return Err(format!("Failed to store channel in database: {}", e));
    }
    println!("✅ Successfully stored channel in database");

    // Edit the deferred response
    if let Ok(handler_lock) = manager.join(guild_id, voice_channel_id).await {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(true).await {
            // Clean up on error
            handler
                .leave()
                .await
                .map_err(|e| format!("Failed to clean up after deafen error: {:?}", e))?;
            let builder =
                EditInteractionResponse::new().content(format!("Failed to deafen: {:?}", e));
            command
                .edit_response(&ctx.http, builder)
                .await
                .map_err(|e| e.to_string())?;
            return Err(format!("Failed to deafen: {:?}", e));
        }

        // Update the listening status in the database
        if let Err(e) = db::set_listening_status(guild_id.get(), voice_channel_id.get(), true) {
            println!("Failed to update listening status: {:?}", e);
        }

        let builder =
            EditInteractionResponse::new().content("Connected to your voice channel and deafened!");
        command
            .edit_response(&ctx.http, builder)
            .await
            .map_err(|e| e.to_string())?;

        // Get guild language and play appropriate connect audio
        let language =
            db::get_guild_language(guild_id.get()).unwrap_or(db::language::Language::English);
        println!("🌐 Current guild language: {:?}", language);
        let audio_path = audio::get_connect_audio_path(language);
        println!("📂 Audio path: {:?}", audio_path);

        // Play the connection audio after sending the message
        if let Ok(audio_data) = fs::read(&audio_path) {
            println!("📊 Audio data size: {} bytes", audio_data.len());
            let input = songbird::input::Input::try_from(audio_data);
            if let Ok(input) = input {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                handler.play_input(input);
                println!("🎵 Playing connect audio for language: {:?}", language);
            } else {
                println!("❌ Failed to create input source from audio data");
            }
        } else {
            println!("❌ Failed to read audio file: {:?}", audio_path);
        }

        Ok("".to_string())
    } else {
        let builder = EditInteractionResponse::new().content("Failed to join voice channel");
        command
            .edit_response(&ctx.http, builder)
            .await
            .map_err(|e| e.to_string())?;
        Err("Failed to join voice channel".to_string())
    }
}

pub async fn run_disconnect(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
    // Defer the response to show we're processing
    if let Err(why) = command.defer(&ctx.http).await {
        println!("❌ Failed to defer response: {}", why);
        return Err(format!("Failed to defer response: {}", why));
    }
    println!("✅ Successfully deferred response");

    let guild_id = command
        .guild_id
        .ok_or("This command can only be used in servers")?;
    let command_channel_id = command.channel_id;

    let manager = get(ctx).await.ok_or("Failed to get voice client")?.clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        // Remove the command's text channel from the database
        if let Err(e) = db::set_listening_status(guild_id.get(), command_channel_id.get(), false) {
            println!("❌ Failed to update database: {}", e);
            return Err(format!("Failed to update database: {}", e));
        }
        println!("✅ Successfully removed channel from database");

        // Disconnect from the voice channel
        let mut handler = handler_lock.lock().await;
        handler.leave().await.map_err(|e| e.to_string())?;

        let builder = EditInteractionResponse::new().content("Disconnected from voice channel!");
        command
            .edit_response(&ctx.http, builder)
            .await
            .map_err(|e| e.to_string())?;
        Ok("".to_string())
    } else {
        let builder = EditInteractionResponse::new().content("Not connected to a voice channel!");
        command
            .edit_response(&ctx.http, builder)
            .await
            .map_err(|e| e.to_string())?;
        Ok("".to_string())
    }
}
