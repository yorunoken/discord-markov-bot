use rand::Rng;
use rusqlite::{params, Connection};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::utils::send_markov_message;

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

        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            send_markov_message(&ctx, &msg, guild_id).await;
            return;
        }

        // Only save to database if message doesn't contain bot user tag
        let sentence = msg.content;
        if !sentence.contains(format!("<@{}>", ctx.cache.current_user().id).as_str()) {
            let conn = Connection::open("messages.db").expect("Unable to open database");
            conn.execute(
                "INSERT INTO messages (sentence, channel_id, guild_id, message_id, author_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sentence, msg.channel_id.get(), guild_id.get(), msg.id.get(), msg.author.id.get()],
            )
            .expect("Failed to insert word into database");
        }
    }
}
