use serenity::builder::CreateCommand;
use serenity::builder::EditInteractionResponse;
use serenity::model::application::CommandInteraction;
use serenity::prelude::*;
use crate::db;
use tokio::time::timeout;
use std::time::Duration;

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

    let guild_id = command.guild_id.ok_or("This command can only be used in servers")?;
    let user_id = command.user.id;
    
    // Get the channel_id before getting the manager
    let channel_id = {
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

    let manager = songbird::get(ctx).await
        .ok_or("Failed to get voice client")?
        .clone();

    // Check if already connected and disconnect first
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.leave().await.map_err(|e| e.to_string())?;
    }

    if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(true).await {
            // Clean up on error
            handler.leave().await.map_err(|e| format!("Failed to clean up after deafen error: {:?}", e))?;
            let builder = EditInteractionResponse::new().content(format!("Failed to deafen: {:?}", e));
            command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to deafen: {:?}", e));
        }

        // Set listening status in database
        if let Err(e) = db::set_listening_status(guild_id.get(), channel_id.get(), true) {
            let builder = EditInteractionResponse::new().content(format!("Failed to update database: {:?}", e));
            command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update database: {:?}", e));
        }

        let builder = EditInteractionResponse::new().content("Connected to your voice channel and deafened!");
        command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
        Ok("".to_string())
    } else {
        let builder = EditInteractionResponse::new().content("Failed to join voice channel");
        command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
        Err("Failed to join voice channel".to_string())
    }
}

pub async fn run_disconnect(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
    println!("🔄 Starting disconnect command");
    // Defer the response to show we're processing
    if let Err(why) = command.defer(&ctx.http).await {
        println!("❌ Failed to defer response: {}", why);
        return Err(format!("Failed to defer response: {}", why));
    }
    println!("✅ Successfully deferred response");
    
    let guild_id = command.guild_id.ok_or("This command can only be used in servers")?;
    println!("📍 Guild ID: {}", guild_id);
    
    let manager = match songbird::get(ctx).await {
        Some(manager) => {
            println!("✅ Got voice manager");
            manager
        },
        None => {
            println!("❌ Failed to get voice manager");
            return Err("Failed to get voice client".into())
        }
    };
    
    if let Some(handler_lock) = manager.get(guild_id) {
        println!("✅ Got voice handler");
        let mut handler = handler_lock.lock().await;
        println!("✅ Acquired handler lock");
        
        // Make sure we're undeafened before leaving
        if let Err(e) = handler.deafen(false).await {
            println!("⚠️ Failed to undeafen: {:?}", e);
        } else {
            println!("✅ Successfully undeafened");
        }
        
        // Get channel ID before leaving for database update
        let maybe_channel_id = handler.current_channel().map(|id| id.0.into());
        
        if let Err(e) = handler.leave().await {
            println!("❌ Failed to leave: {:?}", e);
            let builder = EditInteractionResponse::new().content(format!("Failed to leave: {:?}", e));
            command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
        println!("✅ Successfully left voice channel");

        // Send success response before cleanup
        println!("🎉 Sending success response");
        let builder = EditInteractionResponse::new().content("Disconnected from voice channel!");
        if let Err(e) = command.edit_response(&ctx.http, builder).await {
            println!("❌ Failed to send success response: {:?}", e);
            return Err(e.to_string());
        }
        println!("✅ Successfully sent response");

        // Update database before spawning cleanup
        if let Some(channel_id) = maybe_channel_id {
            println!("📍 Updating database for channel ID: {}", channel_id);
            if let Err(e) = db::set_listening_status(guild_id.get(), channel_id, false) {
                println!("⚠️ Failed to update database: {:?}", e);
            } else {
                println!("✅ Successfully updated database");
            }
        }

        // Drop the handler lock before spawning cleanup
        drop(handler);

        // Spawn cleanup operation in a separate task
        println!("🧹 Starting cleanup operations");
        let guild_id_clone = guild_id;
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            // Try to remove the handler with a timeout
            match timeout(Duration::from_secs(5), manager_clone.remove(guild_id_clone)).await {
                Ok(remove_result) => match remove_result {
                    Ok(_) => println!("✅ Successfully removed voice handler"),
                    Err(e) => println!("⚠️ Failed to remove voice handler: {:?}", e),
                },
                Err(_) => println!("⚠️ Timeout while removing voice handler"),
            }
        });

        Ok("".to_string())
    } else {
        println!("ℹ️ Not connected to a voice channel");
        let builder = EditInteractionResponse::new().content("Not connected to a voice channel");
        command.edit_response(&ctx.http, builder).await.map_err(|e| e.to_string())?;
        Err("Not connected to a voice channel".to_string())
    }
}
