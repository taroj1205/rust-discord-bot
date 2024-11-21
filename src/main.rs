mod commands;
mod api;
mod db;

use std::env;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{
    CreateInteractionResponse, CreateInteractionResponseMessage, 
    CreateAttachment, EditInteractionResponse, CreateInteractionResponseFollowup
};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use songbird::SerenityInit;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "id" => Some(commands::id::run(&command.data.options())),
                "attachmentinput" => Some(commands::attachmentinput::run(&command.data.options())),
                "modal" => {
                    commands::modal::run(&ctx, &command).await.unwrap();
                    None
                },
                "hiroyuki" => {
                    match commands::hiroyuki::run(&ctx, &command, &command.data.options()).await {
                        Ok(audio_data) => {
                            // First, edit the deferred response to show success
                            if let Err(e) = command
                                .edit_response(&ctx.http,
                                    EditInteractionResponse::new()
                                        .content("✅ Voice generated successfully!"))
                                .await
                            {
                                println!("Failed to send success message: {}", e);
                            }

                            // Then send the audio file as a follow-up message
                            let followup = CreateInteractionResponseFollowup::new()
                                .add_file(CreateAttachment::bytes(audio_data, "hiroyuki.wav"));
                                
                            if let Err(e) = command.create_followup(&ctx.http, followup).await {
                                println!("Failed to send audio file: {}", e);
                            }
                        }
                        Err(why) => {
                            if let Err(e) = command
                                .edit_response(&ctx.http, 
                                    EditInteractionResponse::new()
                                        .content(&why))
                                .await
                            {
                                println!("Failed to send error response: {}", e);
                            }
                        }
                    }
                    None
                },
                "connect" => {
                    match commands::voice::run(&command, &ctx).await {
                        Ok(response) => Some(response),
                        Err(error) => Some(error),
                    }
                },
                "disconnect" => {
                    match commands::voice::run_disconnect(&command, &ctx).await {
                        Ok(response) => Some(response),
                        Err(error) => Some(error),
                    }
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let global_commands = Command::set_global_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::id::register(),
            commands::welcome::register(),
            commands::numberinput::register(),
            commands::attachmentinput::register(),
            commands::modal::register(),
            commands::wonderful_command::register(),
            commands::hiroyuki::register(),
            commands::voice::register(),
            commands::voice::register_disconnect(),
        ])
        .await;

        match global_commands {
            Ok(_commands) => println!("Successfully registered global commands!"),
            Err(why) => println!("Error registering global commands: {why:?}"),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize database
    if let Err(e) = db::init_db() {
        eprintln!("Failed to initialize database: {:?}", e);
        return;
    }

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}