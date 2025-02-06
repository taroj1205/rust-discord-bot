use rusqlite::{Connection, Result, params};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref DB_CONNECTION: Mutex<Connection> = Mutex::new(
        Connection::open("bot.db").expect("Failed to open database")
    );
}

pub mod language {
    #[derive(Debug, Clone, Copy)]
    pub enum Language {
        English,
        Japanese,
    }

    impl From<&str> for Language {
        fn from(s: &str) -> Self {
            match s.to_lowercase().as_str() {
                "japanese" | "ja" | "jp" => Language::Japanese,
                _ => Language::English, // default to English
            }
        }
    }

    impl From<String> for Language {
        fn from(s: String) -> Self {
            Self::from(s.as_str())
        }
    }
}

use language::Language;

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
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS guild_settings (
            guild_id INTEGER PRIMARY KEY,
            language TEXT NOT NULL DEFAULT 'english'
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
        params![guild_id as i64, channel_id as i64, is_listening as i64],
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
        params![guild_id as i64, channel_id as i64],
        |row| row.get(0)
    );
    
    match result {
        Ok(is_listening) => {
            println!("✅ Found listening status: {}", is_listening);
            Ok(is_listening)
        },
        Err(_) => {
            println!("ℹ️ No listening status found, defaulting to false");
            Ok(false)
        }
    }
}

pub fn set_guild_language(guild_id: u64, language: Language) -> Result<()> {
    println!("🔄 Setting language for guild {}", guild_id);
    let conn = DB_CONNECTION.lock().unwrap();
    let language_str = match language {
        Language::English => "english",
        Language::Japanese => "japanese",
    };
    conn.execute(
        "INSERT OR REPLACE INTO guild_settings (guild_id, language) VALUES (?1, ?2)",
        params![guild_id as i64, language_str],
    )?;
    println!("✅ Successfully updated guild language");
    Ok(())
}

pub fn reset_guild_listening_status(guild_id: u64) -> Result<()> {
    println!("🔄 Resetting all listening statuses for guild {}", guild_id);
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "UPDATE voice_channels SET is_listening = 0 WHERE guild_id = ?1",
        params![guild_id as i64],
    )?;
    println!("✅ Successfully reset all listening statuses for guild {}", guild_id);
    Ok(())
}

pub fn get_guild_language(guild_id: u64) -> Result<Language> {
    println!("🔍 Getting language for guild {}", guild_id);
    let conn = DB_CONNECTION.lock().unwrap();
    let language: String = conn.query_row(
        "SELECT language FROM guild_settings WHERE guild_id = ?1",
        params![guild_id as i64],
        |row| row.get(0),
    ).unwrap_or_else(|_| "english".to_string());
    
    Ok(language.as_str().into())
}
