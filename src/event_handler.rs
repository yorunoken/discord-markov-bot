use rand::Rng;
use rusqlite::{params, Connection};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::markov_chain::Chain;

const DATABASE_MESSAGE_FETCH_LIMIT: usize = 2000;

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

        let mut response = "".to_string();
        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            let mut rng = rand::thread_rng();

            let mut stmt = conn
                .prepare("SELECT sentence FROM messages WHERE guild_id = ?1 LIMIT ?2;")
                .unwrap();

            let sentences_iter = stmt
                .query_map(
                    params![guild_id.get(), DATABASE_MESSAGE_FETCH_LIMIT],
                    |row| row.get(0),
                )
                .unwrap();

            let mut sentences: Vec<String> = Vec::new();
            for sentence_result in sentences_iter {
                sentences.push(sentence_result.unwrap());
            }

            println!("{:#?}", sentences);

            let mut markov_chain = Chain::new();
            markov_chain.train(sentences);

            let max_words = rng.gen_range(1..15);
            response = markov_chain.generate(max_words);
        }

        if !response.is_empty() {
            msg.channel_id.say(&ctx.http, response).await.unwrap();
            return;
        }

        let sentence = msg.content;
        // Only save to database if message doesn't contain bot user tag
        if !sentence.contains(format!("<@{}>", ctx.cache.current_user().id).as_str()) {
            conn.execute(
                "INSERT INTO messages (sentence, channel_id, guild_id, message_id, author_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sentence, msg.channel_id.get(), guild_id.get(), msg.id.get(), msg.author.id.get()],
            )
            .expect("Failed to insert word into database");
        }
    }
}
