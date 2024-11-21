use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub async fn get_hiroyuki_voice(text: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    println!("🎯 Starting to fetch voice for text: {}", text);
    let client = Client::new();
    
    // First API call to get the audio URL
    println!("📡 Making first API call to get audio URL...");
    let res = client
        .post("https://plbwpbyme3.execute-api.ap-northeast-1.amazonaws.com/production/coefonts/19d55439-312d-4a1d-a27b-28f0f31bedc5/try")
        .json(&json!({
            "text": text
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        println!("❌ Failed to create Hiroyuki voice for text: {}", text);
        return Err("Failed to create Hiroyuki voice".into());
    }

    let json: serde_json::Value = res.json().await?;
    let location = json["location"].as_str().ok_or("No location in response")?;
    println!("✅ Got audio URL successfully");

    // Second API call to get the audio data
    println!("📡 Making second API call to fetch audio data...");
    let audio_res = client.get(location).send().await?;
    let audio_data = audio_res.bytes().await?;
    println!("✅ Successfully fetched audio data for text: {}", text);

    Ok(audio_data.to_vec())
}
