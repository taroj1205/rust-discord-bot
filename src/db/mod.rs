use rusqlite::{Connection, Result};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Connection> = Mutex::new(
        Connection::open("bot.db").expect("Failed to open database")
    );
}

pub fn init_db() -> Result<()> {
    println!("🔄 Initializing database");
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS voice_channels (
            guild_id INTEGER NOT NULL,
            channel_id INTEGER NOT NULL,
            is_listening BOOLEAN NOT NULL DEFAULT 0,
            PRIMARY KEY (guild_id, channel_id)
        )",
        [],
    )?;
    
    // Reset all listening status on startup
    conn.execute(
        "UPDATE voice_channels SET is_listening = 0",
        [],
    )?;
    
    println!("✅ Database initialized successfully");
    Ok(())
}

pub fn set_listening_status(guild_id: u64, channel_id: u64, is_listening: bool) -> Result<()> {
    println!("🔄 Setting listening status for guild {} channel {} to {}", guild_id, channel_id, is_listening);
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO voice_channels (guild_id, channel_id, is_listening) 
         VALUES (?1, ?2, ?3)",
        [&(guild_id as i64), &(channel_id as i64), &(is_listening as i64)],
    )?;
    println!("✅ Successfully updated listening status");
    Ok(())
}

pub fn is_listening(guild_id: u64, channel_id: u64) -> Result<bool> {
    println!("🔍 Checking listening status for guild {} channel {}", guild_id, channel_id);
    let conn = DB_CONNECTION.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT is_listening FROM voice_channels 
         WHERE guild_id = ?1 AND channel_id = ?2"
    )?;
    
    let result: Result<bool> = stmt.query_row(
        [guild_id as i64, channel_id as i64],
        |row| row.get(0)
    );
    
    match result {
        Ok(status) => {
            println!("✅ Found listening status: {}", status);
            Ok(status)
        },
        Err(_) => {
            println!("ℹ️ No listening status found, defaulting to false");
            Ok(false)
        }
    }
}
