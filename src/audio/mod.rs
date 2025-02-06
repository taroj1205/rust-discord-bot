use std::path::{Path, PathBuf};
use std::fs;
use crate::api::hiroyuki;
use crate::db::language::Language;

const CONNECT_EN: &str = "Connected to VC";
const CONNECT_JP: &str = "接続しました";

pub async fn ensure_audio_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let audio_dir = Path::new("assets/audio");
    if !audio_dir.exists() {
        println!("📁 Creating audio directory");
        fs::create_dir_all(audio_dir)?;
    }

    // Check and generate English audio
    let en_path = audio_dir.join("connect_en.mp3");
    if !en_path.exists() {
        println!("🎵 Generating English connect audio");
        let audio_data = hiroyuki::get_hiroyuki_voice(CONNECT_EN).await?;
        fs::write(&en_path, audio_data)?;
        println!("✅ Saved English connect audio");
    }

    // Check and generate Japanese audio
    let jp_path = audio_dir.join("connect_jp.mp3");
    if !jp_path.exists() {
        println!("🎵 Generating Japanese connect audio");
        let audio_data = hiroyuki::get_hiroyuki_voice(CONNECT_JP).await?;
        fs::write(&jp_path, audio_data)?;
        println!("✅ Saved Japanese connect audio");
    }

    Ok(())
}

pub fn get_connect_audio_path(language: Language) -> PathBuf {
    let filename = match language {
        Language::English => "connect_en.mp3",
        Language::Japanese => "connect_jp.mp3",
    };
    Path::new("assets/audio").join(filename)
}
