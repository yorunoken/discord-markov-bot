use rand::rngs::OsRng;
use tokio::time::Duration;

use rand::Rng;
use rusqlite::{params, Connection};

use serenity::builder::GetMessages;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{all::CreateMessage, async_trait};

use crate::commands::Command;
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
                                if let Some(builder) =
                                    generate_markov_message(guild_id, channel.id, None).await
                                {
                                    if !messages_have_bot {
                                        channel.send_message(&ctx.http, builder).await.unwrap();
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

        let command_initiated = handle_command(&ctx, &msg, &self.commands).await;

        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            let builder = match generate_markov_message(guild_id, msg.channel_id, None).await {
                Some(s) => s,
                None => CreateMessage::new()
                    .content("Please wait until this channel has over 500 messages."),
            };

            msg.channel_id
                .send_message(&ctx.http, builder)
                .await
                .unwrap();
            return;
        }

        if !command_initiated {
            // Only save to database if message doesn't contain bot user tag
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
    }
}

const PREFIX: &str = "m.";

async fn handle_command(ctx: &Context, msg: &Message, commands: &Vec<Command>) -> bool {
    // Make sure we're dealing with humans :)
    if msg.author.bot || msg.content.len() == 0 {
        return false;
    }

    // Message doesn't start with the prefix, meaning it's not a command. So we return
    if !msg.content.starts_with(PREFIX) {
        return false;
    }

    // Get the arguments
    let mut args: Vec<&str> = msg
        .content
        .strip_prefix(PREFIX)
        .unwrap()
        .split_whitespace()
        .collect();

    // Get the command name by removing the first arg of the args array
    let command_input = args.remove(0);

    for command in commands {
        if command.name == command_input || command.aliases.contains(&command_input.into()) {
            let matched_alias = match command.name == command_input {
                true => None,
                false => Some(command_input),
            };

            // Start typing
            msg.channel_id.start_typing(&ctx.http);

            // Execute command
            if let Err(reason) =
                (command.exec)(&ctx, &msg, args, &command.name, matched_alias).await
            {
                println!(
                    "There was an error while handling command {}: {:#?}",
                    command.name, reason
                )
            }

            return true;
        }
    }

    false
}
