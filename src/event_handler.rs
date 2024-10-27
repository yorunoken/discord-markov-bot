use rand::rngs::OsRng;
use tokio::time::Duration;

use rand::Rng;
use rusqlite::{params, Connection};

use serenity::builder::GetMessages;
use serenity::model::{application::Interaction, channel::Message, gateway::Ready};
use serenity::prelude::*;
use serenity::{
    all::{Command as CommandInteraction, CreateMessage},
    async_trait,
};

use crate::commands::{self, Command};
use crate::utils::{
    change_bot_profile, generate_markov_message, get_most_popular_channel, get_random_pfp,
};

pub struct Handler {
    pub commands: Vec<Command>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        println!("Bot has started as {}", bot.user.name);

        match CommandInteraction::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::generate::register(),
                commands::leaderboard::register(),
            ],
        )
        .await
        {
            Err(e) => {
                eprintln!("There was an error while registering commands: {}", e);
            }
            Ok(_) => {}
        }

        let http = ctx.http.clone();

        // Random message generator on loop
        let mut rng = OsRng;
        tokio::spawn(async move {
            loop {
                // Fetch vector of guilds the bot is in.
                let guild_ids = ctx.cache.guilds();

                // Loop over the guild ids
                for guild_id in guild_ids {
                    // Get the channel id of the most popular channel
                    let popular_channel_id = get_most_popular_channel(guild_id).await;
                    let all_channels = ctx.http.get_channels(guild_id).await.unwrap();

                    if let Some(channel_id) = all_channels
                        .iter()
                        .find(|channel| channel.id.get() == popular_channel_id)
                        .map(|channel| channel.id)
                    {
                        // Fetch the channel
                        let channel = ctx.http.get_channel(channel_id).await.unwrap();

                        match channel.guild() {
                            Some(channel) => {
                                let messages = channel
                                    .messages(&ctx.http, GetMessages::new().limit(100))
                                    .await
                                    .unwrap();

                                let mut messages_have_bot = false;
                                for message in messages {
                                    if message.author.id.get() == ctx.cache.current_user().id.get()
                                    {
                                        messages_have_bot = true;
                                    }
                                }

                                // Only send a message if builder is not None
                                if let Some(markov_message) =
                                    generate_markov_message(guild_id, channel.id, None).await
                                {
                                    if !messages_have_bot {
                                        channel
                                            .send_message(
                                                &ctx.http,
                                                CreateMessage::new().content(markov_message),
                                            )
                                            .await
                                            .unwrap();
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }

                // Wait a random second from 300 to 900
                let range = rng.gen_range(300..900);
                tokio::time::sleep(Duration::from_secs(range)).await;
            }
        });

        // Avatar switcher
        const HOURS_TO_WAIT: u64 = 12;
        tokio::spawn(async move {
            loop {
                match get_random_pfp().await {
                    Ok(Some(avatar_link)) => {
                        match change_bot_profile(&http, &avatar_link).await {
                            Err(err) => {
                                eprintln!("There was an error while changing profile: {}", err);
                            }
                            Ok(_) => {
                                println!("Succesfully changes profile.\n\nProfile details:\navatar link: {}", avatar_link)
                            }
                        }
                    }
                    Ok(None) => {
                        println!("No profile available");
                    }
                    Err(e) => {
                        eprintln!("Error getting random profile: {}", e);
                    }
                }

                // Wait for `HOURS_TO_WAIT` hours before switching avatars again
                tokio::time::sleep(Duration::from_secs(60 * 60 * HOURS_TO_WAIT)).await;
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
            let builder = match generate_markov_message(guild_id, msg.channel_id, None).await {
                Some(markov_message) => CreateMessage::new().content(markov_message),
                None => CreateMessage::new()
                    .content("Please wait until this channel has over 500 messages."),
            };

            msg.channel_id
                .send_message(&ctx.http, builder)
                .await
                .unwrap();
            return;
        }

        let sentence = msg.content;
        if !sentence.contains(format!("<@{}>", ctx.cache.current_user().id).as_str()) {
            let conn = Connection::open("messages.db").expect("Unable to open database");
            conn.execute(
                "INSERT INTO messages (content, channel_id, guild_id, message_id, author_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sentence, msg.channel_id.get(), guild_id.get(), msg.id.get(), msg.author.id.get()],
            )
            .expect("Failed to insert word into database");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(interaction) = interaction {
            for command in &self.commands {
                if interaction.data.name.as_str() == command.name {
                    // Execute command
                    if let Err(reason) = (command.exec)(&ctx, &interaction).await {
                        println!(
                            "There was an error while handling command {}: {:#?}",
                            command.name, reason
                        )
                    }
                }
            }
        }
    }
}
