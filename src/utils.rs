use base64::{engine::general_purpose, Engine as _};
use rand::seq::SliceRandom;
use rand::Rng;

use rusqlite::{params, Connection};

use serde::Deserialize;
use serenity::all::{ChannelId, CreateMessage};
use serenity::all::{GuildId, Http};
use tokio::fs;

use crate::markov_chain;

pub async fn generate_markov_message(
    guild_id: GuildId,
    channel_id: ChannelId,
    custom_word: Option<&str>,
) -> Option<CreateMessage> {
    const DATABASE_MESSAGE_FETCH_LIMIT: usize = 2000;

    let sentences: Vec<String> = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("messages.db").expect("Unable to open database");

        let mut stmt = conn
            .prepare(
                "SELECT content FROM messages WHERE guild_id = ?1 AND channel_id = ?2 ORDER BY RANDOM() LIMIT ?3;",
            )
            .unwrap();

        let sentences_iter = stmt
            .query_map(
                params![
                    guild_id.get(),
                    channel_id.get(),
                    DATABASE_MESSAGE_FETCH_LIMIT
                ],
                |row| row.get(0),
            )
            .unwrap();

        sentences_iter.map(|result| result.unwrap()).collect()
    })
    .await
    .unwrap();

    if sentences.len() < 500 {
        return None;
    }

    let mut rng = rand::thread_rng();

    let mut markov_chain = markov_chain::Chain::new();
    markov_chain.train(sentences);

    let max_words = rng.gen_range(1..15);
    let content = markov_chain.generate(max_words, custom_word);
    Some(CreateMessage::new().content(content))
}

pub async fn get_most_popular_channel(guild_id: GuildId) -> u64 {
    let channel_id: u64 = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("messages.db").expect("Unable to open database");

        let mut stmt = conn
            .prepare("SELECT channel_id FROM messages WHERE guild_id = ?1 GROUP BY channel_id ORDER BY COUNT(*) DESC LIMIT 1;")
            .unwrap();

        let channel_id_result: u64 = stmt.query_row(params![guild_id.get()], |row| row.get(0)).unwrap_or(0);

        channel_id_result
    })
    .await
    .unwrap();

    channel_id
}

#[derive(Deserialize, Clone)]
struct Config {
    pub avatar_link: Vec<String>,
}

pub async fn get_random_pfp() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let toml_content = fs::read_to_string("Avatars.toml").await?;
    let config: Config = toml::from_str(&toml_content)?;

    let mut rng = rand::thread_rng();
    Ok(config.avatar_link.choose(&mut rng).cloned())
}

pub async fn change_bot_profile(http: &Http, avatar_url: &String) -> Result<(), serenity::Error> {
    let avatar_response = reqwest::get(avatar_url)
        .await
        .expect("Failed to fetch avatar");

    let avatar_bytes = avatar_response
        .bytes()
        .await
        .expect("Failed to read avatar bytes");

    let avatar_base64 = general_purpose::STANDARD.encode(&avatar_bytes);
    let avatar_data_uri = format!("data:image/png;base64,{}", avatar_base64);

    http.edit_profile(&serde_json::json!({
        "avatar": avatar_data_uri,
    }))
    .await?;

    Ok(())
}
