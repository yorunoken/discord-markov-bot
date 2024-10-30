use dotenv::dotenv;
use rusqlite::Connection;
use serenity::prelude::*;
use std::env;

mod commands;
mod event_handler;
mod markov_chain;
mod string_cmp;
mod utils;

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenv().ok();

    // Create the messages table if it doesn't exist
    let conn = Connection::open("messages.db").expect("Unable to open database");
    let sql_messages = "
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id INTEGER NOT NULL,
            author_id INTEGER NOT NULL,
            channel_id INTEGER NOT NULL,
            guild_id INTEGER NOT NULL,
            content TEXT NOT NULL
        );
        ";
    let sql_game_ratings = "
        CREATE TABLE IF NOT EXISTS game_ratings (
            user_id TEXT PRIMARY KEY,
            rating REAL
        );
        ";
    conn.execute(sql_messages, [])
        .expect("Failed to create messages table");
    conn.execute(sql_game_ratings, [])
        .expect("Failed to create game_ratings table");

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN to be defined in environment.");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let commands = commands::commands_vecs();
    let registered = commands::register_vecs();

    // Build the Discord client, and pass in our event handler
    let mut client = Client::builder(discord_token, intents)
        .event_handler(event_handler::Handler {
            commands,
            registered,
        })
        .await
        .expect("Error creating client.");

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
