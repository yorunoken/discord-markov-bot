use rand::Rng;
use rusqlite::{params, Connection};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::markov_chain::Chain;

pub struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("Bot has started as {}", bot.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let guild_id = match msg.guild_id {
            Some(s) => s,
            _ => return,
        };

        if msg.author.bot {
            return;
        }

        let conn = Connection::open("messages.db").expect("Unable to open database");

        const DATABASE_MESSAGE_FETCH_LIMIT: usize = 2000;

        let mut response = String::from("");
        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            let mut rng = rand::thread_rng();

            let mut stmt = conn
                .prepare("SELECT sentence FROM messages WHERE guild_id = ?1 AND channel_id = ?2 LIMIT ?3;")
                .unwrap();

            let sentences_iter = stmt
                .query_map(
                    params![
                        guild_id.get(),
                        msg.channel_id.get(),
                        DATABASE_MESSAGE_FETCH_LIMIT
                    ],
                    |row| row.get(0),
                )
                .unwrap();

            let mut sentences: Vec<String> = Vec::new();
            for sentence_result in sentences_iter {
                sentences.push(sentence_result.unwrap());
            }

            response = match sentences.len() >= 1000 {
                true => {
                    let mut markov_chain = Chain::new();
                    markov_chain.train(sentences);

                    let max_words = rng.gen_range(1..15);
                    markov_chain.generate(max_words)
                }
                false => {
                    String::from("The chat must have 1000+ messages for me to generate messages.")
                }
            };
        }

        if !response.is_empty() {
            msg.channel_id.say(&ctx.http, response).await.unwrap();
            return;
        }

        // Only save to database if message doesn't contain bot user tag
        let sentence = msg.content;
        if !sentence.contains(format!("<@{}>", ctx.cache.current_user().id).as_str()) {
            conn.execute(
                "INSERT INTO messages (sentence, channel_id, guild_id, message_id, author_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sentence, msg.channel_id.get(), guild_id.get(), msg.id.get(), msg.author.id.get()],
            )
            .expect("Failed to insert word into database");
        }
    }
}
