use rand::rngs::OsRng;
use tokio::time::Duration;

use rand::Rng;
use rusqlite::{params, Connection};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::utils::{generate_markov_message, get_most_popular_channel};

pub struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        println!("Bot has started as {}", bot.user.name);

        let mut rng = OsRng;
        println!("started loop");
        tokio::spawn(async move {
            loop {
                // Fetch vector of guilds the bot is in.
                let guild_ids = ctx.cache.guilds();

                for guild_id in guild_ids {
                    let popular_channel_id = get_most_popular_channel(guild_id).await;
                    let all_channels = ctx.http.get_channels(guild_id).await.unwrap();
                    let channel_id = all_channels
                        .iter()
                        .find(|channel| channel.id.get() == popular_channel_id)
                        .map(|channel| channel.id)
                        .unwrap();

                    let channel = ctx.http.get_channel(channel_id).await.unwrap();

                    match channel.guild() {
                        Some(channel) => {
                            let builder = generate_markov_message(guild_id, channel.id).await;
                            channel.send_message(&ctx.http, builder).await.unwrap();
                        }
                        None => {}
                    }
                }

                // Wait a random second from 60 to 270
                let range = rng.gen_range(60..270);
                tokio::time::sleep(Duration::from_secs(range)).await;
            }
        });
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
            let builder = generate_markov_message(guild_id, msg.channel_id).await;
            msg.channel_id
                .send_message(&ctx.http, builder)
                .await
                .unwrap();
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
