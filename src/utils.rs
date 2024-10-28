use rand::Rng;

use rusqlite::{params, Connection};
use serenity::all::{ChannelId, GuildId};

use crate::markov_chain;

pub async fn generate_markov_message(
    guild_id: GuildId,
    channel_id: ChannelId,
    custom_word: Option<&str>,
) -> Option<String> {
    const DATABASE_MESSAGE_FETCH_LIMIT: usize = 2000;

    let sentences: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut conn: Option<Connection> = None;
        for i in 0..=5 {
            match Connection::open("messages.db") {
                Ok(conn_ok) => conn = Some(conn_ok),
                Err(err) => {
                    eprintln!("Errored while opening db: {}, i: {}", err, i);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            };
        }

        let conn = conn.expect("Failed to establish database connection after multiple attempts.");

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
    Some(markov_chain.generate(max_words, custom_word))
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
