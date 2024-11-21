use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::*;

pub fn register() -> CreateCommand {
    CreateCommand::new("connect").description("Connect to your voice channel")
}

pub fn register_disconnect() -> CreateCommand {
    CreateCommand::new("disconnect").description("Disconnect from the voice channel")
}

pub async fn run(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
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

    let _handler = manager.join(guild_id, channel_id).await.map_err(|e| e.to_string())?;

    Ok("Connected to your voice channel!".to_string())
}

pub async fn run_disconnect(command: &CommandInteraction, ctx: &Context) -> Result<String, String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in servers")?;
    
    let manager = songbird::get(ctx).await
        .ok_or("Failed to get voice client")?;
    
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.leave().await.map_err(|e| e.to_string())?;
    } else {
        return Err("Not connected to a voice channel".to_string());
    }

    Ok("Disconnected from voice channel!".to_string())
}
