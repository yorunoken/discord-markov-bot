use rand::Rng;
use rusqlite::{params, Connection};

use serenity::prelude::*;
use serenity::{all::GuildId, model::channel::Message};

use crate::markov_chain::Chain;

pub async fn send_markov_message(ctx: &Context, msg: &Message, guild_id: GuildId) {
    const DATABASE_MESSAGE_FETCH_LIMIT: usize = 2000;
    let channel_id = msg.channel_id;

    let sentences: Vec<String> = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("messages.db").expect("Unable to open database");

        let mut stmt = conn
            .prepare(
                "SELECT sentence FROM messages WHERE guild_id = ?1 AND channel_id = ?2 LIMIT ?3;",
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

    let response = match sentences.len() >= 500 {
        true => {
            let mut rng = rand::thread_rng();
            let mut markov_chain = Chain::new();
            markov_chain.train(sentences);

            let max_words = rng.gen_range(1..15);
            markov_chain.generate(max_words)
        }
        false => String::from("The chat must have 500+ messages for me to generate messages."),
    };

    msg.channel_id.say(&ctx.http, response).await;
}
