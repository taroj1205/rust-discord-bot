use serenity::model::voice::VoiceState;
use serenity::prelude::*;
use songbird::get;

pub async fn handle_voice_state_update(ctx: &Context, old: Option<VoiceState>, new: VoiceState) -> Result<(), String> {
    // If user joined a channel, we don't need to do anything
    if new.channel_id.is_some() {
        return Ok(());
    }

    // User left a channel (new.channel_id is None)
    if let Some(old_state) = old {
        if let Some(old_channel_id) = old_state.channel_id {
            let guild_id = new.guild_id.ok_or("Not in a guild")?;

            // Check if the bot was the one who left/was kicked
            if new.user_id == ctx.cache.current_user().id {
                println!("🚫 Bot was disconnected from voice channel in guild {}", guild_id);
                // Reset all listening statuses for this guild
                if let Err(e) = crate::db::reset_guild_listening_status(guild_id.get()) {
                    println!("❌ Failed to reset listening statuses: {}", e);
                }
                return Ok(());
            }

            // Get the guild and count non-bot users before any async calls
            let non_bot_count = {
                let guild = guild_id
                    .to_guild_cached(&ctx.cache)
                    .ok_or("Failed to get guild")?;

                guild.voice_states
                    .values()
                    .filter(|state| {
                        // Check if user is in this channel and is not a bot
                        if let Some(channel_id) = state.channel_id {
                            if channel_id == old_channel_id {
                                if let Some(member) = state.member.as_ref() {
                                    return !member.user.bot;
                                }
                            }
                        }
                        false
                    })
                    .count()
            }; // guild is dropped here

            // If no non-bot users are left in the channel
            if non_bot_count == 0 {
                println!("🚪 No non-bot users left in channel {}, disconnecting", old_channel_id);
                
                // Get the voice manager
                if let Some(manager) = get(ctx).await {
                    if let Some(handler_lock) = manager.get(guild_id) {
                        let mut handler = handler_lock.lock().await;
                        if let Err(e) = handler.leave().await {
                            println!("❌ Error leaving voice channel: {}", e);
                            return Err(format!("Failed to leave voice channel: {}", e));
                        }
                        println!("✅ Successfully left empty voice channel");
                    }
                }
            }
        }
    }

    Ok(())
}
