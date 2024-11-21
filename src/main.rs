#![allow(deprecated)]
mod commands;

// use std::collections::HashSet;
use std::sync::Arc;

use serenity::all::{Message, ShardManager};

use serenity::async_trait;
use serenity::framework::standard::macros::group;
// use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::info;
use shuttle_runtime::SecretStore;
use anyhow::anyhow;

use crate::commands::math::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
        else if msg.content == "!hello" {
            if let Err(why) = msg.channel_id.say(&ctx, "world!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

#[group]
#[commands(multiply)]
struct General;

#[shuttle_runtime::main]
async fn serenity(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secret_store
        .get("DISCORD_TOKEN")
        .ok_or_else(|| anyhow!("'DISCORD_TOKEN' was not found"))?;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}